//! FR-MCP-004 dynamic client registration (RFC 7591), POST /register.
//!
//! Public clients (CLIs, desktop apps - the primary MCP case, DEC-803) carry no secret and prove
//! possession via PKCE; registration is open. Confidential clients require an authenticated
//! tenant-admin (clauses #18, #19): the secret is generated, Argon2-hashed, stored, and returned once.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::authsession::AuthSession;
use super::enums::ClientType;
use super::error::OAuthError;
use super::response::EndpointError;
use super::{audit, scope, secret, store};

/// The RFC 7591 registration request body.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// `public` or `confidential`.
    pub client_type: String,
    /// The exact-match redirect URIs (1 to 5).
    pub redirect_uris: Vec<String>,
    /// Optional human-readable name (<= 64 chars).
    #[serde(default)]
    pub client_name: Option<String>,
    /// Space-separated requested scopes.
    pub scope: String,
}

/// The RFC 7591 registration response.
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    /// The generated client id.
    pub client_id: String,
    /// The client secret - present only for confidential clients, shown exactly once.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    /// Unix timestamp the client was issued.
    pub client_id_issued_at: i64,
}

/// Register a client. `caller` is the authenticated session from a bearer auth JWT (required only for
/// confidential clients).
pub async fn register(
    pool: &PgPool,
    req: RegisterRequest,
    caller: Option<AuthSession>,
) -> Result<RegisterResponse, EndpointError> {
    let ctype = ClientType::from_wire(&req.client_type)
        .ok_or_else(|| OAuthError::invalid_request("client_type must be public or confidential"))?;

    // Common validation.
    if req.redirect_uris.is_empty() || req.redirect_uris.len() > 5 {
        return Err(
            OAuthError::invalid_request("redirect_uris must contain 1 to 5 entries").into(),
        );
    }
    for uri in &req.redirect_uris {
        if !is_allowed_redirect(uri) {
            return Err(
                OAuthError::invalid_request("redirect_uri must be https or http loopback").into(),
            );
        }
    }
    if let Some(name) = req.client_name.as_deref() {
        if name.len() > 64 {
            return Err(OAuthError::invalid_request("client_name exceeds 64 chars").into());
        }
    }
    let requested = scope::parse_scope(&req.scope);
    if requested.is_empty() || requested.iter().any(|s| !scope::is_valid_scope_token(s)) {
        return Err(OAuthError::invalid_scope().into());
    }

    match ctype {
        ClientType::Public => {
            let id = store::insert_client(
                pool,
                "public",
                None,
                None,
                &req.redirect_uris,
                req.client_name.as_deref(),
                &req.scope,
            )
            .await?;
            audit::client_registered(pool, Uuid::nil(), Uuid::nil(), id, "public").await;
            Ok(RegisterResponse {
                client_id: id.to_string(),
                client_secret: None,
                client_id_issued_at: chrono::Utc::now().timestamp(),
            })
        }
        ClientType::Confidential => {
            let caller = caller.ok_or_else(|| {
                OAuthError::invalid_client(
                    "confidential registration requires an authenticated caller",
                )
            })?;
            if !caller.has_role("tenant-admin") {
                return Err(OAuthError::unauthorized_client("tenant-admin role required").into());
            }
            let client_secret = secret::opaque_token_256();
            let secret_hash = hash_secret(&client_secret)?;
            let id = store::insert_client(
                pool,
                "confidential",
                Some(caller.tenant_id),
                Some(&secret_hash),
                &req.redirect_uris,
                req.client_name.as_deref(),
                &req.scope,
            )
            .await?;
            audit::client_registered(
                pool,
                caller.tenant_id,
                caller.subject_id,
                id,
                "confidential",
            )
            .await;
            Ok(RegisterResponse {
                client_id: id.to_string(),
                client_secret: Some(client_secret),
                client_id_issued_at: chrono::Utc::now().timestamp(),
            })
        }
    }
}

/// Argon2id PHC-string hash of a client secret (salt embedded).
fn hash_secret(secret: &str) -> Result<String, EndpointError> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| EndpointError::Internal)
}

/// `https://` anywhere, or `http://` only for loopback hosts (OAuth 2.1 §10.3.3, clause #12).
fn is_allowed_redirect(uri: &str) -> bool {
    if let Some(rest) = uri.strip_prefix("https://") {
        return !rest.is_empty();
    }
    if let Some(rest) = uri.strip_prefix("http://") {
        return rest.starts_with("localhost") || rest.starts_with("127.0.0.1");
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redirect_allows_https_and_loopback_http_only() {
        assert!(is_allowed_redirect("https://app.example.com/cb"));
        assert!(is_allowed_redirect("http://localhost:8080/cb"));
        assert!(is_allowed_redirect("http://127.0.0.1:9000/cb"));
        assert!(!is_allowed_redirect("http://app.example.com/cb"));
        assert!(!is_allowed_redirect("ftp://x"));
        assert!(!is_allowed_redirect("https://"));
    }
}
