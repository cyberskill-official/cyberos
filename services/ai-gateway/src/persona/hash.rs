//! TASK-AI-014 — Source-hash verification for persona tamper detection.

use sha2::{Digest, Sha256};

use super::types::{Persona, PersonaError};

/// Verify that the persona's body hash matches its cached `source_hash`.
///
/// This is the tamper-detection boundary check (TASK-AI-001 §1 #7). On mismatch,
/// returns `PersonaError::Tampered`.
pub fn verify_persona(persona: &Persona) -> Result<(), PersonaError> {
    let actual = sha256(persona.body.as_bytes());
    if actual != persona.source_hash {
        return Err(PersonaError::Tampered {
            handle: persona.handle.clone(),
            expected_hash: Box::new(persona.source_hash),
            actual_hash: Box::new(actual),
        });
    }
    Ok(())
}

/// Compute SHA-256 over the given bytes.
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

/// Return the first 16 hex characters (8 bytes) of a SHA-256 hash.
///
/// Used for the `X-CyberOS-Persona-Source-Hash` response header.
pub fn hex16(hash: &[u8; 32]) -> String {
    hash.iter().take(8).map(|b| format!("{:02x}", b)).collect()
}
