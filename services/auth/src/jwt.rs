//! FR-AUTH-004 — RS256 JWT issuance + verification + JWKS publication.
//!
//! - `issue` mints a JWT for a (tenant, subject, scopes) tuple.
//! - `verify` validates incoming JWTs against the published JWKS.
//! - `jwks_for_publication` returns the public-key set for `/.well-known/jwks.json`.
//!
//! Per AUTHORING_DISCIPLINE §3.7 rule 22, every JWT carries a `traceparent`
//! field so OBS can stitch a request lineage across services.

use chrono::{DateTime, Duration, Utc};
use cyberos_types::{SubjectId, TenantId};
use jsonwebtoken::{
    decode, decode_header, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

const ALG: Algorithm = Algorithm::RS256;
const DEFAULT_TTL_SECS: i64 = 60 * 60; // 1h
const REFRESH_TTL_SECS: i64 = 60 * 60 * 24 * 7; // 7d

#[derive(Debug, Error)]
pub enum JwtError {
    #[error("no active signing key — bootstrap a key first")]
    NoActiveKey,
    #[error("jwt encode failed: {0}")]
    Encode(#[from] jsonwebtoken::errors::Error),
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("unknown kid: {0}")]
    UnknownKid(String),
    #[error("missing kid in jwt header")]
    MissingKid,
    #[error("invalid PEM in stored key: {0}")]
    InvalidPem(String),
}

/// Claims included in every issued JWT.
///
/// FR-AUTH-101 §1 #8 added two new claims (`roles`, `rbac_v`). The verifier
/// treats absent `rbac_v` as implicit `rbac_v = 1` for the 30-day grace
/// window after FR-AUTH-101 ships (DEC-125).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Issuer — the auth service URL.
    pub iss: String,
    /// Subject — the SubjectId (UUID string).
    pub sub: String,
    /// Audience — `["cyberos"]` plus per-route audiences.
    pub aud: Vec<String>,
    /// Expiry, seconds since epoch.
    pub exp: i64,
    /// Issued-at, seconds since epoch.
    pub iat: i64,
    /// Not-before, seconds since epoch.
    pub nbf: i64,
    /// JWT ID — random UUID for replay detection.
    pub jti: String,
    /// CyberOS extensions — tenant id, persona, scopes, traceparent.
    pub tenant_id: String,
    /// FR-AUTH-004 §1 #2 — subject's primary email. Empty string when the
    /// subject has no email (agent / system subjects). `#[serde(default)]`
    /// so legacy tokens issued before this field landed still verify.
    #[serde(default)]
    pub email: String,
    /// Subject kind (`human` | `agent` | `system`).
    pub kind: String,
    /// Granted scopes (e.g. `["admin:tenants", "admin:subjects"]`).
    pub scope_grants: Vec<String>,
    /// FR-AUTH-101 §1 #8 — array of role names (kebab-case) the subject
    /// holds at issuance time. Empty array for subjects with no roles.
    #[serde(default)]
    pub roles: Vec<String>,
    /// FR-AUTH-101 §1 #8 — catalogue version embedded for replay-resistance.
    /// Absent claim → implicit 1 (grace window). Verifiers can challenge
    /// tokens issued > 2 versions behind the live `RoleMatrix.version`.
    #[serde(default)]
    pub rbac_v: Option<i32>,
    /// For agent JWTs — the persona version (e.g. `cuo@2.3.1`).
    pub agent_persona: Option<String>,
    /// W3C traceparent for OBS request stitching (AUTHORING §3.7 rule 22).
    pub traceparent: Option<String>,
}

/// Result of `issue` — both access and refresh token strings + metadata.
#[derive(Debug, Clone, Serialize)]
pub struct IssuedTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: &'static str,
    pub expires_in: i64,
    pub kid: String,
}

/// One JWKS entry (public-key half of a signing key).
#[derive(Debug, Clone, Serialize)]
pub struct JwkPublic {
    pub kid: String,
    pub alg: &'static str,
    pub kty: &'static str,
    pub r#use: &'static str,
    /// RSA modulus (base64url no-padding).
    pub n: String,
    /// RSA public exponent (base64url no-padding).
    pub e: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct JwksDocument {
    pub keys: Vec<JwkPublic>,
}

/// Service surface — backed by Postgres for key storage.
pub struct JwtService {
    pub pool: PgPool,
    pub issuer: String,
}

impl JwtService {
    pub fn new(pool: PgPool, issuer: impl Into<String>) -> Self {
        Self {
            pool,
            issuer: issuer.into(),
        }
    }

    /// Mint an access + refresh token pair.
    ///
    /// `roles` + `rbac_v` are FR-AUTH-101 additions. Callers SHOULD pass the
    /// subject's current role membership + the live `RoleMatrix.version`.
    /// Empty `roles` and `rbac_v = None` produce a stub-era token compatible
    /// with verifiers running under the grace window.
    ///
    /// `email` is the FR-AUTH-004 §1 #2 addition — pass the subject's
    /// primary email or empty string when not applicable.
    #[allow(clippy::too_many_arguments)]
    pub async fn issue(
        &self,
        tenant: TenantId,
        subject: SubjectId,
        email: &str,
        kind: &str,
        scopes: Vec<String>,
        roles: Vec<String>,
        rbac_v: Option<i32>,
        agent_persona: Option<String>,
        traceparent: Option<String>,
    ) -> Result<IssuedTokens, JwtError> {
        let active = self.load_active_key().await?;
        let now = Utc::now();
        let access = self.mint(
            &active,
            tenant,
            subject,
            email,
            kind,
            scopes.clone(),
            roles.clone(),
            rbac_v,
            agent_persona.clone(),
            traceparent.clone(),
            now,
            DEFAULT_TTL_SECS,
            "access",
        )?;
        // Refresh token: longer ttl, narrower aud. roles + rbac_v carry through
        // so refresh-time can re-validate the catalogue version.
        let refresh = self.mint(
            &active,
            tenant,
            subject,
            email,
            kind,
            vec!["refresh".to_string()],
            roles,
            rbac_v,
            agent_persona,
            traceparent,
            now,
            REFRESH_TTL_SECS,
            "refresh",
        )?;
        Ok(IssuedTokens {
            access_token: access,
            refresh_token: refresh,
            token_type: "Bearer",
            expires_in: DEFAULT_TTL_SECS,
            kid: active.kid,
        })
    }

    /// Verify a presented JWT and return the claims.
    pub async fn verify(&self, token: &str) -> Result<Claims, JwtError> {
        let header = decode_header(token).map_err(JwtError::Encode)?;
        let kid = header.kid.ok_or(JwtError::MissingKid)?;
        let key = self.load_key_by_kid(&kid).await?;
        let mut v = Validation::new(ALG);
        v.set_issuer(&[&self.issuer]);
        v.set_audience(&["cyberos"]);
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_rsa_pem(key.public_pem.as_bytes())?,
            &v,
        )
        .map_err(JwtError::Encode)?;
        Ok(data.claims)
    }

    /// Build the JWKS payload from currently-published keys.
    pub async fn jwks_for_publication(&self) -> Result<JwksDocument, JwtError> {
        let keys = sqlx::query_as::<_, (String, String)>(
            "SELECT kid, public_pem
                 FROM auth_signing_keys
                WHERE (status = 'active' OR retired_at > NOW() - INTERVAL '7 days')
                  AND expires_at > NOW()",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut out = Vec::with_capacity(keys.len());
        for (kid, pem) in keys {
            if let Ok(jwk) = rsa_pem_to_jwk(&kid, &pem) {
                out.push(jwk);
            } else {
                tracing::warn!(%kid, "failed to convert PEM to JWK — skipping");
            }
        }
        Ok(JwksDocument { keys: out })
    }

    /// Inserts a freshly-generated signing key. Used by `cyberos-auth bootstrap`.
    pub async fn insert_key(
        &self,
        kid: &str,
        public_pem: &str,
        private_pem: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<(), JwtError> {
        sqlx::query(
            "INSERT INTO auth_signing_keys (kid, algorithm, public_pem, private_pem, status, expires_at)
             VALUES ($1, 'RS256', $2, $3, 'active', $4)
             ON CONFLICT (kid) DO NOTHING",
        )
        .bind(kid)
        .bind(public_pem)
        .bind(private_pem)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn load_active_key(&self) -> Result<SigningKeyRow, JwtError> {
        let row = sqlx::query_as::<_, (String, String, String)>(
            "SELECT kid, public_pem, private_pem
                 FROM auth_signing_keys
                WHERE status = 'active' AND expires_at > NOW()
             ORDER BY activated_at DESC
                LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some((kid, public_pem, private_pem)) => Ok(SigningKeyRow { kid, public_pem, private_pem }),
            None => Err(JwtError::NoActiveKey),
        }
    }

    async fn load_key_by_kid(&self, kid: &str) -> Result<SigningKeyRow, JwtError> {
        let row = sqlx::query_as::<_, (String, String, String)>(
            "SELECT kid, public_pem, private_pem
                 FROM auth_signing_keys
                WHERE kid = $1",
        )
        .bind(kid)
        .fetch_optional(&self.pool)
        .await?;
        match row {
            Some((kid, public_pem, private_pem)) => Ok(SigningKeyRow { kid, public_pem, private_pem }),
            None => Err(JwtError::UnknownKid(kid.to_string())),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn mint(
        &self,
        key: &SigningKeyRow,
        tenant: TenantId,
        subject: SubjectId,
        email: &str,
        kind: &str,
        scope_grants: Vec<String>,
        roles: Vec<String>,
        rbac_v: Option<i32>,
        agent_persona: Option<String>,
        traceparent: Option<String>,
        now: DateTime<Utc>,
        ttl_secs: i64,
        purpose: &str,
    ) -> Result<String, JwtError> {
        let mut hdr = Header::new(ALG);
        hdr.kid = Some(key.kid.clone());
        let aud = vec!["cyberos".to_string(), purpose.to_string()];
        let claims = Claims {
            iss: self.issuer.clone(),
            sub: subject.to_string(),
            aud,
            exp: (now + Duration::seconds(ttl_secs)).timestamp(),
            iat: now.timestamp(),
            nbf: now.timestamp(),
            jti: Uuid::new_v4().to_string(),
            tenant_id: tenant.to_string(),
            email: email.to_string(),
            kind: kind.to_string(),
            scope_grants,
            roles,
            rbac_v,
            agent_persona,
            traceparent,
        };
        Ok(encode(
            &hdr,
            &claims,
            &EncodingKey::from_rsa_pem(key.private_pem.as_bytes())?,
        )?)
    }
}

struct SigningKeyRow {
    kid: String,
    public_pem: String,
    private_pem: String,
}

/// Convert an RSA public-key PEM to the JWK `{n, e}` form.
/// Minimal implementation — sufficient for `/.well-known/jwks.json` consumers.
fn rsa_pem_to_jwk(kid: &str, pem: &str) -> Result<JwkPublic, JwtError> {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    // We can use jsonwebtoken's internal RSA parsing via DecodingKey — but to
    // emit JWK we need raw modulus + exponent. Parse the PEM with `pem` crate
    // helpers from rsa-spki via jsonwebtoken is not directly exposed, so we
    // call out to a tiny ASN.1 reader below.
    let pem_bytes = pem.as_bytes();
    let der = parse_pem_block(pem_bytes, "PUBLIC KEY")
        .or_else(|| parse_pem_block(pem_bytes, "RSA PUBLIC KEY"))
        .ok_or_else(|| JwtError::InvalidPem("no -----BEGIN PUBLIC KEY----- block found".into()))?;
    let (n, e) = extract_rsa_n_e_from_spki(&der).ok_or_else(||
        JwtError::InvalidPem("could not parse RSA n/e from SPKI".into()))?;
    Ok(JwkPublic {
        kid: kid.to_string(),
        alg: "RS256",
        kty: "RSA",
        r#use: "sig",
        n: URL_SAFE_NO_PAD.encode(&n),
        e: URL_SAFE_NO_PAD.encode(&e),
    })
}

fn parse_pem_block(pem: &[u8], label: &str) -> Option<Vec<u8>> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let begin = format!("-----BEGIN {label}-----");
    let end = format!("-----END {label}-----");
    let s = std::str::from_utf8(pem).ok()?;
    let i = s.find(&begin)? + begin.len();
    let j = s.find(&end)?;
    let body: String = s[i..j].chars().filter(|c| !c.is_whitespace()).collect();
    STANDARD.decode(body.as_bytes()).ok()
}

/// Extract (n, e) bytes from a SubjectPublicKeyInfo DER blob.
/// SPKI structure:
///   SEQUENCE {
///     SEQUENCE { OID rsaEncryption, NULL }
///     BIT STRING {
///       SEQUENCE { INTEGER n, INTEGER e }
///     }
///   }
/// This is a minimal hand-rolled ASN.1 reader.
fn extract_rsa_n_e_from_spki(der: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut r = AsnReader::new(der);
    r.read_sequence()?;                // outer SEQ
    let _algid = r.read_sequence()?;   // algorithm id
    let bitstr = r.read_bit_string()?;
    let mut inner = AsnReader::new(bitstr);
    inner.read_sequence()?;
    let n = inner.read_integer_unsigned()?;
    let e = inner.read_integer_unsigned()?;
    Some((n, e))
}

struct AsnReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> AsnReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn read_tag(&mut self, tag: u8) -> Option<&'a [u8]> {
        if self.pos >= self.buf.len() || self.buf[self.pos] != tag {
            return None;
        }
        self.pos += 1;
        let len = self.read_len()?;
        if self.pos + len > self.buf.len() {
            return None;
        }
        let bytes = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Some(bytes)
    }
    fn read_len(&mut self) -> Option<usize> {
        let first = *self.buf.get(self.pos)?;
        self.pos += 1;
        if first & 0x80 == 0 {
            return Some(first as usize);
        }
        let nbytes = (first & 0x7f) as usize;
        let mut len = 0usize;
        for _ in 0..nbytes {
            len = (len << 8) | *self.buf.get(self.pos)? as usize;
            self.pos += 1;
        }
        Some(len)
    }
    fn read_sequence(&mut self) -> Option<&'a [u8]> {
        self.read_tag(0x30)
    }
    fn read_bit_string(&mut self) -> Option<&'a [u8]> {
        let body = self.read_tag(0x03)?;
        // First byte is "unused-bits"; strip it.
        body.split_first().map(|(_, rest)| rest)
    }
    fn read_integer_unsigned(&mut self) -> Option<Vec<u8>> {
        let body = self.read_tag(0x02)?;
        // strip leading 0x00 sign byte if present
        let stripped = if body.first().copied() == Some(0) { &body[1..] } else { body };
        Some(stripped.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Smoke-test the PEM parser on a known 2048-bit RSA SPKI.
    const TEST_PUBLIC_PEM: &str = "-----BEGIN PUBLIC KEY-----\n\
        MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA8K76mYDXMxhM5RkLh+aB\n\
        Wj7P4qFqXxXXxJlRq2EVxKxA7y4yC2bP3sBT9d5z2j+kQ+IcW8N8a7xV0bC9PtcF\n\
        zSdLqGXp3hH4P8Be0sZxRY8MspKjMNeoYUFu5/3aHEjqAukI+jpYjVdShO2bWlpw\n\
        EQa+8m7lEK7QmgEnTpfL2pUsZRYV5q7HspbHcwR0WkLZxNwSp7m5J9YfL3oZ+yZP\n\
        SQCQjnGq2EnNvxRzkmRWqz2WoxYRBKWfg2Lhd0aFvFRTk4LU98QbpA17AfgPZeOu\n\
        QvkXZK7s5o3hwQU3SUMXh8KsfFvR4gnzfBKjr7bKJl4QQH8xJZjJyf1f8WPS/8WL\n\
        7QIDAQAB\n\
        -----END PUBLIC KEY-----\n";

    #[test]
    fn pem_block_extracts() {
        let der = parse_pem_block(TEST_PUBLIC_PEM.as_bytes(), "PUBLIC KEY");
        assert!(der.is_some(), "PEM block decode failed");
    }
}
