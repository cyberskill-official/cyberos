//! FR-AI-010 §1 #9 — Heartbeat task.
//!
//! Emits `event: heartbeat` every 15 seconds during a steady stream to keep
//! proxies (CDNs, corporate firewalls) from timing out idle connections.

use std::time::Duration;

use tokio::sync::mpsc;

use super::StreamEvent;

/// Run the heartbeat loop. Stops when `tx` is dropped (stream ended).
pub async fn run(
    tx: mpsc::Sender<StreamEvent>,
    interval: Duration,
    provider_label: &str,
    model_label: &str,
) {
    let mut tick = tokio::time::interval(interval);
    tick.tick().await; // skip the immediate first tick
    loop {
        tick.tick().await;
        if tx.send(StreamEvent::Heartbeat).await.is_err() {
            return; // receiver dropped; stop heartbeat task
        }
        super::metrics::HEARTBEATS
            .with_label_values(&[provider_label, model_label])
            .inc();
    }
}
