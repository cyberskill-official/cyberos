//! FR-AUTH-102 — MFA HTTP handlers.
//!
//! Endpoints:
//!   * `POST /v1/auth/mfa/factors/totp/enrol`          — start TOTP enrolment
//!   * `POST /v1/auth/mfa/factors/totp/enrol/finish`   — confirm enrolment
//!   * `POST /v1/auth/mfa/verify`                      — runtime TOTP verify
//!   * `GET  /v1/auth/mfa/factors`                     — list enrolled factors
//!   * `DELETE /v1/auth/mfa/factors/:factor_id`         — revoke a factor
//!   * `POST /v1/auth/mfa/recovery/generate`           — generate recovery codes
//!   * `POST /v1/auth/mfa/recovery/verify`             — verify a recovery code
//!
//! All handlers extract `Claims` from the auth middleware extension.

use axum::{
    extract::{Json as JsonInput, Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::Claims;
use crate::AppState;
use super::totp;
use super::repo;
use super::recovery;
// lockout enforcement is consulted in the password-grant path (src/handlers.rs),
// not directly in these MFA handlers.

// ---------------------------------------------------------------------------
// TOTP enrolment — start
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct EnrolStartResponse {
    pub factor_id: Uuid,
    pub secret_base32: String,
    pub otpauth_url: String,
    pub issuer: String,
    pub label: String,
}

#[derive(Debug, Deserialize)]
pub struct EnrolStartBody {
    pub label: Option<String>,
}

pub async fn totp_enrol_start(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<EnrolStartBody>,
) -> Result<(StatusCode, Json<EnrolStartResponse>), (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;

    let secret = totp::generate_totp_secret();
    let secret_b32 = totp::base32_encode(&secret);
    let label = body
        .label
        .as_deref()
        .unwrap_or("CyberOS authenticator")
        .to_string();
    let issuer = "CyberOS".to_string();

    let otpauth = format!(
        "otpauth://totp/{issuer}:{account}?secret={secret}&issuer={issuer}&algorithm=SHA1&digits=6&period=30",
        issuer = issuer,
        account = totp::urlencoding_minimal(&claims.sub),
        secret = secret_b32,
    );

    let factor_id = repo::insert_pending_totp(&state.pg, tenant_id, subject_id, &label, &secret_b32).await?;

    Ok((
        StatusCode::CREATED,
        Json(EnrolStartResponse {
            factor_id,
            secret_base32: secret_b32,
            otpauth_url: otpauth,
            issuer,
            label,
        }),
    ))
}

// ---------------------------------------------------------------------------
// TOTP enrolment — finish
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VerifyBody {
    pub factor_id: Uuid,
    pub code: String,
}

pub async fn totp_enrol_finish(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<VerifyBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;

    let factor = repo::load_totp_factor(&state.pg, tenant_id, subject_id, body.factor_id, "pending").await?;
    let secret = totp::base32_decode(&factor.totp_secret_b32).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "stored secret is not valid base32"})),
        )
    })?;

    if !totp::verify_totp(&secret, &body.code, totp::current_time_step()) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "totp code did not verify"})),
        ));
    }

    repo::activate_factor(&state.pg, tenant_id, subject_id, body.factor_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// TOTP runtime verify
// ---------------------------------------------------------------------------

pub async fn totp_verify(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<VerifyBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;

    let factor = repo::load_totp_factor(&state.pg, tenant_id, subject_id, body.factor_id, "active").await?;
    let secret = totp::base32_decode(&factor.totp_secret_b32).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "stored secret is not valid base32"})),
        )
    })?;

    if !totp::verify_totp(&secret, &body.code, totp::current_time_step()) {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "totp code did not verify"})),
        ));
    }

    repo::touch_factor(&state.pg, tenant_id, body.factor_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// List factors
// ---------------------------------------------------------------------------

pub async fn list_factors(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<repo::FactorSummary>>, (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;
    let factors = repo::list_factors(&state.pg, tenant_id, subject_id).await?;
    Ok(Json(factors))
}

// ---------------------------------------------------------------------------
// Revoke factor
// ---------------------------------------------------------------------------

pub async fn revoke_factor(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(factor_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;
    let revoked = repo::revoke_factor(&state.pg, tenant_id, subject_id, factor_id).await?;
    if revoked {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "factor not found or already revoked"})),
        ))
    }
}

// ---------------------------------------------------------------------------
// Recovery codes — generate
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct RecoveryCodesResponse {
    pub codes: Vec<String>,
    pub message: String,
}

pub async fn generate_recovery_codes(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<(StatusCode, Json<RecoveryCodesResponse>), (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;

    // Must have at least one active MFA factor before generating recovery codes.
    let has_factor = repo::has_active_factor(&state.pg, tenant_id, subject_id).await?;
    if !has_factor {
        return Err((
            StatusCode::PRECONDITION_FAILED,
            Json(json!({"error": "no active MFA factor enrolled; enrol a TOTP or WebAuthn factor first"})),
        ));
    }

    let (batch_id, codes) = recovery::generate_batch();
    let plaintexts: Vec<String> = codes.iter().map(|(p, _)| p.clone()).collect();

    // Invalidate all prior recovery codes by inserting with a new batch_id.
    // Old batches remain in the DB but are filtered out by batch_id on verify.
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;

    // Mark all existing batches as consumed (soft-invalidate).
    sqlx::query(
        "UPDATE mfa_recovery_codes SET consumed = true, consumed_at = NOW()
          WHERE subject_id = $1 AND consumed = false",
    )
    .bind(subject_id)
    .execute(&mut *tx)
    .await
    .map_err(internal_err)?;

    // Insert the new batch.
    for (_plain, hash) in &codes {
        sqlx::query(
            "INSERT INTO mfa_recovery_codes (tenant_id, subject_id, code_bcrypt_hash, batch_id)
              VALUES ($1, $2, $3, $4)",
        )
        .bind(tenant_id)
        .bind(subject_id)
        .bind(hash)
        .bind(batch_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;
    }
    tx.commit().await.map_err(internal_err)?;

    Ok((
        StatusCode::CREATED,
        Json(RecoveryCodesResponse {
            codes: plaintexts,
            message: "Save these codes securely. Each code can only be used once.".to_string(),
        }),
    ))
}

// ---------------------------------------------------------------------------
// Recovery codes — verify
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RecoveryVerifyBody {
    pub code: String,
}

pub async fn verify_recovery_code(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<RecoveryVerifyBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = parse_uuid(&claims.tenant_id)?;
    let subject_id = parse_uuid(&claims.sub)?;

    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal_err)?;

    // Load all unconsumed recovery codes for this subject.
    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, code_bcrypt_hash FROM mfa_recovery_codes
          WHERE subject_id = $1 AND consumed = false
          ORDER BY created_at",
    )
    .bind(subject_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(internal_err)?;

    // Try each unconsumed code — bcrypt compare is O(n) but batch is max 10.
    for (code_id, hash) in &rows {
        if recovery::verify_code(&body.code, hash) {
            // Consume the code.
            sqlx::query(
                "UPDATE mfa_recovery_codes SET consumed = true, consumed_at = NOW()
                  WHERE id = $1",
            )
            .bind(code_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_err)?;
            tx.commit().await.map_err(internal_err)?;
            return Ok(StatusCode::NO_CONTENT);
        }
    }
    tx.commit().await.map_err(internal_err)?;

    Err((
        StatusCode::UNAUTHORIZED,
        Json(json!({"error": "recovery code did not match any unconsumed code"})),
    ))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_uuid(s: &str) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(s).map_err(internal_err)
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}
