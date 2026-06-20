//! The AI Gateway HTTP serving surface — the axum listener that ties the existing pipeline modules
//! (policy loader, alias resolver, router/provider call) into a request handler.
//!
//! Until now the gateway shipped as a library plus the operator CLI: every module existed (alias, policy,
//! redact, router, cost ledger, otel) but nothing bound them behind an HTTP endpoint, so several FRs that
//! say "before binding the HTTP server" referred to a listener that did not exist. This module is that
//! listener. It is also the surface FR-OBS-003 (RED middleware), FR-OBS-004 (LangSmith export), and
//! FR-OBS-005 (TraceContext) attach to.
//!
//! Two seams keep the handler testable and runnable without external systems:
//!   - `PolicySource` — production resolves per-tenant policy via the FR-AI-005 loader; tests inject a
//!     fixed policy.
//!   - `ChatBackend` — the provider call. The real provider adapters (FR-AI-008 Anthropic / OpenAI /
//!     Bedrock) are still stubs, so `EchoBackend` is the in-repo backend that lets the gateway return a
//!     completion for local development, the OBS correlation path, and tests. A real backend that drives
//!     `router::call_provider` is wired when the provider adapters land.

use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{rejection::JsonRejection, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::langsmith::{self, LangSmithMetadata, RedactedPrompt, RedactedResponse};
use crate::policy::TenantPolicy;
use crate::redact;
use crate::router::types::{
    CacheState, ChatCompleteRequest, Choice, FinishReason, Message, ProviderResponse, ProviderUsage,
};

/// Resolves a tenant's policy. Production uses the FR-AI-005 cached loader; tests inject a fixed policy.
/// `Debug` is required so `GatewayState` satisfies the crate's `missing_debug_implementations` lint.
#[async_trait]
pub trait PolicySource: Send + Sync + std::fmt::Debug {
    async fn for_tenant(&self, tenant_id: &str) -> Result<Arc<TenantPolicy>, String>;
}

/// The production policy source: the FR-AI-005 loader (must be `init_loader`-ed at boot).
#[derive(Debug, Default)]
pub struct LoaderPolicySource;

#[async_trait]
impl PolicySource for LoaderPolicySource {
    async fn for_tenant(&self, tenant_id: &str) -> Result<Arc<TenantPolicy>, String> {
        crate::policy::load_for_tenant(tenant_id)
            .await
            .map_err(|e| format!("{e:?}"))
    }
}

/// The provider call. Returns a `ProviderResponse` or an error string the handler maps to HTTP 502.
#[async_trait]
pub trait ChatBackend: Send + Sync + std::fmt::Debug {
    async fn complete(&self, req: &ChatCompleteRequest, model: &str) -> Result<ProviderResponse, String>;
}

/// In-repo backend that echoes the last user message. The real provider adapters (FR-AI-008) are still
/// stubs, so this is what lets the gateway return a completion for local dev, the OBS correlation path,
/// and tests - deterministic, no API key, no network.
#[derive(Debug, Default)]
pub struct EchoBackend;

#[async_trait]
impl ChatBackend for EchoBackend {
    async fn complete(&self, req: &ChatCompleteRequest, _model: &str) -> Result<ProviderResponse, String> {
        let last_user = req
            .messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_default();
        let content = format!("echo: {last_user}");
        let completion_tokens = content.split_whitespace().count().max(1) as u32;
        let prompt_tokens = req
            .messages
            .iter()
            .map(|m| m.content.split_whitespace().count() as u32)
            .sum();
        Ok(ProviderResponse {
            id: format!("echo-{}", uuid::Uuid::new_v4()),
            usage: ProviderUsage {
                prompt_tokens,
                completion_tokens,
                cached_input_tokens: 0,
            },
            choices: vec![Choice {
                index: 0,
                content,
                tool_calls: vec![],
                finish_reason: FinishReason::Stop,
            }],
            finish_reason: FinishReason::Stop,
            latency_ms: 0,
            cache_state: CacheState::None,
            attempts: vec![],
        })
    }
}

/// The shared state behind every request.
#[derive(Clone, Debug)]
pub struct GatewayState {
    pub policy: Arc<dyn PolicySource>,
    pub backend: Arc<dyn ChatBackend>,
}

impl GatewayState {
    /// The production wiring: the FR-AI-005 loader plus the echo backend (until FR-AI-008 providers land).
    pub fn production() -> Self {
        Self {
            policy: Arc::new(LoaderPolicySource),
            backend: Arc::new(EchoBackend),
        }
    }
}

/// One message in the wire request. `ChatCompleteRequest::Message` is not `Deserialize`, so the HTTP body
/// has its own DTO that maps onto it.
#[derive(Debug, Deserialize)]
pub struct ApiMessage {
    pub role: String,
    pub content: String,
}

/// The `POST /v1/chat` request body.
#[derive(Debug, Deserialize)]
pub struct ApiChatRequest {
    pub alias: String,
    pub messages: Vec<ApiMessage>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub temperature: Option<f32>,
}

/// The `POST /v1/chat` response body.
#[derive(Debug, Serialize)]
pub struct ApiChatResponse {
    pub id: String,
    pub model: String,
    pub content: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub finish_reason: String,
}

/// Build the gateway router: liveness, a metrics stub (RED exports via OTLP, not a scrape), and the chat
/// endpoint, with the FR-OBS-003 RED middleware wrapping every route.
pub fn build_router(state: GatewayState) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/metrics", get(|| async { "# cyberos-ai-gateway: RED metrics export via OTLP\n" }))
        .route("/v1/chat", post(chat))
        // FR-OBS-005: ensure every request carries a trace context (extract or generate) and echo it on
        // the response. FR-OBS-003 (ADR-OBS-003-001): tenant_ctx stamps the request's tenant onto the
        // response; red_mw (outer) reads it for the metric's tenant_id label. Same wiring as auth/memory.
        .route_layer(axum::middleware::from_fn(trace_ctx))
        .route_layer(axum::middleware::from_fn(tenant_ctx))
        .layer(axum::middleware::from_fn_with_state(
            cyberos_obs_sdk::RedState::new("ai-gateway"),
            cyberos_obs_sdk::red_mw,
        ))
        .with_state(state)
}

/// FR-OBS-003 - stamp the request's tenant (from `x-tenant-id`) onto the response so `red_mw` can label
/// the metric with the real tenant; absent, the metric falls back to "unknown".
async fn tenant_ctx(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let tenant = req
        .headers()
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .map(str::to_string);
    let mut response = next.run(req).await;
    if let Some(t) = tenant {
        response
            .extensions_mut()
            .insert(cyberos_obs_sdk::TenantCtx(t));
    }
    response
}

/// The canonical W3C trace id + span id for the current request. Stamped by `trace_ctx` as a request
/// extension so the handler (and the LangSmith export, FR-OBS-004) read one consistent value.
#[derive(Debug, Clone)]
pub struct RequestTrace {
    pub trace_id: String,
    pub span_id: String,
}

/// Generate a fresh W3C trace context (16 random bytes trace id, 8 random bytes span id, sampled).
fn generate_trace_context() -> cyberos_obs_sdk::TraceContext {
    let mut rng = rand::thread_rng();
    let trace_id: String = (0..16).map(|_| format!("{:02x}", rng.gen::<u8>())).collect();
    let span_id: String = (0..8).map(|_| format!("{:02x}", rng.gen::<u8>())).collect();
    cyberos_obs_sdk::TraceContext {
        trace_id,
        span_id,
        flags: 1,
    }
}

/// FR-OBS-005 (§1 #1, #4, #11) - ensure every request carries a trace context. Extract the incoming W3C
/// `traceparent` strictly; if it is missing or malformed, generate a fresh one (never reject - trace
/// context is operational, not security, and an attacker-supplied id is not honoured). The resolved trace
/// id is stamped as a request extension and echoed on the response `traceparent` header so a downstream
/// consumer can correlate.
async fn trace_ctx(mut req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    let tc = match cyberos_obs_sdk::extract_traceparent(req.headers()) {
        Ok(tc) => {
            cyberos_obs_sdk::record_tracecontext_extracted("extracted");
            tc
        }
        Err(cyberos_obs_sdk::ExtractError::Missing) => {
            cyberos_obs_sdk::record_tracecontext_extracted("missing_generated_new");
            generate_trace_context()
        }
        Err(cyberos_obs_sdk::ExtractError::Malformed(hash16)) => {
            cyberos_obs_sdk::record_tracecontext_extracted("malformed");
            eprintln!(
                "{{\"sev\":2,\"event\":\"malformed_traceparent\",\"hash16\":\"{hash16}\"}}"
            );
            generate_trace_context()
        }
    };
    req.extensions_mut().insert(RequestTrace {
        trace_id: tc.trace_id.clone(),
        span_id: tc.span_id.clone(),
    });
    let mut response = next.run(req).await;
    cyberos_obs_sdk::inject_traceparent(response.headers_mut(), &tc);
    response
}

/// `POST /v1/chat` - the non-streaming completion path. Pipeline: require a tenant, load its policy,
/// resolve the alias to a model, call the backend, map the provider response to the wire response.
async fn chat(
    State(st): State<GatewayState>,
    axum::Extension(req_trace): axum::Extension<RequestTrace>,
    headers: HeaderMap,
    body: Result<Json<ApiChatRequest>, JsonRejection>,
) -> Response {
    let tenant = match header(&headers, "x-tenant-id") {
        Some(t) => t,
        None => return err(StatusCode::BAD_REQUEST, "missing x-tenant-id header"),
    };
    let Json(req) = match body {
        Ok(j) => j,
        Err(e) => return err(StatusCode::BAD_REQUEST, &format!("invalid request body: {e}")),
    };
    if req.messages.is_empty() {
        return err(StatusCode::BAD_REQUEST, "messages must not be empty");
    }

    let policy = match st.policy.for_tenant(&tenant).await {
        Ok(p) => p,
        Err(e) => return err(StatusCode::NOT_FOUND, &format!("policy unavailable for tenant: {e}")),
    };

    let resolved = match crate::alias::resolve(&req.alias, &policy) {
        Ok(r) => r,
        Err(e) => return err(StatusCode::BAD_REQUEST, &format!("alias resolution failed: {e:?}")),
    };

    let ccr = ChatCompleteRequest {
        alias: req.alias.clone(),
        messages: req
            .messages
            .iter()
            .map(|m| Message {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect(),
        max_tokens: req.max_tokens,
        temperature: req.temperature,
        traceparent: header(&headers, "traceparent"),
        tracestate: header(&headers, "tracestate"),
    };

    let resp = match st.backend.complete(&ccr, &resolved.model).await {
        Ok(r) => r,
        Err(e) => return err(StatusCode::BAD_GATEWAY, &format!("provider call failed: {e}")),
    };

    let content = resp.choices.first().map(|c| c.content.clone()).unwrap_or_default();

    // FR-OBS-004 - opt-in LangSmith export of the (redacted) call, correlated by the request trace id.
    // Gated on the tenant's opt-in so the default path makes no redaction (Presidio) call; the export
    // itself is fire-and-forget, so the response is never blocked on LangSmith.
    if policy.ai_policy.langsmith_export {
        export_to_langsmith(&policy, &ccr, &resolved, &resp, &content, &req_trace).await;
    }

    let api = ApiChatResponse {
        id: resp.id,
        model: resolved.model,
        content,
        prompt_tokens: resp.usage.prompt_tokens,
        completion_tokens: resp.usage.completion_tokens,
        finish_reason: format!("{:?}", resp.finish_reason),
    };
    (StatusCode::OK, Json(api)).into_response()
}

/// Redact the prompt and response (FR-AI-011 / Presidio) and dispatch the LangSmith export (FR-OBS-004).
/// Called only when the tenant has opted in. Redaction failure skips the export (never exports raw text,
/// never fails the response). The cost is wired from the cost ledger when the non-streaming cost path lands.
async fn export_to_langsmith(
    policy: &TenantPolicy,
    ccr: &ChatCompleteRequest,
    resolved: &crate::alias::ResolvedModel,
    resp: &ProviderResponse,
    content: &str,
    trace: &RequestTrace,
) {
    let prompt_text = ccr
        .messages
        .iter()
        .filter(|m| m.role == "user")
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let (redacted_prompt, redacted_response) =
        match (redact::redact(&prompt_text, policy).await, redact::redact(content, policy).await) {
            (Ok(rp), Ok(rr)) => (rp.redacted_text, rr.redacted_text),
            _ => {
                eprintln!(
                    "{{\"sev\":2,\"event\":\"langsmith_redaction_failed_skipping_export\",\"trace_id\":\"{}\"}}",
                    trace.trace_id
                );
                return;
            }
        };

    let metadata = LangSmithMetadata {
        model_alias: ccr.alias.clone(),
        resolved_model: resolved.model.clone(),
        provider: resolved.provider_kind.as_metric_label().to_string(),
        temperature: ccr.temperature,
        max_tokens: ccr.max_tokens,
        latency_ms: resp.latency_ms,
        cost_usd: 0.0,
        persona_handle: String::new(),
        tenant_id: policy.tenant_id.clone(),
        trace_id: trace.trace_id.clone(),
    };
    langsmith::export(
        true,
        RedactedPrompt(redacted_prompt),
        RedactedResponse(redacted_response),
        metadata,
    );
}

fn header(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|h| h.to_str().ok())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn err(status: StatusCode, msg: &str) -> Response {
    (status, Json(serde_json::json!({ "error": msg }))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt; // oneshot

    /// A policy source that always errors - used for tests whose request is rejected before policy load.
    #[derive(Debug)]
    struct UnreachablePolicy;
    #[async_trait]
    impl PolicySource for UnreachablePolicy {
        async fn for_tenant(&self, _t: &str) -> Result<Arc<TenantPolicy>, String> {
            Err("policy source must not be reached in this test".into())
        }
    }

    fn test_state() -> GatewayState {
        GatewayState {
            policy: Arc::new(UnreachablePolicy),
            backend: Arc::new(EchoBackend),
        }
    }

    fn msg(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    #[tokio::test]
    async fn echo_backend_echoes_last_user_message() {
        let req = ChatCompleteRequest {
            alias: "chat.smart".into(),
            messages: vec![msg("system", "be terse"), msg("user", "hello there world")],
            max_tokens: None,
            temperature: None,
            traceparent: None,
            tracestate: None,
        };
        let resp = EchoBackend.complete(&req, "any-model").await.unwrap();
        assert_eq!(resp.choices[0].content, "echo: hello there world");
        assert_eq!(resp.choices[0].finish_reason, FinishReason::Stop);
        // "echo: hello there world" is 4 whitespace-separated words.
        assert_eq!(resp.usage.completion_tokens, 4);
        assert!(resp.id.starts_with("echo-"));
    }

    #[tokio::test]
    async fn healthz_is_ok() {
        let app = build_router(test_state());
        let res = app
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn chat_without_tenant_header_is_400() {
        let app = build_router(test_state());
        let body = r#"{"alias":"chat.smart","messages":[{"role":"user","content":"hi"}]}"#;
        let res = app
            .oneshot(
                Request::post("/v1/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn chat_with_empty_messages_is_400() {
        let app = build_router(test_state());
        let body = r#"{"alias":"chat.smart","messages":[]}"#;
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
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn chat_with_malformed_json_is_400() {
        let app = build_router(test_state());
        let res = app
            .oneshot(
                Request::post("/v1/chat")
                    .header("content-type", "application/json")
                    .header("x-tenant-id", "org:cyberskill")
                    .body(Body::from("not json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn response_carries_a_generated_traceparent_when_absent() {
        let app = build_router(test_state());
        let res = app
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let tp = res
            .headers()
            .get("traceparent")
            .expect("a traceparent must be stamped on the response");
        let s = tp.to_str().unwrap();
        assert!(
            cyberos_obs_sdk::parse_w3c_traceparent(s).is_some(),
            "generated traceparent must be valid W3C: {s}"
        );
    }

    #[tokio::test]
    async fn response_echoes_a_valid_inbound_traceparent() {
        let app = build_router(test_state());
        let valid = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let res = app
            .oneshot(
                Request::get("/healthz")
                    .header("traceparent", valid)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.headers().get("traceparent").unwrap().to_str().unwrap(),
            valid
        );
    }
}
