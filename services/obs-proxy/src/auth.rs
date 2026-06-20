//! JWT verification for the proxy (FR-OBS-002 §1 #1).
//!
//! Tokens come from the auth service (auth/src/jwt.rs): RS256, verified against the JWKS published at
//! `/.well-known/jwks.json`, with a per-key `kid` in the token header. `tenant_id` is a string claim
//! and the nil-UUID tenant (tenant 0) marks a root-admin caller (FR §1 #11). The proxy keeps a minimal
//! local `Claims` and does not depend on the cyberos-auth crate. Tests build the verifier from a
//! symmetric key (HS256); production builds it from the fetched JWKS.

use crate::error::ProxyError;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tenant 0 (the nil UUID) is the root-admin tenant - see auth's memory_bridge note.
pub const NIL_TENANT: &str = "00000000-0000-0000-0000-000000000000";

/// The subset of the auth JWT claims the proxy needs. Extra claims (iss, roles, rbac_v, traceparent,
/// ...) are ignored by serde, so auth's full tokens verify here unchanged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    #[serde(default)]
    pub aud: Vec<String>,
    pub exp: i64,
}

impl Claims {
    /// Root-admin queries cross-tenant by design (FR §1 #11); detected by the nil-UUID tenant.
    pub fn is_root_admin(&self) -> bool {
        self.tenant_id == NIL_TENANT
    }
}

/// Verifies bearer JWTs. Signature and expiry are enforced; failures map to `AuthFailed` (-> 401).
///
/// Holds either a set of JWKS keys selected by the token's `kid` (production), or a single fallback
/// key used when the token carries no `kid` (the HS256 test path).
pub struct Authenticator {
    by_kid: HashMap<String, DecodingKey>,
    fallback: Option<DecodingKey>,
    validation: Validation,
}

impl Authenticator {
    /// Build a verifier from a symmetric secret (tests / HS256 deployments).
    pub fn from_hs256_secret(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_aud = false;
        Self {
            by_kid: HashMap::new(),
            fallback: Some(DecodingKey::from_secret(secret)),
            validation,
        }
    }

    /// Build a verifier from the auth JWKS document (production; RS256). The binary fetches the JWKS
    /// from `/.well-known/jwks.json` at boot (and refreshes per FR §1 #12) and passes the JSON here.
    pub fn from_jwks(jwks_json: &str) -> Result<Self, ProxyError> {
        let set: jsonwebtoken::jwk::JwkSet = serde_json::from_str(jwks_json)
            .map_err(|e| ProxyError::AuthFailed(format!("jwks parse failed: {e}")))?;
        let mut by_kid = HashMap::new();
        for jwk in &set.keys {
            if let Some(kid) = jwk.common.key_id.clone() {
                let key = DecodingKey::from_jwk(jwk)
                    .map_err(|e| ProxyError::AuthFailed(format!("jwks key build failed: {e}")))?;
                by_kid.insert(kid, key);
            }
        }
        if by_kid.is_empty() {
            return Err(ProxyError::AuthFailed(
                "jwks has no usable (kid-bearing) keys".into(),
            ));
        }
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        Ok(Self {
            by_kid,
            fallback: None,
            validation,
        })
    }

    /// Verify a raw JWT and return its claims, or `AuthFailed` (-> 401).
    pub fn verify(&self, token: &str) -> Result<Claims, ProxyError> {
        let key = if self.by_kid.is_empty() {
            self.fallback
                .as_ref()
                .ok_or_else(|| ProxyError::AuthFailed("no verifying key configured".into()))?
        } else {
            let kid = decode_header(token)
                .map_err(|e| ProxyError::AuthFailed(e.to_string()))?
                .kid
                .ok_or_else(|| ProxyError::AuthFailed("token has no kid".into()))?;
            self.by_kid
                .get(&kid)
                .ok_or_else(|| ProxyError::AuthFailed(format!("unknown kid: {kid}")))?
        };
        decode::<Claims>(token, key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| ProxyError::AuthFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn future() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600
    }

    fn mint(secret: &[u8], tenant_id: &str, exp: i64) -> String {
        let claims = Claims {
            sub: "subject-1".into(),
            tenant_id: tenant_id.into(),
            aud: vec!["cyberos".into()],
            exp,
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(secret),
        )
        .unwrap()
    }

    #[test]
    fn verifies_and_extracts_tenant() {
        let a = Authenticator::from_hs256_secret(b"k");
        let c = a.verify(&mint(b"k", "org:cyberskill", future())).unwrap();
        assert_eq!(c.tenant_id, "org:cyberskill");
        assert!(!c.is_root_admin());
    }

    #[test]
    fn nil_tenant_is_root_admin() {
        let a = Authenticator::from_hs256_secret(b"k");
        let c = a.verify(&mint(b"k", NIL_TENANT, future())).unwrap();
        assert!(c.is_root_admin());
    }

    #[test]
    fn wrong_signing_key_rejected() {
        let a = Authenticator::from_hs256_secret(b"k");
        assert!(a.verify(&mint(b"different", "T", future())).is_err());
    }

    #[test]
    fn expired_token_rejected() {
        let a = Authenticator::from_hs256_secret(b"k");
        assert!(a.verify(&mint(b"k", "T", 1_000_000_000)).is_err());
    }

    #[test]
    fn malformed_token_rejected() {
        let a = Authenticator::from_hs256_secret(b"k");
        assert!(a.verify("not.a.jwt").is_err());
    }

    #[test]
    fn from_jwks_rejects_malformed_json() {
        assert!(Authenticator::from_jwks("{ not json").is_err());
    }

    #[test]
    fn from_jwks_rejects_empty_keyset() {
        assert!(Authenticator::from_jwks(r#"{"keys":[]}"#).is_err());
    }
}
