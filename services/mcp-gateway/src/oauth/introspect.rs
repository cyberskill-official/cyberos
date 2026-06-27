//! FR-MCP-004 token introspection (RFC 7662), POST /introspect.
//!
//! Verifies an access token against the FR-AUTH-004 JWKS, checks the revocation list, and returns the
//! RFC 7662 §2.2 shape. An invalid, expired, audience-wrong, or revoked token returns
//! `{"active": false}` - never an error (§2.2).
//!
//! NOTE (clause #22): this endpoint should be restricted to confidential clients holding the
//! `mcp_introspect` scope and exposed on the closed network only; caller authentication is a
//! follow-up. Deploy it behind network policy until then.

use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use super::response::EndpointError;
use super::{jwt, store};

/// The RFC 7662 introspection request body.
#[derive(Debug, Deserialize)]
pub struct IntrospectRequest {
    /// The access token to introspect.
    pub token: String,
}

/// Introspect an access token. `issuer` is the gateway URL; `resource` is this server's canonical URL
/// (the audience the token must carry).
pub async fn introspect(
    pool: &PgPool,
    issuer: &str,
    resource: &str,
    req: IntrospectRequest,
) -> Result<Value, EndpointError> {
    let claims = match jwt::verify_access_token(pool, &req.token, issuer, resource).await {
        Ok(c) => c,
        Err(_) => return Ok(json!({ "active": false })),
    };
    if let Ok(jti) = uuid::Uuid::parse_str(&claims.jti) {
        if store::is_jti_revoked(pool, jti).await? {
            return Ok(json!({ "active": false }));
        }
    }
    Ok(json!({
        "active": true,
        "scope": claims.scope,
        "client_id": claims.client_id,
        "sub": claims.sub,
        "aud": claims.aud,
        "iss": claims.iss,
        "exp": claims.exp,
        "iat": claims.iat,
        "jti": claims.jti,
        "tenant_id": claims.tenant_id,
    }))
}
