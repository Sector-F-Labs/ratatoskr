use async_trait::async_trait;
use std::pin::Pin;
use tokio_stream::Stream;

pub type BoxStream<'a, T> = Pin<Box<dyn Stream<Item = T> + Send + 'a>>;

#[async_trait]
pub trait MessageBroker: Send + Sync {
    async fn publish(&self, payload: &[u8]) -> anyhow::Result<()>;
    async fn subscribe<'a>(&'a self) -> anyhow::Result<BoxStream<'a, Vec<u8>>>;
}

pub mod pipe;
pub mod user_pipes;
