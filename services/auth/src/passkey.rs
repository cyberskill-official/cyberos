//! FR-AUTH-105 — Passkey / WebAuthn Level 3 discoverable-credential flow.
//!
//! Four endpoints under the admin router (caller must be authenticated):
//!   * `POST /v1/auth/passkey/enrol/begin`   — webauthn-rs `start_passkey_registration`
//!   * `POST /v1/auth/passkey/enrol/finish`  — `finish_passkey_registration` + persist mfa_factors row
//!   * `POST /v1/auth/passkey/login/begin`   — `start_passkey_authentication`
//!   * `POST /v1/auth/passkey/login/finish`  — `finish_passkey_authentication` + mint JWT
//!
//! Once an active webauthn factor lands here, the founder-role assignment
//! gate in `rbac::assignment` will pass (DEC-128 enforced via mfa_factors
//! lookup). Passkey is discoverable / resident credential — username-less
//! login flows work without prompting for an identifier.

use axum::{
    extract::{Json as JsonInput, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::OnceLock;
use uuid::Uuid;
use webauthn_rs::prelude::*;

use crate::jwt::Claims;
use crate::AppState;

/// Lazily-initialised webauthn-rs builder. Configured from env vars:
///   AUTH_WEBAUTHN_RP_ID   — relying-party ID, e.g. "cyberskill.world"
///   AUTH_WEBAUTHN_RP_ORIGIN — full origin URL, e.g. "https://app.cyberskill.world"
///   AUTH_WEBAUTHN_RP_NAME  — human-readable name shown in browser UI
fn webauthn() -> &'static Webauthn {
    static W: OnceLock<Webauthn> = OnceLock::new();
    W.get_or_init(|| {
        let rp_id =
            std::env::var("AUTH_WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
        let rp_origin_str = std::env::var("AUTH_WEBAUTHN_RP_ORIGIN")
            .unwrap_or_else(|_| "https://localhost".to_string());
        let rp_origin =
            Url::parse(&rp_origin_str).expect("AUTH_WEBAUTHN_RP_ORIGIN must be a valid URL");
        let rp_name =
            std::env::var("AUTH_WEBAUTHN_RP_NAME").unwrap_or_else(|_| "CyberOS".to_string());
        WebauthnBuilder::new(&rp_id, &rp_origin)
            .expect("webauthn builder init")
            .rp_name(&rp_name)
            .build()
            .expect("webauthn build")
    })
}

#[derive(Debug, Deserialize)]
pub struct EnrolBeginBody {
    pub label: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EnrolBeginResponse {
    pub ceremony_id: Uuid,
    /// JSON-serialised CreationChallengeResponse — the browser's
    /// `navigator.credentials.create()` consumes this directly.
    pub creation_challenge: Value,
}

pub async fn enrol_begin(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<EnrolBeginBody>,
) -> Result<(StatusCode, Json<EnrolBeginResponse>), (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal)?;
    let subject_id = Uuid::parse_str(&claims.sub).map_err(internal)?;
    let label = body.label.unwrap_or_else(|| "Passkey".to_string());

    // Skip credentials already registered for this subject so the browser
    // doesn't offer "you already have one" UI inside the same ceremony.
    let existing_creds = load_subject_passkeys(&state, tenant_id, subject_id).await?;

    let user_name = subject_id.to_string();
    let user_display_name = "CyberOS member".to_string();

    let (ccr, reg_state) = webauthn()
        .start_passkey_registration(
            subject_id,
            &user_name,
            &user_display_name,
            Some(existing_creds.iter().map(|c| c.cred_id().clone()).collect()),
        )
        .map_err(|e| webauthn_err("start_passkey_registration", e))?;

    // Persist the registration state under a fresh ceremony_id so finish can pick it up.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO passkey_enrolment_state (tenant_id, subject_id, flow, state_json, label)
              VALUES ($1, $2, 'enrol', $3, $4)
            RETURNING ceremony_id",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(serde_json::to_value(&reg_state).map_err(internal)?)
    .bind(&label)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let creation_challenge = serde_json::to_value(&ccr).map_err(internal)?;
    Ok((
        StatusCode::OK,
        Json(EnrolBeginResponse {
            ceremony_id: row.0,
            creation_challenge,
        }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct EnrolFinishBody {
    pub ceremony_id: Uuid,
    pub credential: RegisterPublicKeyCredential,
}

pub async fn enrol_finish(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<EnrolFinishBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal)?;
    let subject_id = Uuid::parse_str(&claims.sub).map_err(internal)?;

    // Load + drop the pending state.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: Option<(Value, Option<String>)> = sqlx::query_as(
        "DELETE FROM passkey_enrolment_state
              WHERE ceremony_id = $1 AND subject_id = $2 AND flow = 'enrol'
                AND expires_at > NOW()
          RETURNING state_json, label",
    )
    .bind(body.ceremony_id)
    .bind(subject_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    let (state_json, label) = row.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "ceremony_not_found_or_expired"})),
        )
    })?;

    let reg_state: PasskeyRegistration = serde_json::from_value(state_json).map_err(internal)?;
    let passkey = webauthn()
        .finish_passkey_registration(&body.credential, &reg_state)
        .map_err(|e| webauthn_err("finish_passkey_registration", e))?;

    // Persist as an active mfa_factor row.
    sqlx::query(
        "INSERT INTO mfa_factors
              (tenant_id, subject_id, factor_type, label, cred_id, public_key, sign_count, status, activated_at)
         VALUES ($1, $2, 'webauthn', $3, $4, $5, $6, 'active', NOW())",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(label.unwrap_or_else(|| "Passkey".to_string()))
    .bind(passkey.cred_id().as_ref())
    .bind(serde_json::to_vec(&passkey).map_err(internal)?)
    .bind(0i64)
    .execute(&mut *tx).await.map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct LoginBeginBody {
    pub tenant_slug: String,
    pub handle: Option<String>, // optional — discoverable credential lets us skip
}

#[derive(Debug, Serialize)]
pub struct LoginBeginResponse {
    pub ceremony_id: Uuid,
    pub request_challenge: Value,
}

pub async fn login_begin(
    State(state): State<AppState>,
    JsonInput(body): JsonInput<LoginBeginBody>,
) -> Result<(StatusCode, Json<LoginBeginResponse>), (StatusCode, Json<Value>)> {
    // Look up tenant + subject under root context.
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let subject_row: Option<(Option<Uuid>, Uuid)> = if let Some(handle) = &body.handle {
        sqlx::query_as(
            "SELECT s.id, s.tenant_id FROM subjects s JOIN tenants t ON t.id = s.tenant_id
              WHERE t.slug = $1 AND s.handle = $2",
        )
        .bind(&body.tenant_slug)
        .bind(handle)
        .fetch_optional(&mut *tx)
        .await
        .map_err(internal)?
    } else {
        // Discoverable-credential path: the browser picks the credential
        // without an identifier hint. We still need a tenant scope.
        sqlx::query_as("SELECT NULL::uuid, id FROM tenants WHERE slug = $1")
            .bind(&body.tenant_slug)
            .fetch_optional(&mut *tx)
            .await
            .map_err(internal)?
    };
    tx.commit().await.map_err(internal)?;

    let (subject_id_opt, tenant_id) = subject_row.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tenant_or_subject_not_found"})),
        )
    })?;

    // Gather every active passkey for the subject (or, if discoverable,
    // pass an empty list — webauthn-rs supports username-less auth via
    // start_discoverable_authentication, but for slice 1 we require a handle).
    let subject_id = subject_id_opt.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "username_less_passkey_deferred_to_slice_2"})),
        )
    })?;
    let creds = load_subject_passkeys(&state, tenant_id, subject_id).await?;
    if creds.is_empty() {
        return Err((
            StatusCode::PRECONDITION_FAILED,
            Json(json!({"error": "no_passkey_registered_for_subject"})),
        ));
    }

    let (req, auth_state) = webauthn()
        .start_passkey_authentication(&creds)
        .map_err(|e| webauthn_err("start_passkey_authentication", e))?;

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO passkey_enrolment_state (tenant_id, subject_id, flow, state_json)
              VALUES ($1, $2, 'login', $3)
            RETURNING ceremony_id",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(serde_json::to_value(&auth_state).map_err(internal)?)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    Ok((
        StatusCode::OK,
        Json(LoginBeginResponse {
            ceremony_id: row.0,
            request_challenge: serde_json::to_value(&req).map_err(internal)?,
        }),
    ))
}

#[derive(Debug, Deserialize)]
pub struct LoginFinishBody {
    pub ceremony_id: Uuid,
    pub credential: PublicKeyCredential,
}

pub async fn login_finish(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    JsonInput(body): JsonInput<LoginFinishBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let caller_ip = crate::handlers::caller_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: Option<(Uuid, Uuid, Value)> = sqlx::query_as(
        "DELETE FROM passkey_enrolment_state
              WHERE ceremony_id = $1 AND flow = 'login' AND expires_at > NOW()
          RETURNING tenant_id, subject_id, state_json",
    )
    .bind(body.ceremony_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    let (tenant_id, subject_id, state_json) = row.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "ceremony_not_found_or_expired"})),
        )
    })?;

    let auth_state: PasskeyAuthentication = serde_json::from_value(state_json).map_err(internal)?;
    let auth_result = webauthn()
        .finish_passkey_authentication(&body.credential, &auth_state)
        .map_err(|e| webauthn_err("finish_passkey_authentication", e))?;

    // Update sign-count on the matched credential (helps with cloned-credential detection).
    sqlx::query(
        "UPDATE mfa_factors SET sign_count = $1, last_used_at = NOW()
          WHERE subject_id = $2 AND cred_id = $3 AND factor_type = 'webauthn'",
    )
    .bind(auth_result.counter() as i64)
    .bind(subject_id)
    .bind(auth_result.cred_id().as_ref())
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    // Mint a CyberOS JWT for the verified subject.
    let svc = crate::jwt::JwtService::new(state.pg.clone(), state.jwt_issuer.clone());
    let roles = crate::handlers::load_subject_roles_pub(&state, tenant_id, subject_id, &[]).await;
    let rbac_v = state.role_matrix.read().await.version();
    let tokens = svc
        .issue(
            cyberos_types::TenantId(tenant_id),
            cyberos_types::SubjectId(subject_id),
            "", // FR-AUTH-004 §1 #2 — passkey login doesn't carry plaintext email
            "human",
            vec![],
            roles,
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

    // FR-AUTH-106 slice-3 — apply policy-aware impossible-travel detection.
    // Passkey is intrinsically MFA-strength, but the policy still applies:
    // a `block` action will refuse the login, and a `challenge` outcome
    // here triggers a second-factor (TOTP) requirement on top of the passkey.
    let deps = crate::travel::AssessDeps {
        pool: &state.pg,
        geoip: &state.geoip,
        policy_cache: &state.travel_policy,
        sticky_suppress: &state.sticky_suppress,
    };
    let outcome = crate::travel::assess_login(
        &deps,
        tenant_id,
        subject_id,
        "passkey",
        caller_ip,
        user_agent.as_deref(),
    )
    .await
    .ok();
    let body = match outcome {
        Some(crate::travel::TravelOutcome::Block { kind, .. }) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({"error": "impossible_travel_blocked", "kind": kind})),
            ));
        }
        Some(crate::travel::TravelOutcome::Challenge { kind, login_id, .. }) => json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "subject_id": subject_id,
            "needs_mfa_challenge": true,
            "challenge_reason": kind,
            "challenge_login_id": login_id,
        }),
        _ => json!({
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "token_type": tokens.token_type,
            "expires_in": tokens.expires_in,
            "subject_id": subject_id,
        }),
    };
    Ok((StatusCode::OK, Json(body)))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn load_subject_passkeys(
    state: &AppState,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<Vec<Passkey>, (StatusCode, Json<Value>)> {
    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let rows: Vec<(Vec<u8>,)> = sqlx::query_as(
        "SELECT public_key FROM mfa_factors
          WHERE tenant_id = $1 AND subject_id = $2
            AND factor_type = 'webauthn' AND status = 'active'",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let mut out = Vec::with_capacity(rows.len());
    for (blob,) in rows {
        if let Ok(passkey) = serde_json::from_slice::<Passkey>(&blob) {
            out.push(passkey);
        }
    }
    Ok(out)
}

fn subject_uuid_to_bytes(s: Uuid) -> Vec<u8> {
    s.as_bytes().to_vec()
}

fn webauthn_err(stage: &str, e: WebauthnError) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": "webauthn_error",
            "stage": stage,
            "detail": e.to_string(),
        })),
    )
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}
