use crate::auth::AuthService;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{RwLock, mpsc};

/// Per-user pipe router. Routes incoming messages to each user's `in.pipe`
/// and reads responses from each user's `out.pipe`.
pub struct UserPipeRouter {
    auth: Arc<RwLock<AuthService>>,
    response_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl UserPipeRouter {
    pub fn new(auth: Arc<RwLock<AuthService>>) -> (Self, mpsc::UnboundedReceiver<Vec<u8>>) {
        let (response_tx, response_rx) = mpsc::unbounded_channel();
        (Self { auth, response_tx }, response_rx)
    }

    /// Write a JSONL payload to the user's `in.pipe`.
    /// Opens the pipe lazily; logs and returns Ok on failure (caller should not retry).
    pub async fn route(&self, user_index: usize, payload: &[u8]) -> Result<()> {
        let pipe_dir = {
            let auth = self.auth.read().await;
            let user = auth
                .get_user(user_index)
                .context("user index out of bounds")?;
            user.pipe_dir.clone()
        };

        let in_pipe = PathBuf::from(&pipe_dir).join("in.pipe");

        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&in_pipe)
            .await
            .with_context(|| format!("Failed to open {}", in_pipe.display()))?;

        file.write_all(payload).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;

        tracing::debug!(pipe = %in_pipe.display(), user_index, "Routed message to user pipe");
        Ok(())
    }

    /// Spawn reader tasks for all configured users' `out.pipe` files.
    /// Each task reads JSONL lines and forwards them through the response channel.
    pub async fn start_readers(&self) {
        let auth = self.auth.read().await;
        let users: Vec<(usize, String)> = (0..auth.user_count())
            .filter_map(|i| {
                auth.get_user(i).map(|u| {
                    let dir = u.pipe_dir.clone();
                    (i, dir)
                })
            })
            .collect();
        drop(auth);

        for (idx, pipe_dir) in users {
            let out_pipe = PathBuf::from(&pipe_dir).join("out.pipe");
            let tx = self.response_tx.clone();
            tokio::spawn(async move {
                let mut backoff = Duration::from_millis(500);
                let max_backoff = Duration::from_secs(30);

                loop {
                    match File::open(&out_pipe).await {
                        Ok(file) => {
                            backoff = Duration::from_millis(500); // reset on success
                            let mut reader = BufReader::new(file);
                            let mut line = String::new();
                            loop {
                                line.clear();
                                match reader.read_line(&mut line).await {
                                    Ok(0) => break, // writer closed, reopen
                                    Ok(_) => {
                                        if line.trim().is_empty() {
                                            continue;
                                        }
                                        if tx.send(line.as_bytes().to_vec()).is_err() {
                                            tracing::warn!(user_index = idx, "Response channel closed, stopping out.pipe reader");
                                            return;
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(pipe = %out_pipe.display(), user_index = idx, error = %e, "Error reading out.pipe");
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!(pipe = %out_pipe.display(), user_index = idx, error = %e, backoff_ms = backoff.as_millis(), "out.pipe not available, retrying");
                            tokio::time::sleep(backoff).await;
                            backoff = (backoff * 2).min(max_backoff);
                        }
                    }
                }
            });
        }
    }
}
