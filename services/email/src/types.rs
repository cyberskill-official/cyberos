//! FR-EMAIL-001 §1 #7 + §1 #9 — domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// FR-EMAIL-001 §1 #9 closed enum — direction of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_direction", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    Inbound,
    Outbound,
    Internal,
}

/// FR-EMAIL-001 §1 #9 closed enum — delivery status of a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "message_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageStatus {
    Received,
    Quarantined,
    Delivered,
    Sent,
    Bounced,
    Dropped,
}

impl MessageStatus {
    /// Status derived from spam score per §1 #18 — threshold 5.0.
    pub fn from_spam_score(score: f32) -> Self {
        if score >= 5.0 {
            Self::Quarantined
        } else {
            Self::Received
        }
    }
}

/// FR-EMAIL-001 §3.3 — bounce classification per RFC 3463.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BounceKind {
    Hard,
    Soft,
    Transient,
}

impl BounceKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hard => "hard",
            Self::Soft => "soft",
            Self::Transient => "transient",
        }
    }
}

/// DKIM key algorithm per DEC-304. Slice 1 ships RSA-2048; Ed25519 deferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum KeyAlgorithm {
    Rsa2048,
    Ed25519,
}

impl KeyAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Rsa2048 => "rsa-2048",
            Self::Ed25519 => "ed25519",
        }
    }
}

/// DKIM key status — exactly one `Active` per (tenant, selector).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DkimKeyStatus {
    Active,
    Rotated,
    Revoked,
}

impl DkimKeyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Rotated => "rotated",
            Self::Revoked => "revoked",
        }
    }
}

/// FR-EMAIL-001 §1 #7 — append-only message metadata row.
///
/// Bodies live in S3 (encrypted via KMS, residency-pinned per tenant).
/// Postgres holds only headers + delivery state. Status transitions
/// create NEW rows linked via `prior_message_id`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailMessage {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub stalwart_message_id: i64,
    pub thread_id: String,
    pub direction: MessageDirection,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub subject: Option<String>,
    pub received_at: DateTime<Utc>,
    pub s3_body_key: String,
    pub s3_body_kms_key_id: String,
    pub body_sha256_hex: String,
    pub byte_size: i64,
    pub status: MessageStatus,
    pub prior_message_id: Option<Uuid>,
    pub spam_score: Option<f32>,
    pub dkim_pass: Option<bool>,
    pub spf_pass: Option<bool>,
    pub dmarc_pass: Option<bool>,
    pub bimi_present: Option<bool>,
    pub created_at: DateTime<Utc>,
}

/// FR-EMAIL-001 §1 #8 — materialised thread view for fast list queries.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmailThread {
    pub thread_id: String,
    pub tenant_id: Uuid,
    pub subject_normalised: Option<String>,
    pub last_message_at: DateTime<Utc>,
    pub message_count: i32,
    pub participant_addresses: Vec<String>,
}

/// FR-EMAIL-001 §1 #17 — append-only bounce log entry.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BounceEvent {
    pub id: i64,
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub bounce_kind: String,
    pub bounce_reason: String,
    pub bounce_code: Option<String>,
    pub remote_peer: Option<String>,
    pub ts: DateTime<Utc>,
}

/// FR-EMAIL-001 §1 #5 — per-tenant DKIM key registry row.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DkimKey {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub dkim_selector: String,
    pub key_algorithm: String,
    pub public_key_pem: String,
    /// Private key is KMS-encrypted; the plaintext is never stored.
    pub private_key_kms_encrypted_blob: Vec<u8>,
    pub kms_key_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

/// FR-EMAIL-001 §3.5 — residency binding for body storage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmailStorageBinding {
    pub residency: String,
    pub region: String,
    pub bucket: String,
    pub kms_key_id: String,
}

/// Request body for `cyberos-email-cli provision`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionRequest {
    pub tenant_id: Uuid,
    pub local_part: String,
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spam_threshold_is_5_0() {
        assert_eq!(MessageStatus::from_spam_score(4.9), MessageStatus::Received);
        assert_eq!(MessageStatus::from_spam_score(5.0), MessageStatus::Quarantined);
        assert_eq!(MessageStatus::from_spam_score(7.5), MessageStatus::Quarantined);
    }

    #[test]
    fn key_algorithm_str_is_kebab() {
        assert_eq!(KeyAlgorithm::Rsa2048.as_str(), "rsa-2048");
        assert_eq!(KeyAlgorithm::Ed25519.as_str(), "ed25519");
    }

    #[test]
    fn bounce_kind_str_matches_sql_check() {
        assert_eq!(BounceKind::Hard.as_str(), "hard");
        assert_eq!(BounceKind::Soft.as_str(), "soft");
        assert_eq!(BounceKind::Transient.as_str(), "transient");
    }
}
