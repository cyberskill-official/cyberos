//! FR-EMAIL-001 — structured error type for the email crate.

use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum EmailError {
    #[error("body size {0} bytes is outside the accepted range (1..=26214400)")]
    BodyTooLarge(usize),

    #[error("body sha256 mismatch — header={expected}, computed={actual}")]
    BodyHashMismatch { expected: String, actual: String },

    #[error("unknown residency tag: {0} — accepted: sg-1 / vn-1 / eu-1 / us-1")]
    UnknownResidency(String),

    #[error("residency mismatch — message targeted {tenant_id} (residency {expected}) but body landed in {actual}")]
    ResidencyMismatch {
        tenant_id: Uuid,
        expected: String,
        actual: String,
    },

    #[error("tenant {0} has no residency tag — provision required via cyberos-email-cli")]
    NoResidencyForTenant(Uuid),

    #[error("dkim key generation failed: {0}")]
    DkimKeyGen(String),

    #[error("dkim key for tenant {0} (selector={1}) not found")]
    DkimKeyNotFound(Uuid, String),

    #[error(
        "dkim key for tenant {0} already exists at selector={1}; rotate via cyberos-email-cli"
    )]
    DkimKeyAlreadyExists(Uuid, String),

    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("internal error: {0}")]
    Other(String),
}

impl EmailError {
    /// Stable error code for OTel span attributes + JSON responses.
    pub fn code(&self) -> &'static str {
        match self {
            Self::BodyTooLarge(_) => "body_too_large",
            Self::BodyHashMismatch { .. } => "body_hash_mismatch",
            Self::UnknownResidency(_) => "unknown_residency",
            Self::ResidencyMismatch { .. } => "residency_mismatch",
            Self::NoResidencyForTenant(_) => "no_residency_for_tenant",
            Self::DkimKeyGen(_) => "dkim_key_gen_failed",
            Self::DkimKeyNotFound(..) => "dkim_key_not_found",
            Self::DkimKeyAlreadyExists(..) => "dkim_key_already_exists",
            Self::Db(_) => "db_error",
            Self::Io(_) => "io_error",
            Self::Other(_) => "internal_error",
        }
    }
}

pub type EmailResult<T> = Result<T, EmailError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_kebab() {
        let err = EmailError::BodyTooLarge(99_999_999);
        assert_eq!(err.code(), "body_too_large");
        let err = EmailError::UnknownResidency("zz-1".into());
        assert_eq!(err.code(), "unknown_residency");
    }
}
