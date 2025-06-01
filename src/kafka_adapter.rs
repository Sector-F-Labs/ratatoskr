// src/kafka_adapter.rs

use async_trait::async_trait;
use chrono::Utc;
use futures_util::StreamExt;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    error::KafkaError,
    message::{Message as KafkaMessageRd, OwnedMessage},
    producer::{FutureProducer, FutureRecord},
    Message,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use crate::messaging::MessageSystem;
// Message type for consuming from Kafka (e.g., messages to be sent to Telegram)
use crate::kafka_processing::outgoing::OutgoingMessage;
// Message type for publishing to Kafka (e.g., messages received from Telegram)
use crate::telegram_handler::incoming::IncomingMessage;

#[derive(Error, Debug)]
pub enum KafkaAdapterError {
    #[error("Kafka error: {0}")]
    Kafka(#[from] KafkaError),
    #[error("Serialization/Deserialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Failed to send message: {0}")]
    SendError(String),
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Consumer creation error: {0}")]
    ConsumerCreationError(String),
}

#[derive(Clone)] // Added Clone
pub struct KafkaAdapterConfig {
    pub brokers: String,
    pub group_id: String,
    pub consume_topic: String,
    pub produce_topic: String, // Topic to publish messages to
}

pub struct KafkaAdapter {
    config: KafkaAdapterConfig,
    consumer: Option<StreamConsumer>,
    producer: FutureProducer,
}

impl KafkaAdapter {
    pub fn new(config: KafkaAdapterConfig) -> Result<Self, KafkaAdapterError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("message.timeout.ms", "5000")
            .create()?;

        Ok(Self {
            config,
            consumer: None,
            producer,
        })
    }
}

#[async_trait]
impl MessageSystem for KafkaAdapter {
    type ConsumeMessage = OutgoingMessage;
    type PublishMessage = IncomingMessage; // Changed to IncomingMessage
    type Error = KafkaAdapterError;

    async fn connect(&mut self) -> Result<(), Self::Error> {
        tracing::info!(brokers = %self.config.brokers, group_id = %self.config.group_id, consume_topic = %self.config.consume_topic, publish_topic = %self.config.produce_topic, "Attempting to connect Kafka adapter");
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", &self.config.group_id)
            .set("bootstrap.servers", &self.config.brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            //.set("auto.offset.reset", "earliest") // Or "latest" depending on desired behavior
            .create()
            .map_err(|e| KafkaAdapterError::ConsumerCreationError(e.to_string()))?;

        consumer
            .subscribe(&[&self.config.consume_topic])
            .map_err(|e| KafkaAdapterError::ConnectionError(e.to_string()))?;

        self.consumer = Some(consumer);
        tracing::info!("Kafka consumer connected and subscribed successfully.");
        Ok(())
    }

    async fn consume<F, Fut>(&mut self, mut callback: F) -> Result<(), Self::Error>
    where
        F: FnMut(Self::ConsumeMessage) -> Fut + Send + Sync + 'static, // Changed to ConsumeMessage
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let consumer = self
            .consumer
            .as_mut()
            .ok_or_else(|| KafkaAdapterError::ConnectionError("Consumer not initialized. Call connect first.".to_string()))?;

        tracing::info!(topic = %self.config.consume_topic, "Starting Kafka message consumption loop...");
        loop {
            match consumer.recv().await {
                Ok(borrowed_message) => {
                    // Process the message
                    let owned_message = borrowed_message.detach();
                    if let Some(payload) = owned_message.payload() {
                        match serde_json::from_slice::<Self::ConsumeMessage>(payload) { // Changed to ConsumeMessage
                            Ok(consumed_msg) => {
                                tracing::debug!(topic = %self.config.consume_topic, "Successfully deserialized message from Kafka");
                                // Invoke the callback with the deserialized message and await its completion
                                callback(consumed_msg).await;
                            }
                            Err(e) => {
                                tracing::error!(topic = %self.config.consume_topic, error = %e, "Error deserializing message from Kafka payload");
                                tracing::debug!(raw_payload = ?String::from_utf8_lossy(payload), "Problematic Kafka payload");
                                // Potentially, we could pass the error to the callback or an error handler
                            }
                        }
                    } else {
                        tracing::warn!(topic = %self.config.consume_topic, "Received Kafka message with empty payload");
                    }
                }
                Err(e) => {
                    tracing::error!(topic = %self.config.consume_topic, error = %e, "Error consuming message from Kafka");
                    // Decide if this error is fatal or if the loop should continue
                    // For now, we return the error, which will stop the loop.
                    return Err(KafkaAdapterError::Kafka(e));
                }
            }
        }
    }

    async fn publish(&self, message: &Self::PublishMessage) -> Result<(), Self::Error> { // Changed signature
        tracing::debug!(topic = %self.config.produce_topic, "Attempting to publish message type: {:?}", std::any::type_name::<Self::PublishMessage>());
        let payload = serde_json::to_string(message)?; // message is already a reference

        let record = FutureRecord::to(&self.config.produce_topic)
            .payload(&payload)
            .key("default_key"); // TODO: Determine a meaningful key, perhaps from message source or type

        match self.producer.send(record, Duration::from_secs(5)).await { // Increased timeout slightly
            Ok(delivery) => {
                tracing::info!(topic = %self.config.produce_topic, partition = %delivery.0, offset = %delivery.1, "Message published successfully");
                Ok(())
            }
            Err((e, _owned_message)) => {
                tracing::error!(topic = %self.config.produce_topic, error = %e, "Failed to send message");
                Err(KafkaAdapterError::SendError(e.to_string()))
            }
        }
    }
}

// Ensure the module is declared in main.rs or lib.rs
// pub mod kafka_adapter;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kafka_processing::outgoing::{OutgoingMessage, TextMessageData, MessageTarget};
    use crate::telegram_handler::incoming::{IncomingMessage, IncomingMessageType, TelegramMessageData, MessageSource, FileInfo};
    use crate::messaging::MessageSystem;
    use testcontainers_modules::kafka::Kafka;
    use testcontainers::clients;
    use rdkafka::producer::{FutureProducer, FutureRecord, DefaultProducerContext};
    use rdkafka::consumer::{StreamConsumer, Consumer, CommitMode};
    use rdkafka::config::ClientConfig as RdkafkaClientConfig; // Aliased to avoid conflict with KafkaAdapterConfig
    use rdkafka::message::Message as RdkafkaMessage; // Aliased
    use std::time::Duration;
    use uuid::Uuid;
    use chrono::Utc;
    use teloxide::types::Message as TeloxideMessage; // For dummy TelegramMessage

    fn create_sample_incoming_message(text: String) -> IncomingMessage {
        // Create a dummy Teloxide Message
        let dummy_teloxide_msg: TeloxideMessage = serde_json::from_value(serde_json::json!({
            "message_id": 1,
            "date": 0,
            "chat": {"id": 1, "type": "private"},
            "from": {"id": 1, "is_bot": false, "first_name": "Test"},
            "text": text,
        })).unwrap();

        IncomingMessage {
            message_type: IncomingMessageType::TelegramMessage(TelegramMessageData {
                message: dummy_teloxide_msg,
                downloaded_files: Vec::<FileInfo>::new(),
            }),
            timestamp: Utc::now(),
            source: MessageSource {
                platform: "test_platform".to_string(),
                bot_id: Some(123),
                bot_username: Some("test_bot".to_string()),
            },
        }
    }

    fn create_sample_outgoing_message(text: String) -> OutgoingMessage {
        OutgoingMessage {
            message_type: crate::kafka_processing::outgoing::OutgoingMessageType::TextMessage(TextMessageData {
                text,
                buttons: None,
                reply_keyboard: None,
                parse_mode: None,
                disable_web_page_preview: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget {
                platform: "test_platform".to_string(),
                chat_id: 123,
                thread_id: None,
            },
        }
    }

    #[tokio::test]
    async fn test_kafka_publish_incoming_message() {
        let docker = clients::Cli::default();
        let kafka_node = docker.run(Kafka::default());
        let bootstrap_servers = format!("localhost:{}", kafka_node.get_host_port_ipv4(9093));

        let topic_name = format!("test_publish_topic_{}", Uuid::new_v4());

        let adapter_config = KafkaAdapterConfig {
            brokers: bootstrap_servers.clone(),
            group_id: "test_publish_group".to_string(),
            consume_topic: "dummy_consume_for_publish_test".to_string(), // Not used in this test directly by adapter
            produce_topic: topic_name.clone(),
        };

        let mut adapter = KafkaAdapter::new(adapter_config).expect("Failed to create adapter");
        // For publish-only, connect might not be strictly necessary if producer is created in new,
        // but MessageSystem expects connect to be callable.
        // KafkaAdapter's connect also sets up a consumer, which isn't ideal for a publish-only test.
        // However, producer is created in `new` and available.

        let sample_message = create_sample_incoming_message("Hello Kafka from Test!".to_string());
        adapter.publish(&sample_message).await.expect("Publish failed");

        // Setup a separate consumer to verify the message
        let mut consumer_config = RdkafkaClientConfig::new();
        consumer_config
            .set("bootstrap.servers", &bootstrap_servers)
            .set("group.id", "verify_consumer_group")
            .set("auto.offset.reset", "earliest");

        let consumer: StreamConsumer<DefaultProducerContext> = consumer_config.create().expect("Failed to create test consumer");
        consumer.subscribe(&[&topic_name]).expect("Test consumer subscribe failed");

        let message_stream = consumer.stream();
        let received_message = tokio::time::timeout(Duration::from_secs(10), message_stream.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Error receiving message from Kafka");

        consumer.commit_message(&received_message, CommitMode::Async).unwrap();

        let payload_str = std::str::from_utf8(received_message.payload().unwrap()).unwrap();
        let received_deserialized: IncomingMessage = serde_json::from_str(payload_str).expect("Failed to deserialize consumed message");

        if let IncomingMessageType::TelegramMessage(tm_data) = received_deserialized.message_type {
            if let IncomingMessageType::TelegramMessage(sm_data) = sample_message.message_type {
                 assert_eq!(tm_data.message.text(), sm_data.message.text());
            } else {
                panic!("Sample message was not TelegramMessage type");
            }
        } else {
            panic!("Received message was not TelegramMessage type");
        }
        assert_eq!(received_deserialized.source.platform, sample_message.source.platform);
    }


    #[tokio::test]
    async fn test_kafka_consume_outgoing_message() {
        let docker = clients::Cli::default();
        let kafka_node = docker.run(Kafka::default());
        let bootstrap_servers = format!("localhost:{}", kafka_node.get_host_port_ipv4(9093));

        let topic_name = format!("test_consume_topic_{}", Uuid::new_v4());

        let adapter_config = KafkaAdapterConfig {
            brokers: bootstrap_servers.clone(),
            group_id: "test_consume_group".to_string(),
            consume_topic: topic_name.clone(),
            produce_topic: "dummy_produce_for_consume_test".to_string(), // Not used
        };

        let mut adapter = KafkaAdapter::new(adapter_config).expect("Failed to create adapter");
        adapter.connect().await.expect("Adapter connect failed");

        let sample_message = create_sample_outgoing_message("Hello Adapter, from Kafka!".to_string());

        // Setup a producer to send a message to the topic the adapter consumes from
        let producer: FutureProducer = RdkafkaClientConfig::new()
            .set("bootstrap.servers", &bootstrap_servers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create test producer");

        let payload = serde_json::to_string(&sample_message).expect("Failed to serialize sample OutgoingMessage");
        let record = FutureRecord::to(&topic_name).payload(&payload).key("test_key");

        producer.send(record, Duration::from_secs(5)).await.expect("Test producer failed to send").expect("Delivery error");
        tracing::info!("Test message sent to Kafka topic {}", topic_name);

        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        tokio::spawn(async move {
            adapter.consume(move |consumed_message: OutgoingMessage| {
                let tx_clone = tx.clone();
                async move {
                    tx_clone.send(consumed_message).await.expect("Failed to send consumed message via channel");
                }
            }).await.expect("Consume loop failed");
        });

        let received_message = tokio::time::timeout(Duration::from_secs(10), rx.recv())
            .await
            .expect("Timeout waiting for consumed message via channel")
            .expect("Channel closed before message received");

        if let crate::kafka_processing::outgoing::OutgoingMessageType::TextMessage(received_text_data) = received_message.message_type {
             if let crate::kafka_processing::outgoing::OutgoingMessageType::TextMessage(sent_text_data) = sample_message.message_type {
                assert_eq!(received_text_data.text, sent_text_data.text);
             } else {
                panic!("Sample message was not TextMessage type");
             }
        } else {
            panic!("Received message was not TextMessage type");
        }
        assert_eq!(received_message.target.chat_id, sample_message.target.chat_id);
    }
}
