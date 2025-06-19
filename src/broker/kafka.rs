use crate::broker::{BoxStream, MessageBroker};
use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use rdkafka::{
    config::ClientConfig,
    consumer::{Consumer, StreamConsumer},
    message::Message as KafkaMessage,
    producer::{FutureProducer, FutureRecord},
};

pub struct KafkaBroker {
    producer: FutureProducer,
    consumer: StreamConsumer,
}

impl KafkaBroker {
    pub fn new(broker: &str) -> Result<Self> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", broker)
            .create()?;
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", "ratatoskr-broker")
            .set("bootstrap.servers", broker)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            .create()?;
        Ok(Self { producer, consumer })
    }
}

#[async_trait]
impl MessageBroker for KafkaBroker {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        let record: FutureRecord<'_, (), _> = FutureRecord::to(topic).payload(payload);
        self.producer
            .send(record, None)
            .await
            .map(|_| ())
            .map_err(|(e, _)| e.into())
    }

    async fn subscribe<'a>(&'a self, topic: &str) -> Result<BoxStream<'a, Vec<u8>>> {
        self.consumer.subscribe(&[topic])?;
        let stream = self.consumer.stream().filter_map(|msg| async move {
            match msg {
                Ok(m) => m.payload().map(|p| p.to_vec()),
                Err(e) => {
                    tracing::error!(error = %e, "Kafka consume error");
                    None
                }
            }
        });
        Ok(Box::pin(stream))
    }
}
