//! FR-AUTH-102 — Multi-Factor Authentication (TOTP slice).
//!
//! Implements RFC 6238 Time-Based One-Time Password with HMAC-SHA1, 6-digit
//! codes, 30-second step. WebAuthn enrolment + recovery codes ship in slice 2.
//!
//! Enrolment FSM:
//!   pending (secret generated; QR shown) → active (one valid code submitted)
//!                                       → revoked (subject or admin revokes)
//!
//! Verify path: `POST /v1/auth/mfa/verify { factor_id, code }` — called by
//! the password-grant flow when the subject has at least one active TOTP
//! factor enrolled. Returns 204 on accept, 401 on reject.

use axum::{
    extract::{Json as JsonInput, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::Claims;
use crate::AppState;

const TOTP_STEP_SECS: u64 = 30;
const TOTP_DIGITS: usize = 6;
const TOTP_SECRET_BYTES: usize = 20; // 160-bit per RFC 6238 §5.1 SHA-1

#[derive(Debug, Serialize)]
pub struct EnrolStartResponse {
    pub factor_id: Uuid,
    pub secret_base32: String,
    pub otpauth_url: String,
    pub issuer: String,
    pub label: String,
}

/// `POST /v1/auth/mfa/factors/totp/enrol` — generates a fresh secret + persists
/// a pending factor row + returns the otpauth:// URI for QR rendering.
pub async fn totp_enrol_start(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<EnrolStartBody>,
) -> Result<(StatusCode, Json<EnrolStartResponse>), (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal_err)?;
    let subject_id = Uuid::parse_str(&claims.sub).map_err(internal_err)?;

    let secret = generate_totp_secret();
    let secret_b32 = base32_encode(&secret);
    let label = body
        .label
        .as_deref()
        .unwrap_or("CyberOS authenticator")
        .to_string();

    let issuer = "CyberOS".to_string();
    let otpauth = format!(
        "otpauth://totp/{issuer}:{account}?secret={secret}&issuer={issuer}&algorithm=SHA1&digits=6&period=30",
        issuer = issuer,
        account = urlencoding_minimal(&claims.sub),
        secret = secret_b32,
    );

    // Persist pending row (RLS via tenant_id GUC).
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await.map_err(internal_err)?;

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO mfa_factors (tenant_id, subject_id, factor_type, label, totp_secret, status)
              VALUES ($1, $2, 'totp', $3, $4, 'pending')
            RETURNING id",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(&label)
    .bind(&secret_b32)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    Ok((StatusCode::CREATED, Json(EnrolStartResponse {
        factor_id: row.0,
        secret_base32: secret_b32,
        otpauth_url: otpauth,
        issuer,
        label,
    })))
}

#[derive(Debug, Deserialize)]
pub struct EnrolStartBody {
    pub label: Option<String>,
}

/// `POST /v1/auth/mfa/factors/totp/enrol/finish` — verifies the user can read
/// their authenticator app correctly (proves enrolment success) + activates the
/// pending factor row.
pub async fn totp_enrol_finish(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<VerifyBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal_err)?;
    let subject_id = Uuid::parse_str(&claims.sub).map_err(internal_err)?;

    let factor = load_factor(&state, tenant_id, subject_id, body.factor_id, "pending").await?;
    let secret = base32_decode(&factor.totp_secret_b32).ok_or_else(|| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "stored secret is not valid base32"})),
    ))?;
    if !verify_totp(&secret, &body.code, current_time_step()) {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "totp code did not verify"}))));
    }

    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await.map_err(internal_err)?;
    sqlx::query(
        "UPDATE mfa_factors
            SET status = 'active', activated_at = NOW(), last_used_at = NOW()
          WHERE id = $1 AND subject_id = $2 AND status = 'pending'",
    )
    .bind(body.factor_id)
    .bind(subject_id)
    .execute(&mut *tx).await.map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    Ok(StatusCode::NO_CONTENT)
}

/// `POST /v1/auth/mfa/verify` — runtime verification used during the password
/// grant when the subject has at least one active TOTP factor.
pub async fn totp_verify(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    JsonInput(body): JsonInput<VerifyBody>,
) -> Result<StatusCode, (StatusCode, Json<Value>)> {
    let tenant_id = Uuid::parse_str(&claims.tenant_id).map_err(internal_err)?;
    let subject_id = Uuid::parse_str(&claims.sub).map_err(internal_err)?;

    let factor = load_factor(&state, tenant_id, subject_id, body.factor_id, "active").await?;
    let secret = base32_decode(&factor.totp_secret_b32).ok_or_else(|| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "stored secret is not valid base32"})),
    ))?;
    if !verify_totp(&secret, &body.code, current_time_step()) {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({"error": "totp code did not verify"}))));
    }

    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await.map_err(internal_err)?;
    sqlx::query("UPDATE mfa_factors SET last_used_at = NOW() WHERE id = $1")
        .bind(body.factor_id)
        .execute(&mut *tx).await.map_err(internal_err)?;
    tx.commit().await.map_err(internal_err)?;

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct VerifyBody {
    pub factor_id: Uuid,
    pub code: String,
}

struct LoadedFactor {
    totp_secret_b32: String,
}

async fn load_factor(
    state: &AppState,
    tenant_id: Uuid,
    subject_id: Uuid,
    factor_id: Uuid,
    required_status: &str,
) -> Result<LoadedFactor, (StatusCode, Json<Value>)> {
    let mut tx = state.pg.begin().await.map_err(internal_err)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx).await.map_err(internal_err)?;
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT totp_secret FROM mfa_factors
          WHERE id = $1 AND subject_id = $2 AND factor_type = 'totp' AND status = $3",
    )
    .bind(factor_id)
    .bind(subject_id)
    .bind(required_status)
    .fetch_optional(&mut *tx).await.map_err(internal_err)?;
    let _ = tx.commit().await;
    let (secret,) = row.ok_or_else(|| (
        StatusCode::NOT_FOUND,
        Json(json!({"error": format!("no {required_status} totp factor for caller")})),
    ))?;
    let totp_secret_b32 = secret.ok_or_else(|| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": "totp factor missing secret"})),
    ))?;
    Ok(LoadedFactor { totp_secret_b32 })
}

// ---------------------------------------------------------------------------
// RFC 6238 TOTP implementation — HMAC-SHA1, 30s step, 6 digits.
// ---------------------------------------------------------------------------

fn current_time_step() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs / TOTP_STEP_SECS
}

fn verify_totp(secret: &[u8], code: &str, now_step: u64) -> bool {
    if code.len() != TOTP_DIGITS || !code.chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    // Accept ±1 step to tolerate small clock drift.
    for step in [now_step.saturating_sub(1), now_step, now_step + 1] {
        if constant_time_eq(&hotp(secret, step), code.as_bytes()) {
            return true;
        }
    }
    false
}

fn hotp(secret: &[u8], counter: u64) -> Vec<u8> {
    use sha1::{Digest, Sha1};
    // HMAC-SHA1 inline (avoids pulling another crate).
    let block_size = 64usize;
    let mut key = secret.to_vec();
    if key.len() > block_size {
        let mut h = Sha1::new();
        h.update(&key);
        key = h.finalize().to_vec();
    }
    if key.len() < block_size { key.resize(block_size, 0); }
    let ipad: Vec<u8> = key.iter().map(|b| b ^ 0x36).collect();
    let opad: Vec<u8> = key.iter().map(|b| b ^ 0x5c).collect();

    let mut inner = Sha1::new();
    inner.update(&ipad);
    inner.update(counter.to_be_bytes());
    let inner_hash = inner.finalize();

    let mut outer = Sha1::new();
    outer.update(&opad);
    outer.update(inner_hash);
    let mac = outer.finalize();

    // Dynamic truncation (RFC 4226 §5.3)
    let offset = (mac[19] & 0xf) as usize;
    let bin_code = ((mac[offset] as u32 & 0x7f) << 24)
        | ((mac[offset + 1] as u32) << 16)
        | ((mac[offset + 2] as u32) << 8)
        | (mac[offset + 3] as u32);
    let modulus = 10u32.pow(TOTP_DIGITS as u32);
    let truncated = bin_code % modulus;
    format!("{:0>width$}", truncated, width = TOTP_DIGITS).into_bytes()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() { return false; }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) { diff |= x ^ y; }
    diff == 0
}

fn generate_totp_secret() -> Vec<u8> {
    use rand::RngCore;
    let mut buf = vec![0u8; TOTP_SECRET_BYTES];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

// ---------------------------------------------------------------------------
// Minimal base32 (RFC 4648 §6) for the otpauth URI. No padding (per most
// authenticator apps' convention).
// ---------------------------------------------------------------------------

const B32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn base32_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len() * 8 / 5 + 1);
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for &b in input {
        buf = (buf << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            let idx = ((buf >> bits) & 0x1f) as usize;
            out.push(B32_ALPHABET[idx] as char);
        }
    }
    if bits > 0 {
        let idx = ((buf << (5 - bits)) & 0x1f) as usize;
        out.push(B32_ALPHABET[idx] as char);
    }
    out
}

fn base32_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 5 / 8);
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for c in s.chars().filter(|c| !c.is_whitespace() && *c != '=') {
        let v = B32_ALPHABET.iter().position(|&b| b == c.to_ascii_uppercase() as u8)? as u32;
        buf = (buf << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

fn urlencoding_minimal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~') {
            out.push(c);
        } else {
            for b in c.to_string().as_bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        }
    }
    out
}

fn internal_err<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": e.to_string()})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base32_round_trip() {
        for input in [&b"hello"[..], &[0u8; 20][..], &[0xff_u8; 13][..]] {
            let enc = base32_encode(input);
            let dec = base32_decode(&enc).expect("decode");
            assert_eq!(dec, input, "round-trip failed for {input:?}");
        }
    }

    #[test]
    fn hotp_matches_rfc4226_test_vectors() {
        // RFC 4226 Appendix D — secret = "12345678901234567890" ASCII
        let secret = b"12345678901234567890";
        // Counter 0 → 755224 / 1 → 287082 / 2 → 359152
        assert_eq!(String::from_utf8(hotp(secret, 0)).unwrap(), "755224");
        assert_eq!(String::from_utf8(hotp(secret, 1)).unwrap(), "287082");
        assert_eq!(String::from_utf8(hotp(secret, 2)).unwrap(), "359152");
    }

    #[test]
    fn verify_accepts_code_within_drift_window() {
        let secret = b"12345678901234567890";
        let now = 5u64;
        let valid_code = String::from_utf8(hotp(secret, now)).unwrap();
        assert!(verify_totp(secret, &valid_code, now));
        // Within ±1 step
        let prev = String::from_utf8(hotp(secret, now - 1)).unwrap();
        let next = String::from_utf8(hotp(secret, now + 1)).unwrap();
        assert!(verify_totp(secret, &prev, now));
        assert!(verify_totp(secret, &next, now));
        // Outside drift window
        let far = String::from_utf8(hotp(secret, now + 10)).unwrap();
        assert!(!verify_totp(secret, &far, now));
    }

    #[test]
    fn verify_rejects_wrong_length() {
        assert!(!verify_totp(b"secret", "12345", 1));
        assert!(!verify_totp(b"secret", "1234567", 1));
    }

    #[test]
    fn verify_rejects_non_digit_code() {
        assert!(!verify_totp(b"secret", "12345a", 1));
    }
}
