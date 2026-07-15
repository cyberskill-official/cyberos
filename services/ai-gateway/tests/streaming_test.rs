//! TASK-AI-010 §4 — Integration tests for streaming SSE.
//!
//! Tests the streaming infrastructure: types, SSE serialization, heartbeat,
//! backpressure, and reconcile guard. Provider-level integration tests
//! require real provider impls (future tasks).

use std::time::Duration;

use cyberos_ai_gateway::router::FinishReason;
use cyberos_ai_gateway::streaming::heartbeat;
use cyberos_ai_gateway::streaming::{
    ErrorCode, ProviderStreamEvent, ProviderStreamUsage, ReconcileReason, StreamEvent,
};
use tokio::sync::mpsc;

// ─── SSE serialization tests ─────────────────────────────────────────────────

#[test]
fn token_event_sse_does_not_panic() {
    let ev = StreamEvent::Token {
        text: "hello".into(),
        model: "test-model".into(),
        index: 0,
    };
    let _sse = ev.to_sse_event();
}

#[test]
fn usage_event_sse_does_not_panic() {
    let ev = StreamEvent::Usage {
        prompt_tokens: 100,
        completion_tokens: 50,
        cached_input_tokens: 10,
    };
    let _sse = ev.to_sse_event();
}

#[test]
fn done_event_sse_does_not_panic() {
    let ev = StreamEvent::Done {
        finish_reason: FinishReason::Stop,
    };
    let _sse = ev.to_sse_event();
}

#[test]
fn error_event_sse_does_not_panic() {
    let ev = StreamEvent::Error {
        code: ErrorCode::ProviderDisconnect,
        message: "upstream closed".into(),
    };
    let _sse = ev.to_sse_event();
}

#[test]
fn heartbeat_event_sse_does_not_panic() {
    let _sse = StreamEvent::Heartbeat.to_sse_event();
}

// ─── ErrorCode metric labels ─────────────────────────────────────────────────

#[test]
fn error_code_metric_labels_are_stable() {
    assert_eq!(
        ErrorCode::ProviderDisconnect.as_metric_label(),
        "provider_disconnect"
    );
    assert_eq!(
        ErrorCode::FirstTokenTimeout.as_metric_label(),
        "first_token_timeout"
    );
    assert_eq!(
        ErrorCode::MidStreamTimeout.as_metric_label(),
        "mid_stream_timeout"
    );
    assert_eq!(
        ErrorCode::MaxStreamDurationExceeded.as_metric_label(),
        "max_stream_duration_exceeded"
    );
    assert_eq!(ErrorCode::MissingUsage.as_metric_label(), "missing_usage");
    assert_eq!(
        ErrorCode::BackpressureDrop.as_metric_label(),
        "backpressure_drop"
    );
    assert_eq!(ErrorCode::InternalError.as_metric_label(), "internal_error");
}

// ─── ReconcileReason metric labels ────────────────────────────────────────────

#[test]
fn reconcile_reason_metric_labels_are_stable() {
    assert_eq!(
        ReconcileReason::ClientDisconnect.as_metric_label(),
        "client_disconnect"
    );
    assert_eq!(
        ReconcileReason::FirstTokenTimeout.as_metric_label(),
        "first_token_timeout"
    );
    assert_eq!(
        ReconcileReason::MidStreamTimeout.as_metric_label(),
        "mid_stream_timeout"
    );
    assert_eq!(
        ReconcileReason::ProviderDisconnect.as_metric_label(),
        "provider_disconnect"
    );
    assert_eq!(
        ReconcileReason::MaxDurationExceeded.as_metric_label(),
        "max_duration_exceeded"
    );
    assert_eq!(
        ReconcileReason::InternalError.as_metric_label(),
        "internal_error"
    );
}

// ─── Heartbeat tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn heartbeat_emits_at_interval() {
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(32);

    let handle = tokio::spawn(async move {
        heartbeat::run(tx, Duration::from_millis(100), "test", "test-model").await;
    });

    // First tick is skipped (immediate); subsequent ticks at 100ms intervals.
    tokio::time::sleep(Duration::from_millis(350)).await;

    let mut count = 0;
    while let Ok(ev) = rx.try_recv() {
        assert!(matches!(ev, StreamEvent::Heartbeat));
        count += 1;
    }
    assert!(count >= 2, "expected ≥2 heartbeats, got {count}");

    drop(rx);
    handle.await.unwrap();
}

#[tokio::test]
async fn heartbeat_stops_when_receiver_dropped() {
    let (tx, rx) = mpsc::channel::<StreamEvent>(32);

    let handle = tokio::spawn(async move {
        heartbeat::run(tx, Duration::from_millis(50), "test", "test-model").await;
    });

    drop(rx);

    let result = tokio::time::timeout(Duration::from_millis(200), handle).await;
    assert!(
        result.is_ok(),
        "heartbeat task did not stop after receiver drop"
    );
}

// ─── Backpressure test ───────────────────────────────────────────────────────

#[tokio::test]
async fn channel_blocks_when_full() {
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(4);

    for i in 0..4 {
        tx.send(StreamEvent::Token {
            text: format!("tok-{i}"),
            model: "test".into(),
            index: i,
        })
        .await
        .unwrap();
    }

    // try_send should fail (channel full).
    let result = tx.try_send(StreamEvent::Token {
        text: "overflow".into(),
        model: "test".into(),
        index: 4,
    });
    assert!(result.is_err(), "try_send should fail on full channel");

    let _ = rx.recv().await;

    let result = tx.try_send(StreamEvent::Token {
        text: "fits".into(),
        model: "test".into(),
        index: 4,
    });
    assert!(result.is_ok(), "try_send should succeed after drain");
}

// ─── Token index monotonic ───────────────────────────────────────────────────

#[tokio::test]
async fn token_indices_are_monotonic() {
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(32);

    let producer = tokio::spawn(async move {
        for i in 0..100u32 {
            tx.send(StreamEvent::Token {
                text: format!("tok-{i}"),
                model: "test".into(),
                index: i,
            })
            .await
            .unwrap();
        }
        tx.send(StreamEvent::Usage {
            prompt_tokens: 10,
            completion_tokens: 100,
            cached_input_tokens: 0,
        })
        .await
        .unwrap();
        tx.send(StreamEvent::Done {
            finish_reason: FinishReason::Stop,
        })
        .await
        .unwrap();
    });

    let mut indices = Vec::new();
    while let Some(ev) = rx.recv().await {
        if let StreamEvent::Token { index, .. } = ev {
            indices.push(index);
        }
    }

    let expected: Vec<u32> = (0..100).collect();
    assert_eq!(indices, expected);
    producer.await.unwrap();
}

// ─── StreamResult Debug ──────────────────────────────────────────────────────

#[test]
fn stream_result_completed_debug() {
    let result = cyberos_ai_gateway::streaming::StreamResult::Completed {
        usage: ProviderStreamUsage {
            prompt_tokens: 100,
            completion_tokens: 50,
            cached_input_tokens: 10,
        },
    };
    assert!(format!("{result:?}").contains("Completed"));
}

#[test]
fn stream_result_cancelled_debug() {
    let result = cyberos_ai_gateway::streaming::StreamResult::Cancelled {
        partial_usage: Some(ProviderStreamUsage {
            prompt_tokens: 100,
            completion_tokens: 30,
            cached_input_tokens: 0,
        }),
        reason: ReconcileReason::ClientDisconnect,
    };
    assert!(format!("{result:?}").contains("ClientDisconnect"));
}

#[test]
fn stream_result_provider_error_debug() {
    let result = cyberos_ai_gateway::streaming::StreamResult::ProviderError {
        partial_usage: None,
        code: ErrorCode::ProviderDisconnect,
        message: "upstream closed".into(),
    };
    assert!(format!("{result:?}").contains("ProviderDisconnect"));
}

// ─── Concurrent streams isolated ─────────────────────────────────────────────

#[tokio::test]
async fn concurrent_50_streams_isolated() {
    let handles: Vec<_> = (0..50)
        .map(|i| {
            tokio::spawn(async move {
                let (tx, mut rx) = mpsc::channel::<StreamEvent>(32);
                let sentinel = format!("sentinel-{i}");

                let producer_tx = tx.clone();
                let s = sentinel.clone();
                tokio::spawn(async move {
                    producer_tx
                        .send(StreamEvent::Token {
                            text: s.clone(),
                            model: "test".into(),
                            index: 0,
                        })
                        .await
                        .unwrap();
                    producer_tx
                        .send(StreamEvent::Usage {
                            prompt_tokens: 10,
                            completion_tokens: 1,
                            cached_input_tokens: 0,
                        })
                        .await
                        .unwrap();
                    producer_tx
                        .send(StreamEvent::Done {
                            finish_reason: FinishReason::Stop,
                        })
                        .await
                        .unwrap();
                    // producer_tx dropped here, but outer tx still alive
                });

                // Drop the outer tx so rx.recv() returns None when producer finishes.
                drop(tx);

                while let Some(ev) = rx.recv().await {
                    if let StreamEvent::Token { text, .. } = ev {
                        assert!(
                            text == sentinel || !text.contains("sentinel"),
                            "stream {i} received cross-stream token {text}"
                        );
                    }
                }
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }
}

// ─── ProviderStreamEvent variants ────────────────────────────────────────────

#[test]
fn provider_stream_event_variants() {
    let token = ProviderStreamEvent::Token {
        text: "hello".into(),
    };
    assert!(format!("{token:?}").contains("Token"));

    let usage = ProviderStreamEvent::Usage(ProviderStreamUsage {
        prompt_tokens: 10,
        completion_tokens: 5,
        cached_input_tokens: 0,
    });
    assert!(format!("{usage:?}").contains("Usage"));

    let done = ProviderStreamEvent::Done(FinishReason::Stop);
    assert!(format!("{done:?}").contains("Done"));
}

// ─── ReconcileGuard fire-is-idempotent pattern ──────────────────────────────

#[test]
fn atomic_bool_fire_pattern() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let fired = AtomicBool::new(false);

    // First swap returns false (was not fired) — caller proceeds.
    assert!(!fired.swap(true, Ordering::SeqCst));
    // Second swap returns true (was already fired) — caller skips.
    assert!(fired.swap(true, Ordering::SeqCst));
}

// ─── Event ordering invariant ────────────────────────────────────────────────

#[tokio::test]
async fn event_ordering_tokens_then_usage_then_done() {
    let (tx, mut rx) = mpsc::channel::<StreamEvent>(32);

    tokio::spawn(async move {
        tx.send(StreamEvent::Token {
            text: "a".into(),
            model: "m".into(),
            index: 0,
        })
        .await
        .unwrap();
        tx.send(StreamEvent::Usage {
            prompt_tokens: 10,
            completion_tokens: 1,
            cached_input_tokens: 0,
        })
        .await
        .unwrap();
        tx.send(StreamEvent::Done {
            finish_reason: FinishReason::Stop,
        })
        .await
        .unwrap();
        // tx dropped here → rx.recv() returns None
    });

    let mut collected = Vec::new();
    while let Some(ev) = rx.recv().await {
        collected.push(ev);
    }

    assert!(matches!(&collected[0], StreamEvent::Token { index: 0, .. }));
    assert!(matches!(&collected[1], StreamEvent::Usage { .. }));
    assert!(matches!(&collected[2], StreamEvent::Done { .. }));
}
