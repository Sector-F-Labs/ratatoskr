use async_trait::async_trait;
use tokio_stream::Stream;
use std::pin::Pin;

pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(&self, topic: &str, payload: &[u8]) -> anyhow::Result<()>;
    async fn subscribe<'a>(&'a self, topic: &str) -> anyhow::Result<BoxStream<'a, Vec<u8>>>;
}

pub mod kafka;
pub mod mqtt;
