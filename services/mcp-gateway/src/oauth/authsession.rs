//! Verify an TASK-AUTH-004 session JWT presented as a bearer to `/authorize` (subject identity) and to
//! confidential `/register` (tenant + role). Reuses the shared `auth_signing_keys`; returns None on any
//! failure so callers can map that to `login_required` or `invalid_client`.

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

/// The authenticated caller behind an auth bearer JWT.
#[derive(Debug, Clone)]
pub struct AuthSession {
    /// The subject's UUID.
    pub subject_id: Uuid,
    /// The subject's tenant UUID.
    pub tenant_id: Uuid,
    /// The subject's role names (kebab-case), e.g. `tenant-admin`.
    pub roles: Vec<String>,
}

impl AuthSession {
    /// Whether the caller holds the given role.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }
}

/// Minimal auth claim set; `exp` is still validated by jsonwebtoken from the payload.
#[derive(Debug, Deserialize)]
struct AuthClaims {
    sub: String,
    tenant_id: String,
    #[serde(default)]
    roles: Vec<String>,
}

/// Verify an auth JWT (RS256, issuer = `auth_issuer`) against the signing key named by its `kid`, and
/// extract subject, tenant, and roles. None on any failure.
pub async fn verify_auth_session(
    pool: &PgPool,
    token: &str,
    auth_issuer: &str,
) -> Option<AuthSession> {
    let kid = decode_header(token).ok()?.kid?;
    let public_pem: String =
        sqlx::query_scalar("SELECT public_pem FROM auth_signing_keys WHERE kid = $1")
            .bind(&kid)
            .fetch_optional(pool)
            .await
            .ok()??;
    let mut v = Validation::new(Algorithm::RS256);
    v.set_issuer(&[auth_issuer]);
    v.validate_aud = false;
    let data = decode::<AuthClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_pem.as_bytes()).ok()?,
        &v,
    )
    .ok()?;
    Some(AuthSession {
        subject_id: Uuid::parse_str(&data.claims.sub).ok()?,
        tenant_id: Uuid::parse_str(&data.claims.tenant_id).ok()?,
        roles: data.claims.roles,
    })
}
