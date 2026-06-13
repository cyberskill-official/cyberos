//! FR-AI-008 — Multi-provider router with retry + failover.
//!
//! Calls the resolved LLM provider, retries on transient failures, fails over to
//! the fallback chain on persistent failures, and enforces a per-call deadline.
//!
//! See FR-AI-008 for normative behaviour and acceptance criteria.

pub mod anthropic;
pub mod bedrock;
pub mod bge_batch_buffer;
pub mod bge_provider;
pub mod failover;
mod http;
pub mod jitter;
mod normalize;
pub mod openai;
mod streaming;
pub mod types;

pub use types::{
    AttemptRecord, AttemptStatus, CacheState, ChatCompleteRequest, Choice, EmbedRequest,
    EmbedResponse, EmbedTask, FinishReason, MadeByGenie, Message, ProviderResponse,
    ProviderStreamResponse, ProviderUsage, RouterError, ToolCall,
};

use std::time::{Duration, Instant};

use async_trait::async_trait;
use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram_vec, CounterVec, HistogramVec};
use tracing::{error, warn};

use crate::alias::ResolvedModel;
use crate::circuit_breaker::{self, CallOutcome};
use crate::otel::{attributes as otel_attributes, spans as otel_spans};
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
    let request_id = uuid::Uuid::new_v4().to_string();
    let baggage =
        otel_spans::baggage_header(&policy.tenant_id, req.agent_persona.as_deref(), &request_id);
    let mut root = otel_spans::start_chat_root(req, &policy.tenant_id, &request_id, false);

    let result = async {
        let mut persona_span = root.child(
            otel_spans::PERSONA_LOAD_SPAN,
            opentelemetry::trace::SpanKind::Internal,
        );
        let persona_result = apply_persona_and_emit(req, policy, &request_id).await;
        match &persona_result {
            Ok(_) => {
                persona_span.set_str(otel_attributes::OUTCOME, "allow");
                persona_span.end_ok();
            }
            Err(err) => {
                persona_span.set_str(otel_attributes::OUTCOME, "error");
                persona_span.end_error(router_error_label(err));
            }
        }
        let persona_applied = persona_result?;

        let mut redact_span = root.child(
            otel_spans::REDACT_SPAN,
            opentelemetry::trace::SpanKind::Internal,
        );
        let redact_result = crate::redact::redact_chat_request(&persona_applied.request, policy)
            .await
            .map_err(redaction_error);
        match &redact_result {
            Ok(_) => {
                redact_span.set_str(otel_attributes::OUTCOME, "allow");
                redact_span.end_ok();
            }
            Err(err) => {
                redact_span.set_str(otel_attributes::OUTCOME, "error");
                redact_span.end_error(router_error_label(err));
            }
        }
        let (mut redacted_req, redactions) = redact_result?;
        redacted_req.baggage = Some(baggage.clone());

        let (cache_key, mut cache_state) = response_cache_key(&redacted_req, resolved, policy);
        if let Some(cache_key) = &cache_key {
            let mut cache_span = root.child(
                otel_spans::CACHE_LOOKUP_SPAN,
                opentelemetry::trace::SpanKind::Internal,
            );
            match crate::cache::lookup(cache_key).await {
                crate::cache::CacheLookupOutcome::Hit(cached, lookup_latency) => {
                    cache_span.set_str(otel_attributes::CACHE_STATE, "hit");
                    cache_span.set_str(
                        otel_attributes::CACHE_KEY_HASH16,
                        cache_key_hash16(cache_key),
                    );
                    cache_span.set_str(otel_attributes::OUTCOME, "allow");
                    cache_span.end_ok();
                    let mut response = provider_response_from_cache(
                        cache_key,
                        *cached,
                        lookup_latency,
                        persona_applied.made_by_genie,
                    );
                    restore_tool_call_arguments(&mut response, &redactions);
                    return Ok(response);
                }
                crate::cache::CacheLookupOutcome::Miss
                | crate::cache::CacheLookupOutcome::SchemaMismatch => {
                    cache_span.set_str(otel_attributes::CACHE_STATE, "miss");
                    cache_span.set_str(
                        otel_attributes::CACHE_KEY_HASH16,
                        cache_key_hash16(cache_key),
                    );
                    cache_span.set_str(otel_attributes::OUTCOME, "allow");
                    cache_span.end_ok();
                    cache_state = CacheState::Miss;
                }
                crate::cache::CacheLookupOutcome::Error(err) => {
                    cache_span.set_str(otel_attributes::CACHE_STATE, "error");
                    cache_span.set_str(
                        otel_attributes::CACHE_KEY_HASH16,
                        cache_key_hash16(cache_key),
                    );
                    cache_span.set_str(otel_attributes::OUTCOME, "error");
                    cache_span.end_error("cache_lookup_error");
                    warn!(error = %err, "response_cache_lookup_failed; continuing to provider");
                    cache_state = CacheState::Error;
                }
            }
        }

        let chain = failover::build_provider_chain(resolved, policy, &req.alias);
        let mut response = call_provider_with_chain_traced(
            &redacted_req,
            deadline,
            chain,
            Some(&root),
            Some(&baggage),
        )
        .await?;
        response.cache_state = cache_state;

        if let Some(cache_key) = &cache_key {
            match crate::cache::insert(cache_key, &response, &req.alias).await {
                crate::cache::CacheInsertOutcome::Inserted { .. } => {}
                crate::cache::CacheInsertOutcome::Skipped(_) => {
                    response.cache_state = CacheState::Skipped;
                }
                crate::cache::CacheInsertOutcome::Error(err) => {
                    warn!(error = %err, "response_cache_insert_failed");
                    response.cache_state = CacheState::Error;
                }
            }
        }

        restore_tool_call_arguments(&mut response, &redactions);
        if response.made_by_genie.is_none() {
            response.made_by_genie = persona_applied.made_by_genie;
        }
        Ok(response)
    }
    .await;

    match &result {
        Ok(response) => {
            root.set_str(otel_attributes::OUTCOME, "allow");
            root.set_i64(
                otel_attributes::PROMPT_TOKENS,
                i64::from(response.usage.prompt_tokens),
            );
            root.set_i64(
                otel_attributes::COMPLETION_TOKENS,
                i64::from(response.usage.completion_tokens),
            );
            root.set_str(
                otel_attributes::CACHE_STATE,
                cache_state_label(response.cache_state),
            );
            root.end_ok();
        }
        Err(err) => {
            root.set_str(otel_attributes::OUTCOME, "error");
            root.end_error(router_error_label(err));
        }
    }

    result
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
    call_provider_with_chain_traced(req, deadline, chain, None, req.baggage.as_deref()).await
}

async fn call_provider_with_chain_traced(
    req: &ChatCompleteRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
    parent_span: Option<&otel_spans::OtelSpan>,
    baggage: Option<&str>,
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
            let mut provider_span = provider_attempt_span(
                parent_span,
                req,
                pk,
                model,
                attempt_num,
                endpoint.fallback_position,
            );
            let traced_req = otel_spans::apply_outgoing_trace(req, &provider_span, baggage);

            // §1 #6: Propagate deadline via tokio::time::timeout.
            let outcome = tokio::time::timeout(
                remaining,
                endpoint
                    .provider
                    .call_chat(&traced_req, model, effective_deadline),
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
                    annotate_provider_error(&mut provider_span, None, "error");
                    provider_span.end_error("deadline_exceeded");
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
                    annotate_provider_error(&mut provider_span, Some(400), "refuse");
                    provider_span.end_error("terminal_400");
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
                    annotate_provider_error(&mut provider_span, Some(404), "refuse");
                    provider_span.end_error("terminal_404");
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
                    annotate_provider_error(&mut provider_span, Some(status), "error");
                    provider_span.end_error("auth_error");
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
                    annotate_provider_error(&mut provider_span, Some(429), "error");
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
                            provider_span.end_error("retry_after_exceeded_deadline");
                            break;
                        }
                        provider_span.add_retry_event(
                            attempt_num.saturating_add(1),
                            sleep.as_millis().min(u128::from(u64::MAX)) as u64,
                            Some(429),
                        );
                        provider_span.end_error("retry_after_429");
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
                    annotate_provider_error(&mut provider_span, None, "error");
                    provider_span.end_error("deadline_exceeded");
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
                    annotate_provider_error(&mut provider_span, status_opt, "error");
                    last_error = Some(e);
                }

                // Success
                Ok(Ok(mut resp)) => {
                    annotate_provider_success(&mut provider_span, &resp);
                    provider_span.end_ok();
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
                    provider_span.end_error("retry_backoff_exceeded_deadline");
                    break;
                }
                let prior_status = attempts.last().and_then(|attempt| attempt.http_status);
                provider_span.add_retry_event(
                    attempt_num.saturating_add(1),
                    u64::from(sleep_ms),
                    prior_status,
                );
                provider_span.end_error("retrying_provider_call");
                tokio::time::sleep(sleep_dur).await;
            } else {
                provider_span.end_error("provider_attempt_failed");
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

/// Call the resolved embedding provider with retry + circuit-breaker semantics.
pub async fn call_embed_provider(
    req: &EmbedRequest,
    resolved: &ResolvedModel,
    deadline: Instant,
    policy: &TenantPolicy,
) -> Result<EmbedResponse, RouterError> {
    let alias = match req.task {
        EmbedTask::Passage => "embed.standard",
        EmbedTask::Code => "embed.code",
    };
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut root = otel_spans::start_embed_root(req, alias, &request_id);
    let chain = failover::build_provider_chain(resolved, policy, alias);
    let result = call_embed_provider_with_chain_traced(req, deadline, chain, Some(&root)).await;
    match &result {
        Ok(response) => {
            root.set_str(otel_attributes::OUTCOME, "allow");
            root.set_i64(
                otel_attributes::PROMPT_TOKENS,
                i64::from(response.usage.prompt_tokens),
            );
            root.set_i64(
                otel_attributes::COMPLETION_TOKENS,
                i64::from(response.usage.completion_tokens),
            );
            root.end_ok();
        }
        Err(err) => {
            root.set_str(otel_attributes::OUTCOME, "error");
            root.end_error(router_error_label(err));
        }
    }
    result
}

/// Test/contract entry point for embedding provider dispatch.
#[doc(hidden)]
pub async fn call_embed_provider_with_chain(
    req: &EmbedRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
) -> Result<EmbedResponse, RouterError> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let mut root = otel_spans::start_embed_root(req, "embed.chain", &request_id);
    let result = call_embed_provider_with_chain_traced(req, deadline, chain, Some(&root)).await;
    match &result {
        Ok(response) => {
            root.set_str(otel_attributes::OUTCOME, "allow");
            root.set_i64(
                otel_attributes::PROMPT_TOKENS,
                i64::from(response.usage.prompt_tokens),
            );
            root.end_ok();
        }
        Err(err) => {
            root.set_str(otel_attributes::OUTCOME, "error");
            root.end_error(router_error_label(err));
        }
    }
    result
}

async fn call_embed_provider_with_chain_traced(
    req: &EmbedRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
    parent_span: Option<&otel_spans::OtelSpan>,
) -> Result<EmbedResponse, RouterError> {
    let mut last_error: Option<RouterError> = None;

    for endpoint in &chain {
        let pk = endpoint.provider.kind();
        let model = endpoint.model.as_str();
        if circuit_breaker::is_open(&pk, model) {
            continue;
        }

        for attempt_num in 1..=MAX_RETRIES_PER_PROVIDER {
            if Instant::now() >= deadline {
                record_breaker_outcome(pk, model, CallOutcome::Timeout);
                return Err(RouterError::DeadlineExceeded);
            }

            let remaining = deadline
                .duration_since(Instant::now())
                .min(PROVIDER_DEFAULT_TIMEOUT);
            let mut provider_span = parent_span
                .map(|parent| {
                    parent.child(
                        otel_spans::PROVIDER_CALL_SPAN,
                        opentelemetry::trace::SpanKind::Client,
                    )
                })
                .unwrap_or_else(|| {
                    let request_id = uuid::Uuid::new_v4().to_string();
                    otel_spans::start_embed_root(req, "embed.chain", &request_id)
                });
            provider_span.set_str(otel_attributes::PROVIDER, pk.as_metric_label());
            provider_span.set_str(otel_attributes::MODEL, model);
            provider_span.set_i64(otel_attributes::ATTEMPT_NUM, i64::from(attempt_num));
            provider_span.set_i64(
                otel_attributes::FALLBACK_POSITION,
                i64::from(endpoint.fallback_position),
            );
            provider_span.set_bool(otel_attributes::RETRIED, attempt_num > 1);
            let outcome = tokio::time::timeout(
                remaining,
                endpoint.provider.call_embed(req, model, deadline),
            )
            .await;

            match outcome {
                Err(_) => {
                    record_breaker_outcome(pk, model, CallOutcome::Timeout);
                    annotate_provider_error(&mut provider_span, None, "error");
                    provider_span.end_error("deadline_exceeded");
                    return Err(RouterError::DeadlineExceeded);
                }
                Ok(Ok(resp)) => {
                    record_breaker_outcome(pk, model, CallOutcome::Success);
                    provider_span.set_i64(otel_attributes::STATUS_CODE, 200);
                    provider_span.set_str(otel_attributes::OUTCOME, "allow");
                    provider_span.set_i64(
                        otel_attributes::PROMPT_TOKENS,
                        i64::from(resp.usage.prompt_tokens),
                    );
                    provider_span.end_ok();
                    return Ok(resp);
                }
                Ok(Err(err @ RouterError::NoSidecarForRegion { .. })) => {
                    record_breaker_outcome(pk, model, CallOutcome::Failure4xx);
                    annotate_provider_error(&mut provider_span, None, "error");
                    provider_span.end_error(router_error_label(&err));
                    return Err(err);
                }
                Ok(Err(
                    err @ RouterError::TerminalProviderError {
                        status: 400..=499, ..
                    },
                )) => {
                    record_breaker_outcome(pk, model, CallOutcome::Failure4xx);
                    let status = match &err {
                        RouterError::TerminalProviderError { status, .. } => Some(*status),
                        _ => None,
                    };
                    annotate_provider_error(&mut provider_span, status, "error");
                    provider_span.end_error(router_error_label(&err));
                    return Err(err);
                }
                Ok(Err(err)) => {
                    record_breaker_outcome(pk, model, breaker_outcome_for_error(&err));
                    let status = match &err {
                        RouterError::TerminalProviderError { status, .. } => Some(*status),
                        _ => None,
                    };
                    annotate_provider_error(&mut provider_span, status, "error");
                    provider_span.end_error(router_error_label(&err));
                    last_error = Some(err);
                }
            }
        }
    }

    Err(RouterError::AllProvidersFailed {
        last_error: Box::new(last_error.unwrap_or(RouterError::InvalidResponse {
            reason: "no embedding providers in chain".into(),
        })),
        attempts: Vec::new(),
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
    let request_id = uuid::Uuid::new_v4().to_string();
    let baggage =
        otel_spans::baggage_header(&policy.tenant_id, req.agent_persona.as_deref(), &request_id);
    let mut root = otel_spans::start_chat_root(req, &policy.tenant_id, &request_id, true);

    let result = async {
        let mut persona_span = root.child(
            otel_spans::PERSONA_LOAD_SPAN,
            opentelemetry::trace::SpanKind::Internal,
        );
        let persona_result = apply_persona_and_emit(req, policy, &request_id).await;
        match &persona_result {
            Ok(_) => {
                persona_span.set_str(otel_attributes::OUTCOME, "allow");
                persona_span.end_ok();
            }
            Err(err) => {
                persona_span.set_str(otel_attributes::OUTCOME, "error");
                persona_span.end_error(router_error_label(err));
            }
        }
        let persona_applied = persona_result?;

        let mut redact_span = root.child(
            otel_spans::REDACT_SPAN,
            opentelemetry::trace::SpanKind::Internal,
        );
        let redact_result = crate::redact::redact_chat_request(&persona_applied.request, policy)
            .await
            .map_err(redaction_error);
        match &redact_result {
            Ok(_) => {
                redact_span.set_str(otel_attributes::OUTCOME, "allow");
                redact_span.end_ok();
            }
            Err(err) => {
                redact_span.set_str(otel_attributes::OUTCOME, "error");
                redact_span.end_error(router_error_label(err));
            }
        }
        let (mut redacted_req, _redactions) = redact_result?;
        redacted_req.baggage = Some(baggage.clone());
        let chain = failover::build_provider_chain(resolved, policy, &req.alias);
        call_provider_streaming_with_chain_traced(
            &redacted_req,
            deadline,
            chain,
            Some(&root),
            Some(&baggage),
        )
        .await
    }
    .await;

    match &result {
        Ok(_) => {
            root.set_str(otel_attributes::OUTCOME, "allow");
            root.end_ok();
        }
        Err(err) => {
            root.set_str(otel_attributes::OUTCOME, "error");
            root.end_error(router_error_label(err));
        }
    }
    result
}

/// Test/contract entry point for exercising streaming retry/failover with injected providers.
#[doc(hidden)]
pub async fn call_provider_streaming_with_chain(
    req: &ChatCompleteRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
) -> Result<ProviderStreamResponse, RouterError> {
    call_provider_streaming_with_chain_traced(req, deadline, chain, None, req.baggage.as_deref())
        .await
}

async fn call_provider_streaming_with_chain_traced(
    req: &ChatCompleteRequest,
    deadline: Instant,
    chain: Vec<ProviderEndpoint>,
    parent_span: Option<&otel_spans::OtelSpan>,
    baggage: Option<&str>,
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
            let mut provider_span = provider_attempt_span(
                parent_span,
                req,
                pk,
                model,
                attempt_num,
                endpoint.fallback_position,
            );
            let traced_req = otel_spans::apply_outgoing_trace(req, &provider_span, baggage);
            let outcome = tokio::time::timeout(
                remaining,
                endpoint
                    .provider
                    .call_chat_streaming(&traced_req, model, effective_deadline),
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
                    annotate_provider_error(&mut provider_span, None, "error");
                    provider_span.end_error("streaming_deadline_exceeded");
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
                    annotate_provider_error(&mut provider_span, Some(400), "refuse");
                    provider_span.end_error("streaming_terminal_400");
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
                    annotate_provider_error(&mut provider_span, Some(404), "refuse");
                    provider_span.end_error("streaming_terminal_404");
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
                    annotate_provider_error(&mut provider_span, Some(status), "error");
                    provider_span.end_error("streaming_auth_error");
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
                    annotate_provider_error(&mut provider_span, Some(429), "error");
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
                            provider_span.end_error("streaming_retry_after_exceeded_deadline");
                            break;
                        }
                        provider_span.add_retry_event(
                            attempt_num.saturating_add(1),
                            sleep.as_millis().min(u128::from(u64::MAX)) as u64,
                            Some(429),
                        );
                        provider_span.end_error("streaming_retry_after_429");
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
                    annotate_provider_error(&mut provider_span, None, "error");
                    provider_span.end_error("streaming_deadline_exceeded");
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
                    annotate_provider_error(&mut provider_span, status_opt, "error");
                    last_error = Some(e);
                }

                Ok(Ok(resp)) => {
                    provider_span.set_i64(otel_attributes::STATUS_CODE, 200);
                    provider_span.set_str(otel_attributes::OUTCOME, "allow");
                    provider_span.end_ok();
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
                    provider_span.end_error("streaming_retry_backoff_exceeded_deadline");
                    break;
                }
                let prior_status = attempts.last().and_then(|attempt| attempt.http_status);
                provider_span.add_retry_event(
                    attempt_num.saturating_add(1),
                    u64::from(sleep_ms),
                    prior_status,
                );
                provider_span.end_error("streaming_retrying_provider_call");
                tokio::time::sleep(sleep_dur).await;
            } else {
                provider_span.end_error("streaming_provider_attempt_failed");
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

fn response_cache_key(
    req: &ChatCompleteRequest,
    resolved: &ResolvedModel,
    policy: &TenantPolicy,
) -> (Option<crate::cache::CacheKey>, CacheState) {
    if crate::cache::ttl::ttl_for_alias(&req.alias).is_none() {
        return (None, CacheState::Skipped);
    }
    let redacted_prompt = cache_prompt_material(req);
    let persona_handle = req.agent_persona.as_deref().unwrap_or("");
    (
        Some(crate::cache::CacheKey::derive(
            &policy.tenant_id,
            &redacted_prompt,
            &resolved.model,
            persona_handle,
        )),
        CacheState::Miss,
    )
}

fn cache_prompt_material(req: &ChatCompleteRequest) -> String {
    let mut material = String::new();
    for message in &req.messages {
        material.push_str(&message.role);
        material.push('\x1e');
        material.push_str(&message.content);
        material.push('\x1f');
    }
    material
}

fn provider_response_from_cache(
    key: &crate::cache::CacheKey,
    cached: crate::cache::CachedResponse,
    lookup_latency: Duration,
    made_by_genie: Option<MadeByGenie>,
) -> ProviderResponse {
    let saved_tokens = cached
        .usage
        .prompt_tokens
        .saturating_add(cached.usage.completion_tokens);
    ProviderResponse {
        id: format!("cache-{}", hex::encode(&key.prompt_hash[..8])),
        usage: cached.usage,
        choices: cached.choices,
        finish_reason: cached.finish_reason,
        latency_ms: lookup_latency.as_millis().min(u128::from(u32::MAX)) as u32,
        cache_state: CacheState::Hit { saved_tokens },
        attempts: vec![],
        made_by_genie,
    }
}

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

fn provider_attempt_span(
    parent_span: Option<&otel_spans::OtelSpan>,
    req: &ChatCompleteRequest,
    provider: ProviderKind,
    model: &str,
    attempt_num: u8,
    fallback_position: u8,
) -> otel_spans::OtelSpan {
    let mut span = parent_span
        .map(|parent| {
            parent.child(
                otel_spans::PROVIDER_CALL_SPAN,
                opentelemetry::trace::SpanKind::Client,
            )
        })
        .unwrap_or_else(|| otel_spans::start_detached_provider_span(req));
    span.set_str(otel_attributes::PROVIDER, provider.as_metric_label());
    span.set_str(otel_attributes::MODEL, model);
    span.set_i64(otel_attributes::ATTEMPT_NUM, i64::from(attempt_num));
    span.set_i64(
        otel_attributes::FALLBACK_POSITION,
        i64::from(fallback_position),
    );
    span.set_bool(otel_attributes::RETRIED, attempt_num > 1);
    span
}

fn annotate_provider_success(span: &mut otel_spans::OtelSpan, response: &ProviderResponse) {
    span.set_i64(otel_attributes::STATUS_CODE, 200);
    span.set_i64(
        otel_attributes::PROMPT_TOKENS,
        i64::from(response.usage.prompt_tokens),
    );
    span.set_i64(
        otel_attributes::COMPLETION_TOKENS,
        i64::from(response.usage.completion_tokens),
    );
    span.set_str(otel_attributes::OUTCOME, "allow");
}

fn annotate_provider_error(
    span: &mut otel_spans::OtelSpan,
    status_code: Option<u16>,
    outcome: &'static str,
) {
    if let Some(status_code) = status_code {
        span.set_i64(otel_attributes::STATUS_CODE, i64::from(status_code));
    }
    span.set_str(otel_attributes::OUTCOME, outcome);
}

fn cache_key_hash16(key: &crate::cache::CacheKey) -> String {
    hex::encode(&key.prompt_hash[..8])
}

fn cache_state_label(state: CacheState) -> &'static str {
    match state {
        CacheState::None => "none",
        CacheState::Hit { .. } => "hit",
        CacheState::Miss => "miss",
        CacheState::Skipped => "skipped",
        CacheState::Error => "error",
    }
}

fn router_error_label(error: &RouterError) -> &'static str {
    match error {
        RouterError::DeadlineExceeded => "deadline_exceeded",
        RouterError::AuthError { .. } => "auth_error",
        RouterError::TerminalProviderError { status: 400, .. } => "terminal_400",
        RouterError::TerminalProviderError { status: 404, .. } => "terminal_404",
        RouterError::TerminalProviderError { status: 429, .. } => "rate_limited",
        RouterError::TerminalProviderError { .. } => "provider_error",
        RouterError::NoSidecarForRegion { .. } => "no_sidecar_for_region",
        RouterError::RedactionFailed { .. } => "redaction_failed",
        RouterError::UnknownPersona { .. } => "unknown_persona",
        RouterError::PersonaTampered { .. } => "persona_tampered",
        RouterError::PersonaAuditFailed { .. } => "persona_audit_failed",
        RouterError::SerializationError { .. } => "serialization_error",
        RouterError::InvalidResponse { .. } => "invalid_response",
        RouterError::AllProvidersFailed { .. } => "all_providers_failed",
        RouterError::StreamingNotImplemented => "streaming_not_implemented",
    }
}

fn breaker_outcome_for_error(error: &RouterError) -> CallOutcome {
    match error {
        RouterError::TerminalProviderError { status: 429, .. } => CallOutcome::Failure429,
        RouterError::TerminalProviderError { status, .. } if *status >= 500 => {
            CallOutcome::Failure5xx
        }
        RouterError::DeadlineExceeded => CallOutcome::Timeout,
        RouterError::AuthError { .. }
        | RouterError::TerminalProviderError { .. }
        | RouterError::NoSidecarForRegion { .. } => CallOutcome::Failure4xx,
        RouterError::SerializationError { .. }
        | RouterError::InvalidResponse { .. }
        | RouterError::RedactionFailed { .. }
        | RouterError::UnknownPersona { .. }
        | RouterError::PersonaTampered { .. }
        | RouterError::PersonaAuditFailed { .. }
        | RouterError::AllProvidersFailed { .. }
        | RouterError::StreamingNotImplemented => CallOutcome::Failure5xx,
    }
}

fn record_breaker_outcome(provider: ProviderKind, model: &str, outcome: CallOutcome) {
    circuit_breaker::record_outcome(&provider, model, outcome);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_prompt_material_includes_roles_and_message_boundaries() {
        let req = ChatCompleteRequest {
            alias: "chat.smart".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "A".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "B".to_string(),
                },
            ],
            max_tokens: None,
            temperature: None,
            agent_persona: Some("cuo-cpo@0.4.1".to_string()),
            traceparent: None,
            tracestate: None,
            baggage: None,
        };

        assert_eq!(
            cache_prompt_material(&req),
            "system\u{1e}A\u{1f}user\u{1e}B\u{1f}"
        );
    }

    #[test]
    fn cache_prompt_material_distinguishes_role_swaps() {
        let mut a = ChatCompleteRequest {
            alias: "chat.smart".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: "same".to_string(),
            }],
            max_tokens: None,
            temperature: None,
            agent_persona: None,
            traceparent: None,
            tracestate: None,
            baggage: None,
        };
        let mut b = a.clone();
        b.messages[0].role = "system".to_string();

        assert_ne!(cache_prompt_material(&a), cache_prompt_material(&b));
        a.messages[0].content = "other".to_string();
        assert_ne!(cache_prompt_material(&a), cache_prompt_material(&b));
    }
}

fn redaction_error(error: crate::redact::RedactError) -> RouterError {
    RouterError::RedactionFailed {
        reason: error.to_string(),
    }
}

async fn apply_persona_and_emit(
    req: &ChatCompleteRequest,
    policy: &TenantPolicy,
    request_id: &str,
) -> Result<crate::persona::AppliedPersona, RouterError> {
    let applied = crate::persona::apply_to_request(req, &policy.tenant_id, request_id)
        .map_err(persona_error)?;
    if let Some(row) = applied.audit_row.clone() {
        crate::memory_writer::emit(row)
            .await
            .map_err(|err| RouterError::PersonaAuditFailed {
                reason: err.to_string(),
            })?;
    }
    Ok(applied)
}

fn persona_error(error: crate::persona::PersonaError) -> RouterError {
    match error {
        crate::persona::PersonaError::UnknownPersona { handle, available } => {
            RouterError::UnknownPersona {
                agent_persona: handle,
                available,
            }
        }
        crate::persona::PersonaError::Tampered { handle, .. } => RouterError::PersonaTampered {
            handle: handle.display(),
        },
        crate::persona::PersonaError::RegistryNotInitialised => RouterError::InvalidResponse {
            reason: "persona registry not initialised".to_string(),
        },
        crate::persona::PersonaError::MemoryReadFailed(reason) => RouterError::InvalidResponse {
            reason: format!("persona memory read failed: {reason}"),
        },
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
