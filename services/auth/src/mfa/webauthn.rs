//! FR-AUTH-102 — WebAuthn MFA factor enrolment + verification.
//!
//! Delegates to the shared `webauthn-rs` instance already initialised by
//! `passkey.rs`. This module only deals with MFA *second-factor* WebAuthn
//! (roaming authenticators like YubiKeys), NOT discoverable-credential
//! passkey login (that's FR-AUTH-105 in `passkey.rs`).
//!
//! Endpoints:
//!   * `POST /v1/auth/mfa/factors/webauthn/enrol/begin`
//!   * `POST /v1/auth/mfa/factors/webauthn/enrol/finish`
//!   * `POST /v1/auth/mfa/webauthn/verify/begin`
//!   * `POST /v1/auth/mfa/webauthn/verify/finish`


use uuid::Uuid;

/// Row shape returned when loading a WebAuthn factor from `mfa_factors`.
#[derive(Debug, Clone)]
pub struct WebauthnFactor {
    pub factor_id: Uuid,
    pub display_name: String,
    pub credential_id: Vec<u8>,
    pub public_key_json: Vec<u8>,
    pub signature_count: i64,
}

/// Validate that a WebAuthn assertion's counter is strictly greater than
/// the stored counter. Equality or regression indicates a cloned credential
/// (FIDO2 spec §6.2.1).
pub fn validate_counter_monotonicity(
    stored_count: i64,
    asserted_count: u32,
) -> Result<(), CounterError> {
    let asserted = asserted_count as i64;
    if asserted <= stored_count && stored_count != 0 {
        return Err(CounterError::Regression {
            stored: stored_count,
            asserted,
        });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum CounterError {
    #[error("webauthn signature counter regression: stored={stored}, asserted={asserted}")]
    Regression { stored: i64, asserted: i64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_monotonicity_accepts_increase() {
        assert!(validate_counter_monotonicity(5, 6).is_ok());
        assert!(validate_counter_monotonicity(0, 1).is_ok());
    }

    #[test]
    fn counter_monotonicity_rejects_regression() {
        assert!(validate_counter_monotonicity(10, 9).is_err());
        assert!(validate_counter_monotonicity(10, 10).is_err());
    }

    #[test]
    fn counter_monotonicity_allows_zero_to_zero() {
        // Some authenticators never increment the counter (always 0).
        assert!(validate_counter_monotonicity(0, 0).is_ok());
    }
}
