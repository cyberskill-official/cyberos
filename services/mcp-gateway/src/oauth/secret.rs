//! Opaque-token generation and hashing for FR-MCP-004. Refresh tokens are opaque 256-bit strings
//! (clause #8), stored only as their SHA-256 hash; never the token itself.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// A 256-bit opaque token as base64url-no-pad (43 chars). Two v4 UUIDs supply the 32 random bytes.
pub fn opaque_token_256() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(bytes)
}

/// SHA-256 of `s` as lowercase hex (64 chars) - the stored refresh-token hash and the memory-chain
/// hash placeholder until the real chain wiring lands.
pub fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opaque_tokens_are_43_chars_and_distinct() {
        let a = opaque_token_256();
        let b = opaque_token_256();
        assert_eq!(a.len(), 43);
        assert_ne!(a, b);
    }

    #[test]
    fn sha256_hex_is_64_lowercase_hex() {
        let h = sha256_hex("hello");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }
}
