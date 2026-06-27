//! FR-MCP-001 §1 #25 + §1 #26 — Axum router mounting `POST /mcp` + `GET /mcp/healthz`,
//! plus the FR-MCP-002 control plane (`/v1/mcp/register`, `/heartbeat`, `/deregister`).

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
use crate::protocol::tools_call::{dispatch as call_dispatch, ToolsCallParams};
use crate::protocol::tools_list::{build_response as build_tools_list, ToolsListParams};
use crate::MCP_PROTOCOL_VERSION;

/// Shared state passed through every handler.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Federated tool registry.
    pub registry: Arc<ToolRegistry>,
    /// Postgres pool for the FR-MCP-004 OAuth endpoints. `None` when `MCP_DATABASE_URL` is unset: the
    /// OAuth endpoints then report unconfigured and the rest of the gateway runs unaffected, so the
    /// dev demo needs no database.
    pub oauth_pool: Option<sqlx::PgPool>,
}

/// FR-MCP-001 §1 #25 healthz payload.
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
    /// Per-module server health (FR-MCP-002).
    pub servers: Vec<serde_json::Value>,
    /// Whether the FR-MCP-004 OAuth endpoints are configured (a database is connected).
    pub oauth_configured: bool,
}

/// Whether the FR-MCP-002 control-plane routes (register/heartbeat/deregister) are enabled.
/// Off unless `MCP_DEV_REGISTRATION=1`, because they mutate what the gateway dispatches to.
fn control_plane_enabled() -> bool {
    std::env::var("MCP_DEV_REGISTRATION").as_deref() == Ok("1")
}

fn control_plane_disabled_response() -> (StatusCode, Json<Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!({
            "error": "registration_disabled",
            "detail": "set MCP_DEV_REGISTRATION=1 to enable the dev control plane; production requires authenticated registration (FR-MCP-004)"
        })),
    )
}

/// Build the Axum router. `POST /mcp` + `GET /mcp/healthz` are the MCP protocol surface;
/// `/v1/mcp/{register,heartbeat,deregister}` are the FR-MCP-002 control plane.
pub fn build_router(state: AppState) -> Router {
    Router::new()
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
        .with_state(state)
}

/// FR-MCP-002 control-plane: a module registers its tool catalogue so `tools/list` and
/// `tools/call` can see it.
///
/// Trust boundary: registration changes what the gateway will forward `tools/call` to, so
/// it is privileged. This dev slice gates the route behind `MCP_DEV_REGISTRATION=1` (off by
/// default). Production must replace this with authenticated registration (FR-MCP-004) plus
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
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "invalid_registration", "detail": e.message() })),
        );
    }

    let n = apply_registration(&state.registry, &req);
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

/// FR-MCP-004 clause #20 - RFC 8414 authorization-server metadata at
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

/// FR-MCP-005 gateway-aggregate Protected Resource Metadata (RFC 9728). Public and unauthenticated per
/// DEC-896; cacheable via ETag. `get(...)` also serves HEAD (axum strips the body).
async fn handle_prm_aggregate(headers: HeaderMap) -> axum::response::Response {
    let doc = crate::oauth::prm::protected_resource_metadata(
        &oauth_resource(),
        &oauth_authorization_servers(),
    );
    prm_http_response(&doc, if_none_match(&headers))
}

/// FR-MCP-005 per-module Protected Resource Metadata. The path segment is `cyberos.<module>` (DEC-897);
/// `scopes_supported` is the union of the module's tools' required scopes from the FR-MCP-002 registry.
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

/// The FR-AUTH-004 issuer whose session JWTs `/authorize` and confidential `/register` accept (env
/// `MCP_AUTH_ISSUER`, dev default). Distinct from `oauth_issuer()`, which issues the gateway's own
/// access tokens.
fn oauth_auth_issuer() -> String {
    std::env::var("MCP_AUTH_ISSUER").unwrap_or_else(|_| "http://localhost:8081".to_string())
}

/// FR-MCP-005 `authorization_servers`: the issuer URLs whose access tokens this MCP resource accepts.
/// The gateway is its own authorization server (it mints the tokens `enforce_oauth` verifies), so this
/// defaults to `[oauth_issuer()]`. `MCP_AUTHORIZATION_SERVERS` (comma- or whitespace-separated)
/// overrides it for a future multi-issuer residency map (FR-AUTH-004).
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

/// The `WWW-Authenticate` challenge for a 401 from the MCP surface (FR-MCP-005 §1 #19, RFC 9728 §5.1):
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

/// Whether the MCP surface requires a valid audience-bound OAuth token (FR-MCP-004 clause #23). Off
/// unless `MCP_REQUIRE_AUTH=1`, so the dev demo works with no token and production sets it on - the
/// same gating shape as `MCP_DEV_REGISTRATION`.
fn require_auth() -> bool {
    std::env::var("MCP_REQUIRE_AUTH").as_deref() == Ok("1")
}

/// Verify the request's bearer access token against this resource server: signature + issuer + expiry +
/// exact audience (enforced inside `verify_access_token`) + not-revoked (clauses #23, #24). Returns the
/// 401 response on any failure, or `Ok(())` to let the request proceed.
async fn enforce_oauth(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), axum::response::Response> {
    // Every 401 carries the FR-MCP-005 `WWW-Authenticate: Bearer ... resource_metadata=...` challenge
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
    Ok(())
}

/// FR-MCP-004 RFC 7591 dynamic client registration. JSON body; 503 when no database is configured.
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

/// FR-MCP-004 token endpoint. Form-encoded body (RFC 6749 §6).
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

/// FR-MCP-004 token revocation (RFC 7009). Always 200.
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

/// FR-MCP-004 token introspection (RFC 7662).
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

/// FR-MCP-004 authorize endpoint. The calling subject's identity comes from an auth-service bearer
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

/// FR-MCP-002 control-plane: a module heartbeats to stay healthy. Body: `{"module": "..."}`.
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

/// FR-MCP-002 control-plane: a module deregisters (its tools are withdrawn until it
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
    // FR-MCP-004 clauses #23/#24: when enabled, the MCP surface requires a valid audience-bound,
    // unrevoked access token. Off in dev (no token) so the demo keeps working. A 401 here carries the
    // FR-MCP-005 `WWW-Authenticate` resource-metadata challenge.
    if require_auth() {
        if let Err(resp) = enforce_oauth(&state, &headers).await {
            return resp;
        }
    }
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
            let resp = dispatch_one(&state, req).await;
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
                out.push(dispatch_one(&state, r).await);
            }
            (
                StatusCode::OK,
                Json(serde_json::to_value(out).expect("serialise")),
            )
                .into_response()
        }
    }
}

async fn dispatch_one(state: &AppState, req: Request) -> Response {
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
            // Slice-1: caller scopes come from JWT verification (FR-MCP-004) once wired;
            // for now we accept a permissive default and rely on FR-MCP-002+004 to
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
        let r = dispatch_one(&state, req).await;
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
        let r = dispatch_one(&state, req).await;
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
        let r = dispatch_one(&state, req).await;
        assert!(r.error.is_none());
        let tools = &r.result.unwrap()["tools"];
        assert_eq!(tools.as_array().unwrap().len(), 3);
    }

    // ---- FR-MCP-005 Protected Resource Metadata ------------------------------------------

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
        assert_eq!(res.headers()["cache-control"].to_str().unwrap(), "max-age=3600");
        assert!(res.headers().contains_key("etag"));
        assert!(res.headers().contains_key("x-robots-tag"));
        assert_eq!(res.headers()["access-control-allow-origin"].to_str().unwrap(), "*");
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
}
