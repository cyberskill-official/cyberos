//! FR-MCP-004 access-token minting and verification (clauses #7, #23, #24; DEC-805, DEC-814).
//!
//! Access tokens are RS256 JWTs signed with the FR-AUTH-004 keys in the shared `auth_signing_keys`
//! table (the same keys the auth service publishes at `/.well-known/jwks.json`), so any MCP resource
//! server can verify them against that JWKS. The claim set is clause #7; the audience is bound at mint
//! time and checked exactly at verify time, which is the cross-server-replay defense (clause #23).
//!
//! The queries are runtime-checked `sqlx::query_as` (no compile-time database), so this module
//! compiles without a live database; the sign/verify round trip is exercised by the integration tests
//! against Postgres seeded with an active signing key.

use chrono::Utc;
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use super::audience;

/// Signing algorithm. RS256 only - the auth service is RS256-only despite the spec naming ES256 too.
const ALG: Algorithm = Algorithm::RS256;

/// Access-token lifetime in seconds (DEC-805: `exp = iat + 3600`).
pub const ACCESS_TTL_SECS: i64 = 3600;

/// Why minting or verifying an access token failed.
#[derive(Debug, Error)]
pub enum JwtError {
    /// No active, unexpired key in `auth_signing_keys` to sign with.
    #[error("no active signing key - bootstrap an auth signing key first")]
    NoActiveKey,
    /// The presented JWT carried no `kid` header, so the verifying key cannot be selected.
    #[error("jwt header has no kid")]
    MissingKid,
    /// The JWT's `kid` does not match any key in `auth_signing_keys`.
    #[error("unknown signing key id: {0}")]
    UnknownKid(String),
    /// The token is structurally valid and correctly signed but its `aud` is not this resource server.
    #[error("audience mismatch")]
    AudienceMismatch,
    /// A `jsonwebtoken` encode or decode error (bad PEM, bad signature, expired, wrong issuer).
    #[error("jwt codec error: {0}")]
    Codec(#[from] jsonwebtoken::errors::Error),
    /// A database error loading the signing key.
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// The claims carried by an MCP access token (FR-MCP-004 clause #7).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAccessClaims {
    /// Issuer - the MCP-gateway URL.
    pub iss: String,
    /// Audience - the canonical URL of the MCP resource server this token is bound to (RFC 8707).
    pub aud: Vec<String>,
    /// Subject - the authorizing subject's UUID.
    pub sub: String,
    /// Space-separated granted scopes.
    pub scope: String,
    /// 16-byte random nonce as hex (replay distinctiveness).
    pub nonce: String,
    /// Issued-at, seconds since the epoch.
    pub iat: i64,
    /// Expiry, seconds since the epoch (`iat + ACCESS_TTL_SECS`).
    pub exp: i64,
    /// JWT ID (UUIDv4) - the key the revocation list (clause #24) is keyed on.
    pub jti: String,
    /// The requesting client's UUID.
    pub client_id: String,
    /// The tenant binding.
    pub tenant_id: String,
}

/// A freshly minted access token plus the metadata the caller needs to persist and respond.
#[derive(Debug, Clone)]
pub struct MintedAccessToken {
    /// The signed compact JWT.
    pub access_token: String,
    /// The token's `jti` (store it if the token may later be revoked).
    pub jti: String,
    /// Lifetime in seconds, for the `expires_in` field of the token response.
    pub expires_in: i64,
}

/// Build the claim set for an access token. Pure: every time-and-randomness input is injected, so the
/// shape (notably `exp = iat + ACCESS_TTL_SECS` and the single-resource `aud`) is unit-testable.
#[allow(clippy::too_many_arguments)]
pub fn build_access_claims(
    iss: &str,
    resource: &str,
    sub: &str,
    scope: &str,
    client_id: &str,
    tenant_id: &str,
    now_unix: i64,
    jti: &str,
    nonce: &str,
) -> McpAccessClaims {
    McpAccessClaims {
        iss: iss.to_string(),
        aud: audience::bind_audience(resource),
        sub: sub.to_string(),
        scope: scope.to_string(),
        nonce: nonce.to_string(),
        iat: now_unix,
        exp: now_unix + ACCESS_TTL_SECS,
        jti: jti.to_string(),
        client_id: client_id.to_string(),
        tenant_id: tenant_id.to_string(),
    }
}

/// Mint a signed access token for `(sub, client, tenant)` bound to `resource`, signed with the active
/// auth signing key. Generates the `jti` and `nonce`.
#[allow(clippy::too_many_arguments)]
pub async fn mint_access_token(
    pool: &PgPool,
    iss: &str,
    resource: &str,
    sub: &str,
    scope: &str,
    client_id: &str,
    tenant_id: &str,
) -> Result<MintedAccessToken, JwtError> {
    let (kid, private_pem) = load_active_signing_key(pool).await?;
    let now = Utc::now().timestamp();
    let jti = Uuid::new_v4().to_string();
    let nonce = Uuid::new_v4().simple().to_string(); // 32 hex chars = 16 random bytes
    let claims = build_access_claims(
        iss, resource, sub, scope, client_id, tenant_id, now, &jti, &nonce,
    );
    let mut header = Header::new(ALG);
    header.kid = Some(kid);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(private_pem.as_bytes())?,
    )?;
    Ok(MintedAccessToken {
        access_token: token,
        jti,
        expires_in: ACCESS_TTL_SECS,
    })
}

/// Verify an access token: signature (against the `kid`'s public key), issuer, expiry, and - strictly -
/// that `aud` is exactly `expected_aud` (clause #23). Returns the claims on success.
///
/// Audience is checked here, not by `jsonwebtoken`, because the spec requires exact equality with this
/// resource server's URL, whereas `jsonwebtoken`'s built-in check is looser membership - so
/// `validate_aud` is disabled and [`audience::audience_matches`] is applied instead. The caller must
/// still consult the revocation list for the returned `jti` (clause #24).
pub async fn verify_access_token(
    pool: &PgPool,
    token: &str,
    expected_iss: &str,
    expected_aud: &str,
) -> Result<McpAccessClaims, JwtError> {
    let kid = decode_header(token)?.kid.ok_or(JwtError::MissingKid)?;
    let public_pem = load_signing_key_public_pem(pool, &kid).await?;
    let mut v = Validation::new(ALG);
    v.set_issuer(&[expected_iss]);
    v.validate_aud = false; // we enforce exact audience below, not jsonwebtoken's membership rule
    let data = decode::<McpAccessClaims>(
        token,
        &DecodingKey::from_rsa_pem(public_pem.as_bytes())?,
        &v,
    )?;
    if !audience::audience_matches(&data.claims.aud, expected_aud) {
        return Err(JwtError::AudienceMismatch);
    }
    Ok(data.claims)
}

/// Load the active, unexpired signing key (kid, private PEM) - the newest active key.
async fn load_active_signing_key(pool: &PgPool) -> Result<(String, String), JwtError> {
    let row = sqlx::query_as::<_, (String, String)>(
        "SELECT kid, private_pem
           FROM auth_signing_keys
          WHERE status = 'active' AND expires_at > NOW()
       ORDER BY activated_at DESC
          LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;
    row.ok_or(JwtError::NoActiveKey)
}

/// Load a signing key's public PEM by `kid` (for verification).
async fn load_signing_key_public_pem(pool: &PgPool, kid: &str) -> Result<String, JwtError> {
    let row =
        sqlx::query_as::<_, (String,)>("SELECT public_pem FROM auth_signing_keys WHERE kid = $1")
            .bind(kid)
            .fetch_optional(pool)
            .await?;
    row.map(|(pem,)| pem)
        .ok_or_else(|| JwtError::UnknownKid(kid.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claims_set_exp_one_hour_after_iat() {
        let c = build_access_claims(
            "https://mcp.cyberos.world",
            "https://mcp.cyberos.world",
            "sub-1",
            "mcp:tools",
            "client-1",
            "tenant-1",
            1_000_000,
            "jti-1",
            "nonce-1",
        );
        assert_eq!(c.exp - c.iat, ACCESS_TTL_SECS);
        assert_eq!(c.iat, 1_000_000);
    }

    #[test]
    fn claims_bind_a_single_resource_audience() {
        let c = build_access_claims(
            "https://mcp.cyberos.world",
            "https://server-a.cyberos.world",
            "sub-1",
            "mcp:tools",
            "client-1",
            "tenant-1",
            1_000_000,
            "jti-1",
            "nonce-1",
        );
        assert_eq!(c.aud, vec!["https://server-a.cyberos.world".to_string()]);
        // The bound audience must satisfy the exact-match check and reject a different server.
        assert!(audience::audience_matches(
            &c.aud,
            "https://server-a.cyberos.world"
        ));
        assert!(!audience::audience_matches(
            &c.aud,
            "https://server-b.cyberos.world"
        ));
    }

    #[test]
    fn claims_carry_scope_client_and_tenant_through() {
        let c = build_access_claims(
            "iss",
            "res",
            "sub-9",
            "read write",
            "client-9",
            "tenant-9",
            42,
            "j",
            "n",
        );
        assert_eq!(c.scope, "read write");
        assert_eq!(c.client_id, "client-9");
        assert_eq!(c.tenant_id, "tenant-9");
        assert_eq!(c.sub, "sub-9");
    }
}
