//! Auditor JWT verification.

use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::ViewError;

/// Maximum engagement token TTL supported by FR-OBS-008.
pub const AUDITOR_ENGAGEMENT_TTL_DAYS: i64 = 30;

/// Auth configuration.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// HS256 local-dev secret.
    pub hs256_secret: Option<String>,
    /// RS256 public PEM for production verification.
    pub rs256_public_pem: Option<String>,
    /// Expected issuer.
    pub issuer: Option<String>,
    /// Expected audience.
    pub audience: Option<String>,
}

impl AuthConfig {
    /// Local-development HS256 config.
    pub fn local(secret: impl Into<String>) -> Self {
        Self {
            hs256_secret: Some(secret.into()),
            rs256_public_pem: None,
            issuer: None,
            audience: None,
        }
    }
}

/// JWT claims used by the compliance view service.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claims {
    /// Subject id.
    pub sub: String,
    /// Tenant id.
    pub tenant_id: String,
    /// Role names.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Expiry seconds since epoch.
    pub exp: i64,
    /// Issued-at seconds since epoch.
    #[serde(default)]
    pub iat: i64,
    /// Issuer.
    #[serde(default)]
    pub iss: Option<String>,
    /// Audience.
    #[serde(default)]
    pub aud: Option<Vec<String>>,
    /// Request id, if upstream supplied one.
    #[serde(default)]
    pub request_id: Option<String>,
}

impl Claims {
    /// Request id for audit rows.
    pub fn request_id(&self) -> String {
        self.request_id
            .clone()
            .unwrap_or_else(|| format!("compliance_view_{}", Uuid::new_v4()))
    }

    /// True when the token has the auditor role required by FR-OBS-008.
    pub fn has_external_auditor_role(&self) -> bool {
        self.roles
            .iter()
            .any(|role| role == "external_auditor" || role == "auditor")
    }

    /// True when token TTL is at most the per-engagement max.
    pub fn supports_engagement_ttl(&self) -> bool {
        self.iat == 0
            || self.exp - self.iat <= Duration::days(AUDITOR_ENGAGEMENT_TTL_DAYS).num_seconds()
    }
}

/// Extract and verify a bearer token.
pub fn verify_authorization(
    header: Option<&str>,
    config: &AuthConfig,
) -> Result<Claims, ViewError> {
    let token = header
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or(ViewError::AuthFailed)?;
    let mut last_error = None;
    if let Some(secret) = &config.hs256_secret {
        match decode_claims(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            Algorithm::HS256,
            config,
        ) {
            Ok(claims) => return require_auditor_role(claims),
            Err(err) => last_error = Some(err),
        }
    }
    if let Some(pem) = &config.rs256_public_pem {
        let key = DecodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| ViewError::AuthFailed)?;
        match decode_claims(token, &key, Algorithm::RS256, config) {
            Ok(claims) => return require_auditor_role(claims),
            Err(err) => last_error = Some(err),
        }
    }
    Err(last_error.unwrap_or(ViewError::AuthFailed))
}

fn decode_claims(
    token: &str,
    key: &DecodingKey,
    algorithm: Algorithm,
    config: &AuthConfig,
) -> Result<Claims, ViewError> {
    let mut validation = Validation::new(algorithm);
    if let Some(issuer) = &config.issuer {
        validation.set_issuer(&[issuer]);
    }
    if let Some(audience) = &config.audience {
        validation.set_audience(&[audience]);
    } else {
        validation.validate_aud = false;
    }
    decode::<Claims>(token, key, &validation)
        .map(|data| data.claims)
        .map_err(|_| ViewError::AuthFailed)
}

fn require_auditor_role(claims: Claims) -> Result<Claims, ViewError> {
    if !claims.has_external_auditor_role() {
        return Err(ViewError::Forbidden);
    }
    Ok(claims)
}

/// Test/dev helper for issuing an auditor token.
pub fn issue_local_auditor_token(
    secret: &str,
    tenant_id: &str,
    subject_id: &str,
    roles: Vec<String>,
    ttl_days: i64,
) -> String {
    let now = Utc::now();
    let claims = Claims {
        sub: subject_id.to_string(),
        tenant_id: tenant_id.to_string(),
        roles,
        exp: (now + Duration::days(ttl_days)).timestamp(),
        iat: now.timestamp(),
        iss: None,
        aud: None,
        request_id: Some("test-request".to_string()),
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}
