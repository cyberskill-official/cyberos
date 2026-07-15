//! Caller identity (TASK-EVAL-001 slice 2). Verifies the CyberOS access token (TASK-AUTH-110 provider,
//! TASK-AUTH-004 JWKS, RS256) and extracts the caller. Mirrors `cyberos_chat::auth` exactly - the same
//! JwkSet parse, the same per-kid `DecodingKey` cache held for the process lifetime, the same bearer
//! extraction - so the two services verify CyberOS tokens identically. The HS256 path is for tests and
//! local dev only.
//!
//! The JWKS is fetched once at boot from `EVAL_AUTH_JWKS_URL` (see `main::build_authenticator`) and the
//! parsed keys are cached in [`Authenticator::by_kid`] for the lifetime of the process; that is the
//! fetch-and-cache the gate relies on, the same shape chat uses.

use std::collections::HashMap;

use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

/// The role string that grants the founder-can-read-anyone path (clause 7a) and founder-only mutations.
/// Matches AUTH's `Role::Founder` wire form (`services/auth/src/rbac/catalogue.rs`).
pub const ROLE_FOUNDER: &str = "founder";

/// Roles permitted to record an HR acknowledgment (clause 10a / Operating mode: the HR action that flips
/// the consent gate). Founder, the tenant admin, the Chief Human Resources Officer, and the Data
/// Protection Officer - all AUTH wire forms. The signed-contract acknowledgment is recorded BY these
/// roles on the employee's behalf; the employee never self-acknowledges in the quiet operating mode.
pub const ACK_RECORDER_ROLES: &[&str] = &[ROLE_FOUNDER, "tenant-admin", "chro", "dpo"];

/// Roles permitted to ADMINISTER the TASK-EVAL-002 rubric - create / add items / publish (DEC-2601 §1 #10:
/// "founder + designated rubric admins"). The founder always counts (via `is_founder`); `rubric-admin` is
/// the AUTH wire form for a delegate the founder designates. This is the authoring grant the task requires;
/// it deliberately does not invent an access rule, only names which AUTH roles hold the existing
/// founder/admin authority over the rubric.
pub const RUBRIC_ADMIN_ROLES: &[&str] = &[ROLE_FOUNDER, "rubric-admin"];

/// The claims eval needs from the CyberOS access token. Identical shape to `cyberos_chat::auth::Claims`.
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

/// The verified caller identity threaded through every handler (clause 7, 12). `subject_id` and
/// `tenant_id` are parsed from the token's `sub` / `tenant_id`; `is_founder` and the convenience
/// role checks are derived from the `roles` claim (TASK-AUTH-101 `founder`).
#[derive(Debug, Clone)]
pub struct Caller {
    pub subject_id: Uuid,
    pub tenant_id: Uuid,
    pub is_founder: bool,
    pub roles: Vec<String>,
}

impl Caller {
    /// Build a `Caller` from verified `Claims` (after signature + exp checks pass).
    pub fn from_claims(claims: &Claims) -> Result<Self, AuthError> {
        let subject_id = claims.subject_id()?;
        let tenant_id = claims.tenant_uuid()?;
        let is_founder = claims.roles.iter().any(|r| r == ROLE_FOUNDER);
        Ok(Self {
            subject_id,
            tenant_id,
            is_founder,
            roles: claims.roles.clone(),
        })
    }

    /// Whether the caller holds any of `roles` (founder always counts via `is_founder`).
    pub fn has_any_role(&self, roles: &[&str]) -> bool {
        self.roles.iter().any(|r| roles.contains(&r.as_str()))
    }

    /// Whether the caller may record a subject acknowledgment (founder / admin / HR - clause 10a).
    pub fn may_record_ack(&self) -> bool {
        self.has_any_role(ACK_RECORDER_ROLES)
    }

    /// Whether the caller may administer the rubric - author / publish (TASK-EVAL-002 §1 #10). Founder or a
    /// designated rubric admin.
    pub fn may_administer_rubric(&self) -> bool {
        self.has_any_role(RUBRIC_ADMIN_ROLES)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

/// Verifies tokens against the auth JWKS (RS256), or an HS256 secret in tests. Mirrors
/// `cyberos_chat::auth::Authenticator`.
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

    /// RS256 verifier built from the auth service JWKS (TASK-AUTH-004). The parsed per-kid keys are the
    /// cache: they live for the process lifetime, so token verification never re-fetches.
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

/// Verify the request's bearer token, returning the claims or a 401. Mirrors
/// `cyberos_chat::auth::authenticate`.
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

/// The one entry every handler uses: verify the bearer token and return the [`Caller`] (subject +
/// tenant + roles), or a 401. A token whose `sub` / `tenant_id` is not a UUID is a 401, not a 500.
pub fn caller(
    state: &crate::AppState,
    headers: &HeaderMap,
) -> Result<Caller, (StatusCode, String)> {
    let claims = authenticate(state, headers)?;
    Caller::from_claims(&claims).map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    const SECRET: &[u8] = b"test-secret";

    fn token(sub: &str, tenant: &str, roles: Vec<&str>, exp_offset: i64) -> String {
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
            roles: roles.into_iter().map(|s| s.to_string()).collect(),
            exp: now + exp_offset,
        };
        encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET)).unwrap()
    }

    #[test]
    fn valid_token_verifies_and_builds_caller() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let claims = a
            .verify(&token(
                "cf0f35f7-7770-4598-a656-50493e635351",
                "00000000-0000-0000-0000-000000000000",
                vec!["founder"],
                3600,
            ))
            .unwrap();
        let caller = Caller::from_claims(&claims).unwrap();
        assert!(caller.is_founder);
        assert!(caller.may_record_ack());
    }

    #[test]
    fn non_founder_is_not_founder_and_cannot_record_ack_unless_hr() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let claims = a
            .verify(&token(
                "cf0f35f7-7770-4598-a656-50493e635351",
                "00000000-0000-0000-0000-000000000000",
                vec!["tenant-member"],
                3600,
            ))
            .unwrap();
        let caller = Caller::from_claims(&claims).unwrap();
        assert!(!caller.is_founder);
        assert!(!caller.may_record_ack());
    }

    #[test]
    fn expired_token_is_refused() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let err = a
            .verify(&token(
                "s",
                "00000000-0000-0000-0000-000000000000",
                vec![],
                -3600,
            ))
            .unwrap_err();
        assert!(matches!(err, AuthError::AuthFailed(_)));
    }

    #[test]
    fn garbage_token_is_refused() {
        let a = Authenticator::from_hs256_secret(SECRET);
        assert!(a.verify("not.a.jwt").is_err());
    }
}
