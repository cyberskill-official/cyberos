//! Verifies the CyberOS access token (FR-AUTH-110 provider, FR-AUTH-004 JWKS, RS256) and extracts the
//! caller identity. Mirrors `obs-compliance-view::auth`. The HS256 path is for tests and local dev only.

use std::collections::HashMap;
use std::sync::RwLock;

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
    /// Present only to enforce audience when required (a string or array). jsonwebtoken checks a WRONG aud but
    /// not a MISSING one, so we reject a token that carries no aud at all when `require_aud` is on.
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
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

/// Parse a JWKS document into a kid -> key map. Skips keys without a kid; errors on a malformed set or when
/// no usable key is found.
fn parse_jwks(jwks_json: &str) -> Result<HashMap<String, DecodingKey>, AuthError> {
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
    Ok(by_kid)
}

/// Verifies tokens against the auth JWKS (RS256), or an HS256 secret in tests. The key set lives behind an
/// RwLock so a background refresher can swap in rotated keys without a restart; `verify` stays synchronous.
pub struct Authenticator {
    by_kid: RwLock<HashMap<String, DecodingKey>>,
    fallback: Option<DecodingKey>,
    validation: Validation,
    /// When set, `refresh()` re-fetches the JWKS from here (the source the boot fetch used).
    jwks_url: Option<String>,
    /// True once an audience is required; a token with no `aud` claim is then rejected (see Claims::aud).
    require_aud: bool,
}

impl Authenticator {
    /// HS256 verifier for tests and local dev.
    pub fn from_hs256_secret(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_aud = false;
        Self {
            by_kid: RwLock::new(HashMap::new()),
            fallback: Some(DecodingKey::from_secret(secret)),
            validation,
            jwks_url: None,
            require_aud: false,
        }
    }

    /// RS256 verifier built from the auth service JWKS (FR-AUTH-004).
    pub fn from_jwks(jwks_json: &str) -> Result<Self, AuthError> {
        let by_kid = parse_jwks(jwks_json)?;
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_aud = false;
        Ok(Self {
            by_kid: RwLock::new(by_kid),
            fallback: None,
            validation,
            jwks_url: None,
            require_aud: false,
        })
    }

    /// Remember where the JWKS came from so `refresh()` can re-fetch it (a key rotation then heals in place).
    pub fn set_jwks_url(&mut self, url: String) {
        self.jwks_url = Some(url);
    }

    pub fn has_jwks_url(&self) -> bool {
        self.jwks_url.is_some()
    }

    /// Require the token's `aud` claim to match `aud`. Opt-in (the auth service's exact audience must be known)
    /// so turning it on can never lock everyone out by surprise.
    pub fn require_audience(&mut self, aud: String) {
        self.validation.set_audience(&[aud]);
        self.validation.validate_aud = true;
        self.require_aud = true;
    }

    /// Re-fetch the JWKS and swap the key set in. No-op without a `jwks_url`. Called on an interval so a rotated
    /// signing key is picked up within the refresh window instead of breaking chat until a restart.
    pub async fn refresh(&self) -> Result<(), AuthError> {
        let Some(url) = &self.jwks_url else {
            return Ok(());
        };
        let json = reqwest::get(url)
            .await
            .map_err(|e| AuthError::AuthFailed(format!("fetch jwks: {e}")))?
            .text()
            .await
            .map_err(|e| AuthError::AuthFailed(format!("read jwks: {e}")))?;
        let fresh = parse_jwks(&json)?;
        *self
            .by_kid
            .write()
            .map_err(|_| AuthError::AuthFailed("authenticator key lock poisoned".into()))? = fresh;
        Ok(())
    }

    /// Verify a bearer token and return its claims.
    pub fn verify(&self, token: &str) -> Result<Claims, AuthError> {
        let keys = self
            .by_kid
            .read()
            .map_err(|_| AuthError::AuthFailed("authenticator key lock poisoned".into()))?;
        let data = if keys.is_empty() {
            // HS256 test / local-dev path.
            let key = self
                .fallback
                .as_ref()
                .ok_or_else(|| AuthError::AuthFailed("no verification key".into()))?;
            decode::<Claims>(token, key, &self.validation)
                .map_err(|e| AuthError::AuthFailed(e.to_string()))?
        } else {
            let header = decode_header(token).map_err(|e| AuthError::AuthFailed(e.to_string()))?;
            let kid = header
                .kid
                .ok_or_else(|| AuthError::AuthFailed("token has no kid".into()))?;
            let key = keys
                .get(&kid)
                .ok_or_else(|| AuthError::AuthFailed(format!("unknown kid {kid}")))?;
            decode::<Claims>(token, key, &self.validation)
                .map_err(|e| AuthError::AuthFailed(e.to_string()))?
        };
        // jsonwebtoken rejects a wrong aud but not a missing one; reject the missing case ourselves.
        if self.require_aud && data.claims.aud.is_none() {
            return Err(AuthError::AuthFailed("token missing required aud".into()));
        }
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

    #[test]
    fn audience_is_enforced_only_when_required() {
        #[derive(serde::Serialize)]
        struct C {
            sub: String,
            tenant_id: String,
            roles: Vec<String>,
            exp: i64,
            aud: String,
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mk = |aud: &str| {
            let c = C {
                sub: "cf0f35f7-7770-4598-a656-50493e635351".into(),
                tenant_id: "00000000-0000-0000-0000-000000000000".into(),
                roles: vec![],
                exp: now + 3600,
                aud: aud.into(),
            };
            encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET)).unwrap()
        };
        // No audience required: a token with any aud (or none) verifies.
        let open = Authenticator::from_hs256_secret(SECRET);
        assert!(open.verify(&mk("anything")).is_ok());

        // Audience required: only the matching aud verifies; a wrong one and a token without aud are refused.
        let mut strict = Authenticator::from_hs256_secret(SECRET);
        strict.require_audience("cyberos-chat".into());
        assert!(strict.verify(&mk("cyberos-chat")).is_ok());
        assert!(strict.verify(&mk("someone-else")).is_err());
        assert!(strict
            .verify(&token(
                "cf0f35f7-7770-4598-a656-50493e635351",
                "00000000-0000-0000-0000-000000000000",
                3600
            ))
            .is_err());
    }
}

/// FR-CHAT-269 §1 #2 — the moderation role gate.
///
/// Workspace-level roles only. A CHANNEL role (`owner`, `admin`, `member` in `chat_channel_members`) grants
/// nothing here, deliberately: a channel owner is not a workspace moderator, and a report raised in a
/// channel they own may well be *about* them (§1 #3).
pub const MODERATOR_ROLES: [&str; 2] = ["tenant-admin", "root-admin"];

/// Fail CLOSED. An absent or empty `roles` claim is not "unknown, therefore allow" — it is "unknown,
/// therefore no".
///
/// This matters more than it looks. FR-AUTH-101 permits a grace window in which a token may carry no roles
/// claim at all. Chat has never read `roles` before, so it has no legacy tokens to be gentle with — and an
/// else-allow branch here would make every pre-FR-AUTH-101 token in circulation a moderator. There is no
/// else-allow branch. `Claims::roles` is `#[serde(default)]`-shaped as a `Vec`, so a missing claim
/// deserialises to an empty vec, which matches nothing and is refused.
pub fn require_moderator(claims: &Claims) -> Result<(), (StatusCode, String)> {
    if claims
        .roles
        .iter()
        .any(|r| MODERATOR_ROLES.contains(&r.as_str()))
    {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        "moderation requires tenant-admin".to_string(),
    ))
}

#[cfg(test)]
mod moderator_gate_tests {
    use super::*;

    fn claims(roles: &[&str]) -> Claims {
        Claims {
            sub: uuid::Uuid::nil().to_string(),
            tenant_id: uuid::Uuid::nil().to_string(),
            roles: roles.iter().map(|s| s.to_string()).collect(),
            exp: 0,
            aud: None,
        }
    }

    #[test]
    fn moderator_roles_are_admitted() {
        assert!(require_moderator(&claims(&["tenant-admin"])).is_ok());
        assert!(require_moderator(&claims(&["root-admin"])).is_ok());
        // Extra unrelated roles alongside do not break the match.
        assert!(require_moderator(&claims(&["tenant-member", "tenant-admin"])).is_ok());
    }

    #[test]
    fn the_gate_fails_closed() {
        // AC 1 — the whole point. NONE of these may pass.
        for roles in [
            vec![],                 // no roles at all (a pre-FR-AUTH-101 token)
            vec!["tenant-member"],  // an ordinary member
            vec!["owner"],          // AC 2 — a CHANNEL role is not a workspace role
            vec!["admin"],          // ...nor is the channel-level "admin"
            vec!["tenant-admin-x"], // no prefix/substring matching
            vec!["TENANT-ADMIN"],   // exact, case-sensitive
        ] {
            let c = claims(&roles);
            assert_eq!(
                require_moderator(&c).unwrap_err().0,
                StatusCode::FORBIDDEN,
                "roles {roles:?} must NOT grant moderation"
            );
        }
    }
}
