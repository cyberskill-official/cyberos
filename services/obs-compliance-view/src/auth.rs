//! Auditor authentication + tenant scoping for the compliance views (TASK-OBS-008 §1 #2, #3, #13). Access
//! requires a JWT carrying the `external_auditor` role - tenant-admin does NOT grant it (§1 #2). The
//! auditor sees only their assigned tenant, and a cross-tenant `?tenant_id=` is refused (§1 #3). The
//! verifier is the TASK-AUTH-004 JWKS (RS256), with an HS256 constructor for tests and local dev.

use std::collections::HashMap;

use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;

/// The role a JWT must carry to read any compliance view (§1 #2). Provisioned per engagement via
/// `cyberos-auth issue-auditor-token` (§1 #13).
pub const AUDITOR_ROLE: &str = "external_auditor";

/// The claims the compliance view needs from the auditor JWT.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    #[serde(default)]
    pub roles: Vec<String>,
    #[serde(default)]
    pub aud: Vec<String>,
    pub exp: i64,
}

impl Claims {
    pub fn is_auditor(&self) -> bool {
        self.roles.iter().any(|r| r == AUDITOR_ROLE)
    }
}

/// Why access was refused.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AuthError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
    #[error("caller lacks the external_auditor role")]
    NotAuditor,
    #[error("cross-tenant access refused")]
    CrossTenant,
}

/// Verifies auditor JWTs against the auth JWKS (RS256), or an HS256 secret in tests.
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

    /// RS256 verifier built from the auth service JWKS (TASK-AUTH-004).
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

    /// Verify and require the auditor role (§1 #2).
    pub fn authorize_auditor(&self, token: &str) -> Result<Claims, AuthError> {
        let claims = self.verify(token)?;
        if !claims.is_auditor() {
            return Err(AuthError::NotAuditor);
        }
        Ok(claims)
    }
}

/// §1 #3 - an auditor may only query their own tenant; a cross-tenant `?tenant_id=` query parameter is
/// refused. `None` means no explicit parameter, so the JWT tenant is used.
pub fn enforce_tenant_scope(
    claims_tenant: &str,
    requested_tenant: Option<&str>,
) -> Result<(), AuthError> {
    match requested_tenant {
        None => Ok(()),
        Some(t) if t == claims_tenant => Ok(()),
        Some(_) => Err(AuthError::CrossTenant),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    const SECRET: &[u8] = b"test-secret";

    fn token(roles: &[&str], tenant: &str) -> String {
        #[derive(serde::Serialize)]
        struct C {
            sub: String,
            tenant_id: String,
            roles: Vec<String>,
            aud: Vec<String>,
            exp: i64,
        }
        let exp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600;
        let c = C {
            sub: "auditor-1".into(),
            tenant_id: tenant.into(),
            roles: roles.iter().map(|s| (*s).to_string()).collect(),
            aud: vec![],
            exp,
        };
        encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET)).unwrap()
    }

    fn auth() -> Authenticator {
        Authenticator::from_hs256_secret(SECRET)
    }

    #[test]
    fn auditor_role_is_authorized() {
        let claims = auth()
            .authorize_auditor(&token(&["external_auditor"], "org:acme"))
            .unwrap();
        assert_eq!(claims.tenant_id, "org:acme");
    }

    #[test]
    fn missing_auditor_role_is_refused() {
        let e = auth()
            .authorize_auditor(&token(&["tenant_admin"], "org:acme"))
            .unwrap_err();
        assert_eq!(e, AuthError::NotAuditor);
    }

    #[test]
    fn a_bad_token_fails_auth() {
        let e = auth().authorize_auditor("not.a.jwt").unwrap_err();
        assert!(matches!(e, AuthError::AuthFailed(_)));
    }

    #[test]
    fn tenant_scope_allows_own_tenant_and_no_param() {
        assert_eq!(enforce_tenant_scope("org:acme", None), Ok(()));
        assert_eq!(enforce_tenant_scope("org:acme", Some("org:acme")), Ok(()));
    }

    #[test]
    fn cross_tenant_param_is_refused() {
        assert_eq!(
            enforce_tenant_scope("org:acme", Some("org:other")),
            Err(AuthError::CrossTenant)
        );
    }
}
