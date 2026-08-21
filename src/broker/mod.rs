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

#[cfg(test)]
pub(crate) mod test_support {
    use super::{BoxStream, MessageBroker};
    use async_trait::async_trait;
    use std::sync::Mutex;
    use tokio_stream::wrappers::ReceiverStream;

    /// Records every published message for later assertions instead of sending
    /// anywhere. `subscribe` returns a stream that never yields, since nothing
    /// in this test double ever publishes to it.
    #[derive(Default)]
    pub(crate) struct MockMessageBroker {
        published: Mutex<Vec<(Option<String>, Vec<u8>)>>,
    }

    impl MockMessageBroker {
        pub(crate) fn new() -> Self {
            Self::default()
        }

        pub(crate) fn published(&self) -> Vec<(Option<String>, Vec<u8>)> {
            self.published.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl MessageBroker for MockMessageBroker {
        async fn publish(&self, key: Option<&str>, payload: &[u8]) -> anyhow::Result<()> {
            self.published
                .lock()
                .unwrap()
                .push((key.map(String::from), payload.to_vec()));
            Ok(())
        }

        async fn subscribe<'a>(&'a self) -> anyhow::Result<BoxStream<'a, Vec<u8>>> {
            let (_tx, rx) = tokio::sync::mpsc::channel(1);
            Ok(Box::pin(ReceiverStream::new(rx)))
        }
    }
}
