//! FR-MCP-004 authorization endpoint (GET /authorize -> 302): issues a one-time, 30-second PKCE-bound
//! authorization code (clauses #2, #10, #13, #14, #28, #29).
//!
//! Subject identity comes from an auth-service bearer JWT (the non-interactive realization of "existing
//! AUTH session", clause #28): the client first authenticates to FR-AUTH, then presents that JWT here.
//! Errors that occur after the client and redirect_uri are validated are returned to the redirect_uri
//! as `error=...&state=...` (OAuth 2.1 §4.1.2.1); errors before that (unknown client, redirect
//! mismatch) are returned directly, never by redirecting to an unvalidated URI.
//!
//! Headless notes: with no login or consent UI in the gateway, a missing or invalid subject JWT
//! redirects `error=login_required`, and consent is auto-recorded rather than prompted. An interactive
//! consent screen is a UI follow-up.

use chrono::{Duration, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use super::authsession;
use super::error::OAuthError;
use super::response::EndpointError;
use super::{audit, scope, secret, store};

/// The query parameters of an authorization request.
#[derive(Debug, Deserialize)]
pub struct AuthorizeParams {
    /// Must be `code`.
    pub response_type: String,
    /// The requesting client's id (UUID).
    pub client_id: String,
    /// The redirect URI; must exactly match one registered for the client.
    pub redirect_uri: String,
    /// PKCE challenge (required for public clients).
    #[serde(default)]
    pub code_challenge: Option<String>,
    /// PKCE method; must be `S256`.
    #[serde(default)]
    pub code_challenge_method: Option<String>,
    /// Space-separated requested scopes.
    #[serde(default)]
    pub scope: String,
    /// Opaque CSRF state, required and echoed back unchanged.
    #[serde(default)]
    pub state: String,
    /// Optional client nonce.
    #[serde(default)]
    pub nonce: Option<String>,
    /// Optional `prompt` (e.g. `none` for silent re-auth).
    #[serde(default)]
    pub prompt: Option<String>,
}

/// Process an authorization request and return the URL to redirect the user-agent to (carrying either
/// `code` + `state` on success, or `error` + `state`). A direct `EndpointError` is returned only when
/// the client or redirect_uri cannot be validated, since redirecting then would be unsafe.
pub async fn authorize(
    pool: &PgPool,
    resource: &str,
    auth_issuer: &str,
    params: AuthorizeParams,
    bearer: Option<String>,
) -> Result<String, EndpointError> {
    // 1. Validate the client and the redirect_uri before trusting the redirect target.
    let client_id = Uuid::parse_str(&params.client_id)
        .map_err(|_| OAuthError::invalid_request("client_id must be a UUID"))?;
    let client = store::get_client(pool, client_id)
        .await?
        .ok_or_else(|| OAuthError::invalid_request("unknown client_id"))?;
    if client.revoked_at.is_some() {
        return Err(OAuthError::invalid_request("client is revoked").into());
    }
    if !client
        .redirect_uris
        .iter()
        .any(|u| u == &params.redirect_uri)
    {
        return Err(OAuthError::invalid_request("redirect_uri_mismatch").into());
    }

    // From here, surface errors back to the (validated) redirect_uri.
    let redirect_uri = params.redirect_uri.clone();
    let state = params.state.clone();
    let err = |code: &str| {
        Ok(build_redirect(
            &redirect_uri,
            &[("error", code), ("state", &state)],
        ))
    };

    if state.is_empty() {
        return Ok(build_redirect(
            &redirect_uri,
            &[("error", "invalid_request")],
        ));
    }
    if params.response_type != "code" {
        return err("unsupported_response_type");
    }
    let Some(code_challenge) = params.code_challenge.clone() else {
        return err("invalid_request"); // PKCE required
    };
    if params.code_challenge_method.as_deref() != Some("S256") {
        return err("invalid_request"); // S256 only
    }

    // 2. Subject identity from the auth bearer JWT.
    let subject = match &bearer {
        Some(token) => match authsession::verify_auth_session(pool, token, auth_issuer).await {
            Some(s) => s,
            None => return err("login_required"),
        },
        None => return err("login_required"),
    };

    // 3. Scope must be a subset of the client's registered scope (DEC-813).
    let registered = scope::parse_scope(&client.scope);
    let requested = scope::parse_scope(&params.scope);
    if scope::validate_scopes(&requested, &registered).is_err() {
        return err("invalid_scope");
    }

    // 4. Record consent (headless) and 5. issue the 30-second one-time code.
    store::upsert_consent(pool, subject.subject_id, client_id, &params.scope).await?;
    let code = secret::opaque_token_256(); // 43 chars, matches the oauth_codes length check
    let nonce = params
        .nonce
        .clone()
        .unwrap_or_else(|| Uuid::new_v4().simple().to_string());
    let expires_at = Utc::now() + Duration::seconds(30);
    store::insert_code(
        pool,
        &code,
        client_id,
        subject.subject_id,
        subject.tenant_id,
        &redirect_uri,
        &code_challenge,
        &params.scope,
        resource,
        &nonce,
        &state,
        expires_at,
        &secret::sha256_hex(&code),
    )
    .await?;

    audit::authorize_started(pool, subject.tenant_id, subject.subject_id, client_id).await;

    Ok(build_redirect(
        &redirect_uri,
        &[("code", &code), ("state", &state)],
    ))
}

/// Build `base?k1=v1&k2=v2`, percent-encoding each value. Assumes `base` has no existing query.
fn build_redirect(base: &str, params: &[(&str, &str)]) -> String {
    let query: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{k}={}", percent_encode(v)))
        .collect();
    format!("{base}?{}", query.join("&"))
}

/// Percent-encode a query value, leaving RFC 3986 unreserved characters untouched.
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encode_leaves_unreserved_and_escapes_the_rest() {
        assert_eq!(percent_encode("abcXYZ0-9_.~"), "abcXYZ0-9_.~");
        assert_eq!(percent_encode("a b&c=d"), "a%20b%26c%3Dd");
    }

    #[test]
    fn build_redirect_joins_encoded_params() {
        let url = build_redirect("https://app/cb", &[("code", "x_y-1"), ("state", "a b")]);
        assert_eq!(url, "https://app/cb?code=x_y-1&state=a%20b");
    }
}
