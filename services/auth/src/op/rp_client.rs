//! FR-AUTH-110 §1 #1 + #15 - the first-party OIDC relying-party (RP) registry.
//!
//! Admin-registered confidential clients (CHAT/Mattermost, PORTAL). The
//! client_secret is generated here, returned to the admin exactly once, and
//! stored only as a SHA-256 hash (verify-only; migration 0027's ADR). Most
//! reads/writes run under the caller's tenant GUC; [`get_by_client_id`] resolves
//! a globally-unique client_id under the root-tenant GUC bypass, because the
//! token + authorize endpoints know only the client_id, not its tenant.
//!
//! Runtime-checked `sqlx::query(...)`; compiles without a live database.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use super::errors::OpError;

/// The root tenant nil-UUID that bypasses RLS (matches the 0021/0027 policy).
const ROOT_TENANT: &str = "00000000-0000-0000-0000-000000000000";

#[derive(Debug, Clone)]
pub struct RpClient {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub client_id: String,
    pub client_secret_hash: String,
    pub redirect_uris: Vec<String>,
    pub post_logout_redirect_uris: Vec<String>,
    pub allow_refresh: bool,
    pub is_active: bool,
}

/// Values for a new RP registration.
pub struct NewRpClient<'a> {
    pub tenant_id: Uuid,
    pub name: &'a str,
    pub client_id: &'a str,
    pub redirect_uris: Vec<String>,
    pub post_logout_redirect_uris: Vec<String>,
    pub allow_refresh: bool,
    pub created_by_subject_id: Uuid,
}

/// Generate a 256-bit high-entropy secret, base64url-nopad encoded (43 chars).
pub fn generate_secret() -> String {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

/// SHA-256 hex of a secret - what the DB stores (never the secret itself).
pub fn secret_hash(secret: &str) -> String {
    let mut h = Sha256::new();
    h.update(secret.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Constant-time check of a presented secret against a stored hash.
pub fn verify_secret(presented: &str, stored_hash: &str) -> bool {
    let computed = secret_hash(presented);
    let (a, b) = (computed.as_bytes(), stored_hash.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Register an RP. Returns `(id, one-time client_secret)`; only the hash is
/// persisted (DEC-2497).
pub async fn create(pool: &PgPool, c: &NewRpClient<'_>) -> Result<(Uuid, String), OpError> {
    let id = Uuid::new_v4();
    let secret = generate_secret();
    let hash = secret_hash(&secret);
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(c.tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let insert = sqlx::query(
        "INSERT INTO auth_oidc_rp_clients
            (id, tenant_id, name, client_id, client_secret_hash, redirect_uris,
             post_logout_redirect_uris, allow_refresh, created_by_subject_id)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
    )
    .bind(id)
    .bind(c.tenant_id)
    .bind(c.name)
    .bind(c.client_id)
    .bind(&hash)
    .bind(&c.redirect_uris)
    .bind(&c.post_logout_redirect_uris)
    .bind(c.allow_refresh)
    .bind(c.created_by_subject_id)
    .execute(&mut *tx)
    .await;
    match insert {
        Ok(_) => {}
        Err(sqlx::Error::Database(db)) if db.is_unique_violation() => {
            let _ = tx.rollback().await;
            return Err(OpError::Conflict);
        }
        Err(_) => {
            let _ = tx.rollback().await;
            return Err(OpError::ServerError);
        }
    }
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok((id, secret))
}

type RpRow = (
    Uuid,
    Uuid,
    String,
    String,
    String,
    Vec<String>,
    Vec<String>,
    bool,
    bool,
);

fn row_to_client(r: RpRow) -> RpClient {
    let (
        id,
        tenant_id,
        name,
        client_id,
        client_secret_hash,
        redirect_uris,
        post_logout_redirect_uris,
        allow_refresh,
        is_active,
    ) = r;
    RpClient {
        id,
        tenant_id,
        name,
        client_id,
        client_secret_hash,
        redirect_uris,
        post_logout_redirect_uris,
        allow_refresh,
        is_active,
    }
}

const RP_COLUMNS: &str = "id, tenant_id, name, client_id, client_secret_hash, redirect_uris, post_logout_redirect_uris, allow_refresh, is_active";

/// Resolve a globally-unique active client_id to its RP, reading under the
/// root-tenant GUC bypass (the caller knows only the client_id).
pub async fn get_by_client_id(pool: &PgPool, client_id: &str) -> Result<Option<RpClient>, OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(ROOT_TENANT)
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let row: Option<RpRow> = sqlx::query_as(&format!(
        "SELECT {RP_COLUMNS} FROM auth_oidc_rp_clients WHERE client_id = $1 AND is_active = true"
    ))
    .bind(client_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(row.map(row_to_client))
}

/// List a tenant's RPs (admin view).
pub async fn list_for_tenant(pool: &PgPool, tenant_id: Uuid) -> Result<Vec<RpClient>, OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let rows: Vec<RpRow> = sqlx::query_as(&format!(
        "SELECT {RP_COLUMNS} FROM auth_oidc_rp_clients WHERE tenant_id = $1 ORDER BY created_at DESC"
    ))
    .bind(tenant_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(rows.into_iter().map(row_to_client).collect())
}

/// Soft-delete (is_active = false) an RP by client_id within a tenant. Returns
/// whether a row was affected.
pub async fn soft_delete(pool: &PgPool, tenant_id: Uuid, client_id: &str) -> Result<bool, OpError> {
    let mut tx = pool.begin().await.map_err(|_| OpError::ServerError)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(|_| OpError::ServerError)?;
    let done = sqlx::query(
        "UPDATE auth_oidc_rp_clients SET is_active = false WHERE tenant_id = $1 AND client_id = $2",
    )
    .bind(tenant_id)
    .bind(client_id)
    .execute(&mut *tx)
    .await
    .map_err(|_| OpError::ServerError)?;
    tx.commit().await.map_err(|_| OpError::ServerError)?;
    Ok(done.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_secret_is_unique_and_256_bit() {
        let a = generate_secret();
        let b = generate_secret();
        assert_ne!(a, b);
        // 32 bytes base64url-nopad = 43 chars.
        assert_eq!(a.len(), 43);
    }

    #[test]
    fn secret_hash_is_sha256_hex() {
        let h = secret_hash("abc");
        assert_eq!(h.len(), 64);
        assert!(h.starts_with("ba7816bf"));
    }

    #[test]
    fn verify_secret_round_trip() {
        let s = generate_secret();
        let h = secret_hash(&s);
        assert!(verify_secret(&s, &h));
        assert!(!verify_secret("wrong-secret", &h));
        assert!(!verify_secret(&s, "too-short"));
    }
}
