//! FR-AUTH-110 §1 #7 - PKCE S256 verification (RFC 7636).
//!
//! The authorize endpoint stores the RP's `code_challenge`; the token endpoint
//! presents a `code_verifier`. We recompute the S256 challenge from the verifier
//! and compare it to the stored challenge in constant time. Only S256 is supported
//! (DEC-2484); the `plain` method is not accepted.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

/// The S256 code challenge for a verifier: `base64url-nopad(SHA256(verifier))`.
pub fn s256_challenge(code_verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
}

/// Constant-time check that `code_verifier` satisfies `code_challenge` under S256.
pub fn verify_s256(code_verifier: &str, code_challenge: &str) -> bool {
    constant_time_eq(
        s256_challenge(code_verifier).as_bytes(),
        code_challenge.as_bytes(),
    )
}

/// Length-checked constant-time byte comparison (no early return on first diff).
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    // RFC 7636 Appendix B test vector.
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    #[test]
    fn rfc7636_vector_challenge_matches() {
        assert_eq!(s256_challenge(VERIFIER), CHALLENGE);
    }

    #[test]
    fn verify_accepts_matching_verifier() {
        assert!(verify_s256(VERIFIER, CHALLENGE));
    }

    #[test]
    fn verify_rejects_wrong_verifier() {
        assert!(!verify_s256("not-the-verifier", CHALLENGE));
        assert!(!verify_s256(VERIFIER, "not-the-challenge"));
    }

    #[test]
    fn constant_time_eq_handles_length_mismatch() {
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(constant_time_eq(b"abc", b"abc"));
    }
}
