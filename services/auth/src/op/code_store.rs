//! FR-AUTH-110 §1 #2 + #16 - the authorization-code store.
//!
//! Codes are single-use, 60-second-TTL, PKCE-bound. The table key is the
//! SHA-256 hash of the code (never the code itself). Single-use is enforced by
//! the sibling `auth_oidc_code_consumptions` first-insert-wins guard (ADR
//! OPEN-001 #1): [`consume`] INSERTs the hash; the first wins, a second hits the
//! unique violation and returns `InvalidGrant` (= replay). All writes run under
//! the tenant GUC, mirroring `oidc.rs` and `travel_policy.rs`.
//!
//! These functions use runtime-checked `sqlx::query(...)`, so the module compiles
//! without a live database; the integration tests need Postgres.

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::errors::OpError;

/// Auth-code lifetime (DEC-2490).
pub const CODE_TTL_SECS: i64 = 60;

/// SHA-256 hex of a code (the stored key; the code itself never lands in the DB).
pub fn code_hash(code: &str) -> String {
    let mut h = Sha256::new();
    h.update(code.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// The values bound into a fresh code at `/authorize`.
pub struct NewAuthCode<'a> {
    pub code: &'a str,
    pub tenant_id: Uuid,
    pub rp_client_id: &'a str,
    pub subject_id: Uuid,
    pub redirect_uri: &'a str,
    pub code_challenge: &'a str,
    pub nonce: Option<&'a str>,
    pub scope: &'a str,
    pub sso_session_id: Uuid,
}

/// The state recovered at `/token` for a still-valid code.
#[derive(Debug, Clone)]
pub struct StoredAuthCode {
    pub tenant_id: Uuid,
    pub rp_client_id: String,
    pub subject_id: Uuid,
    pub redirect_uri: String,
    pub code_challenge: String,
    pub nonce: Option<String>,
    pub scope: String,
    pub sso_session_id: Uuid,
}

/// Insert a fresh code (expires `now + 60s`) under the tenant GUC.
pub async fn insert_code(pool: &PgPool, c: &NewAuthCode<'_>) -> Result<(), OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(c.tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    sqlx::query(
        "INSERT INTO auth_oidc_auth_codes
            (code_hash, tenant_id, rp_client_id, subject_id, redirect_uri,
             code_challenge, nonce, scope, sso_session_id, expires_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(code_hash(c.code))
    .bind(c.tenant_id)
    .bind(c.rp_client_id)
    .bind(c.subject_id)
    .bind(c.redirect_uri)
    .bind(c.code_challenge)
    .bind(c.nonce)
    .bind(c.scope)
    .bind(c.sso_session_id)
    .bind(Utc::now() + Duration::seconds(CODE_TTL_SECS))
    .execute(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(())
}

/// Look up a non-expired code by its raw value, returning the bound state, or
/// `InvalidGrant` when missing / expired. Does NOT consume - the caller verifies
/// PKCE + redirect_uri first, then calls [`consume`].
pub async fn lookup_code(pool: &PgPool, tenant_id: Uuid, code: &str) -> Result<StoredAuthCode, OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let row: Option<(Uuid, String, Uuid, String, String, Option<String>, String, Uuid)> =
        sqlx::query_as(
            "SELECT tenant_id, rp_client_id, subject_id, redirect_uri,
                    code_challenge, nonce, scope, sso_session_id
               FROM auth_oidc_auth_codes
              WHERE code_hash = $1 AND expires_at > NOW()",
        )
        .bind(code_hash(code))
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    match row {
        Some((tenant_id, rp_client_id, subject_id, redirect_uri, code_challenge, nonce, scope, sso_session_id)) => {
            Ok(StoredAuthCode {
                tenant_id,
                rp_client_id,
                subject_id,
                redirect_uri,
                code_challenge,
                nonce,
                scope,
                sso_session_id,
            })
        }
        None => Err(OpError::InvalidGrant),
    }
}

/// Single-use guard: the first INSERT into `auth_oidc_code_consumptions` wins; a
/// unique violation means the code was already exchanged (= replay) and returns
/// `InvalidGrant`.
pub async fn consume(pool: &PgPool, tenant_id: Uuid, code: &str) -> Result<(), OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let res = sqlx::query("INSERT INTO auth_oidc_code_consumptions (code_hash, tenant_id) VALUES ($1, $2)")
        .bind(code_hash(code))
        .bind(tenant_id)
        .execute(&mut *tx)
        .await;
    match res {
        Ok(_) => {
            tx.commit().await.map_err(|_| OpError::ServerError)?;
            Ok(())
        }
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            let _ = tx.rollback().await;
            Err(OpError::InvalidGrant)
        }
        Err(_) => {
            let _ = tx.rollback().await;
            Err(OpError::ServerError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_hash_is_sha256_hex() {
        let h = code_hash("abc");
        assert_eq!(h.len(), 64);
        // SHA-256("abc") canonical vector.
        assert!(h.starts_with("ba7816bf"));
    }

    #[test]
    fn code_hash_is_deterministic_and_distinct() {
        assert_eq!(code_hash("same"), code_hash("same"));
        assert_ne!(code_hash("a"), code_hash("b"));
    }
}
