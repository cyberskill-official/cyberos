//! FR-AUTH-102 — MFA repository (database CRUD).
//!
//! All DB access for the MFA subsystem goes through this module.
//! Every query sets `app.current_tenant_id` for RLS enforcement.

use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

/// A loaded TOTP factor from the database.
#[derive(Debug)]
pub struct LoadedTotpFactor {
    pub factor_id: Uuid,
    pub totp_secret_b32: String,
    pub display_name: String,
}

/// Load a TOTP factor by id, subject, and required status.
pub async fn load_totp_factor(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    factor_id: Uuid,
    required_status: &str,
) -> Result<LoadedTotpFactor, (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: Option<(Uuid, Option<String>, String)> = sqlx::query_as(
        "SELECT id, totp_secret, label FROM mfa_factors
          WHERE id = $1 AND subject_id = $2 AND factor_type = 'totp' AND status = $3",
    )
    .bind(factor_id)
    .bind(subject_id)
    .bind(required_status)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    let (fid, secret_opt, label) = row.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("no {required_status} totp factor for caller")})),
        )
    })?;
    let totp_secret_b32 = secret_opt.ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "totp factor missing secret"})),
        )
    })?;

    Ok(LoadedTotpFactor {
        factor_id: fid,
        totp_secret_b32,
        display_name: label,
    })
}

/// Insert a new pending TOTP factor row.
pub async fn insert_pending_totp(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    label: &str,
    secret_b32: &str,
) -> Result<Uuid, (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO mfa_factors (tenant_id, subject_id, factor_type, label, totp_secret, status)
              VALUES ($1, $2, 'totp', $3, $4, 'pending')
            RETURNING id",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(label)
    .bind(secret_b32)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(row.0)
}

/// Activate a pending TOTP factor.
pub async fn activate_factor(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    factor_id: Uuid,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query(
        "UPDATE mfa_factors
            SET status = 'active', activated_at = NOW(), last_used_at = NOW()
          WHERE id = $1 AND subject_id = $2 AND status = 'pending'",
    )
    .bind(factor_id)
    .bind(subject_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(())
}

/// Touch `last_used_at` on a factor.
pub async fn touch_factor(
    pool: &PgPool,
    tenant_id: Uuid,
    factor_id: Uuid,
) -> Result<(), (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    sqlx::query("UPDATE mfa_factors SET last_used_at = NOW() WHERE id = $1")
        .bind(factor_id)
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(())
}

/// Check if a subject has any active MFA factors.
pub async fn has_active_factor(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM mfa_factors
          WHERE subject_id = $1 AND status = 'active'",
    )
    .bind(subject_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(matches!(row, Some((n,)) if n > 0))
}

/// List all active factors for a subject.
#[derive(Debug, serde::Serialize)]
pub struct FactorSummary {
    pub id: Uuid,
    pub factor_type: String,
    pub label: String,
    pub status: String,
    pub enrolled_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn list_factors(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<Vec<FactorSummary>, (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    let rows: Vec<(
        Uuid,
        String,
        String,
        String,
        chrono::DateTime<chrono::Utc>,
        Option<chrono::DateTime<chrono::Utc>>,
    )> = sqlx::query_as(
        "SELECT id, factor_type, label, status, enrolled_at, last_used_at
              FROM mfa_factors
             WHERE subject_id = $1
             ORDER BY enrolled_at",
    )
    .bind(subject_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;

    Ok(rows
        .into_iter()
        .map(|(id, ft, label, status, enrolled, used)| FactorSummary {
            id,
            factor_type: ft,
            label,
            status,
            enrolled_at: enrolled,
            last_used_at: used,
        })
        .collect())
}

/// Revoke (soft-delete) a factor.
pub async fn revoke_factor(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    factor_id: Uuid,
) -> Result<bool, (StatusCode, Json<Value>)> {
    let mut tx = pool.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;
    let result = sqlx::query(
        "UPDATE mfa_factors SET status = 'revoked', revoked_at = NOW()
          WHERE id = $1 AND subject_id = $2 AND status IN ('active', 'pending')",
    )
    .bind(factor_id)
    .bind(subject_id)
    .execute(&mut *tx)
    .await
    .map_err(internal)?;
    tx.commit().await.map_err(internal)?;
    Ok(result.rows_affected() > 0)
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}
