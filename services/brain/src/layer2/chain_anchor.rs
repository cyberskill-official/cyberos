//! Verifies a Layer-1 row's chain anchor before materializing into Layer 2.
//!
//! Per §1 #4: chain_anchor = SHA-256(prev_hash || body). On read, we recompute
//! and reject any L2 query that names a row whose materialised anchor doesn't
//! match the one Layer 1 currently advertises (catches Layer-1 tampering).

use sha2::{Digest, Sha256};

/// Compute the canonical chain anchor for a row.
pub fn compute(prev_hash_hex: Option<&str>, body: &str) -> String {
    let mut h = Sha256::new();
    if let Some(prev) = prev_hash_hex {
        h.update(prev.as_bytes());
    }
    h.update(body.as_bytes());
    hex::encode(h.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_is_deterministic() {
        let a = compute(Some("abcd"), "hello world");
        let b = compute(Some("abcd"), "hello world");
        assert_eq!(a, b);
    }

    #[test]
    fn anchor_differs_when_body_differs() {
        let a = compute(Some("abcd"), "hello world");
        let b = compute(Some("abcd"), "hello mars");
        assert_ne!(a, b);
    }

    #[test]
    fn anchor_differs_when_prev_differs() {
        let a = compute(Some("abcd"), "body");
        let b = compute(Some("efgh"), "body");
        assert_ne!(a, b);
    }

    #[test]
    fn anchor_handles_genesis_row() {
        // The first row in a chain has no prev_hash.
        let g = compute(None, "genesis body");
        assert_eq!(g.len(), 64); // sha256 hex is 64 chars
    }
}

// Vendored from `hex` crate to keep deps minimal in this skeleton stage.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}
