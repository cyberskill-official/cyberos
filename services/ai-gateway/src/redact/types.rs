//! FR-AI-011 §3 — PII redaction types.

use std::collections::HashMap;

/// PII entity types. Closed enum; EN baseline + VN extensions (FR-AI-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PiiType {
    CreditCard,
    UsSsn,
    EmailAddress,
    PhoneNumber,
    Person,
    Location,
    IpAddress,
    IbanCode,
    UsBankNumber,
    MedicalLicense,
    // Slice-3 extensions for FR-AI-012 (declared here for ABI stability)
    VnCccd,
    VnMst,
    VnPhone,
    VnAddress,
}

impl PiiType {
    /// Stable string for OBS metric labels and audit-row keys.
    pub fn as_metric_label(self) -> &'static str {
        match self {
            Self::CreditCard => "credit_card",
            Self::UsSsn => "us_ssn",
            Self::EmailAddress => "email_address",
            Self::PhoneNumber => "phone_number",
            Self::Person => "person",
            Self::Location => "location",
            Self::IpAddress => "ip_address",
            Self::IbanCode => "iban_code",
            Self::UsBankNumber => "us_bank_number",
            Self::MedicalLicense => "medical_license",
            Self::VnCccd => "vn_cccd",
            Self::VnMst => "vn_mst",
            Self::VnPhone => "vn_phone",
            Self::VnAddress => "vn_address",
        }
    }

    /// Maps Presidio's UPPER_SNAKE entity type to our enum.
    pub fn from_presidio(s: &str) -> Option<Self> {
        match s {
            "CREDIT_CARD" => Some(Self::CreditCard),
            "US_SSN" => Some(Self::UsSsn),
            "EMAIL_ADDRESS" => Some(Self::EmailAddress),
            "PHONE_NUMBER" => Some(Self::PhoneNumber),
            "PERSON" => Some(Self::Person),
            "LOCATION" => Some(Self::Location),
            "IP_ADDRESS" => Some(Self::IpAddress),
            "IBAN_CODE" => Some(Self::IbanCode),
            "US_BANK_NUMBER" => Some(Self::UsBankNumber),
            "MEDICAL_LICENSE" => Some(Self::MedicalLicense),
            "VN_CCCD" => Some(Self::VnCccd),
            "VN_MST" => Some(Self::VnMst),
            "VN_PHONE" => Some(Self::VnPhone),
            "VN_ADDRESS" => Some(Self::VnAddress),
            _ => None,
        }
    }
}

/// Result of a successful redaction.
#[derive(Debug, Clone)]
pub struct RedactionResult {
    /// Text with PII replaced by typed placeholders.
    pub redacted_text: String,
    /// Ephemeral restoration map (zeroized on Drop).
    pub map: RestorationMap,
    /// Per-type PII counts for audit row.
    pub counts: HashMap<PiiType, u32>,
    /// Redaction latency in ms.
    pub latency_ms: u32,
}

/// Restoration map. Drop impl zeroizes the underlying string memory.
#[derive(Debug, Default, Clone)]
pub struct RestorationMap {
    inner: HashMap<String, zeroize::Zeroizing<String>>,
}

impl RestorationMap {
    pub fn get(&self, placeholder: &str) -> Option<&str> {
        self.inner.get(placeholder).map(|z| z.as_str())
    }

    pub fn insert(&mut self, placeholder: String, value: String) {
        self.inner
            .insert(placeholder, zeroize::Zeroizing::new(value));
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.inner.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

/// Error taxonomy for redaction.
#[derive(Debug)]
pub enum RedactError {
    /// Sidecar process unreachable.
    SidecarUnreachable { reason: String },
    /// Sidecar didn't respond within timeout.
    SidecarTimeout { waited_ms: u32 },
    /// Sidecar returned non-2xx.
    SidecarError { status: u16, message: String },
    /// Prompt failed pre-validation.
    InvalidPrompt { reason: String },
}

impl std::fmt::Display for RedactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SidecarUnreachable { reason } => {
                write!(f, "sidecar unreachable: {reason}")
            }
            Self::SidecarTimeout { waited_ms } => {
                write!(f, "sidecar timeout after {waited_ms}ms")
            }
            Self::SidecarError { status, message } => {
                write!(f, "sidecar error {status}: {message}")
            }
            Self::InvalidPrompt { reason } => {
                write!(f, "invalid prompt: {reason}")
            }
        }
    }
}

impl std::error::Error for RedactError {}
