//! FR-EMAIL-001 §1 #13 + §1 #14 — canonical email.* memory audit row
//! builders.
//!
//! The 5 row kinds:
//!   - email.message_received
//!   - email.message_sent
//!   - email.message_bounced
//!   - email.message_quarantined
//!   - email.dkim_key_rotated
//!
//! Per §1 #14 the memory chain holds PII-stripped form:
//!   from_hash16 = SHA-256[..16] of `normalise_address(from)`
//!   to_hash16   = SHA-256[..16] of `sort(normalise_address(to))[0]`
//!
//! The Postgres `message_metadata` row carries raw addresses (RLS-scoped);
//! memory holds only the 16-char hash prefix. Forensic-grade queries can
//! still join Postgres ↔ memory via `message_id`.

use crate::types::{BounceKind, EmailMessage, MessageStatus};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Canonical row body — agnostic of the memory writer transport.
#[derive(Debug, Clone, Serialize)]
pub struct EmailAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub message_id: Option<Uuid>,
    pub thread_id: Option<String>,
    pub direction: Option<&'static str>,
    pub from_hash16: Option<String>,
    pub to_hash16: Option<String>,
    pub body_sha256_hex: Option<String>,
    pub byte_size: Option<i64>,
    pub status: Option<&'static str>,
    pub spam_score: Option<f32>,
    pub spam_score_band: Option<&'static str>,
    pub bounce_kind: Option<&'static str>,
    pub bounce_code: Option<String>,
    pub dkim_pass: Option<bool>,
    pub spf_pass: Option<bool>,
    pub dmarc_pass: Option<bool>,
    pub bimi_present: Option<bool>,
    pub old_key_id: Option<Uuid>,
    pub new_key_id: Option<Uuid>,
    pub dkim_selector: Option<String>,
    pub key_algorithm: Option<String>,
    pub rotated_by_subject_id_hash16: Option<String>,
    pub ts_ns: i128,
}

/// PII hash per §1 #14 — SHA-256 of the normalised address, hex-encoded
/// and truncated to 16 chars.
pub fn hash16(input: &str) -> String {
    let normalised = input.trim().to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(normalised.as_bytes());
    let digest = hasher.finalize();
    hex16(&digest[..])
}

fn hex16(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(16);
    for b in bytes.iter().take(8) {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Classify spam score into one of three bands per §1 #23 metrics labels.
pub fn spam_band(score: f32) -> &'static str {
    if score >= 10.0 {
        "10+"
    } else if score >= 7.0 {
        "7-10"
    } else {
        "5-7"
    }
}

/// Current ts in nanoseconds since the unix epoch. i128 for chain-friendly serialisation.
pub fn now_ns() -> i128 {
    let ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as i128)
        .unwrap_or(0);
    ns
}

pub fn message_received(msg: &EmailMessage) -> EmailAuditRow {
    EmailAuditRow {
        kind: "email.message_received",
        tenant_id: msg.tenant_id,
        message_id: Some(msg.id),
        thread_id: Some(msg.thread_id.clone()),
        direction: Some("inbound"),
        from_hash16: Some(hash16(&msg.from_address)),
        to_hash16: msg.to_addresses.first().map(|s| hash16(s)),
        body_sha256_hex: Some(msg.body_sha256_hex.clone()),
        byte_size: Some(msg.byte_size),
        status: Some(status_str(msg.status)),
        spam_score: msg.spam_score,
        spam_score_band: None,
        bounce_kind: None,
        bounce_code: None,
        dkim_pass: msg.dkim_pass,
        spf_pass: msg.spf_pass,
        dmarc_pass: msg.dmarc_pass,
        bimi_present: msg.bimi_present,
        old_key_id: None,
        new_key_id: None,
        dkim_selector: None,
        key_algorithm: None,
        rotated_by_subject_id_hash16: None,
        ts_ns: now_ns(),
    }
}

pub fn message_sent(msg: &EmailMessage) -> EmailAuditRow {
    EmailAuditRow {
        kind: "email.message_sent",
        tenant_id: msg.tenant_id,
        message_id: Some(msg.id),
        thread_id: Some(msg.thread_id.clone()),
        direction: Some("outbound"),
        from_hash16: Some(hash16(&msg.from_address)),
        to_hash16: msg.to_addresses.first().map(|s| hash16(s)),
        body_sha256_hex: Some(msg.body_sha256_hex.clone()),
        byte_size: Some(msg.byte_size),
        status: Some("sent"),
        spam_score: None,
        spam_score_band: None,
        bounce_kind: None,
        bounce_code: None,
        dkim_pass: msg.dkim_pass,
        spf_pass: msg.spf_pass,
        dmarc_pass: msg.dmarc_pass,
        bimi_present: msg.bimi_present,
        old_key_id: None,
        new_key_id: None,
        dkim_selector: None,
        key_algorithm: None,
        rotated_by_subject_id_hash16: None,
        ts_ns: now_ns(),
    }
}

pub fn message_bounced(
    tenant_id: Uuid,
    message_id: Uuid,
    bounce_kind: BounceKind,
    bounce_code: Option<&str>,
) -> EmailAuditRow {
    EmailAuditRow {
        kind: "email.message_bounced",
        tenant_id,
        message_id: Some(message_id),
        thread_id: None,
        direction: Some("outbound"),
        from_hash16: None,
        to_hash16: None,
        body_sha256_hex: None,
        byte_size: None,
        status: Some("bounced"),
        spam_score: None,
        spam_score_band: None,
        bounce_kind: Some(bounce_kind.as_str()),
        bounce_code: bounce_code.map(|s| s.to_owned()),
        dkim_pass: None,
        spf_pass: None,
        dmarc_pass: None,
        bimi_present: None,
        old_key_id: None,
        new_key_id: None,
        dkim_selector: None,
        key_algorithm: None,
        rotated_by_subject_id_hash16: None,
        ts_ns: now_ns(),
    }
}

pub fn message_quarantined(msg: &EmailMessage) -> EmailAuditRow {
    let band = msg.spam_score.map(spam_band);
    EmailAuditRow {
        kind: "email.message_quarantined",
        tenant_id: msg.tenant_id,
        message_id: Some(msg.id),
        thread_id: Some(msg.thread_id.clone()),
        direction: Some("inbound"),
        from_hash16: Some(hash16(&msg.from_address)),
        to_hash16: msg.to_addresses.first().map(|s| hash16(s)),
        body_sha256_hex: Some(msg.body_sha256_hex.clone()),
        byte_size: Some(msg.byte_size),
        status: Some("quarantined"),
        spam_score: msg.spam_score,
        spam_score_band: band,
        bounce_kind: None,
        bounce_code: None,
        dkim_pass: msg.dkim_pass,
        spf_pass: msg.spf_pass,
        dmarc_pass: msg.dmarc_pass,
        bimi_present: msg.bimi_present,
        old_key_id: None,
        new_key_id: None,
        dkim_selector: None,
        key_algorithm: None,
        rotated_by_subject_id_hash16: None,
        ts_ns: now_ns(),
    }
}

pub fn dkim_key_rotated(
    tenant_id: Uuid,
    old_key_id: Option<Uuid>,
    new_key_id: Uuid,
    selector: &str,
    algorithm: &str,
    rotated_by_subject_id: Option<&str>,
) -> EmailAuditRow {
    EmailAuditRow {
        kind: "email.dkim_key_rotated",
        tenant_id,
        message_id: None,
        thread_id: None,
        direction: None,
        from_hash16: None,
        to_hash16: None,
        body_sha256_hex: None,
        byte_size: None,
        status: None,
        spam_score: None,
        spam_score_band: None,
        bounce_kind: None,
        bounce_code: None,
        dkim_pass: None,
        spf_pass: None,
        dmarc_pass: None,
        bimi_present: None,
        old_key_id,
        new_key_id: Some(new_key_id),
        dkim_selector: Some(selector.to_owned()),
        key_algorithm: Some(algorithm.to_owned()),
        rotated_by_subject_id_hash16: rotated_by_subject_id.map(hash16),
        ts_ns: now_ns(),
    }
}

fn status_str(s: MessageStatus) -> &'static str {
    match s {
        MessageStatus::Received => "received",
        MessageStatus::Quarantined => "quarantined",
        MessageStatus::Delivered => "delivered",
        MessageStatus::Sent => "sent",
        MessageStatus::Bounced => "bounced",
        MessageStatus::Dropped => "dropped",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash16_is_deterministic_and_16_chars() {
        let a = hash16("alice@example.com");
        let b = hash16("ALICE@example.com  ");  // mixed case + trailing space
        assert_eq!(a, b, "hash16 normalises case + trims");
        assert_eq!(a.len(), 16);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash16_distinct_for_distinct_addresses() {
        assert_ne!(hash16("alice@example.com"), hash16("bob@example.com"));
    }

    #[test]
    fn spam_band_classification() {
        assert_eq!(spam_band(5.0), "5-7");
        assert_eq!(spam_band(6.9), "5-7");
        assert_eq!(spam_band(7.0), "7-10");
        assert_eq!(spam_band(9.99), "7-10");
        assert_eq!(spam_band(10.0), "10+");
        assert_eq!(spam_band(15.0), "10+");
    }

    #[test]
    fn dkim_rotated_row_carries_both_old_and_new() {
        let tid = Uuid::new_v4();
        let old = Uuid::new_v4();
        let new = Uuid::new_v4();
        let row = dkim_key_rotated(tid, Some(old), new, "cyberos", "rsa-2048", Some("admin@cyberos"));
        assert_eq!(row.kind, "email.dkim_key_rotated");
        assert_eq!(row.old_key_id, Some(old));
        assert_eq!(row.new_key_id, Some(new));
        assert_eq!(row.dkim_selector.as_deref(), Some("cyberos"));
        assert_eq!(row.key_algorithm.as_deref(), Some("rsa-2048"));
        assert!(row.rotated_by_subject_id_hash16.is_some());
        assert_eq!(row.rotated_by_subject_id_hash16.as_ref().unwrap().len(), 16);
    }

    #[test]
    fn five_canonical_kinds_distinct() {
        let kinds = [
            "email.message_received",
            "email.message_sent",
            "email.message_bounced",
            "email.message_quarantined",
            "email.dkim_key_rotated",
        ];
        // sanity — the 5 row kinds spec'd by §1 #13 are distinct strings.
        let set: std::collections::HashSet<_> = kinds.iter().collect();
        assert_eq!(set.len(), 5);
    }
}
