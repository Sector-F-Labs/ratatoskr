use crate::broker::{BoxStream, MessageBroker};
use anyhow::{Context, Result};
use async_trait::async_trait;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::client::DefaultClientContext;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::OwnedHeaders;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::Message;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

/// KafkaBroker implements MessageBroker using Apache Kafka.
/// - Publishes incoming Telegram messages to `{prefix}.in` topic
/// - Consumes outgoing messages from `{prefix}.out` topic
pub struct KafkaBroker {
    producer: FutureProducer,
    brokers: String,
    topic_in: String,
    topic_out: String,
    group_id: String,
}

impl KafkaBroker {
    /// Create a new KafkaBroker.
    ///
    /// # Arguments
    /// * `brokers` - Comma-separated list of Kafka brokers (e.g., "localhost:9092")
    /// * `topic_prefix` - Prefix for topic names (default: "ratatoskr")
    /// * `group_id` - Consumer group ID (default: "ratatoskr")
    pub fn new(brokers: &str, topic_prefix: Option<&str>, group_id: Option<&str>) -> Result<Self> {
        let prefix = topic_prefix.unwrap_or("ratatoskr");
        let group = group_id.unwrap_or("ratatoskr");

        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .set("queue.buffering.max.ms", "0") // Send immediately
            .create()
            .context("Failed to create Kafka producer")?;

        Ok(Self {
            producer,
            brokers: brokers.to_string(),
            topic_in: format!("{}.in", prefix),
            topic_out: format!("{}.out", prefix),
            group_id: group.to_string(),
        })
    }

    /// Ensure topics exist, creating them if necessary.
    pub async fn ensure_topics(&self) -> Result<()> {
        let admin: AdminClient<DefaultClientContext> = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .create()
            .context("Failed to create Kafka admin client")?;

        let topics = [
            NewTopic::new(&self.topic_in, 1, TopicReplication::Fixed(1)),
            NewTopic::new(&self.topic_out, 1, TopicReplication::Fixed(1)),
        ];

        let opts = AdminOptions::new().operation_timeout(Some(Duration::from_secs(5)));

        let results = admin.create_topics(&topics, &opts).await?;

        for result in results {
            match result {
                Ok(topic) => {
                    tracing::info!(topic = %topic, "Created Kafka topic");
                }
                Err((topic, err)) => {
                    // Ignore "topic already exists" error
                    if err == rdkafka::types::RDKafkaErrorCode::TopicAlreadyExists {
                        tracing::debug!(topic = %topic, "Kafka topic already exists");
                    } else {
                        tracing::warn!(topic = %topic, error = ?err, "Failed to create Kafka topic");
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl MessageBroker for KafkaBroker {
    async fn publish(&self, key: Option<&str>, payload: &[u8]) -> Result<()> {
        let mut record = FutureRecord::to(&self.topic_in)
            .payload(payload)
            .headers(OwnedHeaders::new());

        if let Some(k) = key {
            record = record.key(k);
        }

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(err, _)| anyhow::anyhow!("Failed to send message to Kafka: {}", err))?;

        tracing::debug!(
            topic = %self.topic_in,
            key = ?key,
            payload_size = payload.len(),
            "Published message to Kafka"
        );

        Ok(())
    }

    async fn subscribe<'a>(&'a self) -> Result<BoxStream<'a, Vec<u8>>> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", &self.brokers)
            .set("group.id", &self.group_id)
            .set("enable.auto.commit", "true")
            .set("auto.commit.interval.ms", "1000")
            .set("auto.offset.reset", "latest")
            .set("session.timeout.ms", "6000")
            .create()
            .context("Failed to create Kafka consumer")?;

        consumer
            .subscribe(&[&self.topic_out])
            .context("Failed to subscribe to topic")?;

        tracing::info!(
            topic = %self.topic_out,
            group_id = %self.group_id,
            "Subscribed to Kafka topic"
        );

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut stream = consumer.stream();
            while let Some(result) = stream.next().await {
                match result {
                    Ok(msg) => {
                        if let Some(payload) = msg.payload() {
                            tracing::debug!(
                                topic = %msg.topic(),
                                partition = msg.partition(),
                                offset = msg.offset(),
                                key = ?msg.key().map(|k| String::from_utf8_lossy(k)),
                                payload_size = payload.len(),
                                "Received message from Kafka"
                            );
                            if tx.send(payload.to_vec()).await.is_err() {
                                tracing::warn!("Kafka consumer channel closed");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Error consuming from Kafka");
                    }
                }
            }
            tracing::warn!("Kafka consumer stream ended");
        });

        let stream = ReceiverStream::new(rx);
        Ok(Box::pin(stream))
    }
}
