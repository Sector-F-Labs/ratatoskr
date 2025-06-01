// src/messaging.rs
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// A generic trait for interacting with a messaging system.
#[async_trait]
pub trait MessageSystem: Send + Sync + 'static {
    /// The type of message consumed from the system.
    /// Must be Deserializable, Send, Sync, Debug, and Clone.
    /// 'static lifetime for Deserialize ensures it can be deserialized from any lifetime, common for message queues.
    type ConsumeMessage: for<'de> Deserialize<'de> + Send + Sync + std::fmt::Debug + Clone;

    /// The type of message published to the system.
    /// Must be Serializable, Send, Sync, Debug, and Clone.
    type PublishMessage: Serialize + Send + Sync + std::fmt::Debug + Clone;

    /// The type of error that can occur when interacting with the system.
    /// Must implement standard Error trait and be Send + Sync.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Establishes a connection to the messaging system.
    /// This might be a no-op for some systems or involve complex setup for others.
    async fn connect(&mut self) -> Result<(), Self::Error>;

    /// Starts consuming messages from the system.
    ///
    /// The callback `F` is an async function/closure that processes each `ConsumeMessage`.
    /// The consumer loop should continue until an unrecoverable error occurs or
    /// the system signals a shutdown (if applicable).
    async fn consume<F, Fut>(&mut self, callback: F) -> Result<(), Self::Error>
    where
        F: FnMut(Self::ConsumeMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static;

    /// Sends a message to the system.
    ///
    /// Takes a reference to `PublishMessage` to allow for non-consuming publication
    /// if the message needs to be retained or sent elsewhere.
    async fn publish(&self, message: &Self::PublishMessage) -> Result<(), Self::Error>;
}
