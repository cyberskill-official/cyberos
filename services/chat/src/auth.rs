//! Verifies the CyberOS access token (FR-AUTH-110 provider, FR-AUTH-004 JWKS, RS256) and extracts the
//! caller identity. Mirrors `obs-compliance-view::auth`. The HS256 path is for tests and local dev only.

use std::collections::HashMap;

use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

/// The claims chat needs from the CyberOS access token.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    #[serde(default)]
    pub roles: Vec<String>,
    pub exp: i64,
}

impl Claims {
    pub fn subject_id(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.sub).map_err(|_| AuthError::AuthFailed("sub is not a uuid".into()))
    }
    pub fn tenant_uuid(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.tenant_id)
            .map_err(|_| AuthError::AuthFailed("tenant_id is not a uuid".into()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

/// Verifies tokens against the auth JWKS (RS256), or an HS256 secret in tests.
pub struct Authenticator {
    by_kid: HashMap<String, DecodingKey>,
    fallback: Option<DecodingKey>,
    validation: Validation,
}

impl Authenticator {
    /// HS256 verifier for tests and local dev.
    pub fn from_hs256_secret(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_aud = false;
        Self {
            by_kid: HashMap::new(),
            fallback: Some(DecodingKey::from_secret(secret)),
            validation,
        }
    }

    /// RS256 verifier built from the auth service JWKS (FR-AUTH-004).
    pub fn from_jwks(jwks_json: &str) -> Result<Self, AuthError> {
        let set: JwkSet = serde_json::from_str(jwks_json)
            .map_err(|e| AuthError::AuthFailed(format!("malformed jwks: {e}")))?;
        let mut by_kid = HashMap::new();
        for jwk in &set.keys {
            if let Some(kid) = jwk.common.key_id.clone() {
                let key = DecodingKey::from_jwk(jwk)
                    .map_err(|e| AuthError::AuthFailed(format!("bad jwk {kid}: {e}")))?;
                by_kid.insert(kid, key);
            }
        }
        if by_kid.is_empty() {
            return Err(AuthError::AuthFailed("jwks has no usable keys".into()));
        }
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        Ok(Self {
            by_kid,
            fallback: None,
            validation,
        })
    }

    /// Verify a bearer token and return its claims.
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let key = if self.by_kid.is_empty() {
            self.fallback
                .as_ref()
                .ok_or_else(|| AuthError::AuthFailed("no verification key".into()))?
        } else {
            let header = decode_header(token).map_err(|e| AuthError::AuthFailed(e.to_string()))?;
            let kid = header
                .kid
                .ok_or_else(|| AuthError::AuthFailed("token has no kid".into()))?;
            self.by_kid
                .get(&kid)
                .ok_or_else(|| AuthError::AuthFailed(format!("unknown kid {kid}")))?
        };
        let data = decode::<Claims>(token, key, &self.validation)
            .map_err(|e| AuthError::AuthFailed(e.to_string()))?;
        Ok(data.claims)
    }
}

/// Pull the bearer token out of the Authorization header.
pub fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
}

/// Verify the request's bearer token, returning the claims or a 401.
pub fn authenticate(
    state: &crate::AppState,
    headers: &HeaderMap,
) -> Result<Claims, (StatusCode, String)> {
    let token =
        bearer(headers).ok_or((StatusCode::UNAUTHORIZED, "missing bearer token".to_string()))?;
    state
        .authenticator
        .verify(token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    const SECRET: &[u8] = b"test-secret";

    fn token(sub: &str, tenant: &str, exp_offset: i64) -> String {
        #[derive(serde::Serialize)]
        struct C {
            sub: String,
            tenant_id: String,
            roles: Vec<String>,
            exp: i64,
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let c = C {
            sub: sub.into(),
            tenant_id: tenant.into(),
            roles: vec![],
            exp: now + exp_offset,
        };
        encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET)).unwrap()
    }

    #[test]
    fn valid_token_verifies_and_parses_ids() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let claims = a
            .verify(&token(
                "cf0f35f7-7770-4598-a656-50493e635351",
                "00000000-0000-0000-0000-000000000000",
                3600,
            ))
            .unwrap();
        assert!(claims.subject_id().is_ok());
        assert!(claims.tenant_uuid().is_ok());
    }

    #[test]
    fn expired_token_is_refused() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let err = a
            .verify(&token("s", "00000000-0000-0000-0000-000000000000", -3600))
            .unwrap_err();
        assert!(matches!(err, AuthError::AuthFailed(_)));
    }

    #[test]
    fn garbage_token_is_refused() {
        let a = Authenticator::from_hs256_secret(SECRET);
        assert!(a.verify("not.a.jwt").is_err());
    }
}
