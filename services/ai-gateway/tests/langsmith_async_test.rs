//! FR-OBS-004 — LangSmith export must not block the gateway hot path.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::body::Body;
use axum::http::{Response, StatusCode};
use axum::Router as AxumRouter;
use cyberos_ai_gateway::langsmith::client::LangSmithConfig;
use cyberos_ai_gateway::langsmith::{self, LangSmithMetadata, RedactedPrompt, RedactedResponse};
use cyberos_ai_gateway::policy::{AiPolicy, EmergencyOverride, Provider, Residency, TenantPolicy};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";

#[tokio::test]
async fn export_returns_before_slow_langsmith_response() {
    let mock = SlowLangSmith::start(Duration::from_millis(250)).await;
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
        "export should spawn and return before LangSmith responds"
    );
    mock.wait_for_request().await;
}

#[derive(Debug)]
struct SlowLangSmith {
    base_url: String,
    captured_count: Arc<Mutex<usize>>,
}

impl SlowLangSmith {
    async fn start(delay: Duration) -> Self {
        let captured_count = Arc::new(Mutex::new(0usize));
        let route_count = Arc::clone(&captured_count);
        let app = AxumRouter::new().fallback(move |_body: String| {
            let route_count = Arc::clone(&route_count);
            async move {
                *route_count.lock().expect("capture mutex") += 1;
                tokio::time::sleep(delay).await;
                Response::builder()
                    .status(StatusCode::OK)
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
            captured_count,
        }
    }

    fn config(&self) -> LangSmithConfig {
        LangSmithConfig::new(&self.base_url, "test-token", Duration::from_secs(2))
    }

    async fn wait_for_request(&self) {
        for _ in 0..50 {
            if *self.captured_count.lock().expect("capture mutex") > 0 {
                return;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        panic!("expected slow LangSmith mock to receive a request");
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
        tool_calls: vec![],
    }
}
