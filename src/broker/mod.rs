use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

#[async_trait]
pub trait MessageBroker: Send + Sync {
    /// Publish a message with an optional key for partitioning.
    /// The key is typically the telegram_user_id for incoming messages.
    async fn publish(&self, key: Option<&str>, payload: &[u8]) -> anyhow::Result<()>;
    async fn subscribe<'a>(&'a self) -> anyhow::Result<BoxStream<'a, Vec<u8>>>;
}

pub mod kafka;
