use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

use crate::broker::{BoxStream, MessageBroker};
use std::sync::Arc;

pub struct MqttBroker {
    client: AsyncClient,
    sender: broadcast::Sender<Vec<u8>>,
}

impl MqttBroker {
    pub async fn new(broker: &str) -> Result<Self> {
        let parts: Vec<&str> = broker.split(':').collect();
        let host = parts.get(0).unwrap_or(&"localhost");
        let port: u16 = parts.get(1).and_then(|p| p.parse().ok()).unwrap_or(1883);
        let mut mqttoptions = MqttOptions::new("ratatoskr-bot", host.to_string(), port);
        mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));
        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        let (tx, _) = broadcast::channel(100);
        let eventloop = Arc::new(tokio::sync::Mutex::new(eventloop));
        let tx_clone = tx.clone();
        let eventloop_clone = eventloop.clone();
        tokio::spawn(async move {
            loop {
                let ev = {
                    let mut evl = eventloop_clone.lock().await;
                    evl.poll().await
                };
                match ev {
                    Ok(Event::Incoming(Packet::Publish(p))) => {
                        let _ = tx_clone.send(p.payload.to_vec());
                    }
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!(error = %e, "MQTT poll error");
                        break;
                    }
                }
            }
        });
        Ok(Self { client, sender: tx })
    }
}

#[async_trait]
impl MessageBroker for MqttBroker {
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<()> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload)
            .await?;
        Ok(())
    }

    async fn subscribe<'a>(&'a self, topic: &str) -> Result<BoxStream<'a, Vec<u8>>> {
        self.client.subscribe(topic, QoS::AtLeastOnce).await?;
        let rx = self.sender.subscribe();
        let stream = BroadcastStream::new(rx).filter_map(|v| async move { v.ok() });
        Ok(Box::pin(stream))
    }
}
