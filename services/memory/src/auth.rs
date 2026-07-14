//! MEM-001 (report R73, F15) — verified-JWT identity for the memory service.
//!
//! Before this module, `/v1/memory/recall` and `/v1/memory/search` derived the caller's tenant and subject
//! from the `x-tenant-id` / `x-subject-id` request headers, so any network caller could claim founder-grade
//! visibility in any tenant. Identity now comes ONLY from a verified CyberOS access token (TASK-AUTH-110
//! provider, TASK-AUTH-004 JWKS, RS256); the HS256 path is for tests and local dev.
//!
//! This mirrors `services/chat/src/auth.rs` deliberately (same `Authenticator`, same `Claims`, same JWKS
//! refresh discipline) so there is ONE verification behaviour across the platform and memory cannot drift
//! from the auth contract. The memory-specific additions are:
//!   * [`authenticate_claims`] — resolves a verified token into the brain's [`Caller`] and rejects a request
//!     whose residual `x-tenant-id` header disagrees with the token (400), so a stale hop header can never
//!     silently override the claim;
//!   * [`require_auth`] — the axum middleware mounted on the `/v1/memory` routes that runs
//!     [`authenticate_claims`] and stamps the [`Caller`] into the request extensions;
//!   * [`build_authenticator`] — the boot-time verifier constructor (JWKS URL / inline JSON / file / HS256).

use std::collections::HashMap;
use std::sync::RwLock;

use axum::extract::{Request, State};
use axum::http::{header::AUTHORIZATION, HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Json, Response};
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use uuid::Uuid;

use crate::brain::Caller;
use crate::state::AppState;

/// The claims the memory service needs from a CyberOS access token. Byte-compatible with chat's `Claims`.
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    #[serde(default)]
    pub roles: Vec<String>,
    pub exp: i64,
    /// Present only to enforce audience when required (a string or array). jsonwebtoken checks a WRONG aud but
    /// not a MISSING one, so a token that carries no aud at all is rejected when `require_aud` is on.
    #[serde(default)]
    pub aud: Option<serde_json::Value>,
}

impl Claims {
    /// The caller's subject id (their `viewer_subject_id` in TASK-EVAL-001's `access_grant`).
    pub fn subject_id(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.sub).map_err(|_| AuthError::AuthFailed("sub is not a uuid".into()))
    }
    /// The caller's tenant.
    pub fn tenant_uuid(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.tenant_id)
            .map_err(|_| AuthError::AuthFailed("tenant_id is not a uuid".into()))
    }
}

/// An authentication failure. `AuthFailed` is a 401 to the caller (never leaks which check failed beyond a
/// short reason); a residual-header mismatch is surfaced separately as a 400 by [`authenticate_claims`].
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("authentication failed: {0}")]
    AuthFailed(String),
}

/// Parse a JWKS document into a `kid -> key` map. Skips keys without a kid; errors on a malformed set or when
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
/// `RwLock` so the background refresher can swap in rotated keys without a restart; `verify` stays synchronous.
pub struct Authenticator {
    by_kid: RwLock<HashMap<String, DecodingKey>>,
    fallback: Option<DecodingKey>,
    validation: Validation,
    /// When set, `refresh()` re-fetches the JWKS from here (the source the boot fetch used).
    jwks_url: Option<String>,
    /// True once an audience is required; a token with no `aud` claim is then rejected (see [`Claims::aud`]).
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

    /// RS256 verifier built from the auth service JWKS (TASK-AUTH-004).
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

    /// Whether a background refresher should run (the key set came from a URL).
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
    /// signing key is picked up within the refresh window instead of breaking recall until a restart.
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

/// A JSON error body, mirroring the handlers' `{"error": ...}` shape.
fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(serde_json::json!({ "error": msg })))
}

/// Resolve a verified token into the brain's [`Caller`] (§1 #7 identity). Identity comes ONLY from the token:
///   * no bearer token -> 401;
///   * token fails verification (bad signature / expired / unknown kid / missing required aud) -> 401;
///   * a `sub` / `tenant_id` claim that is not a UUID -> 401;
///   * a residual `x-tenant-id` header that is present but does NOT equal the token's tenant -> 400 (a stale
///     hop header must never silently override the claim; a matching or absent header is fine).
///
/// The returned `Caller` carries the token's tenant and subject — never a header value.
pub fn authenticate_claims(
    auth: &Authenticator,
    headers: &HeaderMap,
) -> Result<Caller, (StatusCode, Json<serde_json::Value>)> {
    let token =
        bearer(headers).ok_or_else(|| err(StatusCode::UNAUTHORIZED, "missing bearer token"))?;
    let claims = auth
        .verify(token)
        .map_err(|e| err(StatusCode::UNAUTHORIZED, &e.to_string()))?;
    let tenant_id = claims
        .tenant_uuid()
        .map_err(|e| err(StatusCode::UNAUTHORIZED, &e.to_string()))?;
    let viewer_subject_id = claims
        .subject_id()
        .map_err(|e| err(StatusCode::UNAUTHORIZED, &e.to_string()))?;

    // A residual `x-tenant-id` header is allowed only as an internal hop hint; if present it MUST equal the
    // verified tenant. An unparseable or mismatched header is a 400 (never trusted, never silently ignored).
    if let Some(raw) = headers.get("x-tenant-id") {
        let header_tenant = raw
            .to_str()
            .ok()
            .and_then(|s| Uuid::parse_str(s).ok())
            .ok_or_else(|| {
                err(
                    StatusCode::BAD_REQUEST,
                    "x-tenant-id header is not a valid UUID",
                )
            })?;
        if header_tenant != tenant_id {
            return Err(err(
                StatusCode::BAD_REQUEST,
                "x-tenant-id header does not match the authenticated tenant",
            ));
        }
    }

    Ok(Caller {
        tenant_id,
        viewer_subject_id,
    })
}

/// axum middleware for the `/v1/memory` routes: verify the bearer token, resolve the [`Caller`], and stamp it
/// into the request extensions for the handler to read. `/healthz` and `/metrics` are mounted OUTSIDE this
/// layer and stay unauthenticated.
pub async fn require_auth(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    match authenticate_claims(&state.authenticator, req.headers()) {
        Ok(caller) => {
            req.extensions_mut().insert(caller);
            next.run(req).await
        }
        Err(rejection) => rejection.into_response(),
    }
}

/// Build the token verifier from, in priority order: the auth JWKS URL (fetched once at startup, the
/// production-aligned path), inline JWKS JSON, a JWKS file, or an HS256 secret (tests / local). When
/// `MEMORY_AUTH_AUDIENCE` is set the token's `aud` is additionally required to match it.
///
/// Returns an error when NO verifier is configured: the memory service refuses to boot without one, because
/// `/v1/memory/*` must never fall back to header-trust (R73 fail-closed).
pub async fn build_authenticator() -> Result<Authenticator, AuthError> {
    // Read an env var only when it is set AND non-empty. An empty value (e.g. `MEMORY_AUTH_JWKS_URL=` left in
    // .env) must fall through to the next verifier, not crash `reqwest::get("")` with "relative URL without a
    // base" — which would take the whole service down at boot.
    let env_set = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());

    let mut auth = if let Some(url) = env_set("MEMORY_AUTH_JWKS_URL") {
        let json = reqwest::get(&url)
            .await
            .map_err(|e| AuthError::AuthFailed(format!("fetch JWKS from {url}: {e}")))?
            .text()
            .await
            .map_err(|e| AuthError::AuthFailed(format!("read JWKS body from {url}: {e}")))?;
        let mut a = Authenticator::from_jwks(&json)?;
        // Remember the URL so the background refresher can pick up a rotated signing key without a restart.
        a.set_jwks_url(url);
        a
    } else if let Some(json) = env_set("MEMORY_AUTH_JWKS_JSON") {
        Authenticator::from_jwks(&json)?
    } else if let Some(path) = env_set("MEMORY_AUTH_JWKS_PATH") {
        let json = std::fs::read_to_string(&path)
            .map_err(|e| AuthError::AuthFailed(format!("read JWKS file {path}: {e}")))?;
        Authenticator::from_jwks(&json)?
    } else if let Some(secret) = env_set("MEMORY_AUTH_HS256_SECRET") {
        Authenticator::from_hs256_secret(secret.as_bytes())
    } else {
        return Err(AuthError::AuthFailed(
            "no token verifier configured: set a non-empty MEMORY_AUTH_JWKS_URL, MEMORY_AUTH_JWKS_JSON, \
             MEMORY_AUTH_JWKS_PATH, or MEMORY_AUTH_HS256_SECRET"
                .into(),
        ));
    };

    if let Some(aud) = env_set("MEMORY_AUTH_AUDIENCE") {
        auth.require_audience(aud);
    }
    Ok(auth)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;
    use jsonwebtoken::{encode, EncodingKey, Header};

    const SECRET: &[u8] = b"test-secret";
    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";
    const SUBJECT: &str = "cf0f35f7-7770-4598-a656-50493e635351";

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

    fn headers_with(bearer: Option<&str>, x_tenant: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(b) = bearer {
            h.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&format!("Bearer {b}")).unwrap(),
            );
        }
        if let Some(t) = x_tenant {
            h.insert("x-tenant-id", HeaderValue::from_str(t).unwrap());
        }
        h
    }

    #[test]
    fn valid_token_verifies_and_parses_ids() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let claims = a.verify(&token(SUBJECT, TENANT_A, 3600)).unwrap();
        assert!(claims.subject_id().is_ok());
        assert!(claims.tenant_uuid().is_ok());
    }

    #[test]
    fn expired_token_is_refused() {
        let a = Authenticator::from_hs256_secret(SECRET);
        assert!(a.verify(&token(SUBJECT, TENANT_A, -3600)).is_err());
    }

    #[test]
    fn garbage_token_is_refused() {
        let a = Authenticator::from_hs256_secret(SECRET);
        assert!(a.verify("not.a.jwt").is_err());
    }

    // ---- authenticate_claims: identity comes ONLY from the token ----

    #[test]
    fn claims_resolve_to_caller_from_token() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let h = headers_with(Some(&token(SUBJECT, TENANT_A, 3600)), None);
        let caller = authenticate_claims(&a, &h).unwrap();
        assert_eq!(caller.tenant_id, Uuid::parse_str(TENANT_A).unwrap());
        assert_eq!(caller.viewer_subject_id, Uuid::parse_str(SUBJECT).unwrap());
    }

    #[test]
    fn missing_bearer_is_401() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let (status, _) = authenticate_claims(&a, &headers_with(None, None)).unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn expired_token_via_claims_is_401() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let h = headers_with(Some(&token(SUBJECT, TENANT_A, -3600)), None);
        let (status, _) = authenticate_claims(&a, &h).unwrap_err();
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn matching_x_tenant_header_is_allowed() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let h = headers_with(Some(&token(SUBJECT, TENANT_A, 3600)), Some(TENANT_A));
        assert!(authenticate_claims(&a, &h).is_ok());
    }

    #[test]
    fn forged_x_tenant_header_cannot_override_token() {
        // A valid token for tenant A carrying a spoofed `x-tenant-id: B` is rejected (400) — it NEVER
        // resolves to tenant B. This is the core R73/F15 guarantee: identity is claim-derived.
        let a = Authenticator::from_hs256_secret(SECRET);
        let h = headers_with(Some(&token(SUBJECT, TENANT_A, 3600)), Some(TENANT_B));
        let (status, _) = authenticate_claims(&a, &h).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn unparseable_x_tenant_header_is_400() {
        let a = Authenticator::from_hs256_secret(SECRET);
        let h = headers_with(Some(&token(SUBJECT, TENANT_A, 3600)), Some("not-a-uuid"));
        let (status, _) = authenticate_claims(&a, &h).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    #[test]
    fn audience_is_enforced_only_when_required() {
        #[derive(serde::Serialize)]
        struct C {
            sub: String,
            tenant_id: String,
            exp: i64,
            aud: String,
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let mk = |aud: &str| {
            let c = C {
                sub: SUBJECT.into(),
                tenant_id: TENANT_A.into(),
                exp: now + 3600,
                aud: aud.into(),
            };
            encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET)).unwrap()
        };
        let open = Authenticator::from_hs256_secret(SECRET);
        assert!(open.verify(&mk("anything")).is_ok());

        let mut strict = Authenticator::from_hs256_secret(SECRET);
        strict.require_audience("cyberos-memory".into());
        assert!(strict.verify(&mk("cyberos-memory")).is_ok());
        assert!(strict.verify(&mk("someone-else")).is_err());
        // A token with no aud at all is refused when an audience is required.
        assert!(strict.verify(&token(SUBJECT, TENANT_A, 3600)).is_err());
    }
}
