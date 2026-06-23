//! FR-AI-105 - local model providers end to end.
//!
//! One test runs in CI (it needs no server): it proves a local provider resolves through
//! `alias::resolve` without a cost-table row or a ZDR attestation - the zero-cost, inherently-ZDR
//! exemption (clauses 5 and 7).
//!
//! The other is #[ignore]d and owner-run: it drives a real completion through the whole flipped path
//! (HTTP -> handler -> alias::resolve -> RouterBackend -> router::call_provider -> the local adapter ->
//! the live server). A pass means the gateway returned a real answer, not the EchoBackend echo.
//!
//! Run the live one by hand with a model loaded locally:
//!
//!   # LM Studio (OpenAI-compatible, default http://localhost:1234):
//!   LOCAL_TEST_MODEL=qwen2.5-7b-instruct \
//!     cargo test -p cyberos-ai-gateway --test local_provider_roundtrip_test -- --ignored --nocapture
//!
//!   # Ollama (default http://localhost:11434):
//!   LOCAL_TEST_KIND=ollama LOCAL_TEST_MODEL=llama3.1:8b \
//!     cargo test -p cyberos-ai-gateway --test local_provider_roundtrip_test -- --ignored --nocapture

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use cyberos_ai_gateway::policy::schema::{
    AiPolicy, Provider, ProviderKind, Residency, TenantPolicy,
};
use cyberos_ai_gateway::server::{build_router, GatewayState, PolicySource, RouterBackend};
use tower::ServiceExt;

/// Build a tenant policy whose primary provider is a local one, with `chat.smart` mapped to `model`.
/// `kind` is "ollama" or "local_openai" (anything else is treated as local_openai).
fn local_policy(kind: &str, model: &str) -> TenantPolicy {
    let mut map = HashMap::new();
    map.insert("chat.smart".to_string(), model.to_string());
    let primary = if kind == "ollama" {
        Provider::Ollama {
            model_alias_map: map,
        }
    } else {
        Provider::LocalOpenai {
            model_alias_map: map,
        }
    };
    TenantPolicy {
        tenant_id: "org:cyberskill".to_string(),
        ai_policy: AiPolicy {
            // 150.00 USD cap; never reached for a local (zero-cost) provider, and the HTTP handler does
            // not run the cost ledger anyway.
            monthly_cap_usd: "150".parse().unwrap(),
            warn_threshold: 0.8,
            hard_stop: true,
            primary_provider: primary,
            fallback_chain: vec![],
            call_timeout_seconds: 120,
            // zdr_required is true on purpose: the local provider must still pass, because resolve()
            // treats local providers as inherently ZDR (clause 5).
            residency: Residency::Sg1,
            zdr_required: true,
            emergency_override: Default::default(),
            allowed_personas: None,
            alias_overrides: None,
            residency_requires_regional_provider: None,
            pii_redaction_extra: None,
            langsmith_export: false,
        },
    }
}

/// A policy source that always returns the given local policy.
#[derive(Debug)]
struct FixedPolicy(Arc<TenantPolicy>);

#[async_trait]
impl PolicySource for FixedPolicy {
    async fn for_tenant(&self, _tenant_id: &str) -> Result<Arc<TenantPolicy>, String> {
        Ok(self.0.clone())
    }
}

/// CI test (no server): a local provider resolves with no cost-table row and no ZDR attestation, and is
/// marked ZDR. This is the FR-AI-105 clause 5 + 7 exemption; it needs no global init because the local
/// path skips both lookups.
#[test]
fn local_provider_resolves_without_cost_or_zdr_fixture() {
    let policy = local_policy("local_openai", "qwen2.5-7b-instruct");
    let resolved = cyberos_ai_gateway::alias::resolve("chat.smart", &policy)
        .expect("a local provider must resolve without cost-table or ZDR fixtures");
    assert_eq!(resolved.provider_kind, ProviderKind::LocalOpenai);
    assert_eq!(resolved.model, "qwen2.5-7b-instruct");
    assert!(resolved.is_zdr, "local providers are inherently ZDR");
    assert_eq!(resolved.region, None);

    // The Ollama variant resolves the same way.
    let op = local_policy("ollama", "llama3.1:8b");
    let r2 = cyberos_ai_gateway::alias::resolve("chat.smart", &op).expect("ollama local resolves");
    assert_eq!(r2.provider_kind, ProviderKind::Ollama);
    assert_eq!(r2.model, "llama3.1:8b");
}

/// Owner-run: a real completion through the flipped serving path against a live local server.
#[tokio::test]
#[ignore = "owner-run: needs a local LM Studio/Ollama server with LOCAL_TEST_MODEL loaded"]
async fn local_provider_live_round_trip() {
    // Owner-run only: opt in by setting LOCAL_TEST_MODEL (see the module docs above). This test is
    // #[ignore]d so a plain `cargo test` skips it, but the integration job runs `--ignored`, which
    // would otherwise execute it against a local LM Studio/Ollama server that does not exist in CI
    // and fail on an environmental 502. With no model named there is nothing to talk to, so skip
    // cleanly instead of failing - mirroring how the DATABASE_URL/Redis-backed tests skip when their
    // backend is absent.
    let model = match std::env::var("LOCAL_TEST_MODEL") {
        Ok(m) => m,
        Err(_) => {
            eprintln!("LOCAL_TEST_MODEL not set; skipping owner-run live local round trip");
            return;
        }
    };
    let kind = std::env::var("LOCAL_TEST_KIND").unwrap_or_else(|_| "local_openai".to_string());

    let state = GatewayState {
        policy: Arc::new(FixedPolicy(Arc::new(local_policy(&kind, &model)))),
        backend: Arc::new(RouterBackend),
    };
    let app = build_router(state);

    let body = r#"{"alias":"chat.smart","messages":[{"role":"user","content":"Reply with exactly one word: pong"}]}"#;
    let res = app
        .oneshot(
            Request::post("/v1/chat")
                .header("content-type", "application/json")
                .header("x-tenant-id", "org:cyberskill")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    // On a non-200 the handler returns 502 with the provider error in the body. Surface it, since the
    // common causes are environmental (server not started, wrong model id, wrong kind) rather than a bug.
    assert_eq!(
        status,
        StatusCode::OK,
        "live local round trip returned {status} instead of 200.\n  provider error: {}\n  \
         checks: (1) the local server is running and reachable - LM Studio default \
         http://localhost:1234 (override LMSTUDIO_ENDPOINT), Ollama default http://localhost:11434 \
         (override OLLAMA_ENDPOINT); (2) LOCAL_TEST_MODEL={model:?} names a model that is actually \
         loaded and served; (3) for Ollama set LOCAL_TEST_KIND=ollama.",
        v.get("error")
            .and_then(|e| e.as_str())
            .unwrap_or("(no error field in body)")
    );

    let content = v["content"].as_str().unwrap_or("");
    eprintln!("live local completion ({kind} / {model}): {content:?}");
    assert!(!content.is_empty(), "live completion must be non-empty");
    assert!(
        !content.starts_with("echo:"),
        "must be the real router, not EchoBackend - dispatch flip is not in effect"
    );
    assert_eq!(
        v["model"].as_str(),
        Some(model.as_str()),
        "response model echoes the resolved local model"
    );
}
