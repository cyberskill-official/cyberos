//! FR-MCP-004 dynamic client registration (RFC 7591), POST /register.
//!
//! This slice registers public clients (CLIs, desktop apps - the primary MCP case, DEC-803), which
//! carry no secret and prove possession via PKCE. Confidential client registration returns
//! `invalid_request` until the Argon2 secret-hashing slice lands.

use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use super::enums::ClientType;
use super::error::OAuthError;
use super::response::EndpointError;
use super::scope;
use super::store;

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

/// The RFC 7591 registration response (public client: no secret).
#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    /// The generated client id.
    pub client_id: String,
    /// Unix timestamp the client was issued.
    pub client_id_issued_at: i64,
}

/// Register a public client.
pub async fn register(pool: &PgPool, req: RegisterRequest) -> Result<RegisterResponse, EndpointError> {
    let ctype = ClientType::from_wire(&req.client_type)
        .ok_or_else(|| OAuthError::invalid_request("client_type must be public or confidential"))?;
    if ctype != ClientType::Public {
        return Err(
            OAuthError::invalid_request("confidential client registration is not yet supported")
                .into(),
        );
    }
    if req.redirect_uris.is_empty() || req.redirect_uris.len() > 5 {
        return Err(OAuthError::invalid_request("redirect_uris must contain 1 to 5 entries").into());
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
    // Scope syntax only here; membership is enforced at authorize/token (DEC-813).
    let requested = scope::parse_scope(&req.scope);
    if requested.is_empty() || requested.iter().any(|s| !scope::is_valid_scope_token(s)) {
        return Err(OAuthError::invalid_scope().into());
    }

    let id = store::insert_client(
        pool,
        ctype.as_str(),
        None,
        None,
        &req.redirect_uris,
        req.client_name.as_deref(),
        &req.scope,
    )
    .await?;
    Ok(RegisterResponse {
        client_id: id.to_string(),
        client_id_issued_at: chrono::Utc::now().timestamp(),
    })
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
