//! FR-OBS-009 — Chain-of-custody manifest with Ed25519 signature.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Export manifest payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustodyManifest {
    /// Export id.
    pub export_id: String,
    /// Tenant scope.
    pub tenant_id: String,
    /// Sorted included chain hashes.
    pub chains: Vec<String>,
    /// SHA-256 of canonical payload bytes.
    pub payload_sha256: String,
    /// Ed25519 public key as hex.
    pub public_key_hex: String,
    /// Ed25519 signature as hex.
    pub signature_hex: String,
}

/// Sign an export manifest with a deterministic Ed25519 seed.
pub fn sign_manifest(
    export_id: &str,
    tenant_id: &str,
    chains: &[String],
    seed: [u8; 32],
) -> CustodyManifest {
    let mut sorted = chains.to_vec();
    sorted.sort();
    let payload = canonical_payload(export_id, tenant_id, &sorted);
    let payload_sha256 = hex(&Sha256::digest(payload.as_bytes()));
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = VerifyingKey::from(&signing_key);
    let signature = signing_key.sign(payload.as_bytes());
    CustodyManifest {
        export_id: export_id.into(),
        tenant_id: tenant_id.into(),
        chains: sorted,
        payload_sha256,
        public_key_hex: hex(&verifying_key.to_bytes()),
        signature_hex: hex(&signature.to_bytes()),
    }
}

/// Verify a custody manifest.
pub fn verify_manifest(manifest: &CustodyManifest) -> bool {
    let Ok(public_key_bytes) = decode_hex_32(&manifest.public_key_hex) else {
        return false;
    };
    let Ok(signature_bytes) = decode_hex_64(&manifest.signature_hex) else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&public_key_bytes) else {
        return false;
    };
    let signature = Signature::from_bytes(&signature_bytes);
    let payload = canonical_payload(&manifest.export_id, &manifest.tenant_id, &manifest.chains);
    let digest = hex(&Sha256::digest(payload.as_bytes()));
    digest == manifest.payload_sha256
        && verifying_key.verify(payload.as_bytes(), &signature).is_ok()
}

fn canonical_payload(export_id: &str, tenant_id: &str, chains: &[String]) -> String {
    format!(
        "export_id={export_id}\ntenant_id={tenant_id}\nchains={}\n",
        chains.join(",")
    )
}

fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}

fn decode_hex_32(s: &str) -> Result<[u8; 32], ()> {
    let vec = decode_hex(s)?;
    vec.try_into().map_err(|_| ())
}

fn decode_hex_64(s: &str) -> Result<[u8; 64], ()> {
    let vec = decode_hex(s)?;
    vec.try_into().map_err(|_| ())
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ()> {
    if s.len() % 2 != 0 {
        return Err(());
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks(2) {
        let hi = (chunk[0] as char).to_digit(16).ok_or(())?;
        let lo = (chunk[1] as char).to_digit(16).ok_or(())?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custody_manifest_has_real_verifiable_signature() {
        let manifest = sign_manifest("exp-1", "tenant-a", &["b".into(), "a".into()], [7u8; 32]);
        assert_eq!(manifest.chains, vec!["a".to_string(), "b".to_string()]);
        assert!(verify_manifest(&manifest));
    }
}
