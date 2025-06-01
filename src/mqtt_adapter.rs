// src/mqtt_adapter.rs

use async_trait::async_trait;
use rumqttc::{AsyncClient, Event, EventLoop, MqttOptions, Packet, QoS, SubscribeFilter};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::Mutex; // For EventLoop if needed for advanced scenarios, or if consume is &self

use crate::messaging::MessageSystem;
use crate::kafka_processing::outgoing::OutgoingMessage; // ConsumeMessage type
use crate::telegram_handler::incoming::IncomingMessage; // PublishMessage type

#[derive(Debug, Clone)]
pub struct MqttAdapterConfig {
    pub broker_address: String,
    pub broker_port: u16,
    pub client_id_prefix: String, // Will append a random suffix for uniqueness
    pub consume_topic: String,
    pub publish_topic: String,
    pub consume_qos: QoS,
    pub publish_qos: QoS,
    pub keep_alive_interval: Duration,
}

#[derive(Error, Debug)]
pub enum MqttAdapterError {
    #[error("MQTT client error: {0}")]
    MqttClient(#[from] rumqttc::ClientError),
    #[error("MQTT connection error: {0}")]
    MqttConnection(#[from] rumqttc::ConnectionError),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Subscription failed: {0}")]
    SubscriptionFailed(String),
    #[error("Publish failed: {0}")]
    PublishFailed(String),
    #[error("Consume loop not started or failed")]
    ConsumeLoopNotRunning,
    #[error("Message deserialization error: {0}")]
    Deserialization(String), // Specific for payload deserialization issues
}

pub struct MqttAdapter {
    config: MqttAdapterConfig,
    client: Option<AsyncClient>, // Option because it's set in connect
    event_loop: Arc<Mutex<Option<EventLoop>>>, // Option and Arc<Mutex> for shared access if needed
}

impl MqttAdapter {
    pub fn new(config: MqttAdapterConfig) -> Self {
        Self {
            config,
            client: None,
            event_loop: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait]
impl MessageSystem for MqttAdapter {
    type ConsumeMessage = OutgoingMessage;
    type PublishMessage = IncomingMessage;
    type Error = MqttAdapterError;

    async fn connect(&mut self) -> Result<(), Self::Error> {
        let random_suffix = nanoid::nanoid!(8);
        let client_id = format!("{}-{}", self.config.client_id_prefix, random_suffix);

        let mut mqtt_options = MqttOptions::new(client_id, &self.config.broker_address, self.config.broker_port);
        mqtt_options.set_keep_alive(self.config.keep_alive_interval);
        // Add other options like TLS, authentication if needed

        tracing::info!(broker = %self.config.broker_address, port = %self.config.broker_port, client_id = %mqtt_options.client_id(), "Connecting to MQTT broker...");

        let (client, event_loop) = AsyncClient::new(mqtt_options, 10); // 10 is the capacity of the event loop queue

        // Subscribe to the consume topic
        client
            .subscribe(&self.config.consume_topic, self.config.consume_qos)
            .await
            .map_err(|e| MqttAdapterError::SubscriptionFailed(e.to_string()))?;

        tracing::info!(topic = %self.config.consume_topic, qos = ?self.config.consume_qos, "Subscribed to MQTT topic successfully");

        self.client = Some(client);
        *self.event_loop.lock().await = Some(event_loop); // Store the event loop

        tracing::info!("MQTT client connected and subscribed successfully.");
        Ok(())
    }

    async fn consume<F, Fut>(&mut self, mut callback: F) -> Result<(), Self::Error>
    where
        F: FnMut(Self::ConsumeMessage) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let event_loop_arc = Arc::clone(&self.event_loop);
        let mut event_loop_guard = event_loop_arc.lock().await;

        let mut event_loop = match event_loop_guard.take() {
            Some(el) => el,
            None => return Err(MqttAdapterError::ConsumeLoopNotRunning),
        };

        // Drop the guard before entering the loop so it's not held indefinitely
        drop(event_loop_guard);

        tracing::info!(consume_topic = %self.config.consume_topic, "Starting MQTT event loop for message consumption...");

        loop {
            match event_loop.poll().await {
                Ok(event) => {
                    if let Event::Incoming(Packet::Publish(publish_packet)) = event {
                        tracing::debug!(topic = %publish_packet.topic, "Received MQTT message");
                        if publish_packet.topic == self.config.consume_topic {
                            match serde_json::from_slice::<Self::ConsumeMessage>(&publish_packet.payload) {
                                Ok(msg) => {
                                    callback(msg).await;
                                }
                                Err(e) => {
                                    tracing::error!(topic = %publish_packet.topic, error = %e, "Failed to deserialize MQTT message payload");
                                    // Optionally, could call an error handler callback
                                }
                            }
                        } else {
                            tracing::trace!(topic = %publish_packet.topic, "Received MQTT message on unhandled topic");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "Error in MQTT event loop");
                    // Depending on the error type, might need to reconnect or can return an error to stop.
                    // For simplicity, we return an error here.
                    // Re-store the event_loop if recoverable or if trying to restart
                    // *self.event_loop.lock().await = Some(event_loop); // Not safe without proper error handling
                    return Err(MqttAdapterError::MqttConnection(e));
                }
            }
        }
    }

    async fn publish(&self, message: &Self::PublishMessage) -> Result<(), Self::Error> {
        let client = self.client.as_ref().ok_or(MqttAdapterError::ConsumeLoopNotRunning)?; // Or a more appropriate error like NotConnected

        tracing::debug!(topic = %self.config.publish_topic, "Attempting to publish MQTT message");
        let payload = serde_json::to_string(message)?;

        client
            .publish(
                &self.config.publish_topic,
                self.config.publish_qos,
                false, // retain flag
                payload.as_bytes(),
            )
            .await
            .map_err(|e| MqttAdapterError::PublishFailed(e.to_string()))?;

        tracing::info!(topic = %self.config.publish_topic, "MQTT Message published successfully.");
        Ok(())
    }
}

// Helper for generating client ID suffix (requires `nanoid` crate to be added to Cargo.toml)
// Example: `nanoid = "0.4"`
// For now, this is commented out as nanoid is not yet added.
// A simpler random suffix can be used if nanoid is not available.
mod nanoid {
    use rand::{Rng, distributions::Alphanumeric};
    pub fn nanoid!(len: usize) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kafka_processing::outgoing::{OutgoingMessage, TextMessageData, MessageTarget}; // Reusing for sample data structure
    use crate::telegram_handler::incoming::{IncomingMessage, IncomingMessageType, TelegramMessageData, MessageSource, FileInfo}; // Reusing
    use crate::messaging::MessageSystem;
    use testcontainers_modules::mosquitto::Mosquitto;
    use testcontainers::clients;
    use rumqttc::{MqttOptions as RumqttcMqttOptions, AsyncClient as RumqttcAsyncClient, QoS as RumqttcQoS, Event as RumqttcEvent, Packet as RumqttcPacket};
    use std::time::Duration;
    use uuid::Uuid;
    use chrono::Utc;
    use teloxide::types::Message as TeloxideMessage; // For dummy TelegramMessage
    use tokio::sync::mpsc as tokio_mpsc; // Explicit import

    // Helper to create a sample IncomingMessage (copied from kafka_adapter tests, ensure consistency or centralize)
    fn create_sample_incoming_message(text: String) -> IncomingMessage {
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
                downloaded_files: Vec::new(),
            }),
            timestamp: Utc::now(),
            source: MessageSource { platform: "test_mqtt".to_string(), bot_id: Some(123), bot_username: Some("test_bot".to_string()) },
        }
    }

    // Helper to create a sample OutgoingMessage (copied from kafka_adapter tests)
    fn create_sample_outgoing_message(text: String) -> OutgoingMessage {
        OutgoingMessage {
            message_type: crate::kafka_processing::outgoing::OutgoingMessageType::TextMessage(TextMessageData {
                text, buttons: None, reply_keyboard: None, parse_mode: None, disable_web_page_preview: None,
            }),
            timestamp: Utc::now(),
            target: MessageTarget { platform: "test_mqtt".to_string(), chat_id: 123, thread_id: None },
        }
    }

    #[tokio::test]
    async fn test_mqtt_publish_incoming_message() {
        let docker = clients::Cli::default();
        let mosquitto_node = docker.run(Mosquitto::default());
        let broker_port = mosquitto_node.get_host_port_ipv4(1883);
        let broker_address = "localhost".to_string();

        let publish_topic = format!("test/publish/{}", Uuid::new_v4());
        let consume_topic_verifier = publish_topic.clone(); // Verifier subscribes to the same topic

        let adapter_config = MqttAdapterConfig {
            broker_address: broker_address.clone(),
            broker_port,
            client_id_prefix: "test_pub_adapter_".to_string(),
            consume_topic: "dummy/consume/topic".to_string(), // Not directly used by this adapter's publish
            publish_topic: publish_topic.clone(),
            consume_qos: QoS::AtLeastOnce, // Adapter's internal QoS for its consumer
            publish_qos: QoS::AtLeastOnce, // Adapter's publish QoS
            keep_alive_interval: Duration::from_secs(5),
        };

        let mut adapter = MqttAdapter::new(adapter_config);
        adapter.connect().await.expect("Adapter connect failed for publish test"); // Connect to init client

        let sample_message = create_sample_incoming_message("Hello MQTT from Test!".to_string());
        adapter.publish(&sample_message).await.expect("Publish failed");

        // Setup a separate rumqttc client to verify the message
        let verifier_client_id = format!("verifier_{}", Uuid::new_v4());
        let mut mqtt_options_verifier = RumqttcMqttOptions::new(verifier_client_id, &broker_address, broker_port);
        mqtt_options_verifier.set_keep_alive(Duration::from_secs(5));
        let (verifier_client, mut verifier_eventloop) = RumqttcAsyncClient::new(mqtt_options_verifier, 10);

        verifier_client.subscribe(&consume_topic_verifier, RumqttcQoS::AtLeastOnce).await.expect("Verifier subscribe failed");

        let mut received_payload = None;
        for _ in 0..5 { // Try to poll a few times with timeout
            match tokio::time::timeout(Duration::from_secs(2), verifier_eventloop.poll()).await {
                Ok(Ok(RumqttcEvent::Incoming(RumqttcPacket::Publish(publish_packet)))) => {
                    if publish_packet.topic == consume_topic_verifier {
                        received_payload = Some(publish_packet.payload);
                        break;
                    }
                }
                Ok(Ok(_)) => {} // Other events
                Ok(Err(e)) => {
                    tracing::error!("Verifier eventloop error: {:?}", e);
                    break;
                } // Eventloop error
                Err(_) => { // Timeout
                    tracing::warn!("Verifier timeout polling for message");
                }
            }
        }

        assert!(received_payload.is_some(), "Did not receive message on verifier client");
        let payload_bytes = received_payload.unwrap();
        let received_deserialized: IncomingMessage = serde_json::from_slice(&payload_bytes).expect("Failed to deserialize consumed message by verifier");

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
    async fn test_mqtt_consume_outgoing_message() {
        let docker = clients::Cli::default();
        let mosquitto_node = docker.run(Mosquitto::default());
        let broker_port = mosquitto_node.get_host_port_ipv4(1883);
        let broker_address = "localhost".to_string();

        let consume_topic = format!("test/consume/{}", Uuid::new_v4());

        let adapter_config = MqttAdapterConfig {
            broker_address: broker_address.clone(),
            broker_port,
            client_id_prefix: "test_consume_adapter_".to_string(),
            consume_topic: consume_topic.clone(),
            publish_topic: "dummy/publish/topic".to_string(), // Not used by this adapter's consume
            consume_qos: QoS::AtLeastOnce,
            publish_qos: QoS::AtLeastOnce,
            keep_alive_interval: Duration::from_secs(5),
        };

        let mut adapter = MqttAdapter::new(adapter_config);
        adapter.connect().await.expect("Adapter connect failed for consume test");

        let sample_message = create_sample_outgoing_message("Hello MQTT Adapter, from Test Producer!".to_string());

        // Setup a separate rumqttc client to publish a message
        let producer_client_id = format!("producer_{}", Uuid::new_v4());
        let mut mqtt_options_producer = RumqttcMqttOptions::new(producer_client_id, &broker_address, broker_port);
        mqtt_options_producer.set_keep_alive(Duration::from_secs(5));
        // Producer client does not need an event loop if it only publishes and doesn't process incoming ACKs explicitly.
        let (producer_client, _producer_eventloop) = RumqttcAsyncClient::new(mqtt_options_producer, 10);

        let payload = serde_json::to_string(&sample_message).expect("Failed to serialize sample OutgoingMessage");
        producer_client.publish(&consume_topic, RumqttcQoS::AtLeastOnce, false, payload.as_bytes()).await.expect("Test producer failed to send");
        tracing::info!("Test message sent to MQTT topic {}", consume_topic);

        let (tx, mut rx) = tokio_mpsc::channel(1);

        tokio::spawn(async move {
            adapter.consume(move |consumed_message: OutgoingMessage| {
                let tx_clone = tx.clone();
                async move {
                    tx_clone.send(consumed_message).await.expect("Failed to send consumed message via channel");
                }
            }).await.expect("Consume loop failed"); // This will run indefinitely unless error
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
