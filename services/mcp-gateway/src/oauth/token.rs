//! FR-MCP-004 token endpoint logic (POST /token): `authorization_code` and `refresh_token` grants.
//!
//! authorization_code: consume the one-time code, verify PKCE, mint an access JWT + a rotating opaque
//! refresh token. refresh_token: rotate (mark the presented token used, issue a child); reuse of a
//! non-active token compromises the whole family (clause #9, DEC-806). This slice serves public
//! clients (PKCE proof); confidential client authentication lands with confidential registration.

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use super::enums::GrantType;
use super::error::OAuthError;
use super::response::EndpointError;
use super::secret::{opaque_token_256, sha256_hex};
use super::store::{self, CodeConsumption};
use super::{audit, jwt, pkce};

/// Refresh-token lifetime in days (DEC-805).
const REFRESH_TTL_DAYS: i64 = 30;

/// The `application/x-www-form-urlencoded` body of a token request (RFC 6749 §4.1.3 / §6).
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    /// `authorization_code` or `refresh_token`.
    pub grant_type: String,
    /// The authorization code (authorization_code grant).
    #[serde(default)]
    pub code: Option<String>,
    /// The redirect URI the code was bound to (authorization_code grant).
    #[serde(default)]
    pub redirect_uri: Option<String>,
    /// The PKCE code_verifier (authorization_code grant).
    #[serde(default)]
    pub code_verifier: Option<String>,
    /// The client id (informational for public clients).
    #[serde(default)]
    pub client_id: Option<String>,
    /// The refresh token (refresh_token grant).
    #[serde(default)]
    pub refresh_token: Option<String>,
}

/// The token response (RFC 6749 §5.1).
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    /// The signed access JWT.
    pub access_token: String,
    /// Always `"Bearer"`.
    pub token_type: &'static str,
    /// Access-token lifetime in seconds.
    pub expires_in: i64,
    /// The new (rotated) opaque refresh token.
    pub refresh_token: String,
    /// The granted scope.
    pub scope: String,
}

/// Dispatch a token request on its grant type.
pub async fn token(
    pool: &PgPool,
    issuer: &str,
    req: TokenRequest,
) -> Result<TokenResponse, EndpointError> {
    match GrantType::from_wire(&req.grant_type) {
        Some(GrantType::AuthorizationCode) => authorization_code(pool, issuer, req).await,
        Some(GrantType::RefreshToken) => refresh(pool, issuer, req).await,
        None => Err(OAuthError::unsupported_grant_type().into()),
    }
}

async fn authorization_code(
    pool: &PgPool,
    issuer: &str,
    req: TokenRequest,
) -> Result<TokenResponse, EndpointError> {
    let code = req
        .code
        .ok_or_else(|| OAuthError::invalid_request("code is required"))?;
    let verifier = req
        .code_verifier
        .ok_or_else(|| OAuthError::invalid_request("code_verifier is required (PKCE)"))?;

    let consumed = match store::consume_code(pool, &code).await? {
        CodeConsumption::Consumed(c) => c,
        CodeConsumption::NotFound => {
            return Err(OAuthError::invalid_grant("unknown or already-used code").into())
        }
        CodeConsumption::Expired => return Err(OAuthError::invalid_grant("code_expired").into()),
        CodeConsumption::Replay(replayed) => {
            audit::code_reuse_detected(pool, replayed.client_id).await;
            return Err(OAuthError::invalid_grant("code_replay_detected").into());
        }
    };

    if let Some(ru) = req.redirect_uri.as_deref() {
        if ru != consumed.redirect_uri {
            return Err(OAuthError::invalid_grant("redirect_uri_mismatch").into());
        }
    }
    if !pkce::verify_pkce(&verifier, &consumed.code_challenge) {
        return Err(OAuthError::invalid_grant("pkce_verification_failed").into());
    }

    issue_pair(
        pool,
        issuer,
        &consumed.audience,
        &consumed.scope,
        consumed.client_id,
        consumed.subject_id,
        consumed.tenant_id,
        None,
    )
    .await
}

async fn refresh(
    pool: &PgPool,
    issuer: &str,
    req: TokenRequest,
) -> Result<TokenResponse, EndpointError> {
    let presented = req
        .refresh_token
        .ok_or_else(|| OAuthError::invalid_request("refresh_token is required"))?;
    let hash = sha256_hex(&presented);
    let row = store::get_refresh_by_hash(pool, &hash)
        .await?
        .ok_or_else(|| OAuthError::invalid_grant("unknown refresh_token"))?;

    // Reuse of a rotated-out (`used`) or poisoned (`compromised`) token poisons the whole family.
    if row.state != "active" {
        store::compromise_family(pool, row.family_id).await?;
        audit::refresh_reuse_detected(pool, row.family_id).await;
        return Err(OAuthError::invalid_grant("refresh_token_reuse_detected").into());
    }
    if Utc::now() >= row.expires_at {
        return Err(OAuthError::invalid_grant("refresh_token_expired").into());
    }

    store::mark_refresh_used(pool, &hash).await?;
    issue_pair(
        pool,
        issuer,
        &row.audience,
        &row.scope,
        row.client_id,
        row.subject_id,
        row.tenant_id,
        Some((row.family_id, hash)),
    )
    .await
}

/// Mint an access token plus a new refresh token, persisting the refresh row. `rotate =
/// Some((family_id, parent_hash))` continues that family; `None` roots a new one.
#[allow(clippy::too_many_arguments)]
async fn issue_pair(
    pool: &PgPool,
    issuer: &str,
    audience: &str,
    scope: &str,
    client_id: Uuid,
    subject_id: Uuid,
    tenant_id: Uuid,
    rotate: Option<(Uuid, String)>,
) -> Result<TokenResponse, EndpointError> {
    let minted = jwt::mint_access_token(
        pool,
        issuer,
        audience,
        &subject_id.to_string(),
        scope,
        &client_id.to_string(),
        &tenant_id.to_string(),
    )
    .await?;

    let new_refresh = opaque_token_256();
    let new_hash = sha256_hex(&new_refresh);
    let (family_id, parent) = match &rotate {
        Some((fid, parent_hash)) => (*fid, Some(parent_hash.as_str())),
        None => (Uuid::new_v4(), None),
    };
    let expires_at = Utc::now() + Duration::days(REFRESH_TTL_DAYS);
    store::insert_refresh(
        pool,
        family_id,
        client_id,
        subject_id,
        tenant_id,
        audience,
        scope,
        &new_hash,
        parent,
        expires_at,
        &sha256_hex(&new_hash),
    )
    .await?;

    match &rotate {
        None => {
            audit::token_issued(pool, tenant_id, subject_id, client_id, &minted.jti, scope).await
        }
        Some(_) => {
            audit::token_refreshed(pool, tenant_id, subject_id, client_id, &minted.jti).await
        }
    }

    Ok(TokenResponse {
        access_token: minted.access_token,
        token_type: "Bearer",
        expires_in: minted.expires_in,
        refresh_token: new_refresh,
        scope: scope.to_string(),
    })
}
