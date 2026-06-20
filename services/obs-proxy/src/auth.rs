//! JWT verification for the proxy (FR-OBS-002 §1 #1).
//!
//! A minimal local Claims mirroring the auth service's token (auth/src/jwt.rs): `tenant_id` is a
//! string, and the nil-UUID tenant (tenant 0) marks a root-admin caller (FR §1 #11). The proxy stays
//! light - it does not depend on the cyberos-auth crate. Tokens are verified against a decoding key;
//! in production that key comes from the auth JWKS at `/.well-known/jwks.json` (the boot fetch lands in
//! slice 4b), and tests build the verifier from a local key.

use crate::error::ProxyError;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

/// Tenant 0 (the nil UUID) is the root-admin tenant - see auth's memory_bridge note.
pub const NIL_TENANT: &str = "00000000-0000-0000-0000-000000000000";

/// The subset of the auth JWT claims the proxy needs.
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

/// Verifies bearer JWTs against a decoding key. Signature and expiry are enforced; a failure becomes
/// `ProxyError::AuthFailed` which the router maps to 401.
pub struct Authenticator {
    key: DecodingKey,
    validation: Validation,
}

impl Authenticator {
    /// Build a verifier from a symmetric secret. Used by tests and by HS256 deployments; the
    /// asymmetric JWKS constructor (production) lands with the boot fetch in slice 4b.
    pub fn from_hs256_secret(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_aud = false; // aud tightening deferred; signature + exp are enforced
        Self {
            key: DecodingKey::from_secret(secret),
            validation,
        }
    }

    /// Verify a raw JWT string and return its claims, or `AuthFailed` (-> 401).
    pub fn verify(&self, token: &str) -> Result<Claims, ProxyError> {
        decode::<Claims>(token, &self.key, &self.validation)
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
}
