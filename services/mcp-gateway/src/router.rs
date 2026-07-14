//! TASK-MCP-001 §1 #25 + §1 #26 — Axum router mounting `POST /mcp` + `GET /mcp/healthz`,
//! plus the TASK-MCP-002 control plane (`/v1/mcp/register`, `/heartbeat`, `/deregister`).

use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::Form;
use axum::Json;
use axum::Router;
use serde::Serialize;
use serde_json::{json, Value};
use tracing::{info, warn};

use crate::federation::register::{
    apply as apply_registration, validate as validate_registration, RegisterRequest,
};
use crate::federation::registry::ToolRegistry;
use crate::protocol::errors::{codes, err};
use crate::protocol::initialize::{build_response_value, InitializeParams};
use crate::protocol::jsonrpc::{Inbound, Request, Response};
use crate::protocol::tools_call::{
    dispatch as call_dispatch, prepare as call_prepare, ToolsCallParams,
};
use crate::protocol::tools_list::{build_response as build_tools_list, ToolsListParams};
use crate::MCP_PROTOCOL_VERSION;

/// Shared state passed through every handler.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Federated tool registry.
    pub registry: Arc<ToolRegistry>,
    /// Postgres pool for the TASK-MCP-004 OAuth endpoints. `None` when `MCP_DATABASE_URL` is unset: the
    /// OAuth endpoints then report unconfigured and the rest of the gateway runs unaffected, so the
    /// dev demo needs no database.
    pub oauth_pool: Option<sqlx::PgPool>,
    /// TASK-MCP-008 in-memory elicitation store. The no-database fallback (dev/demo); when `oauth_pool`
    /// and an authenticated caller are present the router uses the [`crate::elicitation_pg`]
    /// store-of-record against `mcp_elicitations` instead.
    pub elicitations: Arc<crate::elicitation::ElicitationStore>,
    /// TASK-MCP-007 in-memory task store for long-running tool calls (the persistent table + worker pool
    /// land in the DB slice).
    pub tasks: Arc<crate::tasks::TaskStore>,
    /// TASK-MCP-007/008 payload sealer for the DB store-of-record (`None` when `MCP_KMS_KEY` is unset, the
    /// no-database path). The destructive-tool confirmation flow seals caller responses with this.
    pub kms: Option<Arc<dyn crate::kms::Kms>>,
}

/// TASK-MCP-001 §1 #25 healthz payload.
#[derive(Debug, Serialize)]
pub struct HealthZ {
    /// Always `"ok"` while we're up.
    pub status: &'static str,
    /// MCP protocol version we speak.
    pub protocol_version: &'static str,
    /// Distinct modules registered.
    pub registered_modules: usize,
    /// Total tools registered.
    pub registered_tools: usize,
    /// Per-module server health (TASK-MCP-002).
    pub servers: Vec<serde_json::Value>,
    /// Whether the TASK-MCP-004 OAuth endpoints are configured (a database is connected).
    pub oauth_configured: bool,
}

/// Whether the TASK-MCP-002 control-plane routes (register/heartbeat/deregister) are enabled.
/// Off unless `MCP_DEV_REGISTRATION=1`, because they mutate what the gateway dispatches to.
fn control_plane_enabled() -> bool {
    std::env::var("MCP_DEV_REGISTRATION").as_deref() == Ok("1")
}

fn control_plane_disabled_response() -> (StatusCode, Json<Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "registration_disabled",
            "detail": "set MCP_DEV_REGISTRATION=1 to enable the dev control plane; production requires authenticated registration (TASK-MCP-004)"
        })),
    )
}

/// Build the Axum router. `POST /mcp` + `GET /mcp/healthz` are the MCP protocol surface;
/// `/v1/mcp/{register,heartbeat,deregister}` are the TASK-MCP-002 control plane.
pub fn build_router(state: AppState) -> Router {
    let app = Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/mcp/healthz", get(handle_healthz))
        .route(
            "/.well-known/oauth-authorization-server",
            get(handle_oauth_metadata),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(handle_prm_aggregate).options(handle_prm_options),
        )
        .route(
            "/.well-known/oauth-protected-resource/:module",
            get(handle_prm_module).options(handle_prm_options),
        )
        .route("/authorize", get(handle_oauth_authorize))
        .route("/register", post(handle_oauth_register))
        .route("/token", post(handle_oauth_token))
        .route("/revoke", post(handle_oauth_revoke))
        .route("/introspect", post(handle_oauth_introspect))
        .route("/v1/mcp/register", post(handle_register))
        .route("/v1/mcp/heartbeat", post(handle_heartbeat))
        .route("/v1/mcp/deregister", post(handle_deregister))
        .route("/v1/mcp/elicitations", get(handle_elicitation_poll))
        .route(
            "/v1/mcp/elicitations/:id/respond",
            post(handle_elicitation_respond),
        )
        .route(
            "/v1/mcp/elicitations/:id/cancel",
            post(handle_elicitation_cancel),
        )
        .route("/v1/mcp/tasks/:id", get(handle_task_status))
        .route("/v1/mcp/tasks/:id/cancel", post(handle_task_cancel))
        .with_state(state);

    // TASK-APP-004: opt-in permissive CORS so the local browser console (the MCP Registry panel) can call
    // /mcp cross-origin in dev. Off by default; in production Caddy fronts the gateway under one origin.
    // Set MCP_DEV_CORS=1 only for local development.
    if std::env::var("MCP_DEV_CORS").is_ok() {
        app.layer(tower_http::cors::CorsLayer::permissive())
    } else {
        app
    }
}

/// TASK-MCP-002 control-plane: a module registers its tool catalogue so `tools/list` and
/// `tools/call` can see it.
///
/// Trust boundary: registration changes what the gateway will forward `tools/call` to, so
/// it is privileged. This dev slice gates the route behind `MCP_DEV_REGISTRATION=1` (off by
/// default). Production must replace this with authenticated registration (TASK-MCP-004) plus
/// an endpoint allowlist before exposing it, and the heartbeat/health lifecycle
/// (DEC-2350/2351) is the next slice on top of this one.
async fn handle_register(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    if !control_plane_enabled() {
        return control_plane_disabled_response();
    }

    let req: RegisterRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "invalid_body", "detail": e.to_string() })),
            );
        }
    };

    if let Err(e) = validate_registration(&req) {
        let detail = e.message();
        // TASK-MCP-003 DEC-2364: a non-conforming tool ID was rejected at registration (best-effort audit).
        if let Some(pool) = state.oauth_pool.as_ref() {
            crate::oauth::audit::skill_name_rejected(pool, &req.module, detail.as_ref()).await;
        }
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_registration", "detail": detail })),
        );
    }

    let n = apply_registration(&state.registry, &req);
    // TASK-MCP-003 DEC-2364: the module's tool IDs validated (best-effort audit).
    if let Some(pool) = state.oauth_pool.as_ref() {
        crate::oauth::audit::skill_name_validated(pool, &req.module, n).await;
    }
    let names: Vec<&str> = req.tools.iter().map(|t| t.name.as_str()).collect();
    info!(module = %req.module, endpoint = %req.endpoint, tools = n, "module registered");
    (
        StatusCode::OK,
        Json(json!({ "registered": n, "module": req.module, "tools": names })),
    )
}

async fn handle_healthz(State(state): State<AppState>) -> (StatusCode, Json<HealthZ>) {
    let servers: Vec<Value> = state
        .registry
        .server_health(SystemTime::now())
        .into_iter()
        .map(|(module, status)| json!({ "module": module, "status": status.as_str() }))
        .collect();
    (
        StatusCode::OK,
        Json(HealthZ {
            status: "ok",
            protocol_version: MCP_PROTOCOL_VERSION,
            registered_modules: state.registry.modules().len(),
            registered_tools: state.registry.len(),
            servers,
            oauth_configured: state.oauth_pool.is_some(),
        }),
    )
}

/// TASK-MCP-004 clause #20 - RFC 8414 authorization-server metadata at
/// `/.well-known/oauth-authorization-server`. Read-only; the issuer, JWKS URI, and supported scopes
/// come from env (`MCP_ISSUER_URL`, `MCP_JWKS_URI`, `MCP_OAUTH_SCOPES`) with dev defaults. Scopes
/// default to `mcp:tools` - the scope every current tool requires; refine to compute the union from
/// the registry when the scope set diversifies.
async fn handle_oauth_metadata() -> (StatusCode, Json<Value>) {
    let issuer =
        std::env::var("MCP_ISSUER_URL").unwrap_or_else(|_| "http://localhost:8090".to_string());
    let jwks_uri = std::env::var("MCP_JWKS_URI")
        .unwrap_or_else(|_| format!("{}/.well-known/jwks.json", issuer.trim_end_matches('/')));
    let scopes: Vec<String> = std::env::var("MCP_OAUTH_SCOPES")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_else(|| vec!["mcp:tools".to_string()]);
    let doc = crate::oauth::discovery::authorization_server_metadata(&issuer, &jwks_uri, &scopes);
    (StatusCode::OK, Json(doc))
}

/// TASK-MCP-005 gateway-aggregate Protected Resource Metadata (RFC 9728). Public and unauthenticated per
/// DEC-896; cacheable via ETag. `get(...)` also serves HEAD (axum strips the body).
async fn handle_prm_aggregate(headers: HeaderMap) -> axum::response::Response {
    let doc = crate::oauth::prm::protected_resource_metadata(
        &oauth_resource(),
        &oauth_authorization_servers(),
    );
    prm_http_response(&doc, if_none_match(&headers))
}

/// TASK-MCP-005 per-module Protected Resource Metadata. The path segment is `cyberos.<module>` (DEC-897);
/// `scopes_supported` is the union of the module's tools' required scopes from the TASK-MCP-002 registry.
/// An unknown module returns 404 and emits a best-effort `mcp.prm_unknown_module_requested` audit row.
async fn handle_prm_module(
    State(state): State<AppState>,
    Path(segment): Path<String>,
    headers: HeaderMap,
) -> axum::response::Response {
    let module = segment.strip_prefix("cyberos.").unwrap_or(&segment);
    match state.registry.module_scopes(module) {
        Some(scopes) => {
            let resource = format!("{}/{module}", oauth_resource().trim_end_matches('/'));
            let doc = crate::oauth::prm::protected_resource_metadata_for_module(
                &resource,
                &oauth_authorization_servers(),
                &scopes,
            );
            prm_http_response(&doc, if_none_match(&headers))
        }
        None => {
            if let Some(pool) = state.oauth_pool.as_ref() {
                crate::oauth::audit::prm_unknown_module_requested(pool, module).await;
            }
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "not_found", "error_description": "unknown module" })),
            )
                .into_response()
        }
    }
}

/// CORS preflight for the PRM endpoints (§1 #15): metadata is public and identical for every requester,
/// so allow any origin for `GET`/`HEAD`.
async fn handle_prm_options() -> axum::response::Response {
    use axum::http::header;
    axum::http::Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD")
        .header(header::ACCESS_CONTROL_MAX_AGE, "3600")
        .body(axum::body::Body::empty())
        .expect("static response builds")
}

/// Borrow the request's `If-None-Match` header value, if present and valid UTF-8.
fn if_none_match(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(axum::http::header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
}

/// Render a PRM document as an RFC 9728 response with the spec's caching, CORS, and robots headers.
/// Honours `If-None-Match`: a matching ETag yields `304 Not Modified` with an empty body. The body is
/// serialized identically to the ETag input, so revalidation is exact.
fn prm_http_response(doc: &Value, if_none_match: Option<&str>) -> axum::response::Response {
    use axum::http::header;
    let etag = crate::oauth::prm::etag(doc);
    let builder = axum::http::Response::builder()
        .header(header::CACHE_CONTROL, "max-age=3600")
        .header(header::ETAG, etag.as_str())
        .header("X-Robots-Tag", "noindex, nofollow, nosnippet")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, HEAD")
        .header(header::ACCESS_CONTROL_MAX_AGE, "3600");
    if if_none_match == Some(etag.as_str()) {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(axum::body::Body::empty())
            .expect("static response builds");
    }
    let body = serde_json::to_string(doc).unwrap_or_else(|_| "{}".to_string());
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .body(axum::body::Body::from(body))
        .expect("static response builds")
}

/// Gateway issuer URL for minted tokens (env `MCP_ISSUER_URL`, dev default).
fn oauth_issuer() -> String {
    std::env::var("MCP_ISSUER_URL").unwrap_or_else(|_| "http://localhost:8090".to_string())
}

/// This resource server's canonical URL - the audience tokens are bound to (env `MCP_RESOURCE_URL`,
/// defaulting to the issuer).
fn oauth_resource() -> String {
    std::env::var("MCP_RESOURCE_URL").unwrap_or_else(|_| oauth_issuer())
}

/// The TASK-AUTH-004 issuer whose session JWTs `/authorize` and confidential `/register` accept (env
/// `MCP_AUTH_ISSUER`, dev default). Distinct from `oauth_issuer()`, which issues the gateway's own
/// access tokens.
fn oauth_auth_issuer() -> String {
    std::env::var("MCP_AUTH_ISSUER").unwrap_or_else(|_| "http://localhost:8081".to_string())
}

/// TASK-MCP-005 `authorization_servers`: the issuer URLs whose access tokens this MCP resource accepts.
/// The gateway is its own authorization server (it mints the tokens `enforce_oauth` verifies), so this
/// defaults to `[oauth_issuer()]`. `MCP_AUTHORIZATION_SERVERS` (comma- or whitespace-separated)
/// overrides it for a future multi-issuer residency map (TASK-AUTH-004).
fn oauth_authorization_servers() -> Vec<String> {
    std::env::var("MCP_AUTHORIZATION_SERVERS")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| {
            s.split(|c: char| c == ',' || c.is_whitespace())
                .filter(|t| !t.is_empty())
                .map(|t| t.trim_end_matches('/').to_string())
                .collect()
        })
        .unwrap_or_else(|| vec![oauth_issuer().trim_end_matches('/').to_string()])
}

/// The `WWW-Authenticate` challenge for a 401 from the MCP surface (TASK-MCP-005 §1 #19, RFC 9728 §5.1):
/// `Bearer` with the `resource_metadata` parameter pointing at this gateway's Protected Resource
/// Metadata, so a client that is refused can discover where to authenticate.
fn www_authenticate_challenge() -> String {
    format!(
        "Bearer realm=\"cyberos-mcp\", error=\"invalid_token\", resource_metadata=\"{}/.well-known/oauth-protected-resource\"",
        oauth_issuer().trim_end_matches('/')
    )
}

/// Extract a bearer auth JWT from `Authorization` and verify it into an `AuthSession`. `None` when the
/// header is absent or the token does not verify, so confidential registration fails closed (an
/// unauthenticated caller cannot mint a confidential client).
async fn oauth_auth_session(
    pool: &sqlx::PgPool,
    headers: &HeaderMap,
) -> Option<crate::oauth::authsession::AuthSession> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))?;
    crate::oauth::authsession::verify_auth_session(pool, token, &oauth_auth_issuer()).await
}

/// Whether the MCP surface requires a valid audience-bound OAuth token (TASK-MCP-004 clause #23). Off
/// unless `MCP_REQUIRE_AUTH=1`, so the dev demo works with no token and production sets it on - the
/// same gating shape as `MCP_DEV_REGISTRATION`.
fn require_auth() -> bool {
    std::env::var("MCP_REQUIRE_AUTH").as_deref() == Ok("1")
}

/// Verify the request's bearer access token against this resource server: signature + issuer + expiry +
/// exact audience (enforced inside `verify_access_token`) + not-revoked (clauses #23, #24). Returns the
/// 401 response on any failure, or the verified [`McpAccessClaims`](crate::oauth::jwt::McpAccessClaims)
/// (carrying the caller's tenant + subject) to let the request proceed.
async fn enforce_oauth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<crate::oauth::jwt::McpAccessClaims, axum::response::Response> {
    // Every 401 carries the TASK-MCP-005 `WWW-Authenticate: Bearer ... resource_metadata=...` challenge
    // so a refused client can discover the Protected Resource Metadata and re-authenticate.
    let unauth = |reason: &str| -> axum::response::Response {
        let mut resp = (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid_token", "error_description": reason })),
        )
            .into_response();
        if let Ok(v) = axum::http::HeaderValue::from_str(&www_authenticate_challenge()) {
            resp.headers_mut()
                .insert(axum::http::header::WWW_AUTHENTICATE, v);
        }
        resp
    };
    let Some(pool) = state.oauth_pool.as_ref() else {
        return Err(unauth("oauth_not_configured"));
    };
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| unauth("missing_bearer_token"))?;
    let claims = match crate::oauth::jwt::verify_access_token(
        pool,
        token,
        &oauth_issuer(),
        &oauth_resource(),
    )
    .await
    {
        Ok(c) => c,
        Err(crate::oauth::jwt::JwtError::AudienceMismatch) => {
            // Clause #23: a token bound to another resource server was presented here. Record it
            // before refusing, so cross-server replay attempts are visible in the audit chain.
            crate::oauth::audit::audience_mismatch(pool).await;
            return Err(unauth("audience_mismatch"));
        }
        Err(_) => return Err(unauth("token_verification_failed")),
    };
    if let Ok(jti) = uuid::Uuid::parse_str(&claims.jti) {
        if crate::oauth::store::is_jti_revoked(pool, jti)
            .await
            .unwrap_or(false)
        {
            return Err(unauth("token_revoked"));
        }
    }
    Ok(claims)
}

/// Parse a verified token's `(tenant_id, subject)` into UUIDs for the DB store-of-record. `None` if
/// either claim is not a UUID (a malformed token that nonetheless verified - treated as no caller).
fn caller_ids(claims: &crate::oauth::jwt::McpAccessClaims) -> Option<(uuid::Uuid, uuid::Uuid)> {
    Some((
        uuid::Uuid::parse_str(&claims.tenant_id).ok()?,
        uuid::Uuid::parse_str(&claims.sub).ok()?,
    ))
}

/// Whether the elicitation/task store-of-record (DB path) is active: a database is connected and the MCP
/// surface authenticates callers, so every row has a real `tenant_id`/`subject`. Off in dev (no
/// `MCP_REQUIRE_AUTH`), where the in-memory stores are used and isolation is moot. Keying create and the
/// REST reads/mutations on the same flag keeps both stores' rows in one backend.
fn use_db_store(state: &AppState) -> bool {
    require_auth() && state.oauth_pool.is_some()
}

/// TASK-MCP-004 RFC 7591 dynamic client registration. JSON body; 503 when no database is configured.
/// Public clients register openly; confidential clients require a tenant-admin auth JWT in
/// `Authorization` (verified into the `caller` session). The body-consuming `Json` extractor stays
/// last per axum's extractor ordering.
async fn handle_oauth_register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<crate::oauth::dcr::RegisterRequest>,
) -> Result<Json<crate::oauth::dcr::RegisterResponse>, crate::oauth::response::EndpointError> {
    let pool = state
        .oauth_pool
        .as_ref()
        .ok_or(crate::oauth::response::EndpointError::Unconfigured)?;
    let caller = oauth_auth_session(pool, &headers).await;
    Ok(Json(crate::oauth::dcr::register(pool, req, caller).await?))
}

/// TASK-MCP-004 token endpoint. Form-encoded body (RFC 6749 §6).
async fn handle_oauth_token(
    State(state): State<AppState>,
    Form(req): Form<crate::oauth::token::TokenRequest>,
) -> Result<Json<crate::oauth::token::TokenResponse>, crate::oauth::response::EndpointError> {
    let pool = state
        .oauth_pool
        .as_ref()
        .ok_or(crate::oauth::response::EndpointError::Unconfigured)?;
    Ok(Json(
        crate::oauth::token::token(pool, &oauth_issuer(), req).await?,
    ))
}

/// TASK-MCP-004 token revocation (RFC 7009). Always 200.
async fn handle_oauth_revoke(
    State(state): State<AppState>,
    Form(req): Form<crate::oauth::revoke::RevokeRequest>,
) -> Result<StatusCode, crate::oauth::response::EndpointError> {
    let pool = state
        .oauth_pool
        .as_ref()
        .ok_or(crate::oauth::response::EndpointError::Unconfigured)?;
    crate::oauth::revoke::revoke(pool, req).await?;
    Ok(StatusCode::OK)
}

/// TASK-MCP-004 token introspection (RFC 7662).
async fn handle_oauth_introspect(
    State(state): State<AppState>,
    Form(req): Form<crate::oauth::introspect::IntrospectRequest>,
) -> Result<Json<Value>, crate::oauth::response::EndpointError> {
    let pool = state
        .oauth_pool
        .as_ref()
        .ok_or(crate::oauth::response::EndpointError::Unconfigured)?;
    Ok(Json(
        crate::oauth::introspect::introspect(pool, &oauth_issuer(), &oauth_resource(), req).await?,
    ))
}

/// TASK-MCP-004 authorize endpoint. The calling subject's identity comes from an auth-service bearer
/// JWT (`Authorization: Bearer <auth_jwt>`); on success it 302-redirects to the client's redirect_uri
/// carrying a one-time code.
async fn handle_oauth_authorize(
    State(state): State<AppState>,
    Query(params): Query<crate::oauth::authorize::AuthorizeParams>,
    headers: HeaderMap,
) -> Result<Redirect, crate::oauth::response::EndpointError> {
    let pool = state
        .oauth_pool
        .as_ref()
        .ok_or(crate::oauth::response::EndpointError::Unconfigured)?;
    let bearer = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string());
    let url = crate::oauth::authorize::authorize(
        pool,
        &oauth_resource(),
        &oauth_auth_issuer(),
        params,
        bearer,
    )
    .await?;
    Ok(Redirect::to(&url))
}

/// TASK-MCP-002 control-plane: a module heartbeats to stay healthy. Body: `{"module": "..."}`.
async fn handle_heartbeat(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    if !control_plane_enabled() {
        return control_plane_disabled_response();
    }
    let module = match parse_module_field(&body) {
        Ok(m) => m,
        Err(resp) => return resp,
    };
    if state.registry.record_heartbeat(&module, SystemTime::now()) {
        let status = state
            .registry
            .server_status(&module, SystemTime::now())
            .map(|s| s.as_str())
            .unwrap_or("healthy");
        (
            StatusCode::OK,
            Json(json!({ "module": module, "status": status })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(
                json!({ "error": "unknown_module", "detail": format!("{module} is not registered; register before heartbeating") }),
            ),
        )
    }
}

/// TASK-MCP-002 control-plane: a module deregisters (its tools are withdrawn until it
/// registers again). Body: `{"module": "..."}`.
async fn handle_deregister(
    State(state): State<AppState>,
    body: axum::body::Bytes,
) -> (StatusCode, Json<Value>) {
    if !control_plane_enabled() {
        return control_plane_disabled_response();
    }
    let module = match parse_module_field(&body) {
        Ok(m) => m,
        Err(resp) => return resp,
    };
    if state.registry.mark_deregistered(&module) {
        info!(module = %module, "module deregistered");
        (
            StatusCode::OK,
            Json(json!({ "module": module, "status": "deregistered" })),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(
                json!({ "error": "unknown_module", "detail": format!("{module} is not registered") }),
            ),
        )
    }
}

/// Map a [`RespondOutcome`](crate::elicitation::RespondOutcome) to the HTTP status + body. Shared by the
/// in-memory and DB-backed respond paths so both answer identically.
fn respond_outcome_http(outcome: &crate::elicitation::RespondOutcome) -> (StatusCode, Json<Value>) {
    use crate::elicitation::RespondOutcome::{
        AlreadyRecorded, Invalid, NotFound, NotPending, Recorded, ValidationFailed,
    };
    match outcome {
        Recorded { confirmed } | AlreadyRecorded { confirmed } => (
            StatusCode::OK,
            Json(json!({ "status": "responded", "confirmed": confirmed })),
        ),
        Invalid(errors) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "validation_errors": errors })),
        ),
        ValidationFailed(errors) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "status": "validation_failed", "validation_errors": errors })),
        ),
        NotFound => (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" }))),
        NotPending => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "not_pending" })),
        ),
    }
}

/// TASK-MCP-008 caller poll: the pending elicitations awaiting a response. When the store-of-record is
/// active the caller is authenticated and sees only their own pending elicitations (DEC-1159); in dev the
/// in-memory store is returned unscoped.
async fn handle_elicitation_poll(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> (StatusCode, Json<Value>) {
    let elicitations = if use_db_store(&state) {
        let claims = match enforce_oauth(&state, &headers).await {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "unauthorized" })),
                )
            }
        };
        let (Some(pool), Some((_, subject))) = (state.oauth_pool.as_ref(), caller_ids(&claims))
        else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "store_not_ready" })),
            );
        };
        match crate::elicitation_pg::pending(pool, subject).await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "elicitation poll (db) failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal" })),
                );
            }
        }
    } else {
        state.elicitations.pending()
    };
    (
        StatusCode::OK,
        Json(json!({ "elicitations": elicitations })),
    )
}

/// TASK-MCP-008 caller response. Validates against the elicitation's fixed type schema, transitions it,
/// and audits best-effort. 200 recorded, 422 on validation failure (with `validation_errors`), 404
/// unknown, 409 no longer pending, 400 on a malformed id.
async fn handle_elicitation_respond(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(req): Json<crate::elicitation::ElicitationRespondReq>,
) -> (StatusCode, Json<Value>) {
    let Ok(id) = uuid::Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_elicitation_id" })),
        );
    };
    // Store-of-record path: authenticate, scope to the caller, seal the payload through the KMS. Dev
    // path: the in-memory store. Both yield a `RespondOutcome` mapped identically below.
    let outcome = if use_db_store(&state) {
        let claims = match enforce_oauth(&state, &headers).await {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "unauthorized" })),
                )
            }
        };
        let (Some(pool), Some((_, subject)), Some(kms)) = (
            state.oauth_pool.as_ref(),
            caller_ids(&claims),
            state.kms.as_deref(),
        ) else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "store_not_ready" })),
            );
        };
        match crate::elicitation_pg::respond(pool, kms, id, subject, req.response_payload).await {
            Ok(o) => o,
            Err(e) => {
                warn!(error = %e, "elicitation respond (db) failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal" })),
                );
            }
        }
    } else {
        state.elicitations.respond(id, req.response_payload)
    };
    // Best-effort audit (pool-gated), same kinds as before, for whichever backend produced the outcome.
    if let Some(pool) = state.oauth_pool.as_ref() {
        match &outcome {
            crate::elicitation::RespondOutcome::Recorded { .. } => {
                crate::oauth::audit::elicitation_responded(pool, id).await;
            }
            crate::elicitation::RespondOutcome::ValidationFailed(_) => {
                crate::oauth::audit::elicitation_validation_failed(pool, id).await;
            }
            _ => {}
        }
    }
    respond_outcome_http(&outcome)
}

/// TASK-MCP-008 caller cancellation of a pending elicitation. Caller-scoped on the store-of-record path.
async fn handle_elicitation_cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let Ok(id) = uuid::Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_elicitation_id" })),
        );
    };
    let cancelled = if use_db_store(&state) {
        let claims = match enforce_oauth(&state, &headers).await {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "unauthorized" })),
                )
            }
        };
        let (Some(pool), Some((_, subject))) = (state.oauth_pool.as_ref(), caller_ids(&claims))
        else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "store_not_ready" })),
            );
        };
        match crate::elicitation_pg::cancel(pool, id, subject).await {
            Ok(b) => b,
            Err(e) => {
                warn!(error = %e, "elicitation cancel (db) failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal" })),
                );
            }
        }
    } else {
        state.elicitations.cancel(id)
    };
    if cancelled {
        if let Some(pool) = state.oauth_pool.as_ref() {
            crate::oauth::audit::elicitation_cancelled(pool, id).await;
        }
        (StatusCode::OK, Json(json!({ "status": "cancelled" })))
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "not_found_or_not_pending" })),
        )
    }
}

/// TASK-MCP-007 task status poll. 200 with the task's status view, or 404 for an unknown handle (or a
/// malformed id). Caller-scoped on the store-of-record path; the result payload is opened through the KMS.
async fn handle_task_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let Ok(id) = uuid::Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_task_id" })),
        );
    };
    let view: Option<Value> = if use_db_store(&state) {
        let claims = match enforce_oauth(&state, &headers).await {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "unauthorized" })),
                )
            }
        };
        let (Some(pool), Some((_, subject)), Some(kms)) = (
            state.oauth_pool.as_ref(),
            caller_ids(&claims),
            state.kms.as_deref(),
        ) else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "store_not_ready" })),
            );
        };
        match crate::tasks_pg::status_view(pool, kms, id, subject).await {
            Ok(v) => v,
            Err(e) => {
                warn!(error = %e, "task status (db) failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal" })),
                );
            }
        }
    } else {
        state.tasks.get(id).map(|t| t.status_view())
    };
    match view {
        Some(v) => (StatusCode::OK, Json(v)),
        None => (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" }))),
    }
}

/// TASK-MCP-007 task cancellation. 200 cancelled, 404 unknown, 409 already terminal, 400 malformed id.
/// Caller-scoped on the store-of-record path.
async fn handle_task_cancel(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> (StatusCode, Json<Value>) {
    let Ok(id) = uuid::Uuid::parse_str(&id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_task_id" })),
        );
    };
    let outcome = if use_db_store(&state) {
        let claims = match enforce_oauth(&state, &headers).await {
            Ok(c) => c,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({ "error": "unauthorized" })),
                )
            }
        };
        let (Some(pool), Some((_, subject))) = (state.oauth_pool.as_ref(), caller_ids(&claims))
        else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "store_not_ready" })),
            );
        };
        match crate::tasks_pg::cancel(pool, id, subject).await {
            Ok(o) => o,
            Err(e) => {
                warn!(error = %e, "task cancel (db) failed");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": "internal" })),
                );
            }
        }
    } else {
        state.tasks.cancel(id)
    };
    match outcome {
        crate::tasks::CancelOutcome::Cancelled => {
            if let Some(pool) = state.oauth_pool.as_ref() {
                crate::oauth::audit::task_cancelled(pool, id).await;
            }
            (StatusCode::OK, Json(json!({ "status": "cancelled" })))
        }
        crate::tasks::CancelOutcome::NotFound => {
            (StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" })))
        }
        crate::tasks::CancelOutcome::AlreadyTerminal => (
            StatusCode::CONFLICT,
            Json(json!({ "error": "task_terminal" })),
        ),
    }
}

/// Parse `{"module": "..."}` from a control-plane request body, or return the error response.
fn parse_module_field(body: &[u8]) -> Result<String, (StatusCode, Json<Value>)> {
    let v: Value = serde_json::from_slice(body).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_body", "detail": e.to_string() })),
        )
    })?;
    match v.get("module").and_then(|m| m.as_str()) {
        Some(m) if !m.trim().is_empty() => Ok(m.to_string()),
        _ => Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "error": "invalid_body", "detail": "expected a non-empty \"module\" string" }),
            ),
        )),
    }
}

async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> axum::response::Response {
    // TASK-MCP-004 clauses #23/#24: when enabled, the MCP surface requires a valid audience-bound,
    // unrevoked access token. Off in dev (no token) so the demo keeps working. A 401 here carries the
    // TASK-MCP-005 `WWW-Authenticate` resource-metadata challenge. The verified claims carry the caller's
    // tenant + subject, which the destructive-tool gate writes into the persisted confirmation row.
    let caller = if require_auth() {
        match enforce_oauth(&state, &headers).await {
            Ok(claims) => Some(claims),
            Err(resp) => return resp,
        }
    } else {
        None
    };
    let inbound = match Inbound::parse(&body) {
        Ok(i) => i,
        Err(e) => {
            warn!(error = %e, "parse failure");
            return (
                StatusCode::OK,
                Json(
                    serde_json::to_value(Response::error(Value::Null, err(codes::PARSE_ERROR, &e)))
                        .expect("serialise"),
                ),
            )
                .into_response();
        }
    };

    match inbound {
        Inbound::Single(req) => {
            if req.is_notification() {
                // Per JSON-RPC 2.0, no response is emitted for notifications.
                return (StatusCode::OK, Json(json!(null))).into_response();
            }
            let resp = dispatch_one(&state, req, caller.as_ref()).await;
            (
                StatusCode::OK,
                Json(serde_json::to_value(resp).expect("serialise")),
            )
                .into_response()
        }
        Inbound::Batch(reqs) => {
            let mut out: Vec<Response> = Vec::with_capacity(reqs.len());
            for r in reqs {
                if r.is_notification() {
                    continue;
                }
                out.push(dispatch_one(&state, r, caller.as_ref()).await);
            }
            (
                StatusCode::OK,
                Json(serde_json::to_value(out).expect("serialise")),
            )
                .into_response()
        }
    }
}

async fn dispatch_one(
    state: &AppState,
    req: Request,
    caller: Option<&crate::oauth::jwt::McpAccessClaims>,
) -> Response {
    let id = req.id.clone().unwrap_or(Value::Null);
    match req.method.as_str() {
        "initialize" => {
            let params: InitializeParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return Response::error(
                            id,
                            err(codes::INVALID_PARAMS, &format!("initialize: {e}")),
                        );
                    }
                },
                None => InitializeParams {
                    protocol_version: String::new(),
                    client_info: None,
                    capabilities: None,
                },
            };
            match build_response_value(&params) {
                Ok(v) => Response::success(id, v),
                Err(e) => Response::error(id, e),
            }
        }
        "tools/list" => {
            let params: ToolsListParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return Response::error(
                            id,
                            err(codes::INVALID_PARAMS, &format!("tools/list: {e}")),
                        );
                    }
                },
                None => ToolsListParams::default(),
            };
            let r = build_tools_list(&state.registry, &params);
            Response::success(id, serde_json::to_value(r).expect("serialise"))
        }
        "tools/call" => {
            // Slice-1: caller scopes come from JWT verification (TASK-MCP-004) once wired;
            // for now we accept a permissive default and rely on TASK-MCP-002+004 to
            // tighten. The `_caller_scopes` is the integration point.
            let params: ToolsCallParams = match req.params {
                Some(p) => match serde_json::from_value(p) {
                    Ok(v) => v,
                    Err(e) => {
                        return Response::error(
                            id,
                            err(codes::INVALID_PARAMS, &format!("tools/call: {e}")),
                        );
                    }
                },
                None => {
                    return Response::error(
                        id,
                        err(codes::INVALID_PARAMS, "tools/call: missing params"),
                    );
                }
            };
            let caller_scopes = vec!["mcp:tools".to_string()];

            // TASK-MCP-006: a destructive tool is held for an elicited confirmation (TASK-MCP-008) before
            // forwarding. Resolve the entry first for its annotations; lookup/scope errors are the same
            // ones `call_dispatch` would raise. Non-destructive tools fall straight through.
            let entry = match call_prepare(&state.registry, &params, &caller_scopes) {
                Ok(e) => e,
                Err(e) => return Response::error(id, e),
            };
            if entry.annotations.destructive_hint {
                // TASK-MCP-008 store-of-record: persist the confirmation to `mcp_elicitations` when a
                // database + authenticated caller are present (so it survives a restart and is
                // caller-scoped); otherwise the in-memory store (dev/demo). `pg` is `Copy` (a `&PgPool`
                // plus two UUIDs), so it is consulted for both the verdict read and the create.
                let pg: Option<(&sqlx::PgPool, uuid::Uuid, uuid::Uuid)> = if use_db_store(state) {
                    match (state.oauth_pool.as_ref(), caller.and_then(caller_ids)) {
                        (Some(pool), Some((tenant, subject))) => Some((pool, tenant, subject)),
                        _ => None,
                    }
                } else {
                    None
                };
                // Fail closed: under the store-of-record the confirmation must be persisted, so an
                // authenticated caller whose tenant/subject did not resolve is an internal error, never a
                // silent drop to the in-memory store (which the respond/cancel handlers would not consult).
                if use_db_store(state) && pg.is_none() {
                    warn!("destructive tools/call under store-of-record with unresolved caller ids; failing closed");
                    return Response::error(id, err(codes::INTERNAL_ERROR, "caller_unresolved"));
                }
                let cid = params
                    .meta
                    .as_ref()
                    .and_then(|m| m.confirmation_id.as_deref())
                    .and_then(|c| uuid::Uuid::parse_str(c).ok());
                let confirmed = match pg {
                    Some((pool, _, subject)) => match cid {
                        Some(c) => crate::elicitation_pg::confirmation_state(pool, c, subject)
                            .await
                            .unwrap_or(None),
                        None => None,
                    },
                    None => cid.and_then(|c| state.elicitations.confirmation_state(c)),
                };
                match crate::gating::evaluate(true, confirmed) {
                    crate::gating::ConfirmationOutcome::Declined => {
                        return Response::success(
                            id,
                            serde_json::to_value(crate::gating::user_rejected_result())
                                .expect("serialise"),
                        );
                    }
                    crate::gating::ConfirmationOutcome::NeedsConfirmation => {
                        let prompt = crate::gating::confirmation_prompt(&params.name);
                        if let Some((pool, tenant, subject)) = pg {
                            match crate::elicitation_pg::create_confirmation(
                                pool,
                                tenant,
                                subject,
                                &params.name,
                                prompt.clone(),
                            )
                            .await
                            {
                                Ok(eid) => {
                                    crate::oauth::audit::elicitation_requested(
                                        pool,
                                        eid,
                                        &params.name,
                                        "confirmation",
                                    )
                                    .await;
                                    return Response::success(
                                        id,
                                        serde_json::to_value(crate::gating::held_result_parts(
                                            eid,
                                            &params.name,
                                            &prompt,
                                        ))
                                        .expect("serialise"),
                                    );
                                }
                                Err(e) => {
                                    // Fail closed: a destructive call is never forwarded if its
                                    // confirmation could not be persisted.
                                    warn!(error = %e, "elicitation persist failed");
                                    return Response::error(
                                        id,
                                        err(codes::INTERNAL_ERROR, "elicitation_persist_failed"),
                                    );
                                }
                            }
                        }
                        let held = state.elicitations.create_confirmation(&params.name, prompt);
                        if let Some(pool) = state.oauth_pool.as_ref() {
                            crate::oauth::audit::elicitation_requested(
                                pool,
                                held.id,
                                &params.name,
                                "confirmation",
                            )
                            .await;
                        }
                        return Response::success(
                            id,
                            serde_json::to_value(crate::gating::held_result(&held))
                                .expect("serialise"),
                        );
                    }
                    crate::gating::ConfirmationOutcome::Proceed => {}
                }
            }
            match call_dispatch(&state.registry, &params, &caller_scopes).await {
                Ok(r) => Response::success(id, serde_json::to_value(r).expect("serialise")),
                Err(e) => Response::error(id, e),
            }
        }
        "notifications/initialized" => {
            // Should have been short-circuited by `is_notification()`; if we got here it
            // means the client sent an id, which the spec allows but is unusual. Just
            // return success.
            Response::success(id, Value::Null)
        }
        other => Response::error(
            id,
            err(
                codes::METHOD_NOT_FOUND,
                &format!("method not found: {other}"),
            ),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_tools(n: usize) -> AppState {
        let r = ToolRegistry::new();
        for i in 0..n {
            r.register(
                format!("cyberos.test.tool_{i}"),
                "test".into(),
                json!({"type":"object"}),
                crate::annotations::ToolAnnotations::read_only_idempotent("t"),
                "test".into(),
                "http://localhost/test".into(),
                vec!["mcp:tools".into()],
            );
        }
        AppState {
            registry: Arc::new(r),
            oauth_pool: None,
            elicitations: Arc::new(crate::elicitation::ElicitationStore::new()),
            tasks: Arc::new(crate::tasks::TaskStore::new()),
            kms: None,
        }
    }

    #[tokio::test]
    async fn dispatch_unknown_method_is_method_not_found() {
        let state = state_with_tools(0);
        let req = Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "no/such/thing".into(),
            params: None,
        };
        let r = dispatch_one(&state, req, None).await;
        assert!(r.error.is_some());
        assert_eq!(r.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn dispatch_initialize_with_correct_version_succeeds() {
        let state = state_with_tools(0);
        let req = Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "initialize".into(),
            params: Some(json!({"protocolVersion": MCP_PROTOCOL_VERSION})),
        };
        let r = dispatch_one(&state, req, None).await;
        assert!(r.error.is_none(), "got {:?}", r.error);
        let result = r.result.unwrap();
        assert_eq!(result["protocolVersion"], MCP_PROTOCOL_VERSION);
    }

    #[tokio::test]
    async fn dispatch_tools_list_returns_descriptors() {
        let state = state_with_tools(3);
        let req = Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "tools/list".into(),
            params: None,
        };
        let r = dispatch_one(&state, req, None).await;
        assert!(r.error.is_none());
        let tools = &r.result.unwrap()["tools"];
        assert_eq!(tools.as_array().unwrap().len(), 3);
    }

    // ---- TASK-MCP-005 Protected Resource Metadata ------------------------------------------

    #[test]
    fn www_authenticate_challenge_points_at_the_prm() {
        let c = www_authenticate_challenge();
        assert!(c.starts_with("Bearer "));
        assert!(c.contains("resource_metadata=\""));
        assert!(c.contains("/.well-known/oauth-protected-resource"));
    }

    #[tokio::test]
    async fn prm_aggregate_is_public_json_with_caching_headers() {
        use tower::ServiceExt;
        let res = build_router(state_with_tools(0))
            .oneshot(
                axum::http::Request::get("/.well-known/oauth-protected-resource")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(
            res.headers()["content-type"].to_str().unwrap(),
            "application/json; charset=utf-8"
        );
        assert_eq!(
            res.headers()["cache-control"].to_str().unwrap(),
            "max-age=3600"
        );
        assert!(res.headers().contains_key("etag"));
        assert!(res.headers().contains_key("x-robots-tag"));
        assert_eq!(
            res.headers()["access-control-allow-origin"]
                .to_str()
                .unwrap(),
            "*"
        );
    }

    #[tokio::test]
    async fn prm_etag_revalidation_returns_304() {
        use tower::ServiceExt;
        let state = state_with_tools(0);
        let first = build_router(state.clone())
            .oneshot(
                axum::http::Request::get("/.well-known/oauth-protected-resource")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let etag = first.headers()["etag"].to_str().unwrap().to_owned();
        let second = build_router(state)
            .oneshot(
                axum::http::Request::get("/.well-known/oauth-protected-resource")
                    .header("if-none-match", &etag)
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn prm_per_module_known_ok_unknown_404() {
        use tower::ServiceExt;
        let state = state_with_tools(0);
        state.registry.register(
            "cyberos.projects.read_board".into(),
            "t".into(),
            json!({"type":"object"}),
            crate::annotations::ToolAnnotations::read_only_idempotent("t"),
            "projects".into(),
            "http://localhost/projects".into(),
            vec!["projects.read".into()],
        );
        let known = build_router(state.clone())
            .oneshot(
                axum::http::Request::get("/.well-known/oauth-protected-resource/cyberos.projects")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(known.status(), StatusCode::OK);

        let unknown = build_router(state)
            .oneshot(
                axum::http::Request::get("/.well-known/oauth-protected-resource/cyberos.bogus")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    }

    // ---- TASK-MCP-008 elicitation HTTP surface ---------------------------------------------

    #[tokio::test]
    async fn elicitation_poll_respond_validate_and_404() {
        use tower::ServiceExt;
        let state = state_with_tools(0);

        // Poll is reachable and empty initially.
        let empty = build_router(state.clone())
            .oneshot(
                axum::http::Request::get("/v1/mcp/elicitations")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(empty.status(), StatusCode::OK);

        // Responding to an unknown id is a 404.
        let unknown = build_router(state.clone())
            .oneshot(
                axum::http::Request::post(format!(
                    "/v1/mcp/elicitations/{}/respond",
                    uuid::Uuid::new_v4()
                ))
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    r#"{"response_payload":{"value":"x"}}"#,
                ))
                .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);

        // A server-created confirmation, approved, records and reads back as confirmed.
        let e = state
            .elicitations
            .create_confirmation("cyberos.kb.bulk_delete", json!({ "title": "ok?" }));
        let ok = build_router(state.clone())
            .oneshot(
                axum::http::Request::post(format!("/v1/mcp/elicitations/{}/respond", e.id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(
                        r#"{"response_payload":{"confirmed":true}}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(ok.status(), StatusCode::OK);
        assert!(state.elicitations.is_confirmed(e.id));

        // A missing required field is a 422.
        let e2 = state.elicitations.create(
            "t",
            crate::elicitation::ElicitationType::StringInput,
            json!({}),
            Vec::new(),
        );
        let invalid = build_router(state)
            .oneshot(
                axum::http::Request::post(format!("/v1/mcp/elicitations/{}/respond", e2.id))
                    .header("content-type", "application/json")
                    .body(axum::body::Body::from(r#"{"response_payload":{}}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    // ---- TASK-MCP-006 destructive-tool gating ----------------------------------------------

    fn register_gated_tool(
        state: &AppState,
        name: &str,
        annotations: crate::annotations::ToolAnnotations,
    ) {
        state.registry.register(
            name.into(),
            "t".into(),
            json!({"type":"object"}),
            annotations,
            "kb".into(),
            "http://127.0.0.1:9/mcp".into(), // refused -> module_unreachable when forwarded
            vec!["mcp:tools".into()],
        );
    }

    fn tools_call_req(params: Value) -> Request {
        Request {
            jsonrpc: "2.0".into(),
            id: Some(json!(1)),
            method: "tools/call".into(),
            params: Some(params),
        }
    }

    #[tokio::test]
    async fn read_only_tool_forwards_through_the_gate() {
        let state = state_with_tools(0);
        register_gated_tool(
            &state,
            "cyberos.kb.search",
            crate::annotations::ToolAnnotations::read_only_idempotent("Search"),
        );
        let r = dispatch_one(
            &state,
            tools_call_req(json!({ "name": "cyberos.kb.search", "arguments": {} })),
            None,
        )
        .await;
        // Not held: it forwards and the (refused) module is unreachable.
        assert_eq!(r.error.unwrap().code, codes::MODULE_UNREACHABLE);
    }

    #[tokio::test]
    async fn destructive_tool_without_confirmation_is_held() {
        let state = state_with_tools(0);
        register_gated_tool(
            &state,
            "cyberos.kb.bulk_delete",
            crate::annotations::ToolAnnotations::destructive("Bulk delete"),
        );
        let r = dispatch_one(
            &state,
            tools_call_req(json!({ "name": "cyberos.kb.bulk_delete", "arguments": {} })),
            None,
        )
        .await;
        assert!(r.error.is_none(), "held is a success result, not an error");
        assert_eq!(
            r.result.unwrap()["structuredContent"]["elicitation_required"],
            true
        );
        assert_eq!(state.elicitations.pending().len(), 1);
    }

    #[tokio::test]
    async fn destructive_tool_with_confirmation_forwards() {
        let state = state_with_tools(0);
        register_gated_tool(
            &state,
            "cyberos.kb.bulk_delete",
            crate::annotations::ToolAnnotations::destructive("Bulk delete"),
        );
        let e = state
            .elicitations
            .create_confirmation("cyberos.kb.bulk_delete", json!({}));
        state
            .elicitations
            .respond(e.id, json!({ "confirmed": true }));
        let r = dispatch_one(
            &state,
            tools_call_req(json!({
                "name": "cyberos.kb.bulk_delete",
                "arguments": {},
                "_meta": { "confirmation_id": e.id.to_string() }
            })),
            None,
        )
        .await;
        // Confirmed: it forwards, and the (refused) module is unreachable.
        assert_eq!(r.error.unwrap().code, codes::MODULE_UNREACHABLE);
    }

    #[tokio::test]
    async fn destructive_tool_declined_aborts_cleanly() {
        let state = state_with_tools(0);
        register_gated_tool(
            &state,
            "cyberos.kb.bulk_delete",
            crate::annotations::ToolAnnotations::destructive("Bulk delete"),
        );
        let e = state
            .elicitations
            .create_confirmation("cyberos.kb.bulk_delete", json!({}));
        state
            .elicitations
            .respond(e.id, json!({ "confirmed": false }));
        let r = dispatch_one(
            &state,
            tools_call_req(json!({
                "name": "cyberos.kb.bulk_delete",
                "arguments": {},
                "_meta": { "confirmation_id": e.id.to_string() }
            })),
            None,
        )
        .await;
        assert!(
            r.error.is_none(),
            "declined returns an in-band tool error result"
        );
        assert_eq!(
            r.result.unwrap()["structuredContent"]["user_rejected"],
            true
        );
    }

    // ---- TASK-MCP-007 tasks HTTP surface ---------------------------------------------------

    #[tokio::test]
    async fn task_status_and_cancel_over_http() {
        use tower::ServiceExt;
        let state = state_with_tools(0);
        let t = state.tasks.start("cyberos.kb.reindex");

        // Status of a running task is reachable.
        let running = build_router(state.clone())
            .oneshot(
                axum::http::Request::get(format!("/v1/mcp/tasks/{}", t.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(running.status(), StatusCode::OK);

        // An unknown handle is a 404.
        let unknown = build_router(state.clone())
            .oneshot(
                axum::http::Request::get(format!("/v1/mcp/tasks/{}", uuid::Uuid::new_v4()))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);

        // Cancel transitions the task; a second cancel is a 409.
        let cancelled = build_router(state.clone())
            .oneshot(
                axum::http::Request::post(format!("/v1/mcp/tasks/{}/cancel", t.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(cancelled.status(), StatusCode::OK);
        assert_eq!(
            state.tasks.get(t.id).unwrap().status,
            crate::tasks::TaskStatus::Cancelled
        );

        let again = build_router(state)
            .oneshot(
                axum::http::Request::post(format!("/v1/mcp/tasks/{}/cancel", t.id))
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(again.status(), StatusCode::CONFLICT);
    }
}
