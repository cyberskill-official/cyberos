//! RS256 key generation for TASK-AUTH-004.
//!
//! Called by `cyberos-auth bootstrap` (and during AppState boot if no
//! active key exists). Generates a 2048-bit RSA keypair and returns the
//! PEM-encoded public + private halves.

use rand::rngs::OsRng;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};
use std::error::Error;
use std::fmt;

pub struct GeneratedKey {
    pub public_pem: String,
    pub private_pem: String,
}

#[derive(Debug)]
pub struct KeygenError(pub String);

impl fmt::Display for KeygenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "key generation failed: {}", self.0)
    }
}
impl Error for KeygenError {}

/// Generate a fresh RSA-2048 keypair. RS256 standard size.
pub fn generate_rsa_2048() -> Result<GeneratedKey, KeygenError> {
    let mut rng = OsRng;
    let private = RsaPrivateKey::new(&mut rng, 2048)
        .map_err(|e| KeygenError(format!("rsa key creation: {e}")))?;
    let public = RsaPublicKey::from(&private);
    let private_pem = private
        .to_pkcs8_pem(LineEnding::LF)
        .map_err(|e| KeygenError(format!("encode private pem: {e}")))?
        .to_string();
    let public_pem = public
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| KeygenError(format!("encode public pem: {e}")))?;
    Ok(GeneratedKey {
        public_pem,
        private_pem,
    })
}
