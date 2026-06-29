//! FR-AUTH-110 slice 1b-endpoints (part A) - the OIDC-provider HTTP surface that
//! does not need the authorize/token round-trip: discovery + userinfo (public),
//! and the admin RP-registry CRUD. The authorize + token handlers (the round
//! trip, with cookie reading, redirects, PKCE, and minting) are the final
//! increment, composing the slice-1a/1b-data pieces.
//!
//! Audit-chain emits (the `op::audit` payloads written into l1) are wired in the
//! authorize/token increment where they carry the most weight; the read-only
//! discovery/userinfo paths and the admin CRUD do not anchor chain rows here.

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Redirect, Response},
    Extension,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::{Claims, JwtService};
use crate::AppState;

use super::code_store::{self, NewAuthCode};
use super::discovery::openid_configuration;
use super::errors::OpError;
use super::id_token;
use super::pkce;
use super::redirect::redirect_uri_registered;
use super::rp_client::{self, NewRpClient};
use super::sso_session;

type ApiErr = (StatusCode, Json<Value>);

/// Map a closed `OpError` to an HTTP error response.
fn op_err(e: OpError) -> ApiErr {
    (
        StatusCode::from_u16(e.http_status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(json!({ "error": e.code() })),
    )
}

/// GET /.well-known/openid-configuration (public, DEC-2492).
pub async fn discovery(State(state): State<AppState>) -> Json<Value> {
    let issuer = state.jwt_issuer.trim_end_matches('/');
    let jwks_uri = format!("{issuer}/.well-known/jwks.json");
    Json(openid_configuration(
        issuer,
        &jwks_uri,
        &["authorization_code"],
    ))
}

/// GET /v1/auth/op/userinfo (public; bearer access_token, DEC-2487). Revoke-gated
/// (§1 #11): a revoked subject's token is refused even if otherwise valid.
pub async fn userinfo(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiErr> {
    let token = bearer(&headers).ok_or_else(|| unauthorized("missing_bearer"))?;
    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let claims = svc
        .verify(token)
        .await
        .map_err(|_| unauthorized("invalid_token"))?;
    if !subject_is_active(&state, &claims.tenant_id, &claims.sub).await {
        return Err(unauthorized("revoked"));
    }
    Ok(Json(json!({
        "sub": claims.sub,
        "email": claims.email,
        "email_verified": !claims.email.is_empty(),
        "tenant_id": claims.tenant_id,
        "roles": claims.roles,
    })))
}

#[derive(Debug, Deserialize)]
pub struct AuthorizeQuery {
    pub client_id: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub response_type: String,
    #[serde(default)]
    pub scope: String,
    #[serde(default)]
    pub state: String,
    #[serde(default)]
    pub code_challenge: String,
    #[serde(default)]
    pub code_challenge_method: String,
    pub nonce: Option<String>,
}

/// GET /v1/auth/op/authorize (public). Validates the RP + redirect + PKCE,
/// resolves the human from the `__Host-cyberos_sso` cookie (silent SSO), revoke-
/// gates, then issues a single-use code and redirects back to the RP. An unknown
/// client or a redirect_uri mismatch renders an error and is NEVER redirected
/// (open-redirect defense, DEC-2491). When there is no active SSO session it
/// redirects back with `error=login_required`; the upstream-Google broker-and-
/// resume that establishes the session is the slice-1b-B follow-up.
pub async fn authorize(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<AuthorizeQuery>,
) -> Response {
    let rp = match rp_client::get_by_client_id(&state.pg, &q.client_id).await {
        Ok(Some(rp)) => rp,
        Ok(None) => return op_error_page(OpError::UnknownClient),
        Err(e) => return op_error_page(e),
    };
    if !redirect_uri_registered(&rp.redirect_uris, &q.redirect_uri) {
        return op_error_page(OpError::RedirectMismatch);
    }
    // Past this point, protocol errors redirect back with ?error=&state=.
    if q.response_type != "code" {
        return redirect_err(&q.redirect_uri, "unsupported_response_type", &q.state);
    }
    if !q.scope.split_whitespace().any(|s| s == "openid") {
        return redirect_err(&q.redirect_uri, "invalid_request", &q.state);
    }
    if q.code_challenge.is_empty() || q.code_challenge_method != "S256" {
        return redirect_err(&q.redirect_uri, "invalid_request", &q.state);
    }
    // Silent SSO: resolve the subject from the cookie-borne session.
    let session = match sso_cookie(&headers) {
        Some(sid) => match sso_session::lookup_active(&state.pg, rp.tenant_id, sid).await {
            Ok(Some(s)) => s,
            Ok(None) => return broker_to_google(&state, &rp, &q).await,
            Err(e) => return op_error_page(e),
        },
        None => return broker_to_google(&state, &rp, &q).await,
    };
    // Revoke gate (DEC-2488): a revoked subject cannot obtain a code.
    if !subject_is_active(
        &state,
        &rp.tenant_id.to_string(),
        &session.subject_id.to_string(),
    )
    .await
    {
        let _ = super::audit::emit(
            &state.pg,
            rp.tenant_id,
            session.subject_id,
            &format!("auth/op/{}/authorize_denied", rp.client_id),
            super::audit::OpAuthorizeDeniedPayload {
                tenant_id: rp.tenant_id,
                rp_client_id: &rp.client_id,
                subject_id: Some(session.subject_id),
                reason: "subject_revoked",
                source_ip: None,
            }
            .to_body_string(),
        )
        .await;
        return redirect_err(&q.redirect_uri, "access_denied", &q.state);
    }
    let _ = sso_session::touch(&state.pg, rp.tenant_id, session.id).await;
    let code = new_code();
    let new = NewAuthCode {
        code: &code,
        tenant_id: rp.tenant_id,
        rp_client_id: &rp.client_id,
        subject_id: session.subject_id,
        redirect_uri: &q.redirect_uri,
        code_challenge: &q.code_challenge,
        nonce: q.nonce.as_deref(),
        scope: &q.scope,
        sso_session_id: session.id,
    };
    if let Err(e) = code_store::insert_code(&state.pg, &new).await {
        return op_error_page(e);
    }
    let _ = super::audit::emit(
        &state.pg,
        rp.tenant_id,
        session.subject_id,
        &format!("auth/op/{}/authorize_issued", rp.client_id),
        super::audit::OpAuthorizeIssuedPayload {
            tenant_id: rp.tenant_id,
            rp_client_id: &rp.client_id,
            subject_id: session.subject_id,
        }
        .to_body_string(),
    )
    .await;
    let url = format!(
        "{}?code={}&state={}",
        q.redirect_uri,
        pct(&code),
        pct(&q.state)
    );
    Redirect::to(&url).into_response()
}

#[derive(Debug, Deserialize)]
pub struct TokenForm {
    pub grant_type: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub redirect_uri: String,
    #[serde(default)]
    pub code_verifier: String,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

/// POST /v1/auth/op/token (public, form-encoded). Exchanges a code for an
/// id_token + access_token after RP authentication, redirect+PKCE re-check,
/// revoke gate, and single-use consume (DEC-2490). RP auth is HTTP Basic or
/// form `client_secret` (client_secret_post), verified against the stored hash.
pub async fn token(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::Form(form): axum::Form<TokenForm>,
) -> Result<Json<Value>, ApiErr> {
    if form.grant_type != "authorization_code" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "unsupported_grant_type" })),
        ));
    }
    let (client_id, client_secret) =
        client_credentials(&headers, &form).ok_or_else(|| op_err(OpError::InvalidClient))?;
    let rp = rp_client::get_by_client_id(&state.pg, &client_id)
        .await
        .map_err(op_err)?
        .ok_or_else(|| op_err(OpError::InvalidClient))?;
    if !rp_client::verify_secret(&client_secret, &rp.client_secret_hash) {
        return Err(op_err(OpError::InvalidClient));
    }
    let stored = code_store::lookup_code(&state.pg, rp.tenant_id, &form.code)
        .await
        .map_err(op_err)?;
    if stored.rp_client_id != rp.client_id
        || stored.redirect_uri != form.redirect_uri
        || !pkce::verify_s256(&form.code_verifier, &stored.code_challenge)
    {
        return Err(op_err(OpError::InvalidGrant));
    }
    if !subject_is_active(
        &state,
        &rp.tenant_id.to_string(),
        &stored.subject_id.to_string(),
    )
    .await
    {
        return Err(op_err(OpError::AccessDenied));
    }
    // Single-use: consume before minting, so a replay yields invalid_grant and
    // no second token is ever issued.
    code_store::consume(&state.pg, rp.tenant_id, &form.code)
        .await
        .map_err(op_err)?;

    let (email, kind) = load_subject_identity(&state, rp.tenant_id, stored.subject_id).await;
    let roles =
        crate::handlers::load_subject_roles_pub(&state, rp.tenant_id, stored.subject_id, &[]).await;
    let rbac_v = state.role_matrix.read().await.version();
    let svc = JwtService::new(state.pg.clone(), state.jwt_issuer.clone());

    let issued = svc
        .issue(
            cyberos_types::TenantId(rp.tenant_id),
            cyberos_types::SubjectId(stored.subject_id),
            &email,
            &kind,
            vec!["openid".to_string()],
            roles.clone(),
            Some(rbac_v),
            None,
            None,
        )
        .await
        .map_err(|_| op_err(OpError::ServerError))?;

    let (kid, pem) = svc
        .active_signing_key()
        .await
        .map_err(|_| op_err(OpError::ServerError))?;
    let issuer = state.jwt_issuer.trim_end_matches('/');
    let username = email.split('@').next().unwrap_or("").to_string();
    let iat = chrono::Utc::now().timestamp();
    let claims = id_token::build_id_token_claims(
        issuer,
        &stored.subject_id.to_string(),
        &rp.client_id,
        &email,
        !email.is_empty(),
        &username,
        &username,
        &rp.tenant_id.to_string(),
        roles,
        stored.nonce.clone(),
        iat,
        3600,
    );
    let id_tok = id_token::sign_id_token(&claims, &kid, &pem).map_err(op_err)?;

    let _ = super::audit::emit(
        &state.pg,
        rp.tenant_id,
        stored.subject_id,
        &format!("auth/op/{}/token_issued", rp.client_id),
        super::audit::OpTokenIssuedPayload {
            tenant_id: rp.tenant_id,
            rp_client_id: &rp.client_id,
            subject_id: stored.subject_id,
            scope: &stored.scope,
        }
        .to_body_string(),
    )
    .await;

    Ok(Json(json!({
        "access_token": issued.access_token,
        "id_token": id_tok,
        "token_type": "Bearer",
        "expires_in": issued.expires_in,
        "scope": stored.scope,
    })))
}

#[derive(Debug, Deserialize)]
pub struct CreateRpClientBody {
    pub name: String,
    pub client_id: String,
    pub redirect_uris: Vec<String>,
    #[serde(default)]
    pub post_logout_redirect_uris: Vec<String>,
    #[serde(default)]
    pub allow_refresh: bool,
}

/// POST /v1/admin/op/rp-clients (tenant-admin). Returns the one-time secret
/// (DEC-2497); only its hash is stored.
pub async fn create_rp_client(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(body): Json<CreateRpClientBody>,
) -> Result<(StatusCode, Json<Value>), ApiErr> {
    let tenant_id = require_tenant_admin(&claims)?;
    let created_by = Uuid::parse_str(&claims.sub).map_err(|_| forbidden())?;
    if body.redirect_uris.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "redirect_uris_required" })),
        ));
    }
    let (id, secret) = rp_client::create(
        &state.pg,
        &NewRpClient {
            tenant_id,
            name: &body.name,
            client_id: &body.client_id,
            redirect_uris: body.redirect_uris.clone(),
            post_logout_redirect_uris: body.post_logout_redirect_uris.clone(),
            allow_refresh: body.allow_refresh,
            created_by_subject_id: created_by,
        },
    )
    .await
    .map_err(op_err)?;
    let _ = super::audit::emit(
        &state.pg,
        tenant_id,
        created_by,
        &format!("auth/op/{}/changed", body.client_id),
        super::audit::OpRpClientChangedPayload {
            tenant_id,
            rp_client_id: &body.client_id,
            changed_by_subject_id: created_by,
            change: "created",
        }
        .to_body_string(),
    )
    .await;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": id.to_string(),
            "client_id": body.client_id,
            "client_secret": secret,
            "redirect_uris": body.redirect_uris,
            "allow_refresh": body.allow_refresh,
        })),
    ))
}

/// GET /v1/admin/op/rp-clients (tenant-admin). Never returns secrets or hashes.
pub async fn list_rp_clients(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, ApiErr> {
    let tenant_id = require_tenant_admin(&claims)?;
    let clients = rp_client::list_for_tenant(&state.pg, tenant_id)
        .await
        .map_err(op_err)?;
    let view: Vec<Value> = clients
        .into_iter()
        .map(|c| {
            json!({
                "id": c.id.to_string(),
                "name": c.name,
                "client_id": c.client_id,
                "redirect_uris": c.redirect_uris,
                "post_logout_redirect_uris": c.post_logout_redirect_uris,
                "allow_refresh": c.allow_refresh,
                "is_active": c.is_active,
            })
        })
        .collect();
    Ok(Json(json!({ "clients": view })))
}

/// DELETE /v1/admin/op/rp-clients/:client_id (tenant-admin). Soft-delete.
pub async fn delete_rp_client(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(client_id): Path<String>,
) -> Result<(StatusCode, Json<Value>), ApiErr> {
    let tenant_id = require_tenant_admin(&claims)?;
    let removed = rp_client::soft_delete(&state.pg, tenant_id, &client_id)
        .await
        .map_err(op_err)?;
    if removed {
        Ok((StatusCode::OK, Json(json!({ "deactivated": client_id }))))
    } else {
        Err((StatusCode::NOT_FOUND, Json(json!({ "error": "not_found" }))))
    }
}

// ---------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------

/// Render a closed `OpError` as an error page (used where redirecting would be
/// unsafe: unknown client / redirect mismatch).
fn op_error_page(e: OpError) -> Response {
    op_err(e).into_response()
}

/// Redirect back to the RP's redirect_uri with `?error=&state=` (used for
/// protocol errors after the client + redirect_uri have been validated).
fn redirect_err(redirect_uri: &str, error: &str, state: &str) -> Response {
    let url = format!("{redirect_uri}?error={}&state={}", pct(error), pct(state));
    Redirect::to(&url).into_response()
}

/// Parse the `__Host-cyberos_sso` session id from the Cookie header.
fn sso_cookie(headers: &HeaderMap) -> Option<Uuid> {
    let raw = headers.get(header::COOKIE)?.to_str().ok()?;
    for part in raw.split(';') {
        if let Some(v) = part.trim().strip_prefix("__Host-cyberos_sso=") {
            return Uuid::parse_str(v).ok();
        }
    }
    None
}

/// A fresh 256-bit authorization code, base64url-nopad.
fn new_code() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

/// Percent-encode a query-parameter value (RFC 3986 unreserved set).
fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Extract a `Bearer <token>` from the Authorization header.
fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
}

fn unauthorized(reason: &str) -> ApiErr {
    (StatusCode::UNAUTHORIZED, Json(json!({ "error": reason })))
}

fn forbidden() -> ApiErr {
    (
        StatusCode::FORBIDDEN,
        Json(json!({ "error": "forbidden", "needed": "tenant-admin" })),
    )
}

/// The create_subject role gate: caller MUST hold `tenant-admin` in their tenant,
/// OR be `root-admin` in tenant 0. Returns the caller's tenant_id.
fn require_tenant_admin(claims: &Claims) -> Result<Uuid, ApiErr> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(|_| forbidden())?;
    let has_tenant_admin = claims.roles.iter().any(|r| r == "tenant-admin");
    let is_root_admin_zero =
        tenant_id == Uuid::nil() && claims.roles.iter().any(|r| r == "root-admin");
    if has_tenant_admin || is_root_admin_zero {
        Ok(tenant_id)
    } else {
        Err(forbidden())
    }
}

/// True iff `subjects.status = 'active'` for the subject, read under its tenant
/// GUC. Any error (bad UUID, db failure, missing row, revoked) returns false so
/// the revoke gate fails closed.
async fn subject_is_active(state: &AppState, tenant_id: &str, subject_id: &str) -> bool {
    let sid = match Uuid::parse_str(subject_id) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut tx = match state.pg.begin().await {
        Ok(t) => t,
        Err(_) => return false,
    };
    if sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id)
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return false;
    }
    let row: Option<(String,)> = sqlx::query_as("SELECT status FROM subjects WHERE id = $1")
        .bind(sid)
        .fetch_optional(&mut *tx)
        .await
        .unwrap_or(None);
    let _ = tx.commit().await;
    matches!(row, Some((s,)) if s == "active")
}

/// Resolve the RP's (client_id, client_secret) from HTTP Basic, falling back to
/// the form body (`client_secret_post`).
fn client_credentials(headers: &HeaderMap, form: &TokenForm) -> Option<(String, String)> {
    if let Some(v) = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
    {
        if let Some(b64) = v.strip_prefix("Basic ") {
            if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim()) {
                if let Ok(s) = String::from_utf8(bytes) {
                    if let Some((id, secret)) = s.split_once(':') {
                        return Some((id.to_string(), secret.to_string()));
                    }
                }
            }
        }
    }
    match (form.client_id.clone(), form.client_secret.clone()) {
        (Some(id), Some(secret)) => Some((id, secret)),
        _ => None,
    }
}

/// Load `(email, kind)` for a subject under its tenant GUC. Defaults to
/// `("", "human")` on any miss so the mint still produces a usable token.
async fn load_subject_identity(
    state: &AppState,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> (String, String) {
    let mut tx = match state.pg.begin().await {
        Ok(t) => t,
        Err(_) => return (String::new(), "human".to_string()),
    };
    if sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .is_err()
    {
        return (String::new(), "human".to_string());
    }
    let row: Option<(Option<String>, String)> =
        sqlx::query_as("SELECT email, kind FROM subjects WHERE id = $1")
            .bind(subject_id)
            .fetch_optional(&mut *tx)
            .await
            .unwrap_or(None);
    let _ = tx.commit().await;
    match row {
        Some((email, kind)) => (email.unwrap_or_default(), kind),
        None => (String::new(), "human".to_string()),
    }
}

/// FR-AUTH-110 broker: no usable SSO session, so send the user through the
/// tenant's Google IdP (FR-AUTH-104) and resume this authorize on return. The
/// callback mints the SSO session, sets the cookie, and 302s back here.
async fn broker_to_google(
    state: &AppState,
    rp: &rp_client::RpClient,
    q: &AuthorizeQuery,
) -> Response {
    let resume = authorize_url(state, q);
    match crate::oidc::begin_idp_flow(state, rp.tenant_id, "google-workspace", Some(resume)).await {
        Ok(url) => Redirect::to(&url).into_response(),
        Err((status, body)) => (status, body).into_response(),
    }
}

/// Reconstruct the original `/v1/auth/op/authorize` URL so the broker can return
/// the browser to it after Google (where the now-present cookie makes it
/// succeed via silent SSO).
fn authorize_url(state: &AppState, q: &AuthorizeQuery) -> String {
    let issuer = state.jwt_issuer.trim_end_matches('/');
    let nonce = q
        .nonce
        .as_deref()
        .map(|n| format!("&nonce={}", pct(n)))
        .unwrap_or_default();
    format!(
        "{issuer}/v1/auth/op/authorize?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256{nonce}",
        pct(&q.client_id),
        pct(&q.redirect_uri),
        pct(&q.scope),
        pct(&q.state),
        pct(&q.code_challenge),
    )
}
