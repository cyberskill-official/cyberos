//! TASK-AUTH-110 §1 #3 + #9 + #26 + DEC-2489 - the AUTH SSO browser session.
//!
//! The server-side truth behind the `__Host-cyberos_sso` cookie. The cookie
//! carries the row id; this table decides validity, so the session is revocable.
//! [`create`] starts one (absolute 24h), [`lookup_active`] is the silent-SSO read
//! (not revoked, within the 8h sliding window and 24h absolute), [`touch`]
//! extends the sliding window, and [`revoke_for_subject`] is the §1 #26 cascade
//! the TASK-AUTH-005 revoke calls so silent SSO stops too, not only new logins.
//!
//! Runtime-checked `sqlx::query(...)`; compiles without a live database.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::errors::OpError;

/// Sliding inactivity window (DEC-2489).
pub const SLIDING_TTL_SECS: i64 = 8 * 60 * 60;
/// Absolute lifetime (DEC-2489).
pub const ABSOLUTE_TTL_SECS: i64 = 24 * 60 * 60;

#[derive(Debug, Clone)]
pub struct SsoSession {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub subject_id: Uuid,
}

/// Create a new SSO session for `subject_id`; returns its id (the cookie value).
pub async fn create(pool: &PgPool, tenant_id: Uuid, subject_id: Uuid) -> Result<Uuid, OpError> {
    let id = Uuid::new_v4();
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    sqlx::query(
        "INSERT INTO auth_sso_sessions (id, tenant_id, subject_id, absolute_expiry)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(tenant_id)
    .bind(subject_id)
    .bind(Utc::now() + Duration::seconds(ABSOLUTE_TTL_SECS))
    .execute(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(id)
}

/// Silent-SSO read: returns the session iff it is not revoked, still inside the
/// 24h absolute window, and was seen within the 8h sliding window.
pub async fn lookup_active(
    pool: &PgPool,
    tenant_id: Uuid,
    id: Uuid,
) -> Result<Option<SsoSession>, OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let row: Option<(Uuid, Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, tenant_id, subject_id
           FROM auth_sso_sessions
          WHERE id = $1
            AND revoked_at IS NULL
            AND absolute_expiry > NOW()
            AND last_seen_at > NOW() - INTERVAL '8 hours'",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(row.map(|(id, tenant_id, subject_id)| SsoSession {
        id,
        tenant_id,
        subject_id,
    }))
}

/// Extend the sliding window (called on each silent-SSO use).
pub async fn touch(pool: &PgPool, tenant_id: Uuid, id: Uuid) -> Result<(), OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    sqlx::query(
        "UPDATE auth_sso_sessions SET last_seen_at = NOW() WHERE id = $1 AND revoked_at IS NULL",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(())
}

/// Revoke every live session for `subject_id` (the §1 #26 cascade). Returns the
/// number of sessions revoked.
pub async fn revoke_for_subject(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
) -> Result<u64, OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let done = sqlx::query(
        "UPDATE auth_sso_sessions SET revoked_at = NOW()
          WHERE subject_id = $1 AND revoked_at IS NULL",
    )
    .bind(subject_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(done.rows_affected())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ttl_constants_are_the_expected_windows() {
        // 8h sliding, 24h absolute (DEC-2489); sliding < absolute by construction.
        assert_eq!(SLIDING_TTL_SECS, 28_800);
        assert_eq!(ABSOLUTE_TTL_SECS, 86_400);
    }
}
