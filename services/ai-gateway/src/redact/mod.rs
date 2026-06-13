//! FR-AI-011 — Presidio EN-base PII redaction in-flight.
//!
//! Redacts PII from every prompt before dispatching to any LLM provider.
//! Uses a localhost-only Presidio sidecar for analysis + anonymization.
//!
//! See FR-AI-011 for normative behaviour and acceptance criteria.

pub mod types;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_histogram_vec, CounterVec, Histogram, HistogramVec,
};
use regex::Regex;
use url::Url;

use crate::policy::TenantPolicy;
use crate::router::{ChatCompleteRequest, Message};

pub use types::{PiiType, RedactError, RedactionResult, RestorationMap};

const SIDECAR_URL: &str = "http://127.0.0.1:5050/redact";
const SIDECAR_URL_ENV: &str = "CYBEROS_AI_GATEWAY_PRESIDIO_URL";
const SIDECAR_TIMEOUT: Duration = Duration::from_secs(2);
const SIDECAR_TIMEOUT_MS_ENV: &str = "CYBEROS_AI_GATEWAY_PRESIDIO_TIMEOUT_MS";
const MAX_PROMPT_BYTES: usize = 64 * 1024;
const LOG_REDACTION_TOKEN: &str = "[REDACTED_PII]";

// ─── Metrics ──────────────────────────────────────────────────────────────────

mod metrics {
    use super::*;

    pub static CALLS: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_redact_calls_total",
            "Redact calls by outcome",
            &["outcome"]
        )
        .unwrap()
    });

    pub static LATENCY_MS: Lazy<HistogramVec> = Lazy::new(|| {
        register_histogram_vec!(
            "ai_redact_latency_ms",
            "Redaction latency in ms",
            &["outcome"],
            vec![5.0, 10.0, 20.0, 30.0, 50.0, 100.0, 250.0]
        )
        .unwrap()
    });

    pub static PII_TYPES: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_redact_pii_types_total",
            "Per-type PII redactions",
            &["type"]
        )
        .unwrap()
    });

    pub static PROMPT_SIZE: Lazy<Histogram> = Lazy::new(|| {
        prometheus::register_histogram!(
            "ai_redact_prompt_size_bytes",
            "Prompt size at redact entry",
            vec![512.0, 1024.0, 4096.0, 8192.0, 16384.0, 65536.0]
        )
        .unwrap()
    });

    pub static UNKNOWN_ENTITIES: Lazy<CounterVec> = Lazy::new(|| {
        register_counter_vec!(
            "ai_redact_unknown_entity_dropped_total",
            "Presidio entities without PiiType variant",
            &["entity"]
        )
        .unwrap()
    });
}

static LOG_REDACTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        Regex::new(r"(?i)\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b").unwrap(),
        Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap(),
        Regex::new(r"\b(?:\d[ -]*?){13,19}\b").unwrap(),
        Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
        Regex::new(r"\+?\d[\d .()/-]{7,}\d").unwrap(),
    ]
});

// ─── Sidecar request/response types ──────────────────────────────────────────

#[derive(serde::Serialize)]
struct SidecarRequest<'a> {
    text: &'a str,
    extra_entities: Vec<&'a str>,
    pii_allowlist: Vec<&'a str>,
}

#[derive(Debug, serde::Deserialize)]
struct SidecarResponse {
    redacted_text: String,
    items: Vec<SidecarItem>,
    #[serde(default)]
    allowlist_hit_count: u32,
}

#[derive(Debug, serde::Deserialize)]
struct SidecarItem {
    entity: String,
    start: usize,
    end: usize,
    original: String,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Redact PII from a prompt using the Presidio sidecar.
///
/// Idempotent: same (prompt, policy) → same redacted_text + same placeholders.
/// Fails closed: any sidecar error → Err (caller returns 503).
pub async fn redact(prompt: &str, policy: &TenantPolicy) -> Result<RedactionResult, RedactError> {
    let started = Instant::now();
    metrics::PROMPT_SIZE.observe(prompt.len() as f64);

    if prompt.len() > MAX_PROMPT_BYTES {
        metrics::CALLS.with_label_values(&["invalid_prompt"]).inc();
        return Err(RedactError::InvalidPrompt {
            reason: format!(
                "prompt size {} exceeds max {} bytes",
                prompt.len(),
                MAX_PROMPT_BYTES
            ),
        });
    }

    let sidecar_url = match sidecar_url() {
        Ok(url) => url,
        Err(err) => {
            metrics::CALLS
                .with_label_values(&["sidecar_unreachable"])
                .inc();
            metrics::LATENCY_MS
                .with_label_values(&["sidecar_unreachable"])
                .observe(started.elapsed().as_millis() as f64);
            return Err(err);
        }
    };
    let timeout = sidecar_timeout();

    let extra_entities: Vec<&str> = policy
        .ai_policy
        .pii_redaction_extra
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();
    let pii_allowlist: Vec<&str> = policy
        .ai_policy
        .pii_allowlist
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|s| s.as_str())
        .collect();

    let req_body = SidecarRequest {
        text: prompt,
        extra_entities,
        pii_allowlist,
    };

    let resp_result = tokio::time::timeout(
        timeout,
        reqwest::Client::new()
            .post(sidecar_url)
            .json(&req_body)
            .send(),
    )
    .await;

    let resp = match resp_result {
        Err(_) => {
            metrics::CALLS.with_label_values(&["sidecar_timeout"]).inc();
            metrics::LATENCY_MS
                .with_label_values(&["sidecar_timeout"])
                .observe(started.elapsed().as_millis() as f64);
            return Err(RedactError::SidecarTimeout {
                waited_ms: timeout.as_millis() as u32,
            });
        }
        Ok(Err(e)) => {
            metrics::CALLS
                .with_label_values(&["sidecar_unreachable"])
                .inc();
            metrics::LATENCY_MS
                .with_label_values(&["sidecar_unreachable"])
                .observe(started.elapsed().as_millis() as f64);
            // §1 #12: error reason is the connection-error class, NEVER the prompt.
            return Err(RedactError::SidecarUnreachable {
                reason: e.without_url().to_string(),
            });
        }
        Ok(Ok(r)) => r,
    };

    if !resp.status().is_success() {
        metrics::CALLS.with_label_values(&["sidecar_error"]).inc();
        metrics::LATENCY_MS
            .with_label_values(&["sidecar_error"])
            .observe(started.elapsed().as_millis() as f64);
        let status = resp.status().as_u16();
        let body = resp.text().await.unwrap_or_default();
        let safe_message = sanitize_sidecar_error_message(&body);
        return Err(RedactError::SidecarError {
            status,
            message: safe_message,
        });
    }

    let body: SidecarResponse = resp.json().await.map_err(|e| {
        metrics::CALLS.with_label_values(&["sidecar_error"]).inc();
        RedactError::SidecarError {
            status: 200,
            message: format!("response_parse_error: {}", e.without_url()),
        }
    })?;

    let (redacted_text, map, counts) = build_placeholder_map_and_counts(prompt, &body);

    for (ty, n) in &counts {
        metrics::PII_TYPES
            .with_label_values(&[ty.as_metric_label()])
            .inc_by(*n as f64);
    }

    let elapsed_ms = started.elapsed().as_millis() as u32;
    metrics::LATENCY_MS
        .with_label_values(&["ok"])
        .observe(elapsed_ms as f64);
    metrics::CALLS.with_label_values(&["ok"]).inc();

    // §1 #13: log only the redacted form + counts; NEVER the raw prompt.
    tracing::debug!(
        latency_ms = elapsed_ms,
        pii_counts = ?counts,
        redacted_size_bytes = redacted_text.len(),
        "redact_success"
    );

    Ok(RedactionResult {
        redacted_text,
        map,
        counts,
        allowlist_hit_count: body.allowlist_hit_count,
        latency_ms: elapsed_ms,
    })
}

/// Redact every chat message before it reaches a provider implementation.
///
/// The returned redaction results intentionally retain restoration maps only in
/// memory; callers may use them to restore tool-call arguments and then drop
/// them. Free-form text responses must not be restored.
pub async fn redact_chat_request(
    req: &ChatCompleteRequest,
    policy: &TenantPolicy,
) -> Result<(ChatCompleteRequest, Vec<RedactionResult>), RedactError> {
    let mut redacted_messages = Vec::with_capacity(req.messages.len());
    let mut redactions = Vec::with_capacity(req.messages.len());

    for message in &req.messages {
        let redaction = redact(&message.content, policy).await?;
        redacted_messages.push(Message {
            role: message.role.clone(),
            content: redaction.redacted_text.clone(),
        });
        redactions.push(redaction);
    }

    Ok((
        ChatCompleteRequest {
            alias: req.alias.clone(),
            messages: redacted_messages,
            max_tokens: req.max_tokens,
            temperature: req.temperature,
            agent_persona: req.agent_persona.clone(),
            traceparent: req.traceparent.clone(),
            tracestate: req.tracestate.clone(),
            baggage: req.baggage.clone(),
        },
        redactions,
    ))
}

/// Restore typed placeholders in the LLM response.
///
/// MUST be called only on tool-call argument fields, NEVER on free-form text
/// response fields (per §1 #5).
pub fn restore(text: &str, map: &RestorationMap) -> String {
    let mut out = text.to_string();
    for (placeholder, value) in map.iter() {
        out = out.replace(placeholder, value);
    }
    out
}

/// Best-effort log sanitizer for code paths that need to mention untrusted text.
///
/// This helper is deliberately conservative and sidecar-free so logging paths
/// never block on Presidio and never emit common PII shapes verbatim.
pub fn redact_for_log(text: &str) -> String {
    let mut out = text.to_string();
    for pattern in LOG_REDACTION_PATTERNS.iter() {
        out = pattern.replace_all(&out, LOG_REDACTION_TOKEN).into_owned();
    }
    out
}

pub mod vn {
    /// Marker trait for VN PII audit display redactors.
    pub trait AuditRedactor {
        fn redact(value: &str) -> String;
    }

    #[derive(Debug)]
    pub struct VnMst;
    #[derive(Debug)]
    pub struct VnCccd;
    #[derive(Debug)]
    pub struct VnPhone;
    #[derive(Debug)]
    pub struct VnBankAccount;

    pub fn redact_for_audit<T: AuditRedactor>(value: &str) -> String {
        T::redact(value)
    }

    impl AuditRedactor for VnMst {
        fn redact(value: &str) -> String {
            redact_prefix_suffix(value, 2, 2, "******")
        }
    }

    impl AuditRedactor for VnCccd {
        fn redact(value: &str) -> String {
            redact_prefix_suffix(value, 3, 3, "******")
        }
    }

    impl AuditRedactor for VnPhone {
        fn redact(value: &str) -> String {
            redact_prefix_suffix(value, 2, 4, "***")
        }
    }

    impl AuditRedactor for VnBankAccount {
        fn redact(value: &str) -> String {
            let digits = digits_only(value);
            if digits.len() <= 4 {
                return "*".repeat(digits.len().max(1));
            }
            format!("***{}", &digits[digits.len() - 4..])
        }
    }

    fn redact_prefix_suffix(value: &str, prefix: usize, suffix: usize, mask: &str) -> String {
        let digits = digits_only(value);
        if digits.len() <= prefix + suffix {
            return "*".repeat(digits.len().max(1));
        }
        format!(
            "{}{}{}",
            &digits[..prefix],
            mask,
            &digits[digits.len() - suffix..]
        )
    }

    fn digits_only(value: &str) -> String {
        value.chars().filter(|c| c.is_ascii_digit()).collect()
    }
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn sidecar_url() -> Result<String, RedactError> {
    let raw = std::env::var(SIDECAR_URL_ENV).unwrap_or_else(|_| SIDECAR_URL.to_string());
    validate_sidecar_url(&raw)?;
    Ok(raw)
}

fn validate_sidecar_url(raw: &str) -> Result<(), RedactError> {
    let url = Url::parse(raw).map_err(|_| RedactError::SidecarUnreachable {
        reason: "invalid sidecar config: URL must parse".to_string(),
    })?;
    if url.scheme() != "http" {
        return Err(RedactError::SidecarUnreachable {
            reason: "invalid sidecar config: sidecar URL must use http".to_string(),
        });
    }
    let Some(host) = url.host_str() else {
        return Err(RedactError::SidecarUnreachable {
            reason: "invalid sidecar config: sidecar URL must include loopback host".to_string(),
        });
    };
    if !matches!(host, "127.0.0.1" | "localhost" | "::1") {
        return Err(RedactError::SidecarUnreachable {
            reason: "invalid sidecar config: sidecar URL must use a loopback host".to_string(),
        });
    }
    Ok(())
}

fn sidecar_timeout() -> Duration {
    std::env::var(SIDECAR_TIMEOUT_MS_ENV)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|millis| *millis > 0)
        .map(Duration::from_millis)
        .unwrap_or(SIDECAR_TIMEOUT)
}

/// Sanitize sidecar error messages to forbid prompt-fragment leaks (§1 #12).
fn sanitize_sidecar_error_message(body: &str) -> String {
    const KNOWN_ERROR_CODES: &[&str] = &[
        "redaction_internal_error",
        "validation_error",
        "recognizer_init_failed",
        "analyzer_timeout",
        "anonymizer_failed",
        "response_parse_error",
    ];
    let trimmed = body.trim();
    for code in KNOWN_ERROR_CODES {
        if trimmed.contains(code) {
            return (*code).to_string();
        }
    }
    "sidecar_returned_unrecognized_message_redacted".to_string()
}

/// Build the restoration map and counts from the sidecar response.
fn build_placeholder_map_and_counts(
    _prompt: &str,
    body: &SidecarResponse,
) -> (String, RestorationMap, HashMap<PiiType, u32>) {
    let mut map = RestorationMap::default();
    let mut counts: HashMap<PiiType, u32> = HashMap::new();
    let mut per_type_counter: HashMap<&str, u32> = HashMap::new();

    // Defensive re-sort by start offset for idempotency (§1 #11).
    let mut sorted_items: Vec<&SidecarItem> = body.items.iter().collect();
    sorted_items.sort_by_key(|item| item.start);

    for item in sorted_items {
        let Some(ty) = PiiType::from_presidio(&item.entity) else {
            tracing::warn!(
                entity = %item.entity,
                "presidio_unknown_entity_dropped; PII not redacted; add variant to PiiType enum"
            );
            metrics::UNKNOWN_ENTITIES
                .with_label_values(&[&item.entity])
                .inc();
            continue;
        };
        let n = per_type_counter
            .entry(ty.as_metric_label())
            .and_modify(|c| *c += 1)
            .or_insert(1);
        let placeholder = format!("<{}_{}>", item.entity, n);
        map.insert(placeholder.clone(), item.original.clone());
        *counts.entry(ty).or_insert(0) += 1;
    }

    (body.redacted_text.clone(), map, counts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pii_type_from_presidio_known_types() {
        assert_eq!(
            PiiType::from_presidio("CREDIT_CARD"),
            Some(PiiType::CreditCard)
        );
        assert_eq!(
            PiiType::from_presidio("EMAIL_ADDRESS"),
            Some(PiiType::EmailAddress)
        );
        assert_eq!(PiiType::from_presidio("US_SSN"), Some(PiiType::UsSsn));
        assert_eq!(
            PiiType::from_presidio("PHONE_NUMBER"),
            Some(PiiType::PhoneNumber)
        );
        assert_eq!(PiiType::from_presidio("PERSON"), Some(PiiType::Person));
        assert_eq!(PiiType::from_presidio("LOCATION"), Some(PiiType::Location));
        assert_eq!(
            PiiType::from_presidio("IP_ADDRESS"),
            Some(PiiType::IpAddress)
        );
        assert_eq!(PiiType::from_presidio("IBAN_CODE"), Some(PiiType::IbanCode));
        assert_eq!(
            PiiType::from_presidio("US_BANK_NUMBER"),
            Some(PiiType::UsBankNumber)
        );
        assert_eq!(
            PiiType::from_presidio("MEDICAL_LICENSE"),
            Some(PiiType::MedicalLicense)
        );
    }

    #[test]
    fn pii_type_from_presidio_vn_types() {
        assert_eq!(PiiType::from_presidio("VN_CCCD"), Some(PiiType::VnCccd));
        assert_eq!(PiiType::from_presidio("VN_MST"), Some(PiiType::VnMst));
        assert_eq!(PiiType::from_presidio("VN_PHONE"), Some(PiiType::VnPhone));
        assert_eq!(PiiType::from_presidio("VN_NDD"), Some(PiiType::VnNdd));
        assert_eq!(
            PiiType::from_presidio("VN_ADDRESS"),
            Some(PiiType::VnAddress)
        );
        assert_eq!(
            PiiType::from_presidio("VN_BANK_ACCOUNT"),
            Some(PiiType::VnBankAccount)
        );
    }

    #[test]
    fn pii_type_from_presidio_unknown() {
        assert_eq!(PiiType::from_presidio("UNKNOWN_TYPE"), None);
    }

    #[test]
    fn pii_type_metric_labels_are_stable() {
        assert_eq!(PiiType::CreditCard.as_metric_label(), "credit_card");
        assert_eq!(PiiType::EmailAddress.as_metric_label(), "email_address");
        assert_eq!(PiiType::UsSsn.as_metric_label(), "us_ssn");
        assert_eq!(PiiType::VnCccd.as_metric_label(), "vn_cccd");
        assert_eq!(PiiType::VnBankAccount.as_metric_label(), "vn_bank_account");
    }

    #[test]
    fn restoration_map_insert_get() {
        let mut map = RestorationMap::default();
        map.insert("<EMAIL_1>".into(), "user@example.com".into());
        assert_eq!(map.get("<EMAIL_1>"), Some("user@example.com"));
        assert_eq!(map.get("<EMAIL_2>"), None);
    }

    #[test]
    fn restoration_map_iter() {
        let mut map = RestorationMap::default();
        map.insert("<A>".into(), "a".into());
        map.insert("<B>".into(), "b".into());
        let mut items: Vec<_> = map.iter().collect();
        items.sort();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn restore_substitutes_placeholders() {
        let mut map = RestorationMap::default();
        map.insert("<EMAIL_1>".into(), "user@example.com".into());
        map.insert("<PHONE_1>".into(), "+1234567890".into());
        let result = restore("Send to <EMAIL_1> or call <PHONE_1>", &map);
        assert_eq!(result, "Send to user@example.com or call +1234567890");
    }

    #[test]
    fn restore_no_placeholders_unchanged() {
        let map = RestorationMap::default();
        let result = restore("No placeholders here", &map);
        assert_eq!(result, "No placeholders here");
    }

    #[test]
    fn vn_audit_redactors_preserve_safe_display_shapes() {
        assert_eq!(
            vn::redact_for_audit::<vn::VnMst>("0312345678"),
            "03******78"
        );
        assert_eq!(
            vn::redact_for_audit::<vn::VnCccd>("031234567678"),
            "031******678"
        );
        assert_eq!(
            vn::redact_for_audit::<vn::VnPhone>("0901234567"),
            "09***4567"
        );
        assert_eq!(
            vn::redact_for_audit::<vn::VnBankAccount>("1234567890"),
            "***7890"
        );
    }

    #[test]
    fn sanitize_known_error_codes() {
        assert_eq!(
            sanitize_sidecar_error_message(r#"{"detail":"redaction_internal_error"}"#),
            "redaction_internal_error"
        );
        assert_eq!(
            sanitize_sidecar_error_message("validation_error"),
            "validation_error"
        );
    }

    #[test]
    fn sanitize_unknown_messages() {
        assert_eq!(
            sanitize_sidecar_error_message("some unknown error with @leak.com"),
            "sidecar_returned_unrecognized_message_redacted"
        );
    }

    #[test]
    fn redact_error_display() {
        let err = RedactError::SidecarUnreachable {
            reason: "connection refused".into(),
        };
        assert!(err.to_string().contains("unreachable"));

        let err = RedactError::SidecarTimeout { waited_ms: 2000 };
        assert!(err.to_string().contains("2000"));

        let err = RedactError::SidecarError {
            status: 500,
            message: "internal".into(),
        };
        assert!(err.to_string().contains("500"));

        let err = RedactError::InvalidPrompt {
            reason: "too large".into(),
        };
        assert!(err.to_string().contains("too large"));
    }

    #[test]
    fn build_placeholder_map_deterministic() {
        let body = SidecarResponse {
            redacted_text: "Hello <EMAIL_ADDRESS_1> and <EMAIL_ADDRESS_2>".into(),
            items: vec![
                SidecarItem {
                    entity: "EMAIL_ADDRESS".into(),
                    start: 19,
                    end: 33,
                    original: "bob@example.com".into(),
                },
                SidecarItem {
                    entity: "EMAIL_ADDRESS".into(),
                    start: 6,
                    end: 20,
                    original: "alice@x.com".into(),
                },
            ],
            allowlist_hit_count: 0,
        };
        // Even though items are out of order, the result should be sorted by start.
        let (text, map, counts) = build_placeholder_map_and_counts("test", &body);
        assert_eq!(text, "Hello <EMAIL_ADDRESS_1> and <EMAIL_ADDRESS_2>");
        assert_eq!(counts.get(&PiiType::EmailAddress), Some(&2));
        // First by position (start=6) should be EMAIL_ADDRESS_1.
        assert_eq!(map.get("<EMAIL_ADDRESS_1>"), Some("alice@x.com"));
        assert_eq!(map.get("<EMAIL_ADDRESS_2>"), Some("bob@example.com"));
    }
}
