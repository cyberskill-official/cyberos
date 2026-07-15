//! TASK-MCP-004 §1.2, §1.3, §3.4 — PKCE S256 verifier
//!
//! DEC-801: S256 only; plain is rejected at the authorize endpoint before this
//! function is ever called. This module verifies at the token endpoint.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use constant_time_eq::constant_time_eq;
use sha2::{Digest, Sha256};

/// Verify a PKCE code_verifier against a stored S256 code_challenge.
///
/// Returns `true` iff SHA-256(code_verifier) base64url-no-pad == stored_challenge,
/// using constant-time comparison to prevent timing-channel leakage (TASK-MCP-004 §11.21).
///
/// Returns `false` (without panicking) when:
/// - verifier length outside [43, 128] (RFC 7636 §4.1)
/// - SHA-256 result does not match stored challenge
pub fn verify_pkce(code_verifier: &str, stored_code_challenge: &str) -> bool {
    if !(43..=128).contains(&code_verifier.len()) {
        return false;
    }
    let computed = URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()));
    constant_time_eq(computed.as_bytes(), stored_code_challenge.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_challenge(verifier: &str) -> String {
        URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
    }

    #[test]
    fn correct_verifier_passes() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"; // 43 chars
        let challenge = make_challenge(verifier);
        assert!(verify_pkce(verifier, &challenge));
    }

    #[test]
    fn wrong_verifier_fails() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = make_challenge(verifier);
        let wrong = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        assert!(!verify_pkce(wrong, &challenge));
    }

    #[test]
    fn short_verifier_fails() {
        assert!(!verify_pkce("tooshort", "anything"));
    }

    #[test]
    fn long_verifier_fails() {
        let v = "a".repeat(129);
        assert!(!verify_pkce(&v, "anything"));
    }

    #[test]
    fn boundary_43_chars_passes() {
        let verifier = "a".repeat(43);
        let challenge = make_challenge(&verifier);
        assert!(verify_pkce(&verifier, &challenge));
    }

    #[test]
    fn boundary_128_chars_passes() {
        let verifier = "a".repeat(128);
        let challenge = make_challenge(&verifier);
        assert!(verify_pkce(&verifier, &challenge));
    }
}
