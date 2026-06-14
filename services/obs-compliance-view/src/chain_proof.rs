//! Ed25519 chain proof for compliance responses.

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::error::ViewError;
use crate::memory::{hex, AuditRow};

/// Proof attached to every response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainProof {
    /// Algorithm name.
    pub algorithm: String,
    /// Public key id.
    pub public_key_id: String,
    /// Signed tree head at query time.
    pub sth_at_query_time: String,
    /// Base64 Ed25519 signature.
    pub signature: String,
}

/// Signer config.
#[derive(Debug, Clone)]
pub struct ChainProofSigner {
    signing_key: SigningKey,
}

impl ChainProofSigner {
    /// Create from 32-byte seed.
    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            signing_key: SigningKey::from_bytes(&seed),
        }
    }

    /// Create from hex seed.
    pub fn from_hex_seed(hex_seed: &str) -> Result<Self, ViewError> {
        let bytes = decode_hex(hex_seed)?;
        let seed: [u8; 32] = bytes
            .try_into()
            .map_err(|_| ViewError::ChainProofFailed("seed must be 32 bytes".to_string()))?;
        Ok(Self::from_seed(seed))
    }

    /// Verifying key.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing_key.verifying_key()
    }

    /// Sign rows and summary.
    pub fn sign(
        &self,
        rows: &[AuditRow],
        summary: &serde_json::Value,
        sth: &str,
    ) -> Result<ChainProof, ViewError> {
        let canonical = canonicalise(rows, summary)?;
        let signature = self.signing_key.sign(&canonical);
        let public_key = self.signing_key.verifying_key();
        Ok(ChainProof {
            algorithm: "Ed25519".to_string(),
            public_key_id: hex(&public_key.to_bytes()[..8]),
            sth_at_query_time: sth.to_string(),
            signature: BASE64.encode(signature.to_bytes()),
        })
    }
}

/// Canonical bytes signed by the proof.
pub fn canonicalise(rows: &[AuditRow], summary: &serde_json::Value) -> Result<Vec<u8>, ViewError> {
    let mut sorted_rows = rows.to_vec();
    sorted_rows
        .sort_by(|a, b| (a.ts, &a.kind, &a.payload_hash).cmp(&(b.ts, &b.kind, &b.payload_hash)));
    serde_json::to_vec(&serde_json::json!({
        "rows": sorted_rows,
        "summary": summary,
    }))
    .map_err(|err| ViewError::ChainProofFailed(err.to_string()))
}

/// Verify a proof.
pub fn verify(
    verifying_key: &VerifyingKey,
    rows: &[AuditRow],
    summary: &serde_json::Value,
    proof: &ChainProof,
) -> bool {
    let Ok(canonical) = canonicalise(rows, summary) else {
        return false;
    };
    let Ok(signature_bytes) = BASE64.decode(&proof.signature) else {
        return false;
    };
    let Ok(signature_bytes) = <[u8; 64]>::try_from(signature_bytes.as_slice()) else {
        return false;
    };
    let signature = Signature::from_bytes(&signature_bytes);
    verifying_key.verify(&canonical, &signature).is_ok()
}

fn decode_hex(s: &str) -> Result<Vec<u8>, ViewError> {
    if s.len() % 2 != 0 {
        return Err(ViewError::ChainProofFailed(
            "hex length must be even".to_string(),
        ));
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    for chunk in s.as_bytes().chunks(2) {
        let hi = (chunk[0] as char)
            .to_digit(16)
            .ok_or_else(|| ViewError::ChainProofFailed("invalid hex".to_string()))?;
        let lo = (chunk[1] as char)
            .to_digit(16)
            .ok_or_else(|| ViewError::ChainProofFailed("invalid hex".to_string()))?;
        out.push(((hi << 4) | lo) as u8);
    }
    Ok(out)
}
