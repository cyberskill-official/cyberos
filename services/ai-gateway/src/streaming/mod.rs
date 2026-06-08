//! FR-AI-010 — Streaming SSE end-to-end (token-by-token to client).
//!
//! Provides `handle_streaming_chat` which returns an SSE stream for chat completions.
//! The actual provider work runs in a spawned task; the HTTP handler returns immediately.
//!
//! See FR-AI-010 for normative behaviour and acceptance criteria.

pub mod heartbeat;
pub mod sse;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::response::sse::{Event as SseEvent, Sse};
use futures::StreamExt;
use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, HistogramVec, IntCounter,
};
use tokio::sync::{mpsc, watch, Mutex};
use tokio_stream::wrappers::ReceiverStream;

use crate::alias::ResolvedModel;
use crate::cost_ledger;
use crate::policy::{ProviderKind, TenantPolicy};
use crate::router;

// ─── Constants ────────────────────────────────────────────────────────────────

const CHANNEL_CAPACITY: usize = 32;
const FIRST_TOKEN_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_STREAM_DURATION: Duration = Duration::from_secs(300);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);
const ABORT_TIMEOUT: Duration = Duration::from_millis(200);

// ─── Metrics ──────────────────────────────────────────────────────────────────

mod metrics {
    use super::*;

    pub static FIRST_TOKEN_MS: Lazy<HistogramVec> = Lazy::new(|| {
        register_histogram_vec!(
            "ai_streaming_first_token_ms",
            "Time from request acceptance to first token streamed to client",
            &["provider", "model"],
            vec![100.0, 250.0, 500.0, 1_000.0, 1_500.0, 2_000.0, 5_000.0]
        )
        .unwrap()
    });

    pub static TOTAL_DURATION_MS: Lazy<HistogramVec> = Lazy::new(|| {
        register_histogram_vec!(
            "ai_streaming_total_duration_ms",
            "Total stream duration",
            &["provider", "model", "outcome"],
            vec![500.0, 1_000.0, 5_000.0, 30_000.0, 60_000.0, 300_000.0]
        )
        .unwrap()
    });

    pub static DISCONNECTS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_streaming_disconnects_total",
            "Client disconnects by phase",
            &["provider", "model", "phase"]
        )
        .unwrap()
    });

    pub static MID_STREAM_ERRORS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_streaming_provider_errors_mid_stream_total",
            "Provider errors after first token",
            &["provider", "model"]
        )
        .unwrap()
    });

    pub static BACKPRESSURE: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_streaming_backpressure_events_total",
            "Per-send blocking events due to slow client",
            &["provider", "model"]
        )
        .unwrap()
    });

    pub static HEARTBEATS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_streaming_heartbeats_emitted_total",
            "Heartbeat events emitted",
            &["provider", "model"]
        )
        .unwrap()
    });

    pub static UNSUPPORTED_FALLBACK: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_streaming_unsupported_fallback_total",
            "Streams that fell back to non-streaming",
            &["model"]
        )
        .unwrap()
    });

    pub static RECONCILES: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_streaming_reconciles_total",
            "Reconciles per stream (sanity check: exactly one fires)",
            &["outcome"]
        )
        .unwrap()
    });

    pub static DROP_OUTSIDE_RUNTIME: Lazy<IntCounter> = Lazy::new(|| {
        prometheus::register_int_counter!(
            "ai_streaming_drop_outside_runtime_total",
            "ReconcileGuard::Drop fired without an available tokio runtime"
        )
        .unwrap()
    });
}

// ─── Public types ─────────────────────────────────────────────────────────────

/// Events yielded by the provider's streaming response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderStreamEvent {
    Token { text: String },
    Usage(ProviderStreamUsage),
    Done(router::FinishReason),
}

/// Usage reported by the provider mid-stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderStreamUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub cached_input_tokens: u32,
}

/// Events sent to the SSE client.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamEvent {
    Token {
        text: String,
        model: String,
        index: u32,
    },
    Usage {
        prompt_tokens: u32,
        completion_tokens: u32,
        cached_input_tokens: u32,
    },
    Done {
        finish_reason: router::FinishReason,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
    Heartbeat,
}

/// Structured error codes for `event: error`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    ProviderDisconnect,
    FirstTokenTimeout,
    MidStreamTimeout,
    MaxStreamDurationExceeded,
    MissingUsage,
    BackpressureDrop,
    InternalError,
}

impl ErrorCode {
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::ProviderDisconnect => "provider_disconnect",
            Self::FirstTokenTimeout => "first_token_timeout",
            Self::MidStreamTimeout => "mid_stream_timeout",
            Self::MaxStreamDurationExceeded => "max_stream_duration_exceeded",
            Self::MissingUsage => "missing_usage",
            Self::BackpressureDrop => "backpressure_drop",
            Self::InternalError => "internal_error",
        }
    }
}

/// Outcome of a stream, consumed by `ReconcileGuard::fire()`.
#[derive(Debug)]
pub enum StreamResult {
    Completed {
        usage: ProviderStreamUsage,
    },
    Cancelled {
        partial_usage: Option<ProviderStreamUsage>,
        reason: ReconcileReason,
    },
    ProviderError {
        partial_usage: Option<ProviderStreamUsage>,
        code: ErrorCode,
        message: String,
    },
}

/// Reason for cancellation (maps to `cost_reconcile::CancelReason`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileReason {
    ClientDisconnect,
    FirstTokenTimeout,
    MidStreamTimeout,
    ProviderDisconnect,
    MaxDurationExceeded,
    InternalError,
}

impl ReconcileReason {
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::ClientDisconnect => "client_disconnect",
            Self::FirstTokenTimeout => "first_token_timeout",
            Self::MidStreamTimeout => "mid_stream_timeout",
            Self::ProviderDisconnect => "provider_disconnect",
            Self::MaxDurationExceeded => "max_duration_exceeded",
            Self::InternalError => "internal_error",
        }
    }
}

/// Error returned by `handle_streaming_chat`.
#[derive(Debug)]
pub enum StreamingHandlerError {
    PrecheckFailed { reason: String, http_status: u16 },
    UnsupportedFallback { model: String },
}

// ─── ReconcileGuard (RAII) ────────────────────────────────────────────────────

/// RAII guard ensuring exactly-one reconcile call per stream.
///
/// `fire()` must be called before Drop in the happy path. If the spawned task
/// panics or the guard is dropped without `fire()`, `Drop` performs the reconcile.
pub struct ReconcileGuard {
    hold_id: uuid::Uuid,
    pool: sqlx::PgPool,
    outcome: Mutex<Option<StreamResult>>,
    fired: AtomicBool,
}

impl std::fmt::Debug for ReconcileGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconcileGuard")
            .field("hold_id", &self.hold_id)
            .field("fired", &self.fired.load(Ordering::SeqCst))
            .finish()
    }
}

impl ReconcileGuard {
    pub fn new(hold_id: uuid::Uuid, pool: sqlx::PgPool) -> Self {
        Self {
            hold_id,
            pool,
            outcome: Mutex::new(None),
            fired: AtomicBool::new(false),
        }
    }

    /// Store the stream outcome. Must be called before `fire()`.
    pub async fn record(&self, outcome: StreamResult) {
        *self.outcome.lock().await = Some(outcome);
    }

    /// Fire the reconcile. Idempotent — second call is a no-op.
    pub async fn fire(&self) {
        if self.fired.swap(true, Ordering::SeqCst) {
            return;
        }
        let outcome = self
            .outcome
            .lock()
            .await
            .take()
            .unwrap_or(StreamResult::Cancelled {
                partial_usage: None,
                reason: ReconcileReason::InternalError,
            });
        let outcome_label = match &outcome {
            StreamResult::Completed { .. } => "success",
            StreamResult::Cancelled { reason, .. } => reason.as_metric_label(),
            StreamResult::ProviderError { .. } => "provider_error",
        };
        metrics::RECONCILES
            .with_label_values(&[outcome_label])
            .inc();
        let call_outcome = stream_result_to_call_outcome(outcome);
        let _ = crate::cost_reconcile::reconcile(self.hold_id, call_outcome, &self.pool).await;
    }
}

impl Drop for ReconcileGuard {
    fn drop(&mut self) {
        if self.fired.load(Ordering::SeqCst) {
            return;
        }
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let hold = self.hold_id;
            let pool = self.pool.clone();
            handle.spawn(async move {
                metrics::RECONCILES
                    .with_label_values(&["panic_recovery"])
                    .inc();
                let outcome = crate::cost_reconcile::CallOutcome::Cancelled {
                    partial_usage: None,
                    reason: crate::cost_reconcile::CancelReason::GatewayShutdown,
                };
                let _ = crate::cost_reconcile::reconcile(hold, outcome, &pool).await;
            });
        } else {
            metrics::DROP_OUTSIDE_RUNTIME.inc();
            tracing::error!(
                hold_id = ?self.hold_id,
                severity = "sev-2",
                "reconcile_guard_drop_outside_runtime; FR-AI-001 cleanup job will sweep within 60s"
            );
        }
    }
}

// ─── Provider streaming support table ─────────────────────────────────────────

/// Returns `true` if the provider supports SSE streaming.
fn provider_supports_streaming(kind: ProviderKind) -> bool {
    match kind {
        ProviderKind::Bedrock | ProviderKind::Anthropic | ProviderKind::Openai => true,
        ProviderKind::Vertex => true,
        ProviderKind::Bge => false,
    }
}

// ─── Main entry point ─────────────────────────────────────────────────────────

/// Handle an SSE chat completion request.
///
/// Returns an SSE stream. The actual provider work runs in a spawned task.
/// The stream yields `event: token`, `event: heartbeat`, `event: usage`,
/// `event: done`, or `event: error` in the canonical order specified in FR-AI-010 §1.
pub async fn handle_streaming_chat(
    req: router::ChatCompleteRequest,
    pool: sqlx::PgPool,
    policy: Arc<TenantPolicy>,
) -> Result<
    Sse<impl futures::Stream<Item = Result<SseEvent, std::convert::Infallible>>>,
    StreamingHandlerError,
> {
    // Step 1: Resolve alias and check streaming support.
    let resolved = crate::alias::resolve(&req.alias, &policy).map_err(|e| match &e {
        crate::alias::AliasError::ZdrViolation { .. } => StreamingHandlerError::PrecheckFailed {
            reason: format!("{e:?}"),
            http_status: 403,
        },
        crate::alias::AliasError::ResidencyViolation {
            resolved_region,
            policy_residency,
            vn1_no_provider,
            ..
        } => {
            let mut body = serde_json::json!({
                "error": "residency_violation",
                "policy_residency": crate::residency::residency_label(*policy_residency),
                "resolved_region": resolved_region,
                "contact": "ops@cyberos.world",
            });
            if *vn1_no_provider {
                body["reason"] = serde_json::json!("no_vn_provider_yet");
            }
            StreamingHandlerError::PrecheckFailed {
                reason: body.to_string(),
                http_status: 403,
            }
        }
        _ => StreamingHandlerError::PrecheckFailed {
            reason: format!("{e:?}"),
            http_status: 400,
        },
    })?;
    if !provider_supports_streaming(resolved.provider_kind) {
        metrics::UNSUPPORTED_FALLBACK
            .with_label_values(&[&resolved.model])
            .inc();
        return Err(StreamingHandlerError::UnsupportedFallback {
            model: resolved.model,
        });
    }

    // Step 2: Synchronous precheck (convert router request to cost_ledger request).
    let precheck_req = cost_ledger::ChatCompleteRequest {
        tenant_id: policy.tenant_id.clone(),
        agent_persona: String::new(), // not available at stream layer; empty
        model_alias: req.alias.clone(),
        prompt_tokens: 0, // unknown until provider responds; precheck uses estimate
        expected_completion_tokens: 1024, // estimate; real impl derives from model
        idempotency_key: req.tracestate.clone().unwrap_or_default(), // placeholder; real impl uses proper key
    };
    let hold_id = match cost_ledger::precheck(&precheck_req, &pool, &policy).await {
        Ok(cost_ledger::PrecheckOutcome::Allow { hold_id, .. }) => hold_id,
        Ok(cost_ledger::PrecheckOutcome::Refuse { reason, .. }) => {
            return Err(StreamingHandlerError::PrecheckFailed {
                reason: format!("{reason:?}"),
                http_status: 429,
            });
        }
        Err(e) => {
            return Err(StreamingHandlerError::PrecheckFailed {
                reason: format!("{e:?}"),
                http_status: 500,
            });
        }
    };

    // Step 3: Channel + disconnect signal + reconcile guard.
    let (tx, rx) = mpsc::channel::<StreamEvent>(CHANNEL_CAPACITY);
    let (disconnect_tx, disconnect_rx) = watch::channel(false);
    let (stream_done_tx, stream_done_rx) = watch::channel(false);
    let guard = Arc::new(ReconcileGuard::new(hold_id, pool.clone()));
    let deadline =
        Instant::now() + Duration::from_secs(policy.ai_policy.call_timeout_seconds as u64);

    // Step 4: Spawn provider task.
    let guard_for_task = guard.clone();
    let req_for_task = req.clone();
    let resolved_for_task = resolved.clone();
    let policy_for_task = policy.clone();
    let tx_for_task = tx.clone();
    let disconnect_rx_for_task = disconnect_rx.clone();

    tokio::spawn(async move {
        let result = run_provider_stream(
            req_for_task,
            resolved_for_task,
            deadline,
            policy_for_task,
            tx_for_task,
            disconnect_rx_for_task,
        )
        .await;
        let _ = stream_done_tx.send(true);
        guard_for_task.record(result).await;
        guard_for_task.fire().await;
    });

    // Step 5: Heartbeat task (cancels when tx drops).
    let tx_for_hb = tx.clone();
    let provider_label = resolved.provider_kind.as_metric_label().to_owned();
    let model_label = resolved.model.clone();
    tokio::spawn(async move {
        heartbeat::run_until_done(
            tx_for_hb,
            HEARTBEAT_INTERVAL,
            &provider_label,
            &model_label,
            stream_done_rx,
        )
        .await;
    });

    // Step 6: Wire disconnect signal on SSE stream drop.
    // The disconnect_tx is dropped when this function returns, but we need it
    // to signal when the SSE response is dropped. We wrap the stream to hold it.
    let stream =
        ReceiverStream::new(rx).map(move |ev| Ok::<_, std::convert::Infallible>(ev.to_sse_event()));
    let stream = DisconnectAwareStream {
        inner: stream,
        disconnect_tx,
    };

    // Step 7: Return SSE.
    Ok(Sse::new(stream))
}

// ─── Provider stream runner ───────────────────────────────────────────────────

async fn run_provider_stream(
    req: router::ChatCompleteRequest,
    resolved: ResolvedModel,
    deadline: Instant,
    policy: Arc<TenantPolicy>,
    tx: mpsc::Sender<StreamEvent>,
    mut disconnect_rx: watch::Receiver<bool>,
) -> StreamResult {
    let started = Instant::now();
    let max_duration_deadline = started + MAX_STREAM_DURATION;
    let effective_deadline = deadline.min(max_duration_deadline);
    let provider_label = resolved.provider_kind.as_metric_label();
    let model_label = &resolved.model;

    // Call provider streaming (pre-first-token retries handled inside).
    let stream_response =
        match router::call_provider_streaming(&req, &resolved, effective_deadline, &policy).await {
            Ok(s) => s,
            Err(e) => {
                let _ = tx
                    .send(StreamEvent::Error {
                        code: ErrorCode::FirstTokenTimeout,
                        message: format!("{e:?}"),
                    })
                    .await;
                return StreamResult::Cancelled {
                    partial_usage: None,
                    reason: ReconcileReason::FirstTokenTimeout,
                };
            }
        };

    let mut provider_stream = provider_stream_from_response(stream_response);

    let mut first_token_at: Option<Instant> = None;
    let mut token_index: u32 = 0;
    let mut last_usage: Option<ProviderStreamUsage> = None;
    let mut got_done = false;

    loop {
        let remaining = effective_deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            let _ = tx
                .send(StreamEvent::Error {
                    code: ErrorCode::MaxStreamDurationExceeded,
                    message: "stream exceeded 300s".into(),
                })
                .await;
            return StreamResult::Cancelled {
                partial_usage: last_usage,
                reason: ReconcileReason::MaxDurationExceeded,
            };
        }

        // First iteration uses FIRST_TOKEN_TIMEOUT; subsequent use remaining.
        // Cap each select at ABORT_TIMEOUT so disconnect is checked every 200ms.
        let iter_timeout = if first_token_at.is_none() {
            FIRST_TOKEN_TIMEOUT.min(remaining)
        } else {
            remaining
        };
        let select_timeout = iter_timeout.min(ABORT_TIMEOUT);

        let next = tokio::select! {
            biased;
            _ = disconnect_rx.changed() => {
                if *disconnect_rx.borrow() {
                    let phase = if first_token_at.is_some() { "after_first_token" } else { "before_first_token" };
                    metrics::DISCONNECTS.with_label_values(&[provider_label, model_label, phase]).inc();
                    return StreamResult::Cancelled {
                        partial_usage: last_usage,
                        reason: ReconcileReason::ClientDisconnect,
                    };
                }
                continue;
            }
            result = tokio::time::timeout(select_timeout, provider_stream.next()) => result,
        };

        match next {
            Err(_timeout) => {
                let code = if first_token_at.is_none() {
                    ErrorCode::FirstTokenTimeout
                } else {
                    ErrorCode::MidStreamTimeout
                };
                let reason = if first_token_at.is_none() {
                    ReconcileReason::FirstTokenTimeout
                } else {
                    ReconcileReason::MidStreamTimeout
                };
                let _ = tx
                    .send(StreamEvent::Error {
                        code,
                        message: format!("timeout after {iter_timeout:?}"),
                    })
                    .await;
                return StreamResult::Cancelled {
                    partial_usage: last_usage,
                    reason,
                };
            }
            Ok(None) => {
                // Provider stream ended.
                if !got_done || last_usage.is_none() {
                    let _ = tx
                        .send(StreamEvent::Error {
                            code: ErrorCode::MissingUsage,
                            message: "stream ended without usage event".into(),
                        })
                        .await;
                    return StreamResult::ProviderError {
                        partial_usage: last_usage,
                        code: ErrorCode::MissingUsage,
                        message: "missing usage".into(),
                    };
                }
                let usage = last_usage.expect("checked above");
                let elapsed = started.elapsed().as_millis() as f64;
                metrics::TOTAL_DURATION_MS
                    .with_label_values(&[provider_label, model_label, "success"])
                    .observe(elapsed);
                return StreamResult::Completed { usage };
            }
            Ok(Some(Err(e))) => {
                // Provider error mid-stream — no retry after first token.
                if first_token_at.is_some() {
                    metrics::MID_STREAM_ERRORS
                        .with_label_values(&[provider_label, model_label])
                        .inc();
                }
                let _ = tx
                    .send(StreamEvent::Error {
                        code: ErrorCode::ProviderDisconnect,
                        message: format!("{e:?}"),
                    })
                    .await;
                return StreamResult::ProviderError {
                    partial_usage: last_usage,
                    code: ErrorCode::ProviderDisconnect,
                    message: format!("{e:?}"),
                };
            }
            Ok(Some(Ok(provider_event))) => match provider_event {
                ProviderStreamEvent::Token { text } => {
                    if first_token_at.is_none() {
                        let elapsed_ms = started.elapsed().as_millis() as f64;
                        metrics::FIRST_TOKEN_MS
                            .with_label_values(&[provider_label, model_label])
                            .observe(elapsed_ms);
                        first_token_at = Some(Instant::now());
                    }
                    // Try non-blocking first; on full channel, block (backpressure).
                    if tx
                        .try_send(StreamEvent::Token {
                            text: text.clone(),
                            model: resolved.model.clone(),
                            index: token_index,
                        })
                        .is_err()
                    {
                        metrics::BACKPRESSURE
                            .with_label_values(&[provider_label, model_label])
                            .inc();
                        if tx
                            .send(StreamEvent::Token {
                                text,
                                model: resolved.model.clone(),
                                index: token_index,
                            })
                            .await
                            .is_err()
                        {
                            let phase = if first_token_at.is_some() {
                                "after_first_token"
                            } else {
                                "before_first_token"
                            };
                            metrics::DISCONNECTS
                                .with_label_values(&[provider_label, model_label, phase])
                                .inc();
                            return StreamResult::Cancelled {
                                partial_usage: last_usage,
                                reason: ReconcileReason::ClientDisconnect,
                            };
                        }
                    }
                    token_index += 1;
                }
                ProviderStreamEvent::Usage(usage) => {
                    last_usage = Some(usage);
                    if tx
                        .send(StreamEvent::Usage {
                            prompt_tokens: usage.prompt_tokens,
                            completion_tokens: usage.completion_tokens,
                            cached_input_tokens: usage.cached_input_tokens,
                        })
                        .await
                        .is_err()
                    {
                        metrics::DISCONNECTS
                            .with_label_values(&[provider_label, model_label, "after_first_token"])
                            .inc();
                        return StreamResult::Cancelled {
                            partial_usage: last_usage,
                            reason: ReconcileReason::ClientDisconnect,
                        };
                    }
                }
                ProviderStreamEvent::Done(finish_reason) => {
                    if tx.send(StreamEvent::Done { finish_reason }).await.is_err() {
                        metrics::DISCONNECTS
                            .with_label_values(&[provider_label, model_label, "after_first_token"])
                            .inc();
                        return StreamResult::Cancelled {
                            partial_usage: last_usage,
                            reason: ReconcileReason::ClientDisconnect,
                        };
                    }
                    got_done = true;
                }
            },
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn provider_stream_from_response(
    resp: router::ProviderStreamResponse,
) -> impl futures::Stream<Item = Result<ProviderStreamEvent, router::RouterError>> {
    resp.into_events()
}

/// Convert `StreamResult` to `cost_reconcile::CallOutcome`.
fn stream_result_to_call_outcome(result: StreamResult) -> crate::cost_reconcile::CallOutcome {
    match result {
        StreamResult::Completed { usage } => crate::cost_reconcile::CallOutcome::Success {
            usage: crate::cost_reconcile::ProviderUsage {
                prompt_tokens: usage.prompt_tokens,
                completion_tokens: usage.completion_tokens,
            },
            latency_ms: 0, // not tracked at stream level
            cache_state: crate::cost_reconcile::CacheState::Miss,
            provider_request_id: String::new(),
        },
        StreamResult::Cancelled {
            partial_usage,
            reason,
        } => crate::cost_reconcile::CallOutcome::Cancelled {
            partial_usage: partial_usage.map(|u| crate::cost_reconcile::ProviderUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
            }),
            reason: match reason {
                ReconcileReason::ClientDisconnect => {
                    crate::cost_reconcile::CancelReason::ClientDisconnect
                }
                ReconcileReason::FirstTokenTimeout => {
                    crate::cost_reconcile::CancelReason::TimeoutBeforeFirstToken
                }
                ReconcileReason::MidStreamTimeout => {
                    crate::cost_reconcile::CancelReason::TimeoutMidStream
                }
                ReconcileReason::ProviderDisconnect => {
                    crate::cost_reconcile::CancelReason::ClientDisconnect
                }
                ReconcileReason::MaxDurationExceeded => {
                    crate::cost_reconcile::CancelReason::TimeoutMidStream
                }
                ReconcileReason::InternalError => {
                    crate::cost_reconcile::CancelReason::GatewayShutdown
                }
            },
        },
        StreamResult::ProviderError {
            partial_usage,
            code,
            message,
        } => {
            // ProviderError maps to Cancelled with partial_usage if we have some,
            // or ProviderError if we don't.
            if let Some(usage) = partial_usage {
                crate::cost_reconcile::CallOutcome::Cancelled {
                    partial_usage: Some(crate::cost_reconcile::ProviderUsage {
                        prompt_tokens: usage.prompt_tokens,
                        completion_tokens: usage.completion_tokens,
                    }),
                    reason: crate::cost_reconcile::CancelReason::ClientDisconnect,
                }
            } else {
                crate::cost_reconcile::CallOutcome::ProviderError {
                    http_status: 502,
                    retryable: false,
                    provider_error_message: format!("{code:?}: {message}"),
                }
            }
        }
    }
}

/// Wrapper stream that signals disconnect when dropped.
struct DisconnectAwareStream<S> {
    inner: S,
    disconnect_tx: watch::Sender<bool>,
}

impl<S> Drop for DisconnectAwareStream<S> {
    fn drop(&mut self) {
        let _ = self.disconnect_tx.send(true);
    }
}

impl<S, T, E> futures::Stream for DisconnectAwareStream<S>
where
    S: futures::Stream<Item = Result<T, E>> + Unpin,
{
    type Item = Result<T, E>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.inner).poll_next(cx)
    }
}
