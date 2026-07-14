//! Envelope encryption for payloads at rest (the DB-slice store-of-record, TASK-MCP-007/008 DEC-1125/1157).
//!
//! The `mcp_elicitations` / `mcp_tasks` tables store caller payloads as `*_kms_blob BYTEA` - never
//! plaintext. This module is the gateway's KMS seam: a [`Kms`] trait with `seal` / `open`, and one
//! in-process implementation, [`EnvKeyKms`], that does real AEAD (XChaCha20-Poly1305) under a 32-byte key
//! the operator supplies via `MCP_KMS_KEY` (base64). It is the dev-real analog of a cloud KMS the same way
//! the in-memory stores are the dev-real analog of the tables: a managed KMS (AWS KMS / GCP KMS / Vault
//! transit) can replace it behind this trait without touching any caller.
//!
//! The key is operator-provided at runtime (a gitignored `.env`), never authored in the repo. When
//! `MCP_KMS_KEY` is unset the gateway runs without DB-backed payloads (the in-memory store path), so the
//! dev demo needs no key.
//!
//! Blob layout: `nonce(24) || ciphertext+tag`. The 24-byte XChaCha nonce is random per seal (two v4 UUIDs
//! supply the bytes, the same source as the opaque-token helper), so sealing the same plaintext twice
//! yields different blobs and a random nonce is safe at this volume.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    XChaCha20Poly1305, XNonce,
};

/// A payload sealer/opener. `seal` encrypts plaintext to an opaque blob; `open` reverses it and
/// authenticates (a tampered or wrong-key blob fails). Implementors are the swap point for a managed KMS.
pub trait Kms: Send + Sync + std::fmt::Debug {
    /// Encrypt `plaintext` to a self-describing blob (`nonce || ciphertext+tag`).
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, KmsError>;
    /// Decrypt and authenticate a blob produced by [`Kms::seal`]. Fails on tamper, truncation, or a key
    /// mismatch.
    fn open(&self, blob: &[u8]) -> Result<Vec<u8>, KmsError>;
}

/// What can go wrong sealing or opening. Carries no key or plaintext material.
#[derive(Debug, thiserror::Error)]
pub enum KmsError {
    /// `MCP_KMS_KEY` is missing, not valid base64, or not exactly 32 bytes once decoded.
    #[error("KMS key missing or not 32 bytes of base64")]
    BadKey,
    /// Encryption failed (should not happen with a valid key; surfaced rather than panicked).
    #[error("KMS seal failed")]
    Seal,
    /// Decryption/authentication failed: tampered blob, truncated blob, or wrong key.
    #[error("KMS open failed")]
    Open,
}

/// The 24-byte XChaCha nonce, drawn from two v4 UUIDs (16 + 8 bytes). Random per call.
fn random_nonce() -> [u8; 24] {
    let mut n = [0u8; 24];
    n[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    n[16..24].copy_from_slice(&uuid::Uuid::new_v4().as_bytes()[..8]);
    n
}

/// AEAD sealer keyed by a 32-byte secret from the environment. The key is held in memory only; its
/// `Debug` is redacted so it never reaches a log line.
pub struct EnvKeyKms {
    key: [u8; 32],
}

impl std::fmt::Debug for EnvKeyKms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EnvKeyKms")
            .field("key", &"<redacted>")
            .finish()
    }
}

impl EnvKeyKms {
    /// Build from a base64 (standard alphabet) encoding of exactly 32 bytes. Whitespace is trimmed.
    pub fn from_base64_key(b64: &str) -> Result<Self, KmsError> {
        let raw = STANDARD.decode(b64.trim()).map_err(|_| KmsError::BadKey)?;
        let key: [u8; 32] = raw.try_into().map_err(|_| KmsError::BadKey)?;
        Ok(Self { key })
    }

    /// Read `MCP_KMS_KEY` from the environment: `Ok(None)` when unset/blank (the no-DB-payload path),
    /// `Ok(Some)` when a valid 32-byte key is present, `Err(BadKey)` when set but malformed (so the
    /// server fails loudly at boot rather than silently running without encryption).
    pub fn from_env() -> Result<Option<Self>, KmsError> {
        match std::env::var("MCP_KMS_KEY") {
            Ok(v) if !v.trim().is_empty() => Ok(Some(Self::from_base64_key(&v)?)),
            _ => Ok(None),
        }
    }
}

impl Kms for EnvKeyKms {
    fn seal(&self, plaintext: &[u8]) -> Result<Vec<u8>, KmsError> {
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key).map_err(|_| KmsError::BadKey)?;
        let nonce = random_nonce();
        let ciphertext = cipher
            .encrypt(XNonce::from_slice(&nonce), plaintext)
            .map_err(|_| KmsError::Seal)?;
        let mut blob = Vec::with_capacity(24 + ciphertext.len());
        blob.extend_from_slice(&nonce);
        blob.extend_from_slice(&ciphertext);
        Ok(blob)
    }

    fn open(&self, blob: &[u8]) -> Result<Vec<u8>, KmsError> {
        if blob.len() < 24 {
            return Err(KmsError::Open);
        }
        let (nonce, ciphertext) = blob.split_at(24);
        let cipher = XChaCha20Poly1305::new_from_slice(&self.key).map_err(|_| KmsError::BadKey)?;
        cipher
            .decrypt(XNonce::from_slice(nonce), ciphertext)
            .map_err(|_| KmsError::Open)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kms() -> EnvKeyKms {
        EnvKeyKms { key: [9u8; 32] }
    }

    #[test]
    fn seal_then_open_round_trips() {
        let k = kms();
        let pt = b"{\"confirmed\":true,\"reason\":\"ok\"}";
        let blob = k.seal(pt).expect("seal");
        assert_ne!(blob.as_slice(), pt, "blob is not plaintext");
        assert_eq!(k.open(&blob).expect("open"), pt);
    }

    #[test]
    fn same_plaintext_seals_differently() {
        let k = kms();
        let pt = b"repeatable";
        assert_ne!(k.seal(pt).unwrap(), k.seal(pt).unwrap(), "random nonce");
    }

    #[test]
    fn tampered_blob_fails_to_open() {
        let k = kms();
        let mut blob = k.seal(b"secret").unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xff;
        assert!(k.open(&blob).is_err(), "AEAD authentication catches tamper");
    }

    #[test]
    fn wrong_key_fails_to_open() {
        let blob = kms().seal(b"secret").unwrap();
        let other = EnvKeyKms { key: [1u8; 32] };
        assert!(other.open(&blob).is_err());
    }

    #[test]
    fn truncated_blob_fails_to_open() {
        let k = kms();
        assert!(k.open(&[0u8; 10]).is_err(), "shorter than the nonce");
    }

    #[test]
    fn base64_key_must_be_32_bytes() {
        // 32 bytes -> ok.
        let good = STANDARD.encode([3u8; 32]);
        assert!(EnvKeyKms::from_base64_key(&good).is_ok());
        // 16 bytes -> rejected.
        let short = STANDARD.encode([3u8; 16]);
        assert!(matches!(
            EnvKeyKms::from_base64_key(&short),
            Err(KmsError::BadKey)
        ));
        // not base64 -> rejected.
        assert!(matches!(
            EnvKeyKms::from_base64_key("not base64!!!"),
            Err(KmsError::BadKey)
        ));
    }

    #[test]
    fn from_base64_key_trims_whitespace() {
        let key = format!("  {}\n", STANDARD.encode([5u8; 32]));
        let k = EnvKeyKms::from_base64_key(&key).expect("trimmed key parses");
        let blob = k.seal(b"x").unwrap();
        assert_eq!(k.open(&blob).unwrap(), b"x");
    }
}
