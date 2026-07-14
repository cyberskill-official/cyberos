//! The AI Gateway HTTP serving surface — the axum listener that ties the existing pipeline modules
//! (policy loader, alias resolver, router/provider call) into a request handler.
//!
//! Until now the gateway shipped as a library plus the operator CLI: every module existed (alias, policy,
//! redact, router, cost ledger, otel) but nothing bound them behind an HTTP endpoint, so several FRs that
//! say "before binding the HTTP server" referred to a listener that did not exist. This module is that
//! listener. It is also the surface TASK-OBS-003 (RED middleware), TASK-OBS-004 (LangSmith export), and
//! TASK-OBS-005 (TraceContext) attach to.
//!
//! Two seams keep the handler testable and runnable without external systems:
//!   - `PolicySource` — production resolves per-tenant policy via the TASK-AI-005 loader; tests inject a
//!     fixed policy.
//!   - `ChatBackend` — the provider call. The real provider adapters (TASK-AI-008 Anthropic / OpenAI /
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
use tracing::Instrument;

use crate::alias::ResolvedModel;
use crate::langsmith::{self, LangSmithMetadata, RedactedPrompt, RedactedResponse};
use crate::policy::TenantPolicy;
use crate::redact;
use crate::router::types::{
    CacheState, ChatCompleteRequest, Choice, FinishReason, Message, ProviderResponse, ProviderUsage,
};

/// Resolves a tenant's policy. Production uses the TASK-AI-005 cached loader; tests inject a fixed policy.
/// `Debug` is required so `GatewayState` satisfies the crate's `missing_debug_implementations` lint.
#[async_trait]
pub trait PolicySource: Send + Sync + std::fmt::Debug {
    async fn for_tenant(&self, tenant_id: &str) -> Result<Arc<TenantPolicy>, String>;
}

/// The production policy source: the TASK-AI-005 loader (must be `init_loader`-ed at boot).
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
///
/// The handler hands the backend the resolved model (TASK-AI-006) and the tenant policy (TASK-AI-005) so a real
/// backend can drive `router::call_provider`. `EchoBackend` ignores both; `RouterBackend` uses them.
#[async_trait]
pub trait ChatBackend: Send + Sync + std::fmt::Debug {
    async fn complete(
        &self,
        req: &ChatCompleteRequest,
        resolved: &ResolvedModel,
        policy: &TenantPolicy,
    ) -> Result<ProviderResponse, String>;
}

/// In-repo backend that echoes the last user message - deterministic, no API key, no network. Since
/// TASK-AI-105 made `RouterBackend` the default serving path, this is now a dev/test-only backend (the OBS
/// correlation path and the handler tests use it); production no longer wires it.
#[derive(Debug, Default)]
pub struct EchoBackend;

#[async_trait]
impl ChatBackend for EchoBackend {
    async fn complete(
        &self,
        req: &ChatCompleteRequest,
        _resolved: &ResolvedModel,
        _policy: &TenantPolicy,
    ) -> Result<ProviderResponse, String> {
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

/// The production serving backend (TASK-AI-105 §1 #6). Drives `router::call_provider`, so a tenant whose
/// resolved provider is a real adapter - local (Ollama, LM Studio) or a keyed cloud provider - gets a real
/// completion with the router's retry and failover. The per-call deadline comes from the tenant policy's
/// `call_timeout_seconds`. Fails closed: an unreachable or erroring provider becomes an `Err`, never a
/// fabricated completion.
#[derive(Debug, Default)]
pub struct RouterBackend;

#[async_trait]
impl ChatBackend for RouterBackend {
    async fn complete(
        &self,
        req: &ChatCompleteRequest,
        resolved: &ResolvedModel,
        policy: &TenantPolicy,
    ) -> Result<ProviderResponse, String> {
        let timeout =
            std::time::Duration::from_secs(u64::from(policy.ai_policy.call_timeout_seconds));
        let deadline = std::time::Instant::now() + timeout;
        crate::router::call_provider(req, resolved, deadline, policy)
            .await
            .map_err(|e| format!("{e:?}"))
    }
}

/// The embeddings call seam (mirrors [`ChatBackend`]). Production resolves the tenant's embedding alias to a
/// provider and calls it once with no failover ([`RouterEmbedBackend`]); tests inject a stub. It returns the
/// wire response, so the handler stays a thin tenant + validation shell.
#[async_trait]
pub trait EmbedBackend: Send + Sync + std::fmt::Debug {
    async fn embed(
        &self,
        req: &ApiEmbedRequest,
        policy: &TenantPolicy,
    ) -> Result<ApiEmbedResponse, EmbedFailure>;
}

/// Why an embeddings call failed, mapped to an HTTP status by the handler.
#[derive(Debug)]
pub enum EmbedFailure {
    /// The tenant's embedding spend cap is exhausted - HTTP 402. The brain marks the row pending and backs
    /// off (it never calls a provider directly). Local providers are zero-cost, so this only fires for a
    /// paid cloud embedding provider over its cap.
    SpendCap,
    /// Alias resolution failed: unknown alias, a ZDR or residency violation, or no provider carries the
    /// embedding alias - HTTP 400. The message names the reason.
    Resolve(String),
    /// The provider was unreachable, timed out, or returned an error - HTTP 502. The brain treats this as
    /// "gateway down" and retries on the next tick.
    Provider(String),
}

/// The production embeddings backend: resolve the tenant's embedding alias (so the policy decides provider,
/// in-region model, ZDR, and residency), then call that one provider with no failover (TASK-MEMORY-123 /
/// DEC-2723). A paid cloud provider would add a cost-ledger pre-check here that returns `SpendCap` over the
/// monthly cap; local providers are zero-cost, so the cyberskill tenant never hits it.
#[derive(Debug, Default)]
pub struct RouterEmbedBackend;

#[async_trait]
impl EmbedBackend for RouterEmbedBackend {
    async fn embed(
        &self,
        req: &ApiEmbedRequest,
        policy: &TenantPolicy,
    ) -> Result<ApiEmbedResponse, EmbedFailure> {
        // The brain requests "bge-m3" (standard embedding). A caller that names an embed alias is honoured;
        // anything else maps to the standard embedding alias so the tenant policy decides the real model.
        let alias = match req.model.as_deref() {
            Some("embed.code") => "embed.code",
            _ => "embed.standard",
        };
        let resolved = crate::alias::resolve(alias, policy)
            .map_err(|e| EmbedFailure::Resolve(format!("{e:?}")))?;

        let ereq = crate::router::types::EmbedRequest {
            input: req.input.clone(),
            model: resolved.model.clone(),
        };
        let timeout =
            std::time::Duration::from_secs(u64::from(policy.ai_policy.call_timeout_seconds));
        let deadline = std::time::Instant::now() + timeout;
        let resp = crate::router::call_embed_provider(&ereq, &resolved, deadline)
            .await
            .map_err(|e| EmbedFailure::Provider(format!("{e:?}")))?;

        Ok(ApiEmbedResponse {
            embeddings: resp.embeddings,
            model: resolved.model.clone(),
            embed_model_version: resolved.model,
        })
    }
}

/// The shared state behind every request.
#[derive(Clone, Debug)]
pub struct GatewayState {
    pub policy: Arc<dyn PolicySource>,
    pub backend: Arc<dyn ChatBackend>,
    pub embed_backend: Arc<dyn EmbedBackend>,
}

impl GatewayState {
    /// The production wiring: the TASK-AI-005 policy loader, the real router chat backend (TASK-AI-105), and the
    /// router embeddings backend (TASK-MEMORY-123).
    pub fn production() -> Self {
        Self {
            policy: Arc::new(LoaderPolicySource),
            backend: Arc::new(RouterBackend),
            embed_backend: Arc::new(RouterEmbedBackend),
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

/// The `POST /v1/embeddings` request body. The brain (TASK-MEMORY-123) sends `{ "input": ["<text>"],
/// "model": "bge-m3" }`; `model` is optional and treated as a hint mapped to an embedding alias.
#[derive(Debug, Deserialize)]
pub struct ApiEmbedRequest {
    pub input: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// The `POST /v1/embeddings` response body, matching the contract the brain's `embed_client` parses:
/// `{ "embeddings": [[..1024 f32..]], "model": "..", "embed_model_version": ".." }`.
#[derive(Debug, Serialize)]
pub struct ApiEmbedResponse {
    pub embeddings: Vec<Vec<f32>>,
    pub model: String,
    pub embed_model_version: String,
}

/// `GET /v1/status` - TASK-APP-003 AI Ops read. Returns the requesting tenant's resolved AI policy: the
/// primary provider and its alias-to-model map, the monthly spend cap and warn threshold, residency, ZDR,
/// and the fallback chain. Read-only over the already-loaded policy; makes no provider call.
async fn status(State(st): State<GatewayState>, headers: HeaderMap) -> Response {
    let tenant = match header(&headers, "x-tenant-id") {
        Some(t) => t,
        None => return err(StatusCode::BAD_REQUEST, "missing x-tenant-id header"),
    };
    match st.policy.for_tenant(&tenant).await {
        Ok(policy) => (StatusCode::OK, Json((*policy).clone())).into_response(),
        Err(e) => err(
            StatusCode::NOT_FOUND,
            &format!("policy unavailable for tenant: {e}"),
        ),
    }
}

/// `POST /v1/embeddings` - TASK-MEMORY-123 / DEC-2723. The one embedding path for the brain: require a tenant,
/// load its policy, then hand off to the embeddings backend (resolve the embedding alias, call the provider
/// once, no failover). Maps the backend's typed failure to the brain's contract: 402 spend cap, 400 bad
/// request or unresolvable alias, 502 provider or gateway down.
async fn embeddings(
    State(st): State<GatewayState>,
    headers: HeaderMap,
    body: Result<Json<ApiEmbedRequest>, JsonRejection>,
) -> Response {
    let tenant = match header(&headers, "x-tenant-id") {
        Some(t) => t,
        None => return err(StatusCode::BAD_REQUEST, "missing x-tenant-id header"),
    };
    let Json(req) = match body {
        Ok(j) => j,
        Err(e) => {
            return err(
                StatusCode::BAD_REQUEST,
                &format!("invalid request body: {e}"),
            )
        }
    };
    if req.input.iter().all(|s| s.trim().is_empty()) {
        return err(
            StatusCode::BAD_REQUEST,
            "input must contain at least one non-empty string",
        );
    }

    let policy = match st.policy.for_tenant(&tenant).await {
        Ok(p) => p,
        Err(e) => {
            return err(
                StatusCode::NOT_FOUND,
                &format!("policy unavailable for tenant: {e}"),
            )
        }
    };

    match st.embed_backend.embed(&req, &policy).await {
        Ok(resp) => (StatusCode::OK, Json(resp)).into_response(),
        Err(EmbedFailure::SpendCap) => err(
            StatusCode::PAYMENT_REQUIRED,
            "embedding spend cap exhausted",
        ),
        Err(EmbedFailure::Resolve(msg)) => err(StatusCode::BAD_REQUEST, &msg),
        Err(EmbedFailure::Provider(msg)) => err(StatusCode::BAD_GATEWAY, &msg),
    }
}

/// Build the gateway router: liveness, a metrics stub (RED exports via OTLP, not a scrape), and the chat
/// endpoint, with the TASK-OBS-003 RED middleware wrapping every route.
pub fn build_router(state: GatewayState) -> Router {
    let mut app = Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route(
            "/metrics",
            get(|| async { "# cyberos-ai-gateway: RED metrics export via OTLP\n" }),
        )
        .route("/v1/chat", post(chat))
        .route("/v1/embeddings", post(embeddings))
        .route("/v1/status", get(status))
        // TASK-OBS-005: ensure every request carries a trace context (extract or generate) and echo it on
        // the response. TASK-OBS-003 (ADR-OBS-003-001): tenant_ctx stamps the request's tenant onto the
        // response; red_mw (outer) reads it for the metric's tenant_id label. Same wiring as auth/memory.
        .route_layer(axum::middleware::from_fn(trace_ctx))
        .route_layer(axum::middleware::from_fn(tenant_ctx))
        .layer(axum::middleware::from_fn_with_state(
            cyberos_obs_sdk::RedState::new("ai-gateway"),
            cyberos_obs_sdk::red_mw,
        ))
        .with_state(state);

    // TASK-APP-001: opt-in permissive CORS so a local browser console (the CDS web console) can call the
    // gateway. Off by default, so the production posture is unchanged; enable for local dev with
    // AI_GATEWAY_DEV_CORS=1. Restrict to a known origin allowlist before exposing the gateway to untrusted
    // browsers.
    if std::env::var("AI_GATEWAY_DEV_CORS").is_ok() {
        app = app.layer(tower_http::cors::CorsLayer::permissive());
    }
    app
}

/// TASK-OBS-003 - stamp the request's tenant (from `x-tenant-id`) onto the response so `red_mw` can label
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
/// extension so the handler (and the LangSmith export, TASK-OBS-004) read one consistent value.
#[derive(Debug, Clone)]
pub struct RequestTrace {
    pub trace_id: String,
    pub span_id: String,
}

/// Generate a fresh W3C trace context (16 random bytes trace id, 8 random bytes span id, sampled).
fn generate_trace_context() -> cyberos_obs_sdk::TraceContext {
    let mut rng = rand::thread_rng();
    let trace_id: String = (0..16)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect();
    let span_id: String = (0..8).map(|_| format!("{:02x}", rng.gen::<u8>())).collect();
    cyberos_obs_sdk::TraceContext {
        trace_id,
        span_id,
        flags: 1,
    }
}

/// TASK-OBS-005 (§1 #1, #4, #11) - ensure every request carries a trace context. Extract the incoming W3C
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
            eprintln!("{{\"sev\":2,\"event\":\"malformed_traceparent\",\"hash16\":\"{hash16}\"}}");
            generate_trace_context()
        }
    };
    req.extensions_mut().insert(RequestTrace {
        trace_id: tc.trace_id.clone(),
        span_id: tc.span_id.clone(),
    });
    // TASK-OBS-005 §1 #2 - instrument the request with the canonical span so every log line emitted while
    // handling it carries trace_id / span_id / tenant_id (the JSON subscriber renders the span scope).
    let tenant = req
        .headers()
        .get("x-tenant-id")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    let span = cyberos_obs_sdk::request_span(&tc.trace_id, &tc.span_id, &tenant);
    let mut response = async move { next.run(req).await }.instrument(span).await;
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
        Err(e) => {
            return err(
                StatusCode::BAD_REQUEST,
                &format!("invalid request body: {e}"),
            )
        }
    };
    if req.messages.is_empty() {
        return err(StatusCode::BAD_REQUEST, "messages must not be empty");
    }

    let policy = match st.policy.for_tenant(&tenant).await {
        Ok(p) => p,
        Err(e) => {
            return err(
                StatusCode::NOT_FOUND,
                &format!("policy unavailable for tenant: {e}"),
            )
        }
    };

    let resolved = match crate::alias::resolve(&req.alias, &policy) {
        Ok(r) => r,
        Err(e) => {
            return err(
                StatusCode::BAD_REQUEST,
                &format!("alias resolution failed: {e:?}"),
            )
        }
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

    let resp = match st.backend.complete(&ccr, &resolved, &policy).await {
        Ok(r) => r,
        Err(e) => {
            return err(
                StatusCode::BAD_GATEWAY,
                &format!("provider call failed: {e}"),
            )
        }
    };

    let content = resp
        .choices
        .first()
        .map(|c| c.content.clone())
        .unwrap_or_default();

    // TASK-OBS-004 - opt-in LangSmith export of the (redacted) call, correlated by the request trace id.
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

/// Redact the prompt and response (TASK-AI-011 / Presidio) and dispatch the LangSmith export (TASK-OBS-004).
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

    let (redacted_prompt, redacted_response) = match (
        redact::redact(&prompt_text, policy).await,
        redact::redact(content, policy).await,
    ) {
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

    /// A policy source that returns the fixture policy - used by the embeddings tests, which must get past
    /// policy load to reach the embeddings backend.
    #[derive(Debug)]
    struct FixedPolicy;
    #[async_trait]
    impl PolicySource for FixedPolicy {
        async fn for_tenant(&self, _t: &str) -> Result<Arc<TenantPolicy>, String> {
            Ok(fixture_policy())
        }
    }

    /// A stub embeddings backend: returns a fixed vector, or forces the spend-cap branch, so the route's
    /// status mapping is testable without a live provider.
    #[derive(Debug)]
    enum StubEmbed {
        Ok,
        SpendCap,
    }
    #[async_trait]
    impl EmbedBackend for StubEmbed {
        async fn embed(
            &self,
            _req: &ApiEmbedRequest,
            _policy: &TenantPolicy,
        ) -> Result<ApiEmbedResponse, EmbedFailure> {
            match self {
                StubEmbed::Ok => Ok(ApiEmbedResponse {
                    embeddings: vec![vec![0.1, 0.2, 0.3]],
                    model: "bge-m3".into(),
                    embed_model_version: "bge-m3@stub".into(),
                }),
                StubEmbed::SpendCap => Err(EmbedFailure::SpendCap),
            }
        }
    }

    fn test_state() -> GatewayState {
        GatewayState {
            policy: Arc::new(UnreachablePolicy),
            backend: Arc::new(EchoBackend),
            embed_backend: Arc::new(StubEmbed::Ok),
        }
    }

    fn msg(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    fn fixture_resolved() -> ResolvedModel {
        ResolvedModel {
            provider_kind: crate::policy::ProviderKind::Anthropic,
            region: None,
            model: "any-model".to_string(),
            fallback_position: 0,
            is_zdr: true,
            latency_class: crate::alias::LatencyClass::Standard,
        }
    }

    fn fixture_policy() -> Arc<TenantPolicy> {
        use crate::policy::schema::{AiPolicy, Provider, Residency};
        use rust_decimal_macros::dec;
        Arc::new(TenantPolicy {
            tenant_id: "org:cyberskill".to_string(),
            ai_policy: AiPolicy {
                monthly_cap_usd: dec!(150),
                warn_threshold: 0.8,
                hard_stop: true,
                primary_provider: Provider::Anthropic {
                    model_alias_map: Default::default(),
                },
                fallback_chain: vec![],
                call_timeout_seconds: 60,
                residency: Residency::Sg1,
                zdr_required: true,
                emergency_override: Default::default(),
                allowed_personas: None,
                alias_overrides: None,
                residency_requires_regional_provider: None,
                pii_redaction_extra: None,
                langsmith_export: false,
            },
        })
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
        let resp = EchoBackend
            .complete(&req, &fixture_resolved(), &fixture_policy())
            .await
            .unwrap();
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

    #[tokio::test]
    async fn embeddings_without_tenant_is_400() {
        let app = build_router(test_state());
        let body = r#"{"input":["hi"],"model":"bge-m3"}"#;
        let res = app
            .oneshot(
                Request::post("/v1/embeddings")
                    .header("content-type", "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn embeddings_empty_input_is_400() {
        let app = build_router(test_state());
        let body = r#"{"input":[],"model":"bge-m3"}"#;
        let res = app
            .oneshot(
                Request::post("/v1/embeddings")
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
    async fn embeddings_success_returns_vectors() {
        let st = GatewayState {
            policy: Arc::new(FixedPolicy),
            backend: Arc::new(EchoBackend),
            embed_backend: Arc::new(StubEmbed::Ok),
        };
        let app = build_router(st);
        let body = r#"{"input":["hi"],"model":"bge-m3"}"#;
        let res = app
            .oneshot(
                Request::post("/v1/embeddings")
                    .header("content-type", "application/json")
                    .header("x-tenant-id", "org:cyberskill")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["embeddings"].as_array().unwrap().len(), 1);
        assert_eq!(v["embed_model_version"], "bge-m3@stub");
    }

    #[tokio::test]
    async fn embeddings_spend_cap_is_402() {
        let st = GatewayState {
            policy: Arc::new(FixedPolicy),
            backend: Arc::new(EchoBackend),
            embed_backend: Arc::new(StubEmbed::SpendCap),
        };
        let app = build_router(st);
        let body = r#"{"input":["hi"],"model":"bge-m3"}"#;
        let res = app
            .oneshot(
                Request::post("/v1/embeddings")
                    .header("content-type", "application/json")
                    .header("x-tenant-id", "org:cyberskill")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::PAYMENT_REQUIRED);
    }
}
