//! TASK-AUTH-104 — OpenID Connect Authorization Code + PKCE SSO flow.
//!
//! Per-tenant IdP config in `oidc_idp_configs`. Three endpoints:
//!   * `GET /v1/auth/oidc/initiate?tenant_slug=...&idp=...&redirect=...`
//!     — generates state + PKCE verifier; returns the IdP authorization URL.
//!   * `GET /v1/auth/oidc/callback?state=...&code=...`
//!     — validates state, exchanges code → tokens, validates ID token,
//!       JIT-creates the subject if missing, mints a CyberOS access JWT.
//!   * `POST /v1/admin/oidc/idp-configs` (under admin router) — operator
//!     creates/updates an IdP config row.
//!
//! Discovery is cached at boot per (tenant_id, idp_config_id) tuple — the
//! `oidc_discovery` map lives in `AppState` (slice 2 — first slice just
//! re-fetches every call to keep state simple).

use axum::{
    extract::{Json as JsonInput, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Redirect, Response},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::AppState;

const STATE_BYTES: usize = 32;
const VERIFIER_BYTES: usize = 48;

// ---------------------------------------------------------------------------
// Admin endpoint — POST /v1/admin/oidc/idp-configs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateIdpConfigBody {
    pub name: String,
    pub discovery_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    #[serde(default)]
    pub scopes: Option<Vec<String>>,
    #[serde(default)]
    pub auto_provision: Option<bool>,
    #[serde(default)]
    pub default_roles: Option<Vec<String>>,
    /// P0 - when non-empty, the callback only admits logins whose verified email
    /// domain is in this list (e.g. ["cyberskill.world"]). Empty = no restriction.
    #[serde(default)]
    pub allowed_domains: Option<Vec<String>>,
}

pub async fn create_idp_config(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<crate::jwt::Claims>,
    JsonInput(body): JsonInput<CreateIdpConfigBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal)?;
    let scopes = body
        .scopes
        .unwrap_or_else(|| vec!["openid".into(), "email".into(), "profile".into()]);
    let auto = body.auto_provision.unwrap_or(true);
    let default_roles = body
        .default_roles
        .unwrap_or_else(|| vec!["tenant-member".into()]);
    // Normalise allowed domains to bare lowercase hostnames ("@CyberSkill.world"
    // and "cyberskill.world" both become "cyberskill.world") so the callback can
    // compare them directly against the email's domain part.
    let allowed_domains = body
        .allowed_domains
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.trim().trim_start_matches('@').to_ascii_lowercase())
        .filter(|d| !d.is_empty())
        .collect::<Vec<String>>();

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO oidc_idp_configs
              (tenant_id, name, discovery_url, client_id, client_secret,
               redirect_uri, scopes, auto_provision, default_roles, allowed_domains)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (tenant_id, name) DO UPDATE
            SET discovery_url = EXCLUDED.discovery_url,
                client_id     = EXCLUDED.client_id,
                client_secret = EXCLUDED.client_secret,
                redirect_uri  = EXCLUDED.redirect_uri,
                scopes        = EXCLUDED.scopes,
                auto_provision = EXCLUDED.auto_provision,
                default_roles = EXCLUDED.default_roles,
                allowed_domains = EXCLUDED.allowed_domains,
                updated_at    = NOW()
       RETURNING id",
    )
    .bind(tenant_id)
    .bind(&body.name)
    .bind(&body.discovery_url)
    .bind(&body.client_id)
    .bind(&body.client_secret)
    .bind(&body.redirect_uri)
    .bind(&scopes)
    .bind(auto)
    .bind(&default_roles)
    .bind(&allowed_domains)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": row.0,
            "tenant_id": tenant_id,
            "name": body.name,
            "allowed_domains": allowed_domains,
        })),
    ))
}

#[derive(Debug, Deserialize)]
pub struct InitiateQuery {
    pub tenant_slug: String,
    pub idp: String,
    /// Where the auth service redirects after success. Must be allow-listed
    /// against the tenant's portal config (slice 2; today accepts any HTTPS).
    pub redirect: Option<String>,
    /// P0 console hand-back - after the callback mints a CyberOS token, the
    /// browser is 302'd here with the token in the URL fragment so a single-page
    /// app can capture it. Allow-listed (AUTH_OIDC_RETURN_ALLOW; localhost in dev).
    pub return_to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InitiateResponse {
    pub authorization_url: String,
    pub state: String,
}

/// `GET /v1/auth/oidc/initiate` — PUBLIC endpoint (no JWT required). Returns
/// the IdP authorization URL the client should redirect the browser to.
pub async fn initiate(
    State(state): State<AppState>,
    Query(q): Query<InitiateQuery>,
) -> Result<(StatusCode, Json<InitiateResponse>), (StatusCode, Json<Value>)> {
    // Look up the IdP config under root context.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: Option<(Uuid, Uuid, String, String, String, Vec<String>)> = sqlx::query_as(
        "SELECT i.id, i.tenant_id, i.discovery_url, i.client_id, i.redirect_uri, i.scopes
             FROM oidc_idp_configs i
             JOIN tenants t ON t.id = i.tenant_id
            WHERE t.slug = $1 AND i.name = $2 AND i.status = 'active'",
    )
    .bind(&q.tenant_slug)
    .bind(&q.idp)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let (idp_config_id, tenant_id, discovery_url, client_id, registered_redirect, scopes) = row
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "idp_not_found", "tenant_slug": q.tenant_slug, "idp": q.idp})),
            )
        })?;

    // Discover authorization_endpoint.
    let discovery = discover(&discovery_url).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": "discovery_failed", "detail": e.to_string()})),
        )
    })?;

    // Generate state + PKCE.
    let state_token = random_b64(STATE_BYTES);
    let code_verifier = random_b64(VERIFIER_BYTES);
    let code_challenge = pkce_challenge(&code_verifier);
    let code_verifier_hash = sha256_hex(code_verifier.as_bytes());

    // Persist the initiate row so the callback can re-find the verifier.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query(
        "INSERT INTO oidc_login_history
                (tenant_id, idp_config_id, flow_state, state_token, code_verifier_hash, ts_ns)
         VALUES ($1, $2, 'initiated', $3, $4, $5)",
    )
    .bind(tenant_id)
    .bind(idp_config_id)
    .bind(&state_token)
    .bind(&code_verifier_hash)
    .bind(chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    // Compose the authorization URL.
    let redirect_uri = q.redirect.unwrap_or(registered_redirect);
    let scope_str = scopes.join(" ");
    let url = format!(
        "{authz}?response_type=code&client_id={cid}&redirect_uri={ru}\
&scope={scope}&state={state}&code_challenge={ch}&code_challenge_method=S256",
        authz = discovery.authorization_endpoint,
        cid = urlencode(&client_id),
        ru = urlencode(&redirect_uri),
        scope = urlencode(&scope_str),
        state = urlencode(&state_token),
        ch = code_challenge,
    );

    // Stash the verifier with the state token so callback can retrieve it.
    // (state_token → code_verifier mapping; in production this would be a
    // short-TTL cache. For slice 1 we re-derive by joining oidc_login_history
    // on state_token + tx-bound state.)
    state.oidc_pending.write().await.insert(
        state_token.clone(),
        PendingState {
            tenant_id,
            idp_config_id,
            redirect_uri,
            code_verifier,
            issued_at_secs: now_secs(),
            op_resume: None,
            return_to: q.return_to.clone(),
        },
    );

    Ok((
        StatusCode::OK,
        Json(InitiateResponse {
            authorization_url: url,
            state: state_token,
        }),
    ))
}

/// TASK-AUTH-110 broker - begin an upstream-IdP (Google) auth-code flow for a
/// tenant **by id**, carrying an `op_resume` URL the callback returns to after
/// it mints the AUTH SSO session. Returns the provider authorization URL the
/// browser should be redirected to. Deliberately kept separate from `initiate`
/// (which is by tenant slug and carries no `op_resume`) so the live Google
/// sign-in path is untouched; the small duplication is the safe trade.
pub(crate) async fn begin_idp_flow(
    state: &AppState,
    tenant_id: Uuid,
    idp_name: &str,
    op_resume: Option<String>,
) -> Result<String, (StatusCode, Json<Value>)> {
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: Option<(Uuid, String, String, String, Vec<String>)> = sqlx::query_as(
        "SELECT id, discovery_url, client_id, redirect_uri, scopes
             FROM oidc_idp_configs
            WHERE tenant_id = $1 AND name = $2 AND status = 'active'",
    )
    .bind(tenant_id)
    .bind(idp_name)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let (idp_config_id, discovery_url, client_id, registered_redirect, scopes) =
        row.ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "idp_not_found",
                    "tenant_id": tenant_id.to_string(),
                    "idp": idp_name
                })),
            )
        })?;

    let discovery = discover(&discovery_url).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": "discovery_failed", "detail": e.to_string()})),
        )
    })?;

    let state_token = random_b64(STATE_BYTES);
    let code_verifier = random_b64(VERIFIER_BYTES);
    let code_challenge = pkce_challenge(&code_verifier);
    let code_verifier_hash = sha256_hex(code_verifier.as_bytes());

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query(
        "INSERT INTO oidc_login_history
                (tenant_id, idp_config_id, flow_state, state_token, code_verifier_hash, ts_ns)
         VALUES ($1, $2, 'initiated', $3, $4, $5)",
    )
    .bind(tenant_id)
    .bind(idp_config_id)
    .bind(&state_token)
    .bind(&code_verifier_hash)
    .bind(chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let scope_str = scopes.join(" ");
    let url = format!(
        "{authz}?response_type=code&client_id={cid}&redirect_uri={ru}\
&scope={scope}&state={state}&code_challenge={ch}&code_challenge_method=S256",
        authz = discovery.authorization_endpoint,
        cid = urlencode(&client_id),
        ru = urlencode(&registered_redirect),
        scope = urlencode(&scope_str),
        state = urlencode(&state_token),
        ch = code_challenge,
    );

    state.oidc_pending.write().await.insert(
        state_token,
        PendingState {
            tenant_id,
            idp_config_id,
            redirect_uri: registered_redirect,
            code_verifier,
            issued_at_secs: now_secs(),
            op_resume,
            return_to: None,
        },
    );

    Ok(url)
}

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub state: String,
    pub code: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// `GET /v1/auth/oidc/callback` — PUBLIC. Exchanges authorization code →
/// access/id token, JIT-creates the subject, then issues a CyberOS JWT.
/// Slice 1 returns the JWT body inline; slice 2 wraps it in a 302 to the
/// caller-supplied `redirect_uri` with `?code=...` for second exchange.
pub async fn callback(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Query(q): Query<CallbackQuery>,
) -> Result<Response, (StatusCode, Json<Value>)> {
    let caller_ip = crate::handlers::caller_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    if let Some(err) = q.error {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "idp_returned_error",
                "idp_error": err,
                "detail": q.error_description.unwrap_or_default(),
            })),
        ));
    }
    let code = q.code.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "missing_code"})),
        )
    })?;

    // Recover the pending state.
    let pending = state
        .oidc_pending
        .write()
        .await
        .remove(&q.state)
        .ok_or_else(|| (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "invalid_state", "detail": "state token not found or already consumed"})),
        ))?;
    if now_secs() - pending.issued_at_secs > 600 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"error": "state_expired", "detail": "state token is older than 10 minutes"}),
            ),
        ));
    }

    // Fetch IdP config + discovery again.
    let idp = load_idp(&state, pending.idp_config_id).await?;
    let discovery = discover(&idp.discovery_url).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": "discovery_failed", "detail": e.to_string()})),
        )
    })?;

    // Exchange code → tokens.
    let token_resp = exchange_code(
        &discovery.token_endpoint,
        &idp.client_id,
        &idp.client_secret,
        &code,
        &pending.redirect_uri,
        &pending.code_verifier,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": "token_exchange_failed", "detail": e.to_string()})),
        )
    })?;

    // Verify the ID token (TASK-AUTH-104, RFC 7517): fetch the IdP JWKS and check
    // the RS256 signature, the issuer, the audience (our client_id), and expiry
    // before trusting any claim. The JWKS is fetched fresh per login so a key
    // rotation at the IdP is picked up without a stale cache.
    let jwks = fetch_jwks(&discovery.jwks_uri).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({"error": "jwks_fetch_failed", "detail": e.to_string()})),
        )
    })?;
    let id_claims = verify_id_token(
        &token_resp.id_token,
        &jwks,
        &discovery.issuer,
        &idp.client_id,
    )
    .map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "id_token_verification_failed", "detail": e.to_string()})),
        )
    })?;
    let idp_sub: String = id_claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": "id_token_missing_sub"})),
            )
        })?
        .to_string();
    let idp_email = id_claims
        .get("email")
        .and_then(|v| v.as_str())
        .map(String::from);

    // P0 Workspace-domain gate (TASK-AUTH-104). When the IdP restricts domains, a
    // verified email is mandatory and its domain must be on the allow-list. This
    // is what keeps Google sign-in scoped to @cyberskill.world: a personal Gmail
    // is rejected here before any subject is touched.
    {
        // Google returns email_verified as a JSON bool; some IdPs use the string
        // "true". Accept both, default to not-verified.
        let verified = id_claims
            .get("email_verified")
            .map(|v| v.as_bool().unwrap_or(v.as_str() == Some("true")))
            .unwrap_or(false);
        if let Err(reason) =
            email_domain_admitted(idp_email.as_deref(), verified, &idp.allowed_domains)
        {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({ "error": reason, "allowed_domains": idp.allowed_domains })),
            ));
        }
    }

    // TASK-AUTH-111 — the person's name, from the ID token we already verified. Read here, where the claims
    // are, so `resolve_subject` is handed a name rather than reaching back for one.
    //
    // Deserialising from the already-verified `id_claims` adds no trust boundary: signature, issuer,
    // audience and expiry were all checked above. Unknown claims — `picture` among them — are dropped on the
    // floor, which is the intent (§1 #9): persisting the avatar would widen the published Data Safety
    // declaration, and that is a policy change with paperwork, not an implementation detail.
    let profile: crate::display_name::Profile =
        serde_json::from_value(id_claims.clone()).unwrap_or_default();
    let (rung, display_name) = crate::display_name::resolve(&profile, idp_email.as_deref());
    // The RUNG, never the name (§1 #8). Knowing which rung fired is most of the diagnostic value; putting
    // the name itself in the log stream is a privacy incident and contradicts the policy we publish.
    tracing::debug!(target: "cyberos_auth::oidc", rung, "resolved display_name");

    // Look up or JIT-create the subject.
    let subject_id =
        resolve_subject(&state, &idp, &idp_sub, idp_email.as_deref(), &display_name).await?;

    // §1 #4 + #5 — repair a name that was never really set; never touch one that was.
    //
    // This runs on EVERY path out of resolve_subject, and it has to. The existing-link fast path returns
    // before any INSERT, so a `CASE` in the JIT upsert would never fire for the people who are actually
    // damaged — every colleague already signed in through Google. A failure to heal must not fail the login:
    // the person gets in with the name they have, and heals on their next sign-in.
    if let Err(e) =
        crate::display_name::heal(&state.pg, idp.tenant_id, subject_id, &display_name).await
    {
        tracing::warn!(target: "cyberos_auth::oidc", error = %e, "display_name heal failed; login proceeds");
    }

    // Mint a CyberOS JWT for the verified subject.
    let svc = crate::jwt::JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let assigned_roles =
        crate::handlers::load_subject_roles_pub(&state, idp.tenant_id, subject_id, &[]).await;
    let rbac_v = state.role_matrix.read().await.version();
    let tokens = svc
        .issue(
            cyberos_types::TenantId(idp.tenant_id),
            cyberos_types::SubjectId(subject_id),
            idp_email.as_deref().unwrap_or(""), // TASK-AUTH-004 §1 #2 — OIDC userinfo carries the email
            "human",
            vec![],
            assigned_roles,
            Some(rbac_v),
            None,
            None,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("jwt issuance failed: {e}")})),
            )
        })?;

    // Audit the success.
    let _ = sqlx::query(
        "INSERT INTO oidc_login_history
                (tenant_id, idp_config_id, subject_id, flow_state, state_token, ts_ns)
         VALUES ($1, $2, $3, 'callback_ok', $4, $5)",
    )
    .bind(idp.tenant_id)
    .bind(pending.idp_config_id)
    .bind(subject_id)
    .bind(&q.state)
    .bind(chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0))
    .execute(&state.pg)
    .await;

    // TASK-AUTH-106 slice-3 — apply policy-aware impossible-travel detection
    // to the OIDC flow. Block → 403; Challenge → token + needs_mfa_challenge
    // flag; Clear → normal token.
    let deps = crate::travel::AssessDeps {
        pool: &state.pg,
        geoip: &state.geoip,
        policy_cache: &state.travel_policy,
        sticky_suppress: &state.sticky_suppress,
    };
    let outcome = crate::travel::assess_login(
        &deps,
        idp.tenant_id,
        subject_id,
        "oidc",
        caller_ip,
        user_agent.as_deref(),
    )
    .await
    .ok();
    // Impossible-travel Block applies to both the API flow and the broker.
    if let Some(crate::travel::TravelOutcome::Block { kind, .. }) = &outcome {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "impossible_travel_blocked", "kind": kind})),
        ));
    }

    // TASK-AUTH-110 broker resume: a no-cookie /v1/auth/op/authorize sent the user
    // here via Google. Mint an AUTH SSO session, set the __Host-cyberos_sso
    // cookie, and 302 back to the original /authorize, which now succeeds via
    // silent SSO. The non-broker path (op_resume = None) is unchanged below.
    if let Some(resume) = pending.op_resume.as_deref() {
        let sso_id =
            match crate::op::sso_session::create(&state.pg, idp.tenant_id, subject_id).await {
                Ok(id) => id,
                Err(_) => {
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": "sso_session_create_failed"})),
                    ))
                }
            };
        let cookie = format!(
            "__Host-cyberos_sso={sso_id}; Path=/; HttpOnly; Secure; SameSite=Lax; Max-Age=86400"
        );
        let mut resp = Redirect::to(resume).into_response();
        if let Ok(hv) = cookie.parse() {
            resp.headers_mut()
                .insert(axum::http::header::SET_COOKIE, hv);
        }
        return Ok(resp);
    }

    // P0 console hand-back. The browser reached this callback via a top-level
    // redirect from Google, so it cannot read a JSON body. When initiate carried
    // a return_to, 302 back to it with the token in the URL *fragment* - never the
    // query string, so the token is not logged server-side or leaked in a Referer
    // header - and let the single-page app capture it. The return_to is
    // allow-listed to stop an open redirect from handing the token to an
    // attacker-chosen origin.
    if let Some(return_to) = pending.return_to.as_deref() {
        if !return_to_allowed(return_to) {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "return_to_not_allowed",
                    "detail": "return_to is not on the allow-list (set AUTH_OIDC_RETURN_ALLOW)"
                })),
            ));
        }
        let challenge = matches!(
            outcome,
            Some(crate::travel::TravelOutcome::Challenge { .. })
        );
        let sep = if return_to.contains('#') { '&' } else { '#' };
        let frag = format!(
            "{sep}access_token={at}&refresh_token={rt}&token_type={tt}&expires_in={ei}{ch}",
            at = urlencode(&tokens.access_token),
            rt = urlencode(&tokens.refresh_token),
            tt = urlencode(&tokens.token_type),
            ei = tokens.expires_in,
            ch = if challenge {
                "&needs_mfa_challenge=1"
            } else {
                ""
            },
        );
        return Ok(Redirect::to(&format!("{return_to}{frag}")).into_response());
    }

    match outcome {
        Some(crate::travel::TravelOutcome::Challenge { kind, login_id, .. }) => Ok(Json(json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "subject_id": subject_id,
            "needs_mfa_challenge": true,
            "challenge_reason": kind,
            "challenge_login_id": login_id,
        }))
        .into_response()),
        _ => Ok(Json(json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "subject_id": subject_id,
        }))
        .into_response()),
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

struct LoadedIdp {
    tenant_id: Uuid,
    discovery_url: String,
    client_id: String,
    client_secret: String,
    auto_provision: bool,
    default_roles: Vec<String>,
    allowed_domains: Vec<String>,
}

async fn load_idp(state: &AppState, idp_id: Uuid) -> Result<LoadedIdp, (StatusCode, Json<Value>)> {
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: Option<(Uuid, String, String, String, bool, Vec<String>, Vec<String>)> =
        sqlx::query_as(
            "SELECT tenant_id, discovery_url, client_id, client_secret, auto_provision,
                    default_roles, allowed_domains
                 FROM oidc_idp_configs WHERE id = $1",
        )
        .bind(idp_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    let r = row.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "idp_config_disappeared"})),
        )
    })?;
    Ok(LoadedIdp {
        tenant_id: r.0,
        discovery_url: r.1,
        client_id: r.2,
        client_secret: r.3,
        auto_provision: r.4,
        default_roles: r.5,
        allowed_domains: r.6,
    })
}

#[derive(Debug, Deserialize)]
struct DiscoveryDoc {
    authorization_endpoint: String,
    token_endpoint: String,
    #[serde(default)]
    jwks_uri: String,
    #[serde(default)]
    issuer: String,
}

async fn discover(url: &str) -> Result<DiscoveryDoc, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("cyberos-auth/0.1")
        .build()?;
    Ok(client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[serde(default)]
    refresh_token: Option<String>,
    id_token: String,
    token_type: String,
    #[serde(default)]
    expires_in: Option<u64>,
}

async fn exchange_code(
    token_endpoint: &str,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
    code_verifier: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()?;
    let form = [
        ("grant_type", "authorization_code"),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code_verifier", code_verifier),
    ];
    let resp = client.post(token_endpoint).form(&form).send().await?;
    if !resp.status().is_success() {
        return Err(format!("token endpoint returned {}", resp.status()).into());
    }
    Ok(resp.json().await?)
}

/// The IdP JWKS document (RFC 7517) and one RSA signing key.
#[derive(Debug, Deserialize)]
struct Jwks {
    #[serde(default)]
    keys: Vec<Jwk>,
}

#[derive(Debug, Deserialize)]
struct Jwk {
    #[serde(default)]
    kid: String,
    #[serde(default)]
    kty: String,
    /// RSA modulus, base64url (RFC 7518 section 6.3.1.1).
    #[serde(default)]
    n: String,
    /// RSA public exponent, base64url.
    #[serde(default)]
    e: String,
}

/// Fetch the IdP JWKS from its `jwks_uri`. Fetched per login so a key rotation
/// is handled without a stale cache (a TTL cache is a later optimisation).
async fn fetch_jwks(jwks_uri: &str) -> Result<Jwks, Box<dyn std::error::Error + Send + Sync>> {
    if jwks_uri.is_empty() {
        return Err("discovery document has no jwks_uri".into());
    }
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent("cyberos-auth/0.1")
        .build()?;
    Ok(client
        .get(jwks_uri)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?)
}

/// Verify the IdP ID token: match its `kid` to a JWKS RSA key, check the RS256
/// signature, and validate `iss` (when the discovery doc names one), `aud` (our
/// client_id), and `exp`. Returns the verified claims. A forged, mis-audienced,
/// or expired token errors rather than being trusted.
fn verify_id_token(
    id_token: &str,
    jwks: &Jwks,
    issuer: &str,
    audience: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};

    let header = decode_header(id_token)?;
    let kid = header.kid.ok_or("id_token header has no kid")?;
    let jwk = jwks
        .keys
        .iter()
        .find(|k| k.kid == kid)
        .ok_or("no JWKS key matches the id_token kid")?;
    if jwk.kty != "RSA" {
        return Err(format!("unsupported JWKS key type: {}", jwk.kty).into());
    }
    let key = DecodingKey::from_rsa_components(&jwk.n, &jwk.e)?;

    let mut validation = Validation::new(Algorithm::RS256);
    validation.set_audience(&[audience]);
    if !issuer.is_empty() {
        validation.set_issuer(&[issuer]);
    }
    validation.validate_exp = true;

    let data = decode::<Value>(id_token, &key, &validation)?;
    Ok(data.claims)
}

async fn resolve_subject(
    state: &AppState,
    idp: &LoadedIdp,
    idp_sub: &str,
    idp_email: Option<&str>,
    // TASK-AUTH-111 — already resolved from the ID token by the caller (`display_name::resolve`). Passed in
    // rather than derived here, so there is one chain and one place it can be got wrong.
    display_name: &str,
) -> Result<Uuid, (StatusCode, Json<Value>)> {
    // 1. Existing link?
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(idp.tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "SELECT subject_id FROM oidc_subject_link
          WHERE idp_config_id = (SELECT id FROM oidc_idp_configs
                                  WHERE tenant_id = $1 AND discovery_url = $2 LIMIT 1)
            AND idp_sub = $3
          LIMIT 1",
    )
    .bind(idp.tenant_id)
    .bind(&idp.discovery_url)
    .bind(idp_sub)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    if let Some((sid,)) = existing {
        let _ = sqlx::query(
            "UPDATE oidc_subject_link SET last_login_at = NOW()
              WHERE subject_id = $1 AND idp_sub = $2",
        )
        .bind(sid)
        .bind(idp_sub)
        .execute(&mut *tx)
        .await;
        tx.commit().await.map_err(internal)?;
        return Ok(sid);
    }
    tx.commit().await.map_err(internal)?;

    // 1.5 Link to an existing subject by verified email (account unification).
    // If a subject in this tenant already has this email - for example a teammate
    // who first signed up with a password - attach the Google identity to THAT
    // subject rather than creating a parallel account. After this, the password
    // and Google both resolve to the one CyberOS account, which is what an
    // operator expects when they sign a colleague in with Google for the first
    // time. (Option<&str> is Copy, so idp_email is still usable below.)
    if let Some(email) = idp_email {
        let mut tx = state.pg.begin().await.map_err(internal)?;
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(idp.tenant_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
        let by_email: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM subjects
              WHERE tenant_id = $1 AND lower(email) = lower($2) AND status = 'active'
              LIMIT 1",
        )
        .bind(idp.tenant_id)
        .bind(email)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?;
        if let Some((sid,)) = by_email {
            sqlx::query(
                "INSERT INTO oidc_subject_link (tenant_id, subject_id, idp_config_id, idp_sub, idp_email)
                      VALUES ($1, $2, (SELECT id FROM oidc_idp_configs WHERE tenant_id=$1 AND discovery_url=$3 LIMIT 1), $4, $5)
                 ON CONFLICT DO NOTHING",
            )
            .bind(idp.tenant_id)
            .bind(sid)
            .bind(&idp.discovery_url)
            .bind(idp_sub)
            .bind(email)
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
            tx.commit().await.map_err(internal)?;
            return Ok(sid);
        }
        tx.commit().await.map_err(internal)?;
    }

    // 2. Auto-provision?
    if !idp.auto_provision {
        return Err((
            StatusCode::FORBIDDEN,
            Json(
                json!({"error": "subject_not_provisioned", "detail": "auto-provision disabled for this IdP"}),
            ),
        ));
    }
    let handle = match idp_email {
        Some(e) => format!(
            "@{}",
            e.split('@')
                .next()
                .unwrap_or(&idp_sub[..idp_sub.len().min(20)])
        ),
        None => format!("@oidc-{}", &idp_sub[..idp_sub.len().min(12)]),
    };
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(idp.tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    // SSO subjects have no local password. The subjects_human_has_password
    // constraint requires a non-null hash for a human, so store a bcrypt hash of
    // an unguessable random value: a password-grant login then fails closed (no
    // one knows it), and the IdP stays the only way in.
    let sso_password_hash = bcrypt::hash(random_b64(32), bcrypt::DEFAULT_COST)
        .map_err(|e| internal(format!("bcrypt hash failed: {e}")))?;
    // TASK-AUTH-111 — the display name comes from the ID token's name claims, resolved by the caller.
    //
    // This bind used to be `idp_email.unwrap_or("")`. THAT was the bug: every human provisioned through
    // Google SSO wore their email address as their display name, in every channel, above every message,
    // forever — and the privacy policy we publish says we take the name from their Google account. We do
    // receive it; we were throwing it away.
    //
    // The `display_name` column is left alone on the ON CONFLICT path on purpose. Repairing an existing row
    // is `display_name::heal`'s job, applied by the caller to whichever subject resolved: putting the rule
    // here would miss the existing-link fast path, which is the path every already-provisioned person takes.
    // One rule, one place.
    let jit_display_name = if display_name.trim().is_empty() {
        handle.as_str() // final rung: no name claims and no email. The handle beats rendering as nothing.
    } else {
        display_name
    };
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO subjects (tenant_id, handle, display_name, email, kind, status, password_hash, roles)
              VALUES ($1, $2, $3, $4, 'human', 'active', $5, $6)
         ON CONFLICT (tenant_id, handle) DO UPDATE
            SET email = COALESCE(EXCLUDED.email, subjects.email),
                updated_at = NOW()
       RETURNING id",
    )
    .bind(idp.tenant_id)
    .bind(&handle)
    .bind(jit_display_name)
    .bind(idp_email)
    .bind(&sso_password_hash)
    .bind(&idp.default_roles)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    let subject_id = row.0;

    // Link the IdP identity for next time.
    sqlx::query(
        "INSERT INTO oidc_subject_link (tenant_id, subject_id, idp_config_id, idp_sub, idp_email)
              VALUES ($1, $2, (SELECT id FROM oidc_idp_configs WHERE tenant_id=$1 AND discovery_url=$3 LIMIT 1), $4, $5)
         ON CONFLICT DO NOTHING",
    )
    .bind(idp.tenant_id)
    .bind(subject_id)
    .bind(&idp.discovery_url)
    .bind(idp_sub)
    .bind(idp_email)
    .execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    // Grant the IdP's default roles in the RBAC table so the minted token carries
    // them (TASK-AUTH-101). Provision-time only - we do NOT re-grant on later logins,
    // so an admin can still remove a role without it reappearing.
    grant_default_roles(state, idp.tenant_id, subject_id, &idp.default_roles).await;
    Ok(subject_id)
}

/// Best-effort: grant a freshly provisioned subject the IdP's default roles in the
/// RBAC `subject_roles` table (TASK-AUTH-101), so the minted token carries them. Runs
/// in its own transaction after the subject is committed - a misconfigured role name
/// (which violates the `roles(name)` FK) rolls back only this grant and never aborts
/// the login. `granted_by` is the nil UUID, marking a system/IdP grant.
async fn grant_default_roles(
    state: &AppState,
    tenant_id: Uuid,
    subject_id: Uuid,
    roles: &[String],
) {
    if roles.is_empty() {
        return;
    }
    let Ok(mut tx) = state.pg.begin().await else {
        return;
    };
    if sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .is_err()
    {
        let _ = tx.rollback().await;
        return;
    }
    for role in roles {
        if sqlx::query(
            "INSERT INTO subject_roles (tenant_id, subject_id, role, granted_by)
                  VALUES ($1, $2, $3, $4)
             ON CONFLICT (subject_id, role) DO NOTHING",
        )
        .bind(tenant_id)
        .bind(subject_id)
        .bind(role)
        .bind(Uuid::nil())
        .execute(&mut *tx)
        .await
        .is_err()
        {
            let _ = tx.rollback().await;
            return;
        }
    }
    let _ = tx.commit().await;
}

fn random_b64(n: usize) -> String {
    let mut buf = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(&buf)
}

fn pkce_challenge(verifier: &str) -> String {
    let mut h = Sha256::new();
    h.update(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(h.finalize())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            for b in c.to_string().as_bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Allow-list check for the P0 console hand-back `return_to`. In production set
/// `AUTH_OIDC_RETURN_ALLOW` to a comma-separated list of allowed URL prefixes
/// (for example "https://os.cyberskill.world/"). When unset, only localhost /
/// 127.0.0.1 URLs are allowed, so local dev works without opening a redirect in
/// production. An unmatched return_to is rejected rather than followed.
/// P0 Workspace-domain gate decision. Given the verified email (if any), whether the IdP marked it
/// verified, and the IdP's allowed domains, return Ok to admit or Err(reason) to reject. An empty
/// allow-list admits everyone (no domain restriction). Kept pure so the security decision is unit-tested
/// without a live Google round-trip.
fn email_domain_admitted(
    email: Option<&str>,
    email_verified: bool,
    allowed: &[String],
) -> Result<(), &'static str> {
    if allowed.is_empty() {
        return Ok(());
    }
    let email = email.ok_or("email_required")?;
    if !email_verified {
        return Err("email_not_verified");
    }
    let domain = email.rsplit('@').next().unwrap_or("").to_ascii_lowercase();
    if allowed.iter().any(|d| d.eq_ignore_ascii_case(&domain)) {
        Ok(())
    } else {
        Err("email_domain_not_allowed")
    }
}

fn return_to_allowed(url: &str) -> bool {
    return_to_allowed_with(url, std::env::var("AUTH_OIDC_RETURN_ALLOW").ok().as_deref())
}

/// Pure core of [`return_to_allowed`], separated so it can be unit-tested without
/// touching process env. `allow` is the raw AUTH_OIDC_RETURN_ALLOW value (None or
/// empty -> dev localhost-only).
fn return_to_allowed_with(url: &str, allow: Option<&str>) -> bool {
    match allow {
        Some(list) if !list.trim().is_empty() => list
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .any(|prefix| url.starts_with(prefix)),
        _ => {
            url.starts_with("http://localhost")
                || url.starts_with("http://127.0.0.1")
                || url.starts_with("https://localhost")
                || url.starts_with("https://127.0.0.1")
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingState {
    pub tenant_id: Uuid,
    pub idp_config_id: Uuid,
    pub redirect_uri: String,
    pub code_verifier: String,
    pub issued_at_secs: u64,
    /// TASK-AUTH-110 broker: when set, the callback mints an AUTH SSO session and
    /// 302s back to this `/v1/auth/op/authorize` URL instead of returning JSON.
    pub op_resume: Option<String>,
    /// P0 console hand-back: when set (and `op_resume` is None), the callback
    /// 302s here with the freshly minted token in the URL fragment.
    pub return_to: Option<String>,
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_challenge_is_deterministic() {
        let v = "abcdef-test-verifier-123";
        let c1 = pkce_challenge(v);
        let c2 = pkce_challenge(v);
        assert_eq!(c1, c2);
        assert_eq!(c1.len(), 43); // base64-url-no-pad of 32-byte SHA-256 = 43 chars
    }

    #[test]
    fn random_b64_returns_correct_length() {
        let s = random_b64(48);
        // 48 bytes → 64 base64-url-no-pad chars
        assert_eq!(s.len(), 64);
    }

    #[test]
    fn jwks_parses_rsa_keys() {
        let body = r#"{"keys":[{"kid":"abc","kty":"RSA","alg":"RS256","use":"sig","n":"AQAB","e":"AQAB"}]}"#;
        let jwks: Jwks = serde_json::from_str(body).unwrap();
        assert_eq!(jwks.keys.len(), 1);
        assert_eq!(jwks.keys[0].kid, "abc");
        assert_eq!(jwks.keys[0].kty, "RSA");
    }

    #[test]
    fn verify_id_token_rejects_unknown_kid() {
        // A well-formed JWT header naming a kid the JWKS does not contain.
        let header = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","kid":"missing"}"#);
        let payload = URL_SAFE_NO_PAD.encode(br#"{"sub":"x","aud":"client","iss":"https://idp"}"#);
        let token = format!("{header}.{payload}.sig");
        let jwks = Jwks { keys: vec![] };
        let err = verify_id_token(&token, &jwks, "https://idp", "client")
            .unwrap_err()
            .to_string();
        assert!(
            err.contains("kid"),
            "expected a kid mismatch error, got: {err}"
        );
    }

    #[test]
    fn verify_id_token_rejects_malformed_token() {
        let jwks = Jwks { keys: vec![] };
        assert!(verify_id_token("not-a-jwt", &jwks, "iss", "client").is_err());
    }

    #[test]
    fn return_to_dev_default_allows_localhost_only() {
        // No allow-list configured -> only localhost / 127.0.0.1 hand-backs.
        assert!(return_to_allowed_with(
            "http://localhost:8090/app.html",
            None
        ));
        assert!(return_to_allowed_with(
            "http://127.0.0.1:8090/app.html",
            Some("")
        ));
        assert!(!return_to_allowed_with("https://evil.example/steal", None));
    }

    #[test]
    fn return_to_uses_configured_prefixes() {
        let allow = Some("https://os.cyberskill.world/, https://admin.cyberskill.world/");
        assert!(return_to_allowed_with(
            "https://os.cyberskill.world/app.html",
            allow
        ));
        // A configured allow-list does NOT implicitly permit localhost.
        assert!(!return_to_allowed_with(
            "http://localhost:8090/app.html",
            allow
        ));
        // An attacker-chosen origin that merely contains the allowed host is rejected.
        assert!(!return_to_allowed_with(
            "https://evil.example/?x=https://os.cyberskill.world/",
            allow
        ));
    }

    #[test]
    fn domain_gate_empty_allowlist_admits_everyone() {
        // No restriction configured -> any verified or unverified email is admitted.
        assert!(email_domain_admitted(Some("anyone@gmail.com"), true, &[]).is_ok());
        assert!(email_domain_admitted(None, false, &[]).is_ok());
    }

    #[test]
    fn domain_gate_restricts_to_workspace() {
        let allowed = vec!["cyberskill.world".to_string()];
        // A verified Workspace email is admitted (case-insensitive on the domain).
        assert!(email_domain_admitted(Some("stephen@cyberskill.world"), true, &allowed).is_ok());
        assert!(email_domain_admitted(Some("Stephen@CyberSkill.World"), true, &allowed).is_ok());
        // A personal Gmail is rejected.
        assert_eq!(
            email_domain_admitted(Some("someone@gmail.com"), true, &allowed),
            Err("email_domain_not_allowed")
        );
        // A Workspace email that the IdP did not verify is rejected.
        assert_eq!(
            email_domain_admitted(Some("stephen@cyberskill.world"), false, &allowed),
            Err("email_not_verified")
        );
        // A restricted IdP with no email claim is rejected.
        assert_eq!(
            email_domain_admitted(None, true, &allowed),
            Err("email_required")
        );
    }
}
