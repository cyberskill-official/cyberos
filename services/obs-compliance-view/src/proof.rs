//! Ed25519 chain-proof over a compliance view response (FR-OBS-008 §1 #5, DEC-176). The view signs the
//! canonical bytes of its response (the rows plus the summary) with the deployment signing key; an
//! auditor verifies the signature against the published public key, independently of CyberOS. Signing is
//! read-only - it never touches the audit chain (DEC-177).

use std::fmt::Write as _;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// The proof attached to a view response footer: the signature and the public key, both hex-encoded so
/// an auditor can verify with any Ed25519 tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub signature_hex: String,
    pub public_key_hex: String,
}

/// Sign `canonical` (the canonical JSON bytes of the response) with the 32-byte Ed25519 secret key.
pub fn sign(secret_key: &[u8; 32], canonical: &[u8]) -> Proof {
    let signing_key = SigningKey::from_bytes(secret_key);
    let signature: Signature = signing_key.sign(canonical);
    Proof {
        signature_hex: to_hex(&signature.to_bytes()),
        public_key_hex: to_hex(&signing_key.verifying_key().to_bytes()),
    }
}

/// Verify a proof over `canonical`, as an auditor would: parse the published public key and the
/// signature, then check them. Returns false on any malformed input or signature mismatch.
pub fn verify(public_key_hex: &str, canonical: &[u8], signature_hex: &str) -> bool {
    let (Some(pk_bytes), Some(sig_bytes)) = (
        from_hex::<32>(public_key_hex),
        from_hex::<64>(signature_hex),
    ) else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&pk_bytes) else {
        return false;
    };
    let signature = Signature::from_bytes(&sig_bytes);
    verifying_key.verify(canonical, &signature).is_ok()
}

pub(crate) fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

pub(crate) fn from_hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2 {
        return None;
    }
    let bytes = s.as_bytes();
    let mut out = [0u8; N];
    for (i, slot) in out.iter_mut().enumerate() {
        let hi = hex_val(bytes[i * 2])?;
        let lo = hex_val(bytes[i * 2 + 1])?;
        *slot = (hi << 4) | lo;
    }
    Some(out)
}

fn hex_val(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEED: [u8; 32] = [7u8; 32];

    #[test]
    fn sign_then_verify_roundtrips() {
        let canonical = br#"{"rows":[],"summary":{"count":0}}"#;
        let proof = sign(&SEED, canonical);
        assert_eq!(proof.signature_hex.len(), 128); // 64 bytes
        assert_eq!(proof.public_key_hex.len(), 64); // 32 bytes
        assert!(verify(
            &proof.public_key_hex,
            canonical,
            &proof.signature_hex
        ));
    }

    #[test]
    fn a_tampered_response_fails_verification() {
        let proof = sign(&SEED, b"original");
        assert!(!verify(
            &proof.public_key_hex,
            b"tampered",
            &proof.signature_hex
        ));
    }

    #[test]
    fn a_wrong_public_key_fails_verification() {
        let proof = sign(&SEED, b"msg");
        let other_pk = sign(&[9u8; 32], b"msg").public_key_hex;
        assert!(!verify(&other_pk, b"msg", &proof.signature_hex));
    }

    #[test]
    fn malformed_hex_fails_closed() {
        assert!(!verify("xyz", b"msg", "nothex"));
        assert!(!verify("", b"msg", ""));
    }
}
