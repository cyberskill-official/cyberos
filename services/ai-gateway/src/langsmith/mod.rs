//! TASK-OBS-004 - LangSmith AI-trace export.
//!
//! Every LLM call can be mirrored to a self-hosted LangSmith for prompt-quality and tool-call analysis,
//! correlated to the operational trace via a shared `trace_id` (TASK-OBS-005 / TASK-AI-022). The export is:
//!   - opt-in per tenant (`ai_policy.langsmith_export`, default false) - even redacted prompts carry
//!     tenant-business semantics, so it is off until the tenant consents (§1 #3);
//!   - redacted-only - the export signature takes `RedactedPrompt` / `RedactedResponse` newtypes, so a
//!     raw `String` is a compile error (§1 #5);
//!   - fire-and-forget - the gateway hot path does not await the POST (§1 #6, #7);
//!   - residency-routed - Sg1/Eu1/Us1 hit per-region hosts; Vn1 drops until a VN deploy exists (§1 #4);
//!   - idempotent - the `trace_id` is the `Idempotency-Key` so a retried delivery is de-duplicated (#11).
//!
//! The live POST (`post_with_retry`) needs a reachable LangSmith and `LANGSMITH_API_TOKEN`, so it is
//! owner-run; the payload build, truncation, opt-in gate, residency map, metrics, and error taxonomy
//! are unit/integration tested.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_gauge, register_histogram, CounterVec, Gauge, Histogram};
use serde::Serialize;
use tracing::warn;

use crate::policy::Residency;

/// A redacted prompt. The export API only accepts this newtype, so a raw `String` (possibly carrying PII)
/// cannot be exported by mistake (§1 #5).
#[derive(Debug, Clone, Serialize)]
pub struct RedactedPrompt(pub String);

/// A redacted response. Same compile-time guard as [`RedactedPrompt`].
#[derive(Debug, Clone, Serialize)]
pub struct RedactedResponse(pub String);

/// One tool call captured in the LangSmith payload (§1 #2).
#[derive(Debug, Clone, Serialize)]
pub struct ToolCallTrace {
    pub tool_name: String,
    pub redacted_args: String,
    pub outcome: String,
}

/// The metadata exported alongside the redacted prompt and response (§1 #2).
#[derive(Debug, Clone, Serialize)]
pub struct LangSmithMetadata {
    pub model_alias: String,
    pub resolved_model: String,
    pub provider: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub latency_ms: u32,
    pub cost_usd: f64,
    pub persona_handle: String,
    pub tenant_id: String,
    /// W3C trace id hex (matches OTel and the Idempotency-Key).
    pub trace_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCallTrace>,
}

/// The export payload (§3). `trace_id` is the W3C hex and the idempotency key.
#[derive(Debug, Clone, Serialize)]
pub struct Payload {
    pub trace_id: String,
    pub prompt: String,
    pub response: String,
    pub metadata: LangSmithMetadata,
}

const MAX_PAYLOAD_BYTES: usize = 100 * 1024;
const TRUNCATION_MARKER: &str = "...[truncated by TASK-OBS-004]";

/// Build the export payload, truncating an over-100 KB redacted prompt or response on a char boundary
/// with a marker (§1 #12). The `trace_id` is taken from the metadata so the two always agree.
pub fn build_payload(
    prompt: RedactedPrompt,
    response: RedactedResponse,
    metadata: LangSmithMetadata,
) -> Payload {
    Payload {
        trace_id: metadata.trace_id.clone(),
        prompt: truncate(prompt.0),
        response: truncate(response.0),
        metadata,
    }
}

fn truncate(s: String) -> String {
    if s.len() <= MAX_PAYLOAD_BYTES {
        return s;
    }
    let mut end = MAX_PAYLOAD_BYTES;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}{}", &s[..end], TRUNCATION_MARKER)
}

/// The outcome of an export attempt, for the `ai_langsmith_exports_total{outcome}` metric (§1 #10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportOutcome {
    /// Opted in and the export was dispatched (the spawned POST records its own delivery result).
    Dispatched,
    /// The tenant has not opted in (§1 #3).
    DroppedOptOut,
    /// Tenant residency has no LangSmith region deploy yet (vn-1).
    DroppedResidencyNoLangsmith,
}

impl ExportOutcome {
    pub fn label(self) -> &'static str {
        match self {
            ExportOutcome::Dispatched => "dispatched",
            ExportOutcome::DroppedOptOut => "dropped_opt_out",
            ExportOutcome::DroppedResidencyNoLangsmith => "dropped_residency_no_langsmith",
        }
    }
}

/// Why a delivery attempt failed (§1 #10).
#[derive(Debug, thiserror::Error)]
pub enum LangSmithError {
    #[error("langsmith auth failed")]
    AuthFailed,
    #[error("langsmith rejected payload (status {0})")]
    InvalidPayload(u16),
    #[error("langsmith server error (status {0})")]
    ServerError(u16),
    #[error("network error: {0}")]
    Network(String),
    #[error("dropped after retries")]
    DroppedAfterRetries,
}

impl LangSmithError {
    fn outcome_label(&self) -> &'static str {
        match self {
            LangSmithError::AuthFailed => "auth_failed",
            LangSmithError::InvalidPayload(_) => "invalid_payload",
            LangSmithError::ServerError(_) | LangSmithError::Network(_) => "langsmith_unreachable",
            LangSmithError::DroppedAfterRetries => "dropped_after_retries",
        }
    }
}

const RETRY_DELAYS_MS: &[u64] = &[100, 250, 500];
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

mod metrics {
    use super::*;

    pub static EXPORTS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_langsmith_exports_total",
            "LangSmith export attempts by outcome and tenant",
            &["outcome", "tenant_id"]
        )
        .unwrap()
    });

    pub static EXPORT_LATENCY_MS: Lazy<Histogram> = Lazy::new(|| {
        register_histogram!(
            "ai_langsmith_export_latency_ms",
            "Async LangSmith export wall-clock latency in milliseconds",
            vec![10.0, 50.0, 100.0, 250.0, 500.0, 1_000.0, 2_000.0, 5_000.0]
        )
        .unwrap()
    });

    pub static QUEUE_DEPTH: Lazy<Gauge> = Lazy::new(|| {
        register_gauge!(
            "ai_langsmith_queue_depth",
            "Pending tokio::spawn'd LangSmith exports"
        )
        .unwrap()
    });
}

static PENDING: AtomicU64 = AtomicU64::new(0);

fn record(outcome: &str, tenant_id: &str) {
    metrics::EXPORTS
        .with_label_values(&[outcome, tenant_id])
        .inc();
}

/// Resolve the LangSmith base URL for a tenant residency (§1 #4).
///
/// `LANGSMITH_URL` overrides the region map (local/dev + integration tests). Vn1 has no deploy yet
/// and returns `None` unless an override is set.
pub fn url_for_residency(residency: Residency) -> Option<String> {
    if let Ok(override_url) = std::env::var("LANGSMITH_URL") {
        if !override_url.is_empty() {
            return Some(override_url);
        }
    }
    match residency {
        Residency::Sg1 => Some("https://langsmith.sg-1.cyberos.world".into()),
        Residency::Eu1 => Some("https://langsmith.eu-1.cyberos.world".into()),
        Residency::Us1 => Some("https://langsmith.us-1.cyberos.world".into()),
        Residency::Vn1 => None,
    }
}

/// Default public hostname used when residency is unknown and no env override is set.
pub fn default_langsmith_url() -> String {
    std::env::var("LANGSMITH_URL")
        .unwrap_or_else(|_| "https://langsmith.cyberos.world".to_string())
}

fn langsmith_token() -> String {
    std::env::var("LANGSMITH_API_TOKEN").unwrap_or_default()
}

/// Export a completed AI call to LangSmith. Opt-in gated (§1 #3); when enabled, the redacted payload is
/// built and POSTed in a spawned task (§1 #6 fire-and-forget) so the gateway response is not blocked on
/// LangSmith availability (§1 #7). Returns the synchronous decision so the caller can observe the metric.
pub fn export(
    enabled: bool,
    residency: Residency,
    prompt: RedactedPrompt,
    response: RedactedResponse,
    metadata: LangSmithMetadata,
) -> ExportOutcome {
    let tenant = metadata.tenant_id.clone();
    if !enabled {
        record(ExportOutcome::DroppedOptOut.label(), &tenant);
        return ExportOutcome::DroppedOptOut;
    }
    let Some(base_url) = url_for_residency(residency) else {
        record(
            ExportOutcome::DroppedResidencyNoLangsmith.label(),
            &tenant,
        );
        return ExportOutcome::DroppedResidencyNoLangsmith;
    };
    let payload = build_payload(prompt, response, metadata);
    let pending = PENDING.fetch_add(1, Ordering::Relaxed) + 1;
    metrics::QUEUE_DEPTH.set(pending as f64);
    tokio::spawn(async move {
        let started = Instant::now();
        let result = post_with_retry(&base_url, &payload).await;
        metrics::EXPORT_LATENCY_MS.observe(started.elapsed().as_secs_f64() * 1000.0);
        let left = PENDING.fetch_sub(1, Ordering::Relaxed).saturating_sub(1);
        metrics::QUEUE_DEPTH.set(left as f64);
        match result {
            Ok(()) => record("ok", &payload.metadata.tenant_id),
            Err(e) => {
                warn!(
                    error = %e,
                    trace_id = %payload.trace_id,
                    "langsmith_export_failed"
                );
                record(e.outcome_label(), &payload.metadata.tenant_id);
            }
        }
    });
    record(ExportOutcome::Dispatched.label(), &tenant);
    ExportOutcome::Dispatched
}

/// POST the payload to LangSmith with retry + exponential backoff (§1 #8) and the `Idempotency-Key`
/// header set to the trace id (§1 #11).
pub async fn post_with_retry(base_url: &str, payload: &Payload) -> Result<(), LangSmithError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| LangSmithError::Network(e.to_string()))?;
    let url = format!("{}/api/v1/traces", base_url.trim_end_matches('/'));
    let token = langsmith_token();
    let mut last = LangSmithError::DroppedAfterRetries;

    for (attempt, delay) in RETRY_DELAYS_MS.iter().enumerate() {
        if attempt > 0 {
            tokio::time::sleep(Duration::from_millis(*delay)).await;
        }
        let res = client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("Idempotency-Key", payload.trace_id.clone())
            .json(payload)
            .send()
            .await;
        match res {
            Ok(r) if r.status().is_success() => return Ok(()),
            Ok(r) if r.status().as_u16() == 401 => return Err(LangSmithError::AuthFailed),
            Ok(r) if r.status().is_client_error() => {
                return Err(LangSmithError::InvalidPayload(r.status().as_u16()))
            }
            Ok(r) => last = LangSmithError::ServerError(r.status().as_u16()),
            Err(e) => last = LangSmithError::Network(e.to_string()),
        }
    }
    Err(last)
}

/// Assert the default / configured host is never the langchain.com SaaS (§1 #4 / #15).
pub fn asserts_self_hosted(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("langsmith.cyberos.world")
        || lower.contains("langsmith.sg-1.cyberos.world")
        || lower.contains("langsmith.eu-1.cyberos.world")
        || lower.contains("langsmith.us-1.cyberos.world")
        || lower.starts_with("http://127.0.0.1")
        || lower.starts_with("http://localhost")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(trace_id: &str) -> LangSmithMetadata {
        LangSmithMetadata {
            model_alias: "chat.smart".into(),
            resolved_model: "claude-3-5-sonnet".into(),
            provider: "anthropic".into(),
            temperature: Some(0.2),
            max_tokens: Some(1024),
            latency_ms: 42,
            cost_usd: 0.0012,
            persona_handle: "default".into(),
            tenant_id: "org:cyberskill".into(),
            trace_id: trace_id.into(),
            tool_calls: vec![],
        }
    }

    #[test]
    fn build_payload_passes_trace_id_and_small_bodies_through() {
        let p = build_payload(
            RedactedPrompt("hello".into()),
            RedactedResponse("hi".into()),
            meta("4bf92f3577b34da6a3ce929d0e0e4736"),
        );
        assert_eq!(p.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(p.prompt, "hello");
        assert_eq!(p.response, "hi");
    }

    #[test]
    fn oversized_bodies_are_truncated_with_a_marker() {
        let big = "a".repeat(MAX_PAYLOAD_BYTES + 500);
        let p = build_payload(
            RedactedPrompt(big.clone()),
            RedactedResponse(big),
            meta("abc"),
        );
        assert!(p.prompt.ends_with(TRUNCATION_MARKER));
        assert!(p.prompt.len() <= MAX_PAYLOAD_BYTES + TRUNCATION_MARKER.len());
        assert!(p.response.ends_with(TRUNCATION_MARKER));
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        let s = "é".repeat(MAX_PAYLOAD_BYTES);
        let out = truncate(s);
        assert!(out.ends_with(TRUNCATION_MARKER));
    }

    #[test]
    fn export_without_opt_in_drops_and_does_not_dispatch() {
        let outcome = export(
            false,
            Residency::Sg1,
            RedactedPrompt("x".into()),
            RedactedResponse("y".into()),
            meta("t"),
        );
        assert_eq!(outcome, ExportOutcome::DroppedOptOut);
        assert_eq!(outcome.label(), "dropped_opt_out");
    }

    #[test]
    fn vn1_without_override_drops_for_residency() {
        // Clear override if a prior test left one; ignore unset.
        std::env::remove_var("LANGSMITH_URL");
        let outcome = export(
            true,
            Residency::Vn1,
            RedactedPrompt("x".into()),
            RedactedResponse("y".into()),
            meta("t-vn"),
        );
        assert_eq!(outcome, ExportOutcome::DroppedResidencyNoLangsmith);
    }

    #[test]
    fn region_urls_are_self_hosted() {
        for r in [Residency::Sg1, Residency::Eu1, Residency::Us1] {
            let u = url_for_residency(r).expect("region url");
            assert!(asserts_self_hosted(&u), "{u}");
            assert!(!u.contains("langchain.com"));
        }
    }

    #[test]
    fn error_taxonomy_renders() {
        assert_eq!(
            LangSmithError::AuthFailed.to_string(),
            "langsmith auth failed"
        );
        assert_eq!(
            LangSmithError::InvalidPayload(422).to_string(),
            "langsmith rejected payload (status 422)"
        );
    }

    #[test]
    fn tool_calls_serialize_in_payload() {
        let mut m = meta("tid");
        m.tool_calls.push(ToolCallTrace {
            tool_name: "search".into(),
            redacted_args: "{\"q\":\"<VN_CCCD_1>\"}".into(),
            outcome: "success".into(),
        });
        let p = build_payload(RedactedPrompt("p".into()), RedactedResponse("r".into()), m);
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("\"tool_name\":\"search\""));
        assert!(json.contains("<VN_CCCD_1>"));
    }
}
