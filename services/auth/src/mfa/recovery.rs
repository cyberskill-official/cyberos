//! FR-AUTH-102 — Recovery codes (single-use, bcrypt-hashed).
//!
//! Each subject gets exactly one *batch* of 10 codes at a time. Regeneration
//! invalidates ALL prior codes (new `batch_id`). Each code is consumed
//! exactly once; after consumption it is marked `consumed=true` and the
//! `consumed_at` timestamp is set.
//!
//! Codes are 8-character alphanumeric strings (no ambiguous chars like 0/O/1/l).

use uuid::Uuid;

/// Characters used for recovery code generation (unambiguous charset).
const RECOVERY_ALPHABET: &[u8] = b"23456789ABCDEFGHJKMNPQRSTUVWXYZ";
const RECOVERY_CODE_LEN: usize = 8;
pub const RECOVERY_BATCH_SIZE: usize = 10;

/// Generate a single recovery code.
fn generate_one_code() -> String {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(RECOVERY_CODE_LEN);
    for _ in 0..RECOVERY_CODE_LEN {
        let idx = (rng.next_u32() as usize) % RECOVERY_ALPHABET.len();
        code.push(RECOVERY_ALPHABET[idx] as char);
    }
    code
}

/// Generate a fresh batch of recovery codes.
/// Returns `(batch_id, Vec<(plaintext_code, bcrypt_hash)>)`.
pub fn generate_batch() -> (Uuid, Vec<(String, String)>) {
    let batch_id = Uuid::new_v4();
    let codes: Vec<(String, String)> = (0..RECOVERY_BATCH_SIZE)
        .map(|_| {
            let plain = generate_one_code();
            let hash = bcrypt::hash(&plain, bcrypt::DEFAULT_COST)
                .expect("bcrypt hash should not fail for short input");
            (plain, hash)
        })
        .collect();
    (batch_id, codes)
}

/// Verify a submitted code against a bcrypt hash.
pub fn verify_code(submitted: &str, hash: &str) -> bool {
    // Normalise input: strip dashes/spaces, uppercase.
    let normalised: String = submitted
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(|c| c.to_uppercase())
        .collect();
    bcrypt::verify(&normalised, hash).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_batch_produces_correct_count() {
        let (batch_id, codes) = generate_batch();
        assert!(!batch_id.is_nil());
        assert_eq!(codes.len(), RECOVERY_BATCH_SIZE);
    }

    #[test]
    fn generated_codes_are_correct_length() {
        let (_, codes) = generate_batch();
        for (plain, _hash) in &codes {
            assert_eq!(plain.len(), RECOVERY_CODE_LEN);
            // All chars from the alphabet
            assert!(plain.chars().all(|c| RECOVERY_ALPHABET.contains(&(c as u8))));
        }
    }

    #[test]
    fn verify_code_accepts_correct_code() {
        let (_, codes) = generate_batch();
        let (plain, hash) = &codes[0];
        assert!(verify_code(plain, hash));
    }

    #[test]
    fn verify_code_rejects_wrong_code() {
        let (_, codes) = generate_batch();
        let (_, hash) = &codes[0];
        assert!(!verify_code("ZZZZZZZZ", hash));
    }

    #[test]
    fn verify_code_handles_dashes_and_case() {
        let (_, codes) = generate_batch();
        let (plain, hash) = &codes[0];
        // Insert dashes and lowercase
        let with_dash = format!(
            "{}-{}",
            &plain[..4].to_lowercase(),
            &plain[4..].to_lowercase()
        );
        assert!(verify_code(&with_dash, hash));
    }
}
