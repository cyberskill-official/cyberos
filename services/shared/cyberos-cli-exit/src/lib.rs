//! `cyberos-cli-exit` — shared exit codes for every CyberOS CLI binary.
//!
//! Defined by AUTHORING_DISCIPLINE §3.3 rule 9 (cross-CLI rule):
//! *"All CyberOS CLIs MUST re-export `cyberos-cli-exit::ExitCode`."*
//!
//! Codes 0-7 are stable cross-CLI contract. Module-specific extensions start
//! at the per-module reserved range:
//!   - 200 = AUTH
//!   - 300 = BRAIN
//!   - 400 = OBS

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::process::Termination;

/// Shared exit codes. Convert to `i32` via `code as i32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    /// All operations succeeded.
    Ok = 0,
    /// Generic failure (use a more specific code when possible).
    Generic = 1,
    /// CLI arg parsing failed.
    UsageError = 2,
    /// Configuration was malformed or missing.
    ConfigError = 3,
    /// A network or external-service call failed.
    NetworkError = 4,
    /// Authentication or authorization failed.
    AuthError = 5,
    /// A precondition was violated (e.g. tenant not found, idempotency mismatch).
    PreconditionFailed = 6,
    /// The operation was interrupted (SIGINT / SIGTERM / timeout).
    Interrupted = 7,
}

impl ExitCode {
    /// Convenience for `std::process::exit`.
    pub fn exit(self) -> ! {
        std::process::exit(self as i32);
    }
}

impl Termination for ExitCode {
    fn report(self) -> std::process::ExitCode {
        std::process::ExitCode::from(self as u8)
    }
}

impl From<ExitCode> for i32 {
    fn from(c: ExitCode) -> i32 {
        c as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ok_is_zero() {
        assert_eq!(ExitCode::Ok as i32, 0);
    }

    #[test]
    fn codes_are_stable() {
        // These values are CONTRACT — changing them breaks every shell wrapper.
        assert_eq!(ExitCode::Ok as i32, 0);
        assert_eq!(ExitCode::Generic as i32, 1);
        assert_eq!(ExitCode::UsageError as i32, 2);
        assert_eq!(ExitCode::ConfigError as i32, 3);
        assert_eq!(ExitCode::NetworkError as i32, 4);
        assert_eq!(ExitCode::AuthError as i32, 5);
        assert_eq!(ExitCode::PreconditionFailed as i32, 6);
        assert_eq!(ExitCode::Interrupted as i32, 7);
    }
}
