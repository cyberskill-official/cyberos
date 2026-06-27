//! FR-MCP-004 token revocation (RFC 7009), POST /revoke.
//!
//! Revoking a refresh token compromises its whole family. This slice handles refresh tokens (the
//! common revoke target); access-token (`jti`) revocation via the revocation list lands with the
//! tools/call verification slice. Per RFC 7009 §2.2 the endpoint always returns 200, whether or not
//! the token was active, to avoid being a probing oracle.

use serde::Deserialize;
use sqlx::PgPool;

use super::response::EndpointError;
use super::secret::sha256_hex;
use super::store;

/// The RFC 7009 revocation request body.
#[derive(Debug, Deserialize)]
pub struct RevokeRequest {
    /// The token to revoke (access or refresh).
    pub token: String,
    /// Optional hint (`access_token` | `refresh_token`); advisory only.
    #[serde(default)]
    pub token_type_hint: Option<String>,
}

/// Revoke a token. A matching refresh token compromises its family; anything else is a no-op. Always
/// `Ok` - the handler returns 200 regardless (RFC 7009 §2.2).
pub async fn revoke(pool: &PgPool, req: RevokeRequest) -> Result<(), EndpointError> {
    let hash = sha256_hex(&req.token);
    if let Some(row) = store::get_refresh_by_hash(pool, &hash).await? {
        store::compromise_family(pool, row.family_id).await?;
    }
    Ok(())
}
