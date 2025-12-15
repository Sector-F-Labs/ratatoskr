use crate::broker::{BoxStream, MessageBroker};
use anyhow::Result;
use async_trait::async_trait;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

/// PipeBroker bridges messages over stdout (Telegram -> handler) and a named pipe (handler -> Telegram).
/// `publish` writes JSON payloads to stdout (newline-delimited).
/// `subscribe` reads newline-delimited JSON messages from a named pipe.
pub struct PipeBroker {
    inbound_pipe: String,
}

impl PipeBroker {
    pub fn new(inbound_pipe: impl Into<String>) -> Self {
        Self {
            inbound_pipe: inbound_pipe.into(),
        }
    }
}

#[async_trait]
impl MessageBroker for PipeBroker {
    async fn publish(&self, payload: &[u8]) -> Result<()> {
        // Newline-delimit messages so downstream handlers can parse a stream.
        let mut stdout = tokio::io::stdout();
        stdout.write_all(payload).await?;
        stdout.write_all(b"\n").await?;
        stdout.flush().await?;
        Ok(())
    }

    async fn subscribe<'a>(&'a self) -> Result<BoxStream<'a, Vec<u8>>> {
        let pipe_path = self.inbound_pipe.clone();
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            loop {
                match File::open(&pipe_path).await {
                    Ok(file) => {
                        let mut reader = BufReader::new(file);
                        let mut line = String::new();
                        loop {
                            line.clear();
                            match reader.read_line(&mut line).await {
                                Ok(0) => break, // Writer closed; reopen.
                                Ok(_) => {
                                    if line.trim().is_empty() {
                                        continue;
                                    }
                                    if tx.send(line.as_bytes().to_vec()).await.is_err() {
                                        return;
                                    }
                                }
                                Err(e) => {
                                    tracing::error!(path = %pipe_path, error = %e, "Error reading from inbound pipe");
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!(path = %pipe_path, error = %e, "Failed to open inbound pipe; retrying");
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            }
        });

        let stream = ReceiverStream::new(rx).map(|bytes| bytes);
        Ok(Box::pin(stream))
    }
}
