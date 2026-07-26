//! RSA-SHA256 signature verification for Wise webhooks (DEC-840).

use base64::Engine;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::pkcs8::DecodePublicKey;
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use sha2::Sha256;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum WiseVerifyError {
    #[error("invalid_signature")]
    InvalidSignature,
    #[error("invalid_key")]
    InvalidKey,
    #[error("invalid_encoding")]
    InvalidEncoding,
}

/// Verify `X-Signature-SHA256` (base64) over the raw request body using a PEM public key.
pub fn verify_signature(pem_public_key: &str, body: &[u8], signature_b64: &str) -> Result<(), WiseVerifyError> {
    let public_key = RsaPublicKey::from_public_key_pem(pem_public_key)
        .map_err(|_| WiseVerifyError::InvalidKey)?;
    let verifying = VerifyingKey::<Sha256>::new(public_key);
    let sig_bytes = base64::engine::general_purpose::STANDARD
        .decode(signature_b64.trim())
        .map_err(|_| WiseVerifyError::InvalidEncoding)?;
    let signature = Signature::try_from(sig_bytes.as_slice()).map_err(|_| WiseVerifyError::InvalidSignature)?;
    verifying
        .verify(body, &signature)
        .map_err(|_| WiseVerifyError::InvalidSignature)
}
