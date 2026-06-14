//! FR-OBS-004 — self-hosted LangSmith trace exports for AI calls.

pub mod client;
pub mod payload;

use std::time::Instant;

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram, register_int_gauge, CounterVec};
use prometheus::{Histogram, IntGauge};
use tracing::warn;

use crate::policy::TenantPolicy;

pub use client::{LangSmithConfig, LangSmithError};
pub use payload::{
    build_payload, cost_usd_for_response, is_w3c_trace_id, prompt_from_messages,
    response_from_choices, tool_calls_from_response, LangSmithMetadata, LangSmithPayload,
    RedactedPrompt, RedactedResponse, ToolCallTrace, TRUNCATION_MARKER,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportDecision {
    DroppedOptOut,
    InvalidPayload,
    Spawned,
}

static EXPORTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_langsmith_exports_total",
        "LangSmith exports by outcome and tenant",
        &["outcome", "tenant_id"]
    )
    .unwrap()
});

static EXPORT_LATENCY_MS: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "ai_langsmith_export_latency_ms",
        "LangSmith export task wall-clock latency in milliseconds",
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1_000.0, 2_500.0]
    )
    .unwrap()
});

static QUEUE_DEPTH: Lazy<IntGauge> = Lazy::new(|| {
    register_int_gauge!(
        "ai_langsmith_queue_depth",
        "Pending fire-and-forget LangSmith export tasks"
    )
    .unwrap()
});

pub async fn export(
    trace_id: &str,
    redacted_prompt: RedactedPrompt,
    redacted_response: RedactedResponse,
    metadata: LangSmithMetadata,
    tenant_policy: &TenantPolicy,
) -> ExportDecision {
    let config = client::LangSmithConfig::from_env(tenant_policy.ai_policy.residency);
    export_with_config(
        trace_id,
        redacted_prompt,
        redacted_response,
        metadata,
        tenant_policy,
        config,
    )
    .await
}

pub async fn export_with_config(
    trace_id: &str,
    redacted_prompt: RedactedPrompt,
    redacted_response: RedactedResponse,
    metadata: LangSmithMetadata,
    tenant_policy: &TenantPolicy,
    config: LangSmithConfig,
) -> ExportDecision {
    if !tenant_policy.ai_policy.langsmith_export {
        record_export("dropped_opt_out", &metadata.tenant_id);
        return ExportDecision::DroppedOptOut;
    }

    if !is_w3c_trace_id(trace_id) || config.validate_self_hosted().is_err() {
        record_export("invalid_payload", &metadata.tenant_id);
        return ExportDecision::InvalidPayload;
    }

    let trace_id = trace_id.to_ascii_lowercase();
    let tenant_id = metadata.tenant_id.clone();
    QUEUE_DEPTH.inc();
    tokio::spawn(async move {
        let _queue_guard = QueueDepthGuard;
        let started = Instant::now();
        let payload = build_payload(&trace_id, redacted_prompt, redacted_response, metadata);
        let outcome = match client::post_with_retry_with_config(&config, &payload).await {
            Ok(()) => "ok",
            Err(err) => {
                warn!(error = %err, trace_id = %trace_id, "langsmith_export_failed");
                err.metric_outcome()
            }
        };
        EXPORT_LATENCY_MS.observe(started.elapsed().as_secs_f64() * 1000.0);
        record_export(outcome, &tenant_id);
    });

    ExportDecision::Spawned
}

pub fn queue_depth() -> i64 {
    QUEUE_DEPTH.get()
}

fn record_export(outcome: &'static str, tenant_id: &str) {
    EXPORTS_TOTAL.with_label_values(&[outcome, tenant_id]).inc();
}

#[derive(Debug)]
struct QueueDepthGuard;

impl Drop for QueueDepthGuard {
    fn drop(&mut self) {
        QUEUE_DEPTH.dec();
    }
}
