//! FR-AI-014 — Source-hash verification for persona tamper detection.

use sha2::{Digest, Sha256};

use super::parse;
use super::types::{Persona, PersonaError};

/// Verify that the persona's body hash matches its cached `source_hash`.
///
/// This is the tamper-detection boundary check (FR-AI-001 §1 #7). On mismatch,
/// returns `PersonaError::Tampered`.
pub fn verify_persona(persona: &Persona) -> Result<(), PersonaError> {
    let actual = match std::fs::read_to_string(&persona.source_path) {
        Ok(raw) => match parse::parse_persona_md(&persona.source_path, &raw) {
            Ok(current) => current.source_hash,
            // Hot reload is all-or-nothing. If a saved file is temporarily
            // malformed, keep serving the last valid cached persona.
            Err(_parse_error) => return Ok(()),
        },
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => sha256(persona.body.as_bytes()),
        Err(err) => {
            return Err(PersonaError::MemoryReadFailed(format!(
                "{}: {err}",
                persona.source_path
            )))
        }
    };
    if actual != persona.source_hash {
        return Err(PersonaError::Tampered {
            handle: persona.handle.clone(),
            expected_hash: persona.source_hash,
            actual_hash: actual,
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
