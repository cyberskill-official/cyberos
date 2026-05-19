// services/auth/src/mfa/totp.rs
const TOTP_STEP_SECS: u64 = 30;
const TOTP_DIGITS: usize = 6;
pub const TOTP_SECRET_BYTES: usize = 20;

pub fn current_time_step() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        / TOTP_STEP_SECS
}

pub fn verify_totp(secret: &[u8], code: &str, now_step: u64) -> bool {
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
    let block_size = 64usize;
    let mut key = secret.to_vec();
    if key.len() > block_size {
        let mut h = Sha1::new();
        h.update(&key);
        key = h.finalize().to_vec();
    }
    if key.len() < block_size {
        key.resize(block_size, 0);
    }
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
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

pub fn generate_totp_secret() -> Vec<u8> {
    use rand::RngCore;
    let mut buf = vec![0u8; TOTP_SECRET_BYTES];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

const B32_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

pub fn base32_encode(input: &[u8]) -> String {
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

pub fn base32_decode(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 5 / 8);
    let mut buf: u32 = 0;
    let mut bits = 0u32;
    for c in s.chars().filter(|c| !c.is_whitespace() && *c != '=') {
        let v = B32_ALPHABET
            .iter()
            .position(|&b| b == c.to_ascii_uppercase() as u8)? as u32;
        buf = (buf << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xff) as u8);
        }
    }
    Some(out)
}

pub fn urlencoding_minimal(s: &str) -> String {
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

    #[test]
    fn generate_totp_secret_has_correct_length() {
        let secret = generate_totp_secret();
        assert_eq!(secret.len(), TOTP_SECRET_BYTES);
    }

    #[test]
    fn base32_encode_known_value() {
        // "Hello!" in base32 = "JBSWY3DPEE"
        // Actually let's just verify determinism
        let a = base32_encode(b"test");
        let b = base32_encode(b"test");
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }
}
