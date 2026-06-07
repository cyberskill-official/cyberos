//! FR-AI-008 — Multi-provider router with retry + failover.
//!
//! Calls the resolved LLM provider, retries on transient failures, fails over to
//! the fallback chain on persistent failures, and enforces a per-call deadline.
//!
//! See FR-AI-008 for normative behaviour and acceptance criteria.

pub mod anthropic;
pub mod bedrock;
pub mod failover;
mod http;
pub mod jitter;
mod normalize;
pub mod openai;
mod streaming;
pub mod types;

pub use types::{
    AttemptRecord, AttemptStatus, CacheState, ChatCompleteRequest, Choice, EmbedRequest,
    EmbedResponse, FinishReason, Message, ProviderResponse, ProviderStreamResponse, ProviderUsage,
    RouterError, ToolCall,
};

use std::time::{Duration, Instant};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec};
use tracing::{error, warn};

use crate::alias::ResolvedModel;
use crate::circuit_breaker::{self, CallOutcome};
use crate::policy::{ProviderKind, TenantPolicy};
pub use failover::ProviderEndpoint;

// ─── Constants ────────────────────────────────────────────────────────────────

/// Max retry attempts per provider.
const MAX_RETRIES_PER_PROVIDER: u8 = 3;

/// Total failover budget across all providers + retries.
const FAILOVER_BUDGET: Duration = Duration::from_secs(30);

/// Safety cap on attempts vec to catch infinite-loop bugs.
const ATTEMPTS_CAP: usize = 16;

/// Exponential backoff delays (ms) for attempts 2 and 3. Attempt 1 is immediate.
const RETRY_DELAYS_MS: &[u32] = &[200, 800];

/// Jitter factor (±20%).
const JITTER_FACTOR: f64 = 0.20;

/// Default per-provider timeout.
const PROVIDER_DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ─── Metrics ──────────────────────────────────────────────────────────────────

static CALLS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_router_calls_total",
        "Router calls by provider, model, and outcome",
        &["provider", "model", "outcome"]
    )
    .unwrap()
});

static RETRIES: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_router_retries_total",
        "Retries by provider and reason",
        &["provider", "reason"]
    )
    .unwrap()
});

static FAILOVERS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_router_failovers_total",
        "Failovers from one provider to another",
        &["from", "to"]
    )
    .unwrap()
});

static LATENCY_MS: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "ai_router_latency_ms",
        "Per-attempt latency in ms",
        &["provider", "model"],
        vec![50.0, 100.0, 250.0, 500.0, 1_000.0, 2_500.0, 5_000.0, 10_000.0, 30_000.0]
    )
    .unwrap()
});

static DEADLINE_EXCEEDED: Lazy<prometheus::IntCounter> = Lazy::new(|| {
    prometheus::register_int_counter!(
        "ai_router_deadline_exceeded_total",
        "Calls that hit the caller deadline"
    )
    .unwrap()
});

static ATTEMPTS_PER_CALL: Lazy<HistogramVec> = Lazy::new(|| {
    register_histogram_vec!(
        "ai_router_attempts_per_call",
        "Total attempts per call",
        &["final_outcome"],
        vec![1.0, 2.0, 3.0, 5.0, 8.0, 13.0, 16.0]
    )
    .unwrap()
});

// ─── Provider trait ───────────────────────────────────────────────────────────

/// Trait for LLM provider implementations.
///
/// Implementors handle HTTP dispatch to a specific provider API.
/// The router handles retry + failover; providers only need to
/// translate request → HTTP call → response.
#[async_trait]
pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    async fn call_chat(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderResponse, RouterError>;

    async fn call_embed(
        &self,
        req: &EmbedRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<EmbedResponse, RouterError>;

    /// Start a provider-native streaming chat call.
    async fn call_chat_streaming(
        &self,
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderStreamResponse, RouterError> {
        Err(RouterError::StreamingNotImplemented)
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Call the resolved LLM provider with retry + failover semantics.
///
/// §1 #1: Accepts (a) the ChatCompleteRequest, (b) ResolvedModel from FR-AI-006,
/// (c) a tokio Instant deadline, (d) a reference to TenantPolicy.
pub async fn call_provider(
    req: &ChatCompleteRequest,
    resolved: &ResolvedModel,
    deadline: Instant,
    policy: &TenantPolicy,
) -> Result<ProviderResponse, RouterError> {
    let (redacted_req, redactions) = crate::redact::redact_chat_request(req, policy)
        .await
        .map_err(redaction_error)?;
    let chain = failover::build_provider_chain(resolved, policy, &req.alias);
    let mut response = call_provider_with_chain(&redacted_req, deadline, chain).await?;
    restore_tool_call_arguments(&mut response, &redactions);
    Ok(response)
}

/// Test/contract entry point for exercising the router loop with injected providers.
///
/// Production callers should use [`call_provider`], which constructs this chain from
/// the resolved alias and tenant policy.
#[doc(hidden)]
pub async fn call_provider_with_chain(
    req: &ChatCompleteRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
) -> Result<ProviderResponse, RouterError> {
    let started = Instant::now();
    let failover_deadline = started + FAILOVER_BUDGET;
    let effective_deadline = deadline.min(failover_deadline);

    let mut attempts: Vec<AttemptRecord> = Vec::with_capacity(ATTEMPTS_CAP);
    let mut last_error: Option<RouterError> = None;
    let mut prev_provider_kind: Option<ProviderKind> = None;
    for endpoint in &chain {
        let pk = endpoint.provider.kind();
        let model = endpoint.model.as_str();

        // §1 #14: Emit failover counter when transitioning between providers.
        if let Some(prev) = prev_provider_kind {
            FAILOVERS
                .with_label_values(&[prev.as_metric_label(), pk.as_metric_label()])
                .inc();
        }
        prev_provider_kind = Some(pk);

        for attempt_num in 1..=MAX_RETRIES_PER_PROVIDER {
            // §1 #13: ATTEMPTS_CAP guard.
            if attempts.len() >= ATTEMPTS_CAP {
                error!(
                    attempts_len = attempts.len(),
                    "router_attempts_cap_exceeded"
                );
                return Err(RouterError::InvalidResponse {
                    reason: format!(
                        "attempts cap exceeded ({ATTEMPTS_CAP}); programmer error in failover loop"
                    ),
                });
            }

            // §1 #15: Check deadline before launching attempt.
            if Instant::now() >= effective_deadline {
                DEADLINE_EXCEEDED.inc();
                ATTEMPTS_PER_CALL
                    .with_label_values(&["deadline_exceeded"])
                    .observe(attempts.len() as f64);
                return Err(RouterError::DeadlineExceeded);
            }

            let remaining = effective_deadline
                .duration_since(Instant::now())
                .min(PROVIDER_DEFAULT_TIMEOUT);
            let call_started = Instant::now();

            // §1 #6: Propagate deadline via tokio::time::timeout.
            let outcome = tokio::time::timeout(
                remaining,
                endpoint.provider.call_chat(req, model, effective_deadline),
            )
            .await;

            let elapsed_ms = call_started.elapsed().as_millis() as u32;
            LATENCY_MS
                .with_label_values(&[pk.as_metric_label(), model])
                .observe(elapsed_ms as f64);

            match outcome {
                // Timeout
                Err(_timeout) => {
                    attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::TimeoutBeforeFirstToken,
                        elapsed_ms,
                        None,
                    ));
                    RETRIES
                        .with_label_values(&[pk.as_metric_label(), "timeout"])
                        .inc();
                    record_breaker_outcome(pk, model, CallOutcome::Timeout);
                    DEADLINE_EXCEEDED.inc();
                    ATTEMPTS_PER_CALL
                        .with_label_values(&["deadline_exceeded"])
                        .observe(attempts.len() as f64);
                    return Err(RouterError::DeadlineExceeded);
                }

                // §1 #7: 400 is terminal — no retry, no failover.
                Ok(Err(RouterError::TerminalProviderError {
                    status: 400,
                    provider: ep,
                    message,
                    ..
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::Terminal400,
                        elapsed_ms,
                        Some(400),
                    ));
                    CALLS
                        .with_label_values(&[ep.as_metric_label(), model, "terminal_4xx"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure4xx);
                    return Err(RouterError::TerminalProviderError {
                        provider: ep,
                        status: 400,
                        message,
                        retry_after_secs: None,
                    });
                }

                // §1 #9: 404 is terminal — no retry, no failover.
                Ok(Err(RouterError::TerminalProviderError {
                    status: 404,
                    provider: ep,
                    message,
                    ..
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::Terminal404,
                        elapsed_ms,
                        Some(404),
                    ));
                    CALLS
                        .with_label_values(&[ep.as_metric_label(), model, "terminal_4xx"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure4xx);
                    warn!(provider = ?ep, model = %model, "router_404_terminal_check_alias_resolver");
                    return Err(RouterError::TerminalProviderError {
                        provider: ep,
                        status: 404,
                        message,
                        retry_after_secs: None,
                    });
                }

                // §1 #8: 401/403 is terminal — sev-1 log.
                Ok(Err(RouterError::AuthError {
                    provider: ep,
                    status,
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::TerminalAuth,
                        elapsed_ms,
                        Some(status),
                    ));
                    CALLS
                        .with_label_values(&[ep.as_metric_label(), model, "auth_error"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure4xx);
                    error!(
                        provider = ?ep,
                        status = status,
                        severity = "sev-1",
                        "router_auth_error_terminal"
                    );
                    return Err(RouterError::AuthError {
                        provider: ep,
                        status,
                    });
                }

                // §1 #10: 429 — honour Retry-After if present.
                Ok(Err(RouterError::TerminalProviderError {
                    status: 429,
                    provider: ep,
                    message,
                    retry_after_secs,
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::RetriedAfter429,
                        elapsed_ms,
                        Some(429),
                    ));
                    RETRIES
                        .with_label_values(&[ep.as_metric_label(), "429"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure429);
                    last_error = Some(RouterError::TerminalProviderError {
                        provider: ep,
                        status: 429,
                        message,
                        retry_after_secs,
                    });

                    if let Some(secs) = retry_after_secs {
                        let sleep = Duration::from_secs(secs);
                        if Instant::now() + sleep > effective_deadline {
                            // Retry-After exceeds budget — fail over immediately.
                            if let Some(last) = attempts.last_mut() {
                                last.status = AttemptStatus::FailedOver;
                            }
                            break;
                        }
                        tokio::time::sleep(sleep).await;
                        continue;
                    }
                    // No Retry-After — fall through to exponential backoff.
                }

                Ok(Err(RouterError::DeadlineExceeded)) => {
                    attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::DeadlineExceededMidCall,
                        elapsed_ms,
                        None,
                    ));
                    DEADLINE_EXCEEDED.inc();
                    record_breaker_outcome(pk, model, CallOutcome::Timeout);
                    ATTEMPTS_PER_CALL
                        .with_label_values(&["deadline_exceeded"])
                        .observe(attempts.len() as f64);
                    return Err(RouterError::DeadlineExceeded);
                }

                // Other errors (5xx, conn reset, etc.)
                Ok(Err(e)) => {
                    let status_opt = match &e {
                        RouterError::TerminalProviderError { status, .. } => Some(*status),
                        _ => None,
                    };
                    attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::RetriedAfter5xx,
                        elapsed_ms,
                        status_opt,
                    ));
                    RETRIES
                        .with_label_values(&[pk.as_metric_label(), "5xx"])
                        .inc();
                    record_breaker_outcome(pk, model, breaker_outcome_for_error(&e));
                    last_error = Some(e);
                }

                // Success
                Ok(Ok(mut resp)) => {
                    resp.attempts = std::mem::take(&mut attempts);
                    resp.attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::Succeeded,
                        elapsed_ms,
                        Some(200),
                    ));
                    CALLS
                        .with_label_values(&[pk.as_metric_label(), model, "succeeded"])
                        .inc();
                    record_breaker_outcome(pk, model, CallOutcome::Success);
                    ATTEMPTS_PER_CALL
                        .with_label_values(&["succeeded"])
                        .observe(resp.attempts.len() as f64);
                    return Ok(resp);
                }
            }

            // §1 #3: Exponential backoff before next retry within same provider.
            if attempt_num < MAX_RETRIES_PER_PROVIDER {
                let base_ms = RETRY_DELAYS_MS[(attempt_num - 1) as usize];
                let sleep_ms = {
                    let mut rng = rand::thread_rng();
                    jitter::jitter_ms(base_ms, JITTER_FACTOR, &mut rng)
                };
                let sleep_dur = Duration::from_millis(sleep_ms as u64);
                if Instant::now() + sleep_dur > effective_deadline {
                    break;
                }
                tokio::time::sleep(sleep_dur).await;
            }
        }

        // §1 #4: All retries exhausted for this provider — mark last attempt as FailedOver.
        if let Some(last) = attempts.last_mut() {
            if matches!(
                last.status,
                AttemptStatus::RetriedAfter5xx
                    | AttemptStatus::RetriedAfter429
                    | AttemptStatus::TimeoutBeforeFirstToken
                    | AttemptStatus::RetriedAfterConnReset
            ) {
                last.status = AttemptStatus::FailedOver;
            }
        }
    }

    // All providers exhausted.
    CALLS
        .with_label_values(&["none", "none", "all_failed"])
        .inc();
    ATTEMPTS_PER_CALL
        .with_label_values(&["all_failed"])
        .observe(attempts.len() as f64);

    Err(RouterError::AllProvidersFailed {
        last_error: Box::new(last_error.unwrap_or(RouterError::InvalidResponse {
            reason: "no providers in chain".into(),
        })),
        attempts,
    })
}

/// Call the resolved LLM provider streaming endpoint with retry + failover.
///
/// Retries and failovers happen only while opening the provider stream. Once a
/// `ProviderStreamResponse` is returned, downstream token delivery is committed
/// to that stream and provider errors are surfaced to the caller as stream items.
pub async fn call_provider_streaming(
    req: &ChatCompleteRequest,
    resolved: &ResolvedModel,
    deadline: Instant,
    policy: &TenantPolicy,
) -> Result<ProviderStreamResponse, RouterError> {
    let (redacted_req, _redactions) = crate::redact::redact_chat_request(req, policy)
        .await
        .map_err(redaction_error)?;
    let chain = failover::build_provider_chain(resolved, policy, &req.alias);
    call_provider_streaming_with_chain(&redacted_req, deadline, chain).await
}

/// Test/contract entry point for exercising streaming retry/failover with injected providers.
#[doc(hidden)]
pub async fn call_provider_streaming_with_chain(
    req: &ChatCompleteRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
) -> Result<ProviderStreamResponse, RouterError> {
    let started = Instant::now();
    let failover_deadline = started + FAILOVER_BUDGET;
    let effective_deadline = deadline.min(failover_deadline);

    let mut attempts: Vec<AttemptRecord> = Vec::with_capacity(ATTEMPTS_CAP);
    let mut last_error: Option<RouterError> = None;
    let mut prev_provider_kind: Option<ProviderKind> = None;
    for endpoint in &chain {
        let pk = endpoint.provider.kind();
        let model = endpoint.model.as_str();

        if let Some(prev) = prev_provider_kind {
            FAILOVERS
                .with_label_values(&[prev.as_metric_label(), pk.as_metric_label()])
                .inc();
        }
        prev_provider_kind = Some(pk);

        for attempt_num in 1..=MAX_RETRIES_PER_PROVIDER {
            if attempts.len() >= ATTEMPTS_CAP {
                error!(
                    attempts_len = attempts.len(),
                    "router_streaming_attempts_cap_exceeded"
                );
                return Err(RouterError::InvalidResponse {
                    reason: format!(
                        "attempts cap exceeded ({ATTEMPTS_CAP}); programmer error in streaming failover loop"
                    ),
                });
            }

            if Instant::now() >= effective_deadline {
                DEADLINE_EXCEEDED.inc();
                ATTEMPTS_PER_CALL
                    .with_label_values(&["streaming_deadline_exceeded"])
                    .observe(attempts.len() as f64);
                return Err(RouterError::DeadlineExceeded);
            }

            let remaining = effective_deadline
                .duration_since(Instant::now())
                .min(PROVIDER_DEFAULT_TIMEOUT);
            let call_started = Instant::now();
            let outcome = tokio::time::timeout(
                remaining,
                endpoint
                    .provider
                    .call_chat_streaming(req, model, effective_deadline),
            )
            .await;

            let elapsed_ms = call_started.elapsed().as_millis() as u32;
            LATENCY_MS
                .with_label_values(&[pk.as_metric_label(), model])
                .observe(elapsed_ms as f64);

            match outcome {
                Err(_timeout) => {
                    attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::TimeoutBeforeFirstToken,
                        elapsed_ms,
                        None,
                    ));
                    RETRIES
                        .with_label_values(&[pk.as_metric_label(), "streaming_timeout"])
                        .inc();
                    record_breaker_outcome(pk, model, CallOutcome::Timeout);
                    DEADLINE_EXCEEDED.inc();
                    ATTEMPTS_PER_CALL
                        .with_label_values(&["streaming_deadline_exceeded"])
                        .observe(attempts.len() as f64);
                    return Err(RouterError::DeadlineExceeded);
                }

                Ok(Err(RouterError::TerminalProviderError {
                    status: 400,
                    provider: ep,
                    message,
                    ..
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::Terminal400,
                        elapsed_ms,
                        Some(400),
                    ));
                    CALLS
                        .with_label_values(&[ep.as_metric_label(), model, "streaming_terminal_4xx"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure4xx);
                    return Err(RouterError::TerminalProviderError {
                        provider: ep,
                        status: 400,
                        message,
                        retry_after_secs: None,
                    });
                }

                Ok(Err(RouterError::TerminalProviderError {
                    status: 404,
                    provider: ep,
                    message,
                    ..
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::Terminal404,
                        elapsed_ms,
                        Some(404),
                    ));
                    CALLS
                        .with_label_values(&[ep.as_metric_label(), model, "streaming_terminal_4xx"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure4xx);
                    warn!(provider = ?ep, model = %model, "router_streaming_404_terminal_check_alias_resolver");
                    return Err(RouterError::TerminalProviderError {
                        provider: ep,
                        status: 404,
                        message,
                        retry_after_secs: None,
                    });
                }

                Ok(Err(RouterError::AuthError {
                    provider: ep,
                    status,
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::TerminalAuth,
                        elapsed_ms,
                        Some(status),
                    ));
                    CALLS
                        .with_label_values(&[ep.as_metric_label(), model, "streaming_auth_error"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure4xx);
                    error!(
                        provider = ?ep,
                        status = status,
                        severity = "sev-1",
                        "router_streaming_auth_error_terminal"
                    );
                    return Err(RouterError::AuthError {
                        provider: ep,
                        status,
                    });
                }

                Ok(Err(RouterError::TerminalProviderError {
                    status: 429,
                    provider: ep,
                    message,
                    retry_after_secs,
                })) => {
                    attempts.push(make_record(
                        ep,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::RetriedAfter429,
                        elapsed_ms,
                        Some(429),
                    ));
                    RETRIES
                        .with_label_values(&[ep.as_metric_label(), "streaming_429"])
                        .inc();
                    record_breaker_outcome(ep, model, CallOutcome::Failure429);
                    last_error = Some(RouterError::TerminalProviderError {
                        provider: ep,
                        status: 429,
                        message,
                        retry_after_secs,
                    });

                    if let Some(secs) = retry_after_secs {
                        let sleep = Duration::from_secs(secs);
                        if Instant::now() + sleep > effective_deadline {
                            if let Some(last) = attempts.last_mut() {
                                last.status = AttemptStatus::FailedOver;
                            }
                            break;
                        }
                        tokio::time::sleep(sleep).await;
                        continue;
                    }
                }

                Ok(Err(RouterError::DeadlineExceeded)) => {
                    attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::DeadlineExceededMidCall,
                        elapsed_ms,
                        None,
                    ));
                    DEADLINE_EXCEEDED.inc();
                    record_breaker_outcome(pk, model, CallOutcome::Timeout);
                    ATTEMPTS_PER_CALL
                        .with_label_values(&["streaming_deadline_exceeded"])
                        .observe(attempts.len() as f64);
                    return Err(RouterError::DeadlineExceeded);
                }

                Ok(Err(e)) => {
                    let status_opt = match &e {
                        RouterError::TerminalProviderError { status, .. } => Some(*status),
                        _ => None,
                    };
                    attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::RetriedAfter5xx,
                        elapsed_ms,
                        status_opt,
                    ));
                    RETRIES
                        .with_label_values(&[pk.as_metric_label(), "streaming_5xx"])
                        .inc();
                    record_breaker_outcome(pk, model, breaker_outcome_for_error(&e));
                    last_error = Some(e);
                }

                Ok(Ok(resp)) => {
                    let mut final_attempts = std::mem::take(&mut attempts);
                    final_attempts.push(make_record(
                        pk,
                        model,
                        attempt_num,
                        endpoint.fallback_position,
                        AttemptStatus::Succeeded,
                        elapsed_ms,
                        Some(200),
                    ));
                    CALLS
                        .with_label_values(&[pk.as_metric_label(), model, "streaming_succeeded"])
                        .inc();
                    record_breaker_outcome(pk, model, CallOutcome::Success);
                    ATTEMPTS_PER_CALL
                        .with_label_values(&["streaming_succeeded"])
                        .observe(final_attempts.len() as f64);
                    return Ok(resp.with_attempts(final_attempts));
                }
            }

            if attempt_num < MAX_RETRIES_PER_PROVIDER {
                let base_ms = RETRY_DELAYS_MS[(attempt_num - 1) as usize];
                let sleep_ms = {
                    let mut rng = rand::thread_rng();
                    jitter::jitter_ms(base_ms, JITTER_FACTOR, &mut rng)
                };
                let sleep_dur = Duration::from_millis(sleep_ms as u64);
                if Instant::now() + sleep_dur > effective_deadline {
                    break;
                }
                tokio::time::sleep(sleep_dur).await;
            }
        }

        if let Some(last) = attempts.last_mut() {
            if matches!(
                last.status,
                AttemptStatus::RetriedAfter5xx
                    | AttemptStatus::RetriedAfter429
                    | AttemptStatus::TimeoutBeforeFirstToken
                    | AttemptStatus::RetriedAfterConnReset
            ) {
                last.status = AttemptStatus::FailedOver;
            }
        }
    }

    CALLS
        .with_label_values(&["none", "none", "streaming_all_failed"])
        .inc();
    ATTEMPTS_PER_CALL
        .with_label_values(&["streaming_all_failed"])
        .observe(attempts.len() as f64);

    Err(RouterError::AllProvidersFailed {
        last_error: Box::new(last_error.unwrap_or(RouterError::InvalidResponse {
            reason: "no providers in streaming chain".into(),
        })),
        attempts,
    })
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_record(
    provider: ProviderKind,
    model: &str,
    attempt_num: u8,
    fallback_position: u8,
    status: AttemptStatus,
    elapsed_ms: u32,
    http_status: Option<u16>,
) -> AttemptRecord {
    AttemptRecord {
        provider,
        model: model.to_string(),
        attempt_num,
        fallback_position,
        status,
        elapsed_ms,
        http_status,
    }
}

fn breaker_outcome_for_error(error: &RouterError) -> CallOutcome {
    match error {
        RouterError::TerminalProviderError { status: 429, .. } => CallOutcome::Failure429,
        RouterError::TerminalProviderError { status, .. } if *status >= 500 => {
            CallOutcome::Failure5xx
        }
        RouterError::DeadlineExceeded => CallOutcome::Timeout,
        RouterError::AuthError { .. } | RouterError::TerminalProviderError { .. } => {
            CallOutcome::Failure4xx
        }
        RouterError::SerializationError { .. }
        | RouterError::InvalidResponse { .. }
        | RouterError::RedactionFailed { .. }
        | RouterError::AllProvidersFailed { .. }
        | RouterError::StreamingNotImplemented => CallOutcome::Failure5xx,
    }
}

fn record_breaker_outcome(provider: ProviderKind, model: &str, outcome: CallOutcome) {
    circuit_breaker::record_outcome(&provider, model, outcome);
}

fn redaction_error(error: crate::redact::RedactError) -> RouterError {
    RouterError::RedactionFailed {
        reason: error.to_string(),
    }
}

fn restore_tool_call_arguments(
    response: &mut ProviderResponse,
    redactions: &[crate::redact::RedactionResult],
) {
    for choice in &mut response.choices {
        for tool_call in &mut choice.tool_calls {
            let mut restored = tool_call.arguments.clone();
            for redaction in redactions {
                restored = crate::redact::restore(&restored, &redaction.map);
            }
            tool_call.arguments = restored;
        }
    }
}
