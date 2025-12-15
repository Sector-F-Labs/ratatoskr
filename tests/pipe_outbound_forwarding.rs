use std::{path::PathBuf, process::Command, time::Duration};

use futures_util::StreamExt;
use ratatoskr::broker::{pipe::PipeBroker, MessageBroker};
use ratatoskr::kafka_processing::outgoing::{OutgoingMessage, OutgoingMessageType};
use tempfile::tempdir;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn pipe_broker_reads_outgoing_message_from_fifo() -> anyhow::Result<()> {
    let dir = tempdir()?;
    let pipe_path = dir.path().join("out.pipe");

    // Create the named pipe used by PipeBroker
    let status = Command::new("mkfifo")
        .arg(&pipe_path)
        .status()
        .expect("mkfifo should be available on Unix-like systems");
    assert!(status.success(), "mkfifo failed: {:?}", status);

    let broker = PipeBroker::new(pipe_path.to_string_lossy());
    let mut stream = broker.subscribe().await?;

    // Load sample OutgoingMessage payload
    let json_path: PathBuf =
        [env!("CARGO_MANIFEST_DIR"), "tests", "data", "outgoing_text.json"]
            .iter()
            .collect();
    let payload = tokio::fs::read_to_string(json_path).await?;

    // Write the payload into the outbound pipe to simulate a handler response
    let writer_path = pipe_path.clone();
    let writer = tokio::spawn(async move {
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .open(&writer_path)
            .await?;
        file.write_all(payload.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await?;
        anyhow::Ok(())
    });

    // Read the first message from the pipe-backed stream
    let bytes = tokio::time::timeout(Duration::from_secs(3), stream.next())
        .await?
        .expect("expected payload from pipe");
    writer.await??;

    let raw = String::from_utf8_lossy(&bytes).to_string();
    println!("RAW_PAYLOAD: {}", raw);
    assert!(
        !raw.trim().is_empty(),
        "received empty payload from pipe"
    );
    let outgoing: OutgoingMessage = serde_json::from_str(&raw)?;
    assert_eq!(outgoing.target.chat_id, 123456789);
    match outgoing.message_type {
        OutgoingMessageType::TextMessage(data) => {
            assert_eq!(data.text, "hello from pipe");
        }
        other => panic!("expected text message, got {:?}", other),
    }

    Ok(())
}
