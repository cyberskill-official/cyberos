//! FR-OBS-004 - LangSmith AI-trace export.
//!
//! Every LLM call can be mirrored to a self-hosted LangSmith for prompt-quality and tool-call analysis,
//! correlated to the operational trace via a shared `trace_id` (FR-OBS-005 / FR-AI-022). The export is:
//!   - opt-in per tenant (`ai_policy.langsmith_export`, default false) - even redacted prompts carry
//!     tenant-business semantics, so it is off until the tenant consents (§1 #3);
//!   - redacted-only - the export signature takes `RedactedPrompt` / `RedactedResponse` newtypes, so a
//!     raw `String` is a compile error (§1 #5);
//!   - fire-and-forget - the gateway hot path does not await the POST (§1 #6, #7);
//!   - idempotent - the `trace_id` is the `Idempotency-Key` so a retried delivery is de-duplicated (#11).
//!
//! The live POST (`post_with_retry`) needs a reachable LangSmith and `LANGSMITH_API_TOKEN`, so it is
//! owner-run; the payload build, the 100 KB truncation, the opt-in gate, and the error taxonomy are pure
//! and unit-tested here.

use std::time::Duration;

use serde::Serialize;

/// A redacted prompt. The export API only accepts this newtype, so a raw `String` (possibly carrying PII)
/// cannot be exported by mistake (§1 #5).
#[derive(Debug, Clone)]
pub struct RedactedPrompt(pub String);

/// A redacted response. Same compile-time guard as [`RedactedPrompt`].
#[derive(Debug, Clone)]
pub struct RedactedResponse(pub String);

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
const TRUNCATION_MARKER: &str = "...[truncated by FR-OBS-004]";

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
}

impl ExportOutcome {
    pub fn label(self) -> &'static str {
        match self {
            ExportOutcome::Dispatched => "dispatched",
            ExportOutcome::DroppedOptOut => "dropped_opt_out",
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

const RETRY_DELAYS_MS: &[u64] = &[100, 250, 500];
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

fn langsmith_url() -> String {
    std::env::var("LANGSMITH_URL").unwrap_or_else(|_| "https://langsmith.cyberos.world".to_string())
}

fn langsmith_token() -> String {
    std::env::var("LANGSMITH_API_TOKEN").unwrap_or_default()
}

/// Export a completed AI call to LangSmith. Opt-in gated (§1 #3); when enabled, the redacted payload is
/// built and POSTed in a spawned task (§1 #6 fire-and-forget) so the gateway response is not blocked on
/// LangSmith availability (§1 #7). Returns the synchronous decision so the caller can record the metric.
pub fn export(
    enabled: bool,
    prompt: RedactedPrompt,
    response: RedactedResponse,
    metadata: LangSmithMetadata,
) -> ExportOutcome {
    if !enabled {
        return ExportOutcome::DroppedOptOut;
    }
    let payload = build_payload(prompt, response, metadata);
    tokio::spawn(async move {
        if let Err(e) = post_with_retry(&payload).await {
            eprintln!(
                "{{\"sev\":2,\"event\":\"langsmith_export_failed\",\"trace_id\":\"{}\",\"error\":\"{}\"}}",
                payload.trace_id, e
            );
        }
    });
    ExportOutcome::Dispatched
}

/// POST the payload to LangSmith with retry + exponential backoff (§1 #8) and the `Idempotency-Key`
/// header set to the trace id (§1 #11). Owner-run live: needs a reachable LangSmith + `LANGSMITH_API_TOKEN`.
pub async fn post_with_retry(payload: &Payload) -> Result<(), LangSmithError> {
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| LangSmithError::Network(e.to_string()))?;
    let url = format!("{}/api/v1/traces", langsmith_url());
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
        // A multi-byte char straddling the limit must not panic or split.
        let s = "é".repeat(MAX_PAYLOAD_BYTES); // 2 bytes each
        let out = truncate(s);
        assert!(out.ends_with(TRUNCATION_MARKER));
    }

    #[test]
    fn export_without_opt_in_drops_and_does_not_dispatch() {
        let outcome = export(
            false,
            RedactedPrompt("x".into()),
            RedactedResponse("y".into()),
            meta("t"),
        );
        assert_eq!(outcome, ExportOutcome::DroppedOptOut);
        assert_eq!(outcome.label(), "dropped_opt_out");
    }

    #[test]
    fn error_taxonomy_renders() {
        assert_eq!(LangSmithError::AuthFailed.to_string(), "langsmith auth failed");
        assert_eq!(
            LangSmithError::InvalidPayload(422).to_string(),
            "langsmith rejected payload (status 422)"
        );
    }
}
