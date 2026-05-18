//! FR-AUTH-109 — stub→full migration state + grace-window enforcer.
//!
//! Two surfaces:
//!   * `MigrationState::load_from_db` — read the singleton row at boot +
//!     on the 60s refresh cycle. Used by the verifier to decide whether
//!     to reject a token that lacks `rbac_v` (DEC-125 grace window).
//!   * `POST /v1/admin/auth/migration/extend-grace { days, reason }` —
//!     extend the grace window (operator action; requires root-admin).
//!   * `GET  /v1/admin/auth/migration/preview` — show how many active
//!     subjects still have legacy tokens (i.e. would fail when grace closes).
//!
//! Wired into `verify_jwt` once both this module + a refresh task are in
//! place. Slice 1: state read + preview + extend; verifier check lands when
//! the refresh-cycle for migration_state is added alongside RBAC refresher.

use axum::{
    extract::{Json as JsonInput, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

use crate::jwt::Claims;
use crate::rbac::Role;
use crate::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct MigrationState {
    pub fr_auth_101_shipped_at: DateTime<Utc>,
    pub grace_window_days: i32,
    pub grace_closes_at: DateTime<Utc>,
    pub extended_by: Option<Uuid>,
    pub extension_reason: Option<String>,
}

impl MigrationState {
    pub async fn load_from_db(pool: &PgPool) -> Result<Option<Self>, sqlx::Error> {
        let row: Option<(DateTime<Utc>, i32, DateTime<Utc>, Option<Uuid>, Option<String>)> =
            sqlx::query_as(
                "SELECT fr_auth_101_shipped_at, grace_window_days, grace_closes_at,
                        extended_by, extension_reason
                   FROM auth_migration_state WHERE id = 1",
            )
            .fetch_optional(pool)
            .await?;
        Ok(row.map(|(shipped, days, closes, ext_by, reason)| Self {
            fr_auth_101_shipped_at: shipped,
            grace_window_days: days,
            grace_closes_at: closes,
            extended_by: ext_by,
            extension_reason: reason,
        }))
    }

    pub fn grace_is_open(&self) -> bool {
        Utc::now() < self.grace_closes_at
    }
}

// ---------------------------------------------------------------------------
// `GET /v1/admin/auth/migration/preview` — caller-impact estimator.
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct PreviewResponse {
    pub shipped_at: DateTime<Utc>,
    pub grace_closes_at: DateTime<Utc>,
    pub grace_open: bool,
    pub seconds_remaining: i64,
    /// Number of active subjects without ANY subject_roles row — these
    /// would fail strict rbac_v checks the moment grace closes.
    pub subjects_without_rbac_rows: i64,
    /// Number of active mfa_factors rows for `webauthn` factor type — proxy
    /// for the WebAuthn enrolment progress.
    pub active_webauthn_factors: i64,
}

pub async fn preview(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<PreviewResponse>, (StatusCode, Json<Value>)> {
    require_root_admin(&claims)?;

    let ms = MigrationState::load_from_db(&state.pg).await.map_err(internal)?;
    let ms = ms.ok_or_else(|| (
        StatusCode::PRECONDITION_FAILED,
        Json(json!({"error": "auth_migration_state row missing — migration 0015 not applied"})),
    ))?;

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&mut *tx).await.map_err(internal)?;

    let (subjects_without_rbac,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM subjects s
          WHERE s.status = 'active'
            AND NOT EXISTS (
                SELECT 1 FROM subject_roles sr WHERE sr.subject_id = s.id
            )",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;

    let (active_webauthn,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM mfa_factors
          WHERE factor_type = 'webauthn' AND status = 'active'",
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    Ok(Json(PreviewResponse {
        shipped_at: ms.fr_auth_101_shipped_at,
        grace_closes_at: ms.grace_closes_at,
        grace_open: ms.grace_is_open(),
        seconds_remaining: (ms.grace_closes_at - Utc::now()).num_seconds().max(0),
        subjects_without_rbac_rows: subjects_without_rbac,
        active_webauthn_factors: active_webauthn,
    }))
}

// ---------------------------------------------------------------------------
// `POST /v1/admin/auth/migration/extend-grace`
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ExtendBody {
    /// Days to ADD to the existing grace_closes_at (not absolute).
    pub days: i32,
    pub reason: String,
}

pub async fn extend_grace(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<ExtendBody>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    require_root_admin(&claims)?;
    if body.days < 1 || body.days > 90 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "extension must be 1-90 days"})),
        ));
    }
    if body.reason.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "reason required — recorded in auth_migration_state.extension_reason"})),
        ));
    }

    let extender = Uuid::parse_str(&claims.sub).map_err(internal)?;

    let row: (DateTime<Utc>,) = sqlx::query_as(
        "UPDATE auth_migration_state
            SET grace_closes_at = grace_closes_at + ($1 || ' days')::INTERVAL,
                grace_window_days = grace_window_days + $1,
                extended_by = $2,
                extension_reason = $3,
                last_updated_at = NOW()
          WHERE id = 1
      RETURNING grace_closes_at",
    )
    .bind(body.days)
    .bind(extender)
    .bind(&body.reason)
    .fetch_one(&state.pg)
    .await
    .map_err(internal)?;

    Ok((StatusCode::OK, Json(json!({
        "grace_closes_at": row.0,
        "extended_by": extender,
        "reason": body.reason,
        "extended_days": body.days,
    }))))
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn require_root_admin(claims: &Claims) -> Result<(), (StatusCode, Json<Value>)> {
    let is_root = claims
        .roles
        .iter()
        .any(|s| Role::from_str(s).map(|r| r == Role::RootAdmin).unwrap_or(false));
    if !is_root {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "root-admin required"})),
        ));
    }
    Ok(())
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}
