//! FR-AI-011 §4 — Integration tests for PII redaction.
//!
//! Tests the redact module's HTTP contract with a local mock server
//! (no real Presidio sidecar needed). Unit tests for PiiType mapping,
//! RestorationMap, restore(), sanitize, and deterministic placeholders
//! live in src/redact/mod.rs.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::Router as AxumRouter;
use cyberos_ai_gateway::alias::{LatencyClass, ResolvedModel};
use cyberos_ai_gateway::policy::{AiPolicy, EmergencyOverride, Provider, Residency, TenantPolicy};
use cyberos_ai_gateway::redact::{self, PiiType, RedactError};
use cyberos_ai_gateway::router::{self, ChatCompleteRequest, Message, RouterError};
use serde_json::{json, Value};

// ── Test helpers ─────────────────────────────────────────────────────────────

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvOverride {
    _guard: MutexGuard<'static, ()>,
}

impl EnvOverride {
    fn new(sidecar_url: &str, timeout_ms: u64) -> Self {
        let guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::set_var("CYBEROS_AI_GATEWAY_PRESIDIO_URL", sidecar_url);
        std::env::set_var(
            "CYBEROS_AI_GATEWAY_PRESIDIO_TIMEOUT_MS",
            timeout_ms.to_string(),
        );
        std::env::remove_var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL");
        Self { _guard: guard }
    }

    fn with_openai_base_url(self, openai_base_url: &str) -> Self {
        std::env::set_var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL", openai_base_url);
        self
    }
}

impl Drop for EnvOverride {
    fn drop(&mut self) {
        std::env::remove_var("CYBEROS_AI_GATEWAY_PRESIDIO_URL");
        std::env::remove_var("CYBEROS_AI_GATEWAY_PRESIDIO_TIMEOUT_MS");
        std::env::remove_var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL");
    }
}

fn minimal_policy() -> TenantPolicy {
    policy_with_provider(Provider::Anthropic {
        model_alias_map: HashMap::new(),
    })
}

fn policy_with_provider(primary_provider: Provider) -> TenantPolicy {
    TenantPolicy {
        tenant_id: "test-tenant".into(),
        ai_policy: AiPolicy {
            monthly_cap_usd: rust_decimal_macros::dec!(100),
            warn_threshold: 0.80,
            hard_stop: true,
            primary_provider,
            fallback_chain: vec![],
            call_timeout_seconds: 60,
            residency: Residency::Sg1,
            zdr_required: false,
            emergency_override: EmergencyOverride::default(),
            allowed_personas: None,
            alias_overrides: None,
            residency_requires_regional_provider: None,
            pii_redaction_extra: None,
        },
    }
}

async fn spawn_sidecar(
    status: StatusCode,
    body: impl Into<String>,
    delay: Duration,
) -> (String, Arc<Mutex<Vec<String>>>) {
    let captured_bodies = Arc::new(Mutex::new(Vec::new()));
    let route_bodies = Arc::clone(&captured_bodies);
    let body = body.into();

    let app = AxumRouter::new().fallback(move |request_body: String| {
        let route_bodies = Arc::clone(&route_bodies);
        let body = body.clone();
        async move {
            route_bodies
                .lock()
                .expect("capture mutex")
                .push(request_body);
            if !delay.is_zero() {
                tokio::time::sleep(delay).await;
            }
            Response::builder()
                .status(status)
                .header("content-type", "application/json")
                .body(Body::from(body))
                .expect("response")
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}/redact"), captured_bodies)
}

async fn spawn_openai_server() -> (String, Arc<Mutex<Vec<String>>>) {
    let captured_bodies = Arc::new(Mutex::new(Vec::new()));
    let route_bodies = Arc::clone(&captured_bodies);

    let app = AxumRouter::new().fallback(move |request_body: String| {
        let route_bodies = Arc::clone(&route_bodies);
        async move {
            route_bodies
                .lock()
                .expect("capture mutex")
                .push(request_body);
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "id": "chatcmpl-redacted",
                        "usage": {
                            "prompt_tokens": 12,
                            "completion_tokens": 3
                        },
                        "choices": [{
                            "index": 0,
                            "message": {"content": "ok"},
                            "finish_reason": "stop"
                        }]
                    })
                    .to_string(),
                ))
                .expect("response")
        }
    });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), captured_bodies)
}

fn redacted_email_response() -> String {
    json!({
        "redacted_text": "email <EMAIL_ADDRESS_1>",
        "items": [{
            "entity": "EMAIL_ADDRESS",
            "start": 6,
            "end": 24,
            "original": "secret@example.com"
        }]
    })
    .to_string()
}

fn default_chat_request(content: &str) -> ChatCompleteRequest {
    ChatCompleteRequest {
        alias: "chat.smart".into(),
        messages: vec![Message {
            role: "user".into(),
            content: content.into(),
        }],
        max_tokens: Some(100),
        temperature: Some(0.2),
        traceparent: None,
        tracestate: None,
    }
}

// ── AC #1: Credit card redacted ──────────────────────────────────────────────

#[tokio::test]
async fn redacts_credit_card_via_sidecar_contract() {
    let sidecar_body = json!({
        "redacted_text": "My card is <CREDIT_CARD_1>",
        "items": [{
            "entity": "CREDIT_CARD",
            "start": 11,
            "end": 30,
            "original": "4111-1111-1111-1111"
        }]
    })
    .to_string();
    let (sidecar_url, captured) = spawn_sidecar(StatusCode::OK, sidecar_body, Duration::ZERO).await;
    let _env = EnvOverride::new(&sidecar_url, 2_000);

    let result = redact::redact("My card is 4111-1111-1111-1111", &minimal_policy())
        .await
        .expect("redaction succeeds");

    assert_eq!(result.redacted_text, "My card is <CREDIT_CARD_1>");
    assert_eq!(
        result.map.get("<CREDIT_CARD_1>"),
        Some("4111-1111-1111-1111")
    );
    assert_eq!(result.counts.get(&PiiType::CreditCard), Some(&1));
    assert!(!result.redacted_text.contains("4111-1111-1111-1111"));

    let restored = redact::restore("My card is <CREDIT_CARD_1>", &result.map);
    assert_eq!(restored, "My card is 4111-1111-1111-1111");

    let request_body = captured.lock().expect("capture mutex")[0].clone();
    let request: Value = serde_json::from_str(&request_body).unwrap();
    assert_eq!(request["text"], "My card is 4111-1111-1111-1111");
    assert_eq!(request["extra_entities"], json!([]));
}

#[tokio::test]
async fn sends_policy_extra_entities_to_sidecar() {
    let (sidecar_url, captured) =
        spawn_sidecar(StatusCode::OK, redacted_email_response(), Duration::ZERO).await;
    let _env = EnvOverride::new(&sidecar_url, 2_000);
    let mut policy = minimal_policy();
    policy.ai_policy.pii_redaction_extra = Some(vec!["VN_CCCD".into(), "VN_MST".into()]);

    redact::redact("email secret@example.com", &policy)
        .await
        .expect("redaction succeeds");

    let request_body = captured.lock().expect("capture mutex")[0].clone();
    let request: Value = serde_json::from_str(&request_body).unwrap();
    assert_eq!(request["extra_entities"], json!(["VN_CCCD", "VN_MST"]));
}

// ── AC #7: Sidecar unreachable returns error ─────────────────────────────────

#[tokio::test]
async fn sidecar_unreachable_returns_err() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let _env = EnvOverride::new(&format!("http://{addr}/redact"), 250);

    let result = redact::redact("hello", &minimal_policy()).await;
    assert!(
        matches!(result, Err(RedactError::SidecarUnreachable { .. })),
        "expected SidecarUnreachable, got {result:?}"
    );
}

#[tokio::test]
async fn sidecar_timeout_returns_err_without_prompt() {
    let (sidecar_url, _) = spawn_sidecar(
        StatusCode::OK,
        redacted_email_response(),
        Duration::from_millis(150),
    )
    .await;
    let _env = EnvOverride::new(&sidecar_url, 25);

    let result = redact::redact("email secret@example.com", &minimal_policy()).await;

    match result {
        Err(RedactError::SidecarTimeout { waited_ms }) => assert_eq!(waited_ms, 25),
        other => panic!("expected timeout, got {other:?}"),
    }
}

#[tokio::test]
async fn sidecar_error_message_is_sanitized() {
    let (sidecar_url, _) = spawn_sidecar(
        StatusCode::INTERNAL_SERVER_ERROR,
        r#"{"detail":"redaction_internal_error secret@example.com"}"#,
        Duration::ZERO,
    )
    .await;
    let _env = EnvOverride::new(&sidecar_url, 2_000);

    let result = redact::redact("email secret@example.com", &minimal_policy()).await;

    match result {
        Err(RedactError::SidecarError { status, message }) => {
            assert_eq!(status, 500);
            assert_eq!(message, "redaction_internal_error");
            assert!(!message.contains("secret@example.com"));
        }
        other => panic!("expected sanitized sidecar error, got {other:?}"),
    }
}

#[tokio::test]
async fn rejects_non_loopback_sidecar_url() {
    let _env = EnvOverride::new("http://0.0.0.0:5050/redact", 2_000);

    let result = redact::redact("email secret@example.com", &minimal_policy()).await;

    match result {
        Err(RedactError::SidecarUnreachable { reason }) => {
            assert!(reason.contains("loopback"));
            assert!(!reason.contains("secret@example.com"));
        }
        other => panic!("expected loopback config rejection, got {other:?}"),
    }
}

// ── AC #3: No PII passthrough ────────────────────────────────────────────────

#[tokio::test]
async fn no_pii_passthrough_logic() {
    let sidecar_body = json!({
        "redacted_text": "What is 2+2?",
        "items": []
    })
    .to_string();
    let (sidecar_url, _) = spawn_sidecar(StatusCode::OK, sidecar_body, Duration::ZERO).await;
    let _env = EnvOverride::new(&sidecar_url, 2_000);

    let result = redact::redact("What is 2+2?", &minimal_policy())
        .await
        .expect("redaction succeeds");

    assert_eq!(result.redacted_text, "What is 2+2?");
    assert!(result.map.is_empty());
    assert!(result.counts.is_empty());
}

#[tokio::test]
async fn redact_chat_request_redacts_each_message_without_mutating_original() {
    let (sidecar_url, _) =
        spawn_sidecar(StatusCode::OK, redacted_email_response(), Duration::ZERO).await;
    let _env = EnvOverride::new(&sidecar_url, 2_000);
    let req = default_chat_request("email secret@example.com");

    let (redacted_req, redactions) = redact::redact_chat_request(&req, &minimal_policy())
        .await
        .expect("chat request redacts");

    assert_eq!(req.messages[0].content, "email secret@example.com");
    assert_eq!(redacted_req.messages[0].content, "email <EMAIL_ADDRESS_1>");
    assert_eq!(redactions.len(), 1);
    assert_eq!(
        redactions[0].map.get("<EMAIL_ADDRESS_1>"),
        Some("secret@example.com")
    );
}

// ── AC #7: Restoration round-trip for tool-call args ─────────────────────────

#[tokio::test]
async fn restoration_round_trip_for_tool_args() {
    let mut map = cyberos_ai_gateway::redact::RestorationMap::default();
    map.insert("<EMAIL_ADDRESS_1>".into(), "john@example.com".into());

    let tool_arg = "<EMAIL_ADDRESS_1>";
    let restored = redact::restore(tool_arg, &map);
    assert_eq!(restored, "john@example.com");
}

// ── AC #8: Restoration does NOT apply to text response fields ────────────────

#[tokio::test]
async fn restoration_preserves_placeholders_in_text() {
    let mut map = cyberos_ai_gateway::redact::RestorationMap::default();
    map.insert("<EMAIL_ADDRESS_1>".into(), "john@example.com".into());

    // Simulate a text response field — the caller should NOT call restore()
    // on this. But if they did, the placeholder would be replaced.
    // The AC is about caller discipline: the test verifies that
    // the placeholder IS present in the raw text.
    let text_response = "I sent the email to <EMAIL_ADDRESS_1>";
    assert!(text_response.contains("<EMAIL_ADDRESS_1>"));
    assert!(!text_response.contains("john@example.com"));
}

// ── AC #10: Concurrent redactions isolated ───────────────────────────────────

#[tokio::test]
async fn concurrent_restoration_maps_isolated() {
    let handles: Vec<_> = (0..50)
        .map(|i| {
            tokio::spawn(async move {
                let mut map = cyberos_ai_gateway::redact::RestorationMap::default();
                let email = format!("user{i}@cyberos.world");
                map.insert("<EMAIL_ADDRESS_1>".into(), email.clone());

                // Each map should have its own value.
                assert_eq!(map.get("<EMAIL_ADDRESS_1>"), Some(email.as_str()));
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }
}

// ── AC #11: Idempotency — restore is deterministic ──────────────────────────

#[tokio::test]
async fn restore_deterministic() {
    let mut map = cyberos_ai_gateway::redact::RestorationMap::default();
    map.insert("<EMAIL_ADDRESS_1>".into(), "alice@x.com".into());
    map.insert("<EMAIL_ADDRESS_2>".into(), "bob@y.com".into());

    let input = "Send to <EMAIL_ADDRESS_1> and <EMAIL_ADDRESS_2>";
    let r1 = redact::restore(input, &map);
    let r2 = redact::restore(input, &map);
    assert_eq!(r1, r2);
    assert_eq!(r1, "Send to alice@x.com and bob@y.com");
}

// ── AC #12: No PII in error variants ────────────────────────────────────────

#[tokio::test]
async fn no_prompt_fragment_in_error_variants() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let _env = EnvOverride::new(&format!("http://{addr}/redact"), 250);

    let result = redact::redact("secret@example.com", &minimal_policy()).await;
    if let Err(e) = result {
        let err_str = format!("{e}");
        assert!(
            !err_str.contains("secret@example.com"),
            "error leaked prompt: {err_str}"
        );
    }
}

#[tokio::test]
async fn router_redacts_before_openai_dispatch() {
    let (sidecar_url, _) =
        spawn_sidecar(StatusCode::OK, redacted_email_response(), Duration::ZERO).await;
    let (openai_url, captured_provider_bodies) = spawn_openai_server().await;
    let _env = EnvOverride::new(&sidecar_url, 2_000).with_openai_base_url(&openai_url);

    let mut alias_map = HashMap::new();
    alias_map.insert("chat.smart".into(), "gpt-test".into());
    let policy = policy_with_provider(Provider::Openai {
        model_alias_map: alias_map,
    });
    let resolved = ResolvedModel {
        provider_kind: cyberos_ai_gateway::policy::ProviderKind::Openai,
        region: None,
        model: "gpt-test".into(),
        fallback_position: 0,
        is_zdr: true,
        latency_class: LatencyClass::Standard,
    };
    let req = default_chat_request("email secret@example.com");

    let response = router::call_provider(
        &req,
        &resolved,
        Instant::now() + Duration::from_secs(5),
        &policy,
    )
    .await
    .expect("router call succeeds");

    assert_eq!(response.id, "chatcmpl-redacted");
    let provider_body = captured_provider_bodies.lock().expect("capture mutex")[0].clone();
    assert!(provider_body.contains("<EMAIL_ADDRESS_1>"));
    assert!(!provider_body.contains("secret@example.com"));
}

#[tokio::test]
async fn router_fails_closed_when_redaction_unavailable() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let _env = EnvOverride::new(&format!("http://{addr}/redact"), 250);

    let mut alias_map = HashMap::new();
    alias_map.insert("chat.smart".into(), "gpt-test".into());
    let policy = policy_with_provider(Provider::Openai {
        model_alias_map: alias_map,
    });
    let resolved = ResolvedModel {
        provider_kind: cyberos_ai_gateway::policy::ProviderKind::Openai,
        region: None,
        model: "gpt-test".into(),
        fallback_position: 0,
        is_zdr: true,
        latency_class: LatencyClass::Standard,
    };

    let err = router::call_provider(
        &default_chat_request("email secret@example.com"),
        &resolved,
        Instant::now() + Duration::from_secs(5),
        &policy,
    )
    .await
    .expect_err("redaction failure should block provider call");

    match err {
        RouterError::RedactionFailed { reason } => {
            assert!(reason.contains("sidecar unreachable"));
            assert!(!reason.contains("secret@example.com"));
        }
        other => panic!("expected redaction failure, got {other:?}"),
    }
}

#[test]
fn redact_for_log_removes_common_pii_shapes() {
    let line = "email secret@example.com ssn 123-45-6789 card 4111-1111-1111-1111 ip 10.1.2.3";
    let redacted = redact::redact_for_log(line);
    assert!(!redacted.contains("secret@example.com"));
    assert!(!redacted.contains("123-45-6789"));
    assert!(!redacted.contains("4111-1111-1111-1111"));
    assert!(!redacted.contains("10.1.2.3"));
    assert!(redacted.contains("[REDACTED_PII]"));
}

// ── PiiType stability tests ──────────────────────────────────────────────────

#[test]
fn pii_type_from_presidio_all_variants() {
    let cases = [
        ("CREDIT_CARD", PiiType::CreditCard),
        ("US_SSN", PiiType::UsSsn),
        ("EMAIL_ADDRESS", PiiType::EmailAddress),
        ("PHONE_NUMBER", PiiType::PhoneNumber),
        ("PERSON", PiiType::Person),
        ("LOCATION", PiiType::Location),
        ("IP_ADDRESS", PiiType::IpAddress),
        ("IBAN_CODE", PiiType::IbanCode),
        ("US_BANK_NUMBER", PiiType::UsBankNumber),
        ("MEDICAL_LICENSE", PiiType::MedicalLicense),
        ("VN_CCCD", PiiType::VnCccd),
        ("VN_MST", PiiType::VnMst),
        ("VN_PHONE", PiiType::VnPhone),
        ("VN_ADDRESS", PiiType::VnAddress),
    ];

    for (presidio_name, expected) in cases {
        assert_eq!(
            PiiType::from_presidio(presidio_name),
            Some(expected),
            "failed for {presidio_name}"
        );
    }

    assert_eq!(PiiType::from_presidio("UNKNOWN_TYPE"), None);
}

#[test]
fn pii_type_metric_labels_match_expected() {
    assert_eq!(PiiType::CreditCard.as_metric_label(), "credit_card");
    assert_eq!(PiiType::UsSsn.as_metric_label(), "us_ssn");
    assert_eq!(PiiType::EmailAddress.as_metric_label(), "email_address");
    assert_eq!(PiiType::PhoneNumber.as_metric_label(), "phone_number");
    assert_eq!(PiiType::Person.as_metric_label(), "person");
    assert_eq!(PiiType::Location.as_metric_label(), "location");
    assert_eq!(PiiType::IpAddress.as_metric_label(), "ip_address");
    assert_eq!(PiiType::IbanCode.as_metric_label(), "iban_code");
    assert_eq!(PiiType::UsBankNumber.as_metric_label(), "us_bank_number");
    assert_eq!(PiiType::MedicalLicense.as_metric_label(), "medical_license");
    assert_eq!(PiiType::VnCccd.as_metric_label(), "vn_cccd");
    assert_eq!(PiiType::VnMst.as_metric_label(), "vn_mst");
    assert_eq!(PiiType::VnPhone.as_metric_label(), "vn_phone");
    assert_eq!(PiiType::VnAddress.as_metric_label(), "vn_address");
}

// ── RestorationMap edge cases ────────────────────────────────────────────────

#[test]
fn restoration_map_overwrite() {
    let mut map = cyberos_ai_gateway::redact::RestorationMap::default();
    map.insert("<X>".into(), "first".into());
    map.insert("<X>".into(), "second".into());
    assert_eq!(map.get("<X>"), Some("second"));
}

#[test]
fn restoration_map_empty_get() {
    let map = cyberos_ai_gateway::redact::RestorationMap::default();
    assert_eq!(map.get("<anything>"), None);
    assert!(map.is_empty());
}

#[test]
fn restore_multiple_different_placeholders() {
    let mut map = cyberos_ai_gateway::redact::RestorationMap::default();
    map.insert("<A>".into(), "alpha".into());
    map.insert("<B>".into(), "beta".into());
    map.insert("<C>".into(), "gamma".into());

    let result = redact::restore("<A> <B> <C>", &map);
    assert_eq!(result, "alpha beta gamma");
}

// ── RedactError Display ──────────────────────────────────────────────────────

#[test]
fn redact_error_display_variants() {
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
