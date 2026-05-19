//! FR-AUTH-005 §1 #10 + G-010 — active-jti session tracking.
//!
//! Each successful JWT issue (FR-AUTH-004 password-grant or refresh-grant)
//! MUST insert a row here so the revoke path (FR-AUTH-005 §1 #3 + G-003) can
//! enumerate the subject's active jtis and push them into the in-memory
//! deny-list (G-011). Without this table, revocation is best-effort
//! (`subjects.status = 'revoked'` blocks NEW logins but lets EXISTING JWTs
//! keep working until natural expiry).
//!
//! The table lives in the shared auth Postgres database; RLS pins each
//! row to its `tenant_id` so tenant-admin cannot enumerate other tenants'
//! active jtis. The `cyberos_ops` BYPASSRLS role is used only by the
//! reaper job (future FR), never by request-path code.

use chrono::{DateTime, Utc};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Insert a new active-session row. Call inside the same Postgres tx as
/// the JWT issue's audit-row emit so partial state ("audit row but no
/// session" or vice-versa) is impossible.
///
/// `source_ip_hash16` MUST be a 16-hex-char string — compute via
/// `memory_bridge::source_ip_hash16(ip)` (date-salted to prevent
/// cross-day correlation). The DB CHECK constraint enforces the length.
pub async fn insert<'c>(
    tx: &mut Transaction<'c, Postgres>,
    jti: &str,
    subject_id: Uuid,
    tenant_id: Uuid,
    expires_at: DateTime<Utc>,
    source_ip_hash16: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO sessions (jti, subject_id, tenant_id, expires_at, source_ip_hash16)
              VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (jti) DO NOTHING",
    )
    .bind(jti)
    .bind(subject_id)
    .bind(tenant_id)
    .bind(expires_at)
    .bind(source_ip_hash16)
    .execute(&mut **tx)
    .await
    .map(|_| ())
}

/// Active session row — minimal projection used by the revoke path.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ActiveSession {
    pub jti: String,
    pub expires_at: DateTime<Utc>,
}

/// Enumerate the subject's currently-active jtis (expires_at > NOW()).
/// The revoke handler walks the returned list and pushes each jti into
/// the in-memory deny-list (per FR-AUTH-005 §1 #3).
pub async fn list_active_for_subject<'c>(
    tx: &mut Transaction<'c, Postgres>,
    subject_id: Uuid,
) -> Result<Vec<ActiveSession>, sqlx::Error> {
    sqlx::query_as::<_, ActiveSession>(
        "SELECT jti, expires_at
           FROM sessions
          WHERE subject_id = $1
            AND expires_at > NOW()",
    )
    .bind(subject_id)
    .fetch_all(&mut **tx)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    // ActiveSession is a sqlx::FromRow — exercise the type compiles via
    // a no-op construction. Real round-trip is in admin_revoke_test.rs
    // (Postgres-gated, `#[ignore]`).
    #[test]
    fn active_session_struct_constructs() {
        let s = ActiveSession {
            jti: "j".into(),
            expires_at: Utc::now(),
        };
        assert_eq!(s.jti, "j");
    }
}
