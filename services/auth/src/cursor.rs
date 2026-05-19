//! FR-AUTH-005 §1 #5 + #9 + G-005/G-009 — HMAC-SHA256-signed pagination cursors.
//!
//! Cursor wire format (decoded from URL-safe base64, no padding):
//!
//! ```text
//!   byte 0       : table tag (0x01 = tenants, 0x02 = subjects)
//!   bytes 1..17  : last-seen id (16-byte UUID)
//!   bytes 17..25 : HMAC-SHA256 prefix (8 bytes) of (table_tag || uuid)
//! ```
//!
//! Total 25 bytes → 34 URL-safe-base64 chars (no padding). The 8-byte HMAC
//! truncation gives 2^64 brute-force resistance which is comfortably above
//! the bar for what cursor-tampering attacks try to do (the goal of signing
//! is to prevent operators from crafting cursors pointing at arbitrary
//! database rows, not to resist nation-state forgery).
//!
//! **Why the table tag?** A cursor minted for `/v1/admin/tenants` MUST NOT
//! be redeemable against `/v1/admin/subjects` — even within the same tenant.
//! Without the tag, an attacker who knows a valid tenant-cursor could
//! redeem it on the subjects endpoint to bypass cursor signing on a
//! different scan. The tag binds the cursor to its table.
//!
//! **Key derivation.** The signing key reads from env var
//! `AUTH_CURSOR_SIGNING_SECRET` (64 hex chars = 32 bytes). In dev/test (env
//! unset) the key falls back to `SHA256("cyberos-cursor-dev-key-only")` with
//! a startup warning so cursors remain functional during local development.
//! In production the operator MUST set the env var alongside the JWT
//! signing key (FR-AUTH-005 §1 #9: "same deployment secret as JWT signing,
//! separate scope: cursor signing key derived via HKDF").
//!
//! See also: FR-AUTH-001 §1 #11 for the structured `{error, field, reason}`
//! response body shape used by [`ParseCursorError::into_response`].

use std::sync::OnceLock;

use axum::http::StatusCode;
use axum::response::Json;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Table tag — selects the namespace this cursor is valid for.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorTable {
    Tenants,
    Subjects,
}

impl CursorTable {
    const fn tag(self) -> u8 {
        match self {
            CursorTable::Tenants => 0x01,
            CursorTable::Subjects => 0x02,
        }
    }
}

/// Parse-time errors. Every variant maps to a `400 BAD_REQUEST` with the
/// structured `{error: "invalid_cursor", field, reason}` body per
/// FR-AUTH-005 §1 #9.
#[derive(Debug, Eq, PartialEq)]
pub enum ParseCursorError {
    Base64,
    Length,
    TableMismatch,
    Signature,
    Uuid,
}

impl ParseCursorError {
    pub fn into_response(self) -> (StatusCode, Json<Value>) {
        let reason = match self {
            ParseCursorError::Base64 => "cursor is not valid URL-safe base64",
            ParseCursorError::Length => "cursor decoded length is not 25 bytes",
            ParseCursorError::TableMismatch => "cursor was minted for a different endpoint",
            ParseCursorError::Signature => "cursor signature does not verify",
            ParseCursorError::Uuid => "cursor contains a malformed UUID payload",
        };
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_cursor",
                "field": "cursor",
                "reason": reason,
            })),
        )
    }
}

/// Mint a fresh cursor for `(table, id)`.
pub fn make_cursor(table: CursorTable, id: Uuid) -> String {
    let mut body = [0u8; 17];
    body[0] = table.tag();
    body[1..17].copy_from_slice(id.as_bytes());
    let mac = hmac_sha256_first8(&body);
    let mut wire = [0u8; 25];
    wire[..17].copy_from_slice(&body);
    wire[17..].copy_from_slice(&mac);
    URL_SAFE_NO_PAD.encode(wire)
}

/// Verify + decode a cursor. The `expected` table tag binds the cursor to
/// the endpoint that minted it.
pub fn parse_cursor(s: &str, expected: CursorTable) -> Result<Uuid, ParseCursorError> {
    let wire = URL_SAFE_NO_PAD
        .decode(s.as_bytes())
        .map_err(|_| ParseCursorError::Base64)?;
    if wire.len() != 25 {
        return Err(ParseCursorError::Length);
    }
    if wire[0] != expected.tag() {
        return Err(ParseCursorError::TableMismatch);
    }
    // Re-derive HMAC over the (tag || uuid) prefix; constant-time compare.
    let body = &wire[..17];
    let provided_mac = &wire[17..25];
    let expected_mac = hmac_sha256_first8(body);
    if !ct_eq(provided_mac, &expected_mac) {
        return Err(ParseCursorError::Signature);
    }
    let id = Uuid::from_slice(&wire[1..17]).map_err(|_| ParseCursorError::Uuid)?;
    Ok(id)
}

/// Cached 32-byte cursor signing key. Lazy-derived on first use.
fn cursor_key() -> &'static [u8; 32] {
    static KEY: OnceLock<[u8; 32]> = OnceLock::new();
    KEY.get_or_init(|| {
        if let Ok(hex) = std::env::var("AUTH_CURSOR_SIGNING_SECRET") {
            if let Ok(bytes) = decode_hex_32(&hex) {
                return bytes;
            }
            tracing::error!(
                "AUTH_CURSOR_SIGNING_SECRET set but not 64-hex; falling back to dev key"
            );
        } else {
            tracing::warn!(
                "AUTH_CURSOR_SIGNING_SECRET unset — using dev fallback. \
                 Set this env var alongside JWT signing in production."
            );
        }
        let mut hasher = Sha256::new();
        hasher.update(b"cyberos-cursor-dev-key-only");
        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    })
}

fn decode_hex_32(s: &str) -> Result<[u8; 32], ()> {
    if s.len() != 64 {
        return Err(());
    }
    let mut out = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let lo = hex_digit(chunk[0])?;
        let hi = hex_digit(chunk[1])?;
        out[i] = (lo << 4) | hi;
    }
    Ok(out)
}

fn hex_digit(b: u8) -> Result<u8, ()> {
    match b {
        b'0'..=b'9' => Ok(b - b'0'),
        b'a'..=b'f' => Ok(b - b'a' + 10),
        b'A'..=b'F' => Ok(b - b'A' + 10),
        _ => Err(()),
    }
}

/// HMAC-SHA256 first 8 bytes — minimal hand-rolled HMAC to avoid pulling
/// in an extra crate when we already depend on `sha2`. Block size = 64.
fn hmac_sha256_first8(msg: &[u8]) -> [u8; 8] {
    const BLOCK: usize = 64;
    let key = cursor_key();
    // Key is already 32 bytes ≤ BLOCK; pad with zeros.
    let mut key_block = [0u8; BLOCK];
    key_block[..32].copy_from_slice(key);

    let mut ipad = [0x36u8; BLOCK];
    let mut opad = [0x5cu8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] ^= key_block[i];
        opad[i] ^= key_block[i];
    }

    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(msg);
    let inner_digest = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    let outer_digest = outer.finalize();

    let mut out = [0u8; 8];
    out.copy_from_slice(&outer_digest[..8]);
    out
}

/// Constant-time byte comparison (subtle::ConstantTimeEq equivalent).
fn ct_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn an_id() -> Uuid {
        Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap()
    }

    #[test]
    fn roundtrip_tenants() {
        let id = an_id();
        let c = make_cursor(CursorTable::Tenants, id);
        let got = parse_cursor(&c, CursorTable::Tenants).unwrap();
        assert_eq!(got, id);
    }

    #[test]
    fn roundtrip_subjects() {
        let id = an_id();
        let c = make_cursor(CursorTable::Subjects, id);
        let got = parse_cursor(&c, CursorTable::Subjects).unwrap();
        assert_eq!(got, id);
    }

    #[test]
    fn table_mismatch_rejected() {
        let c = make_cursor(CursorTable::Tenants, an_id());
        let err = parse_cursor(&c, CursorTable::Subjects).unwrap_err();
        assert_eq!(err, ParseCursorError::TableMismatch);
    }

    #[test]
    fn tampered_signature_rejected() {
        // Flip a byte in the HMAC region (last bytes of the wire form).
        let c = make_cursor(CursorTable::Tenants, an_id());
        let mut wire = URL_SAFE_NO_PAD.decode(c.as_bytes()).unwrap();
        wire[24] ^= 0xff; // flip last byte (inside HMAC region)
        let tampered = URL_SAFE_NO_PAD.encode(&wire);
        let err = parse_cursor(&tampered, CursorTable::Tenants).unwrap_err();
        assert_eq!(err, ParseCursorError::Signature);
    }

    #[test]
    fn tampered_uuid_rejected() {
        // Flipping the UUID bytes invalidates the HMAC → signature failure
        // is the right error (not "uuid"), because parse_cursor verifies
        // before extracting the UUID.
        let c = make_cursor(CursorTable::Tenants, an_id());
        let mut wire = URL_SAFE_NO_PAD.decode(c.as_bytes()).unwrap();
        wire[5] ^= 0x01; // flip a UUID byte
        let tampered = URL_SAFE_NO_PAD.encode(&wire);
        let err = parse_cursor(&tampered, CursorTable::Tenants).unwrap_err();
        assert_eq!(err, ParseCursorError::Signature);
    }

    #[test]
    fn malformed_base64_rejected() {
        let err = parse_cursor("not!base64", CursorTable::Tenants).unwrap_err();
        assert_eq!(err, ParseCursorError::Base64);
    }

    #[test]
    fn short_cursor_rejected() {
        let err = parse_cursor("abc", CursorTable::Tenants).unwrap_err();
        // 'abc' decodes to 2 bytes — Length variant.
        assert_eq!(err, ParseCursorError::Length);
    }

    #[test]
    fn error_response_shape() {
        let (status, Json(body)) = ParseCursorError::Signature.into_response();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"], "invalid_cursor");
        assert_eq!(body["field"], "cursor");
        assert!(body["reason"].as_str().unwrap().contains("signature"));
    }

    #[test]
    fn hex_decode_round_trip() {
        let h = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let bytes = decode_hex_32(h).unwrap();
        assert_eq!(bytes[0], 0x01);
        assert_eq!(bytes[1], 0x23);
        assert_eq!(bytes[31], 0xef);
    }

    #[test]
    fn hex_decode_wrong_length_rejected() {
        assert!(decode_hex_32("abcd").is_err());
        assert!(decode_hex_32(&"ab".repeat(33)).is_err());
    }

    #[test]
    fn ct_eq_works() {
        assert!(ct_eq(b"abc", b"abc"));
        assert!(!ct_eq(b"abc", b"abd"));
        assert!(!ct_eq(b"abc", b"abcd"));
    }
}
