//! FR-OBS-004 — LangSmith export tests.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use axum::Router as AxumRouter;
use cyberos_ai_gateway::langsmith::client::{self, LangSmithConfig, LangSmithError};
use cyberos_ai_gateway::langsmith::{
    self, build_payload, LangSmithMetadata, RedactedPrompt, RedactedResponse, ToolCallTrace,
    TRUNCATION_MARKER,
};
use cyberos_ai_gateway::policy::{AiPolicy, EmergencyOverride, Provider, Residency, TenantPolicy};
use serde_json::Value;

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";
static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Clone)]
struct CapturedRequest {
    headers: HeaderMap,
    body: String,
}

#[derive(Debug, Clone)]
struct MockLangSmith {
    base_url: String,
    captured: Arc<Mutex<Vec<CapturedRequest>>>,
}

struct EnvGuard {
    _guard: MutexGuard<'static, ()>,
}

impl EnvGuard {
    fn set_region_url(key: &str, value: &str) -> Self {
        let guard = ENV_LOCK
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        std::env::remove_var("LANGSMITH_URL");
        std::env::set_var(key, value);
        std::env::set_var("LANGSMITH_API_TOKEN", "env-token");
        Self { _guard: guard }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        std::env::remove_var("LANGSMITH_URL");
        std::env::remove_var("LANGSMITH_URL_SG_1");
        std::env::remove_var("LANGSMITH_URL_EU_1");
        std::env::remove_var("LANGSMITH_URL_US_1");
        std::env::remove_var("LANGSMITH_URL_VN_1");
        std::env::remove_var("LANGSMITH_API_TOKEN");
    }
}

impl MockLangSmith {
    async fn start(status: StatusCode, delay: Duration) -> Self {
        let captured = Arc::new(Mutex::new(Vec::new()));
        let route_captured = Arc::clone(&captured);
        let app = AxumRouter::new().fallback(move |headers: HeaderMap, body: String| {
            let route_captured = Arc::clone(&route_captured);
            async move {
                route_captured
                    .lock()
                    .expect("capture mutex")
                    .push(CapturedRequest { headers, body });
                if !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }
                Response::builder()
                    .status(status)
                    .header("content-type", "application/json")
                    .body(Body::from("{}"))
                    .expect("response")
            }
        });

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        Self {
            base_url: format!("http://{addr}"),
            captured,
        }
    }

    fn config(&self) -> LangSmithConfig {
        LangSmithConfig::new(&self.base_url, "test-token", Duration::from_secs(2))
    }

    fn count(&self) -> usize {
        self.captured.lock().expect("capture mutex").len()
    }

    fn last(&self) -> CapturedRequest {
        self.captured
            .lock()
            .expect("capture mutex")
            .last()
            .expect("captured request")
            .clone()
    }
}

fn policy_with_langsmith_export(enabled: bool) -> TenantPolicy {
    TenantPolicy {
        tenant_id: "org:test".to_string(),
        tenant_jurisdiction: None,
        ai_policy: AiPolicy {
            monthly_cap_usd: rust_decimal_macros::dec!(100),
            warn_threshold: 0.8,
            hard_stop: true,
            primary_provider: Provider::Anthropic {
                model_alias_map: HashMap::new(),
            },
            fallback_chain: vec![],
            call_timeout_seconds: 60,
            residency: Residency::Sg1,
            residency_override: None,
            zdr_required: false,
            langsmith_export: enabled,
            emergency_override: EmergencyOverride::default(),
            allowed_personas: None,
            alias_overrides: None,
            residency_requires_regional_provider: None,
            pii_redaction_extra: None,
            pii_allowlist: None,
        },
    }
}

fn metadata() -> LangSmithMetadata {
    LangSmithMetadata {
        model_alias: "chat.smart".to_string(),
        resolved_model: "claude-3-5-sonnet".to_string(),
        provider: "anthropic".to_string(),
        temperature: Some(0.2),
        max_tokens: Some(100),
        latency_ms: 42,
        cost_usd: 0.0078,
        persona_handle: "cuo-cpo@0.4.1".to_string(),
        tenant_id: "org:test".to_string(),
        trace_id: TRACE_ID.to_string(),
        tool_calls: vec![ToolCallTrace {
            tool_name: "lookup_customer".to_string(),
            redacted_args: RedactedPrompt(r#"{"email":"<EMAIL_ADDRESS_1>"}"#.to_string()),
            outcome: "success".to_string(),
        }],
    }
}

#[tokio::test]
async fn opt_in_tenant_produces_langsmith_trace_with_idempotency_key() {
    let mock = MockLangSmith::start(StatusCode::OK, Duration::ZERO).await;
    let policy = policy_with_langsmith_export(true);

    let decision = langsmith::export_with_config(
        TRACE_ID,
        RedactedPrompt("user: hello".to_string()),
        RedactedResponse("assistant: hi".to_string()),
        metadata(),
        &policy,
        mock.config(),
    )
    .await;
    assert_eq!(decision, langsmith::ExportDecision::Spawned);

    wait_for_count(&mock, 1).await;
    let request = mock.last();
    assert_eq!(
        request.headers["idempotency-key"].to_str().unwrap(),
        TRACE_ID
    );
    assert_eq!(
        request.headers["authorization"].to_str().unwrap(),
        "Bearer test-token"
    );
    let body: Value = serde_json::from_str(&request.body).unwrap();
    assert_eq!(body["trace_id"], TRACE_ID);
    assert_eq!(body["metadata"]["trace_id"], TRACE_ID);
    assert_eq!(
        body["metadata"]["tool_calls"][0]["tool_name"],
        "lookup_customer"
    );
}

#[tokio::test]
async fn opt_out_tenant_does_not_export() {
    let mock = MockLangSmith::start(StatusCode::OK, Duration::ZERO).await;
    let policy = policy_with_langsmith_export(false);

    let decision = langsmith::export_with_config(
        TRACE_ID,
        RedactedPrompt("prompt".to_string()),
        RedactedResponse("response".to_string()),
        metadata(),
        &policy,
        mock.config(),
    )
    .await;

    assert_eq!(decision, langsmith::ExportDecision::DroppedOptOut);
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(mock.count(), 0);
}

#[tokio::test]
async fn server_errors_retry_three_attempts_then_drop() {
    let mock = MockLangSmith::start(StatusCode::INTERNAL_SERVER_ERROR, Duration::ZERO).await;
    let payload = build_payload(
        TRACE_ID,
        RedactedPrompt("prompt".to_string()),
        RedactedResponse("response".to_string()),
        metadata(),
    );

    let err = client::post_with_retry_with_config(&mock.config(), &payload)
        .await
        .expect_err("500 should drop after retries");
    assert!(matches!(err, LangSmithError::DroppedAfterRetries { .. }));
    assert_eq!(mock.count(), 3);
}

#[tokio::test]
async fn auth_errors_do_not_retry() {
    let mock = MockLangSmith::start(StatusCode::UNAUTHORIZED, Duration::ZERO).await;
    let payload = build_payload(
        TRACE_ID,
        RedactedPrompt("prompt".to_string()),
        RedactedResponse("response".to_string()),
        metadata(),
    );

    let err = client::post_with_retry_with_config(&mock.config(), &payload)
        .await
        .expect_err("401 should fail terminally");
    assert!(matches!(err, LangSmithError::AuthFailed));
    assert_eq!(mock.count(), 1);
}

#[test]
fn payload_truncates_redacted_prompts_and_responses_at_100kb() {
    let big = "a".repeat(110 * 1024);
    let payload = build_payload(
        TRACE_ID,
        RedactedPrompt(big.clone()),
        RedactedResponse(big),
        metadata(),
    );

    assert!(payload.prompt.ends_with(TRUNCATION_MARKER));
    assert!(payload.response.ends_with(TRUNCATION_MARKER));
    assert!(payload.prompt.len() <= 100 * 1024 + TRUNCATION_MARKER.len());
}

#[test]
fn exported_payload_contains_only_redacted_pii_shapes() {
    let payload = build_payload(
        TRACE_ID,
        RedactedPrompt("email <EMAIL_ADDRESS_1> phone <PHONE_NUMBER_1>".to_string()),
        RedactedResponse("ok".to_string()),
        metadata(),
    );
    let encoded = serde_json::to_string(&payload).unwrap();

    assert!(!encoded.contains("alice@example.com"));
    assert!(!encoded.contains("0901234567"));
    assert!(encoded.contains("<EMAIL_ADDRESS_1>"));
}

#[test]
fn self_hosted_region_defaults_and_saas_rejection_are_enforced() {
    assert_eq!(
        client::default_base_url(Residency::Sg1),
        "https://langsmith.sg-1.cyberos.world"
    );
    assert_eq!(
        client::default_base_url(Residency::Eu1),
        "https://langsmith.eu-1.cyberos.world"
    );
    let forbidden = LangSmithConfig::new(
        "https://api.smith.langchain.com",
        "test-token",
        Duration::from_secs(2),
    );
    assert!(forbidden.validate_self_hosted().is_err());
}

#[test]
fn region_env_override_uses_uppercase_residency_key() {
    let _env = EnvGuard::set_region_url("LANGSMITH_URL_EU_1", "https://eu-langsmith.example.test");

    let config = LangSmithConfig::from_env(Residency::Eu1);

    assert_eq!(config.base_url, "https://eu-langsmith.example.test");
    assert_eq!(config.token, "env-token");
}

#[tokio::test]
async fn export_is_fire_and_forget_on_slow_langsmith() {
    let mock = MockLangSmith::start(StatusCode::OK, Duration::from_millis(250)).await;
    let policy = policy_with_langsmith_export(true);
    let started = Instant::now();

    let decision = langsmith::export_with_config(
        TRACE_ID,
        RedactedPrompt("prompt".to_string()),
        RedactedResponse("response".to_string()),
        metadata(),
        &policy,
        mock.config(),
    )
    .await;

    assert_eq!(decision, langsmith::ExportDecision::Spawned);
    assert!(
        started.elapsed() < Duration::from_millis(50),
        "export should not wait for LangSmith response"
    );
    wait_for_count(&mock, 1).await;
}

async fn wait_for_count(mock: &MockLangSmith, expected: usize) {
    for _ in 0..50 {
        if mock.count() >= expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!(
        "expected {expected} captured requests, got {}",
        mock.count()
    );
}
