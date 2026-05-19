//! FR-EMAIL-001 §4 #5 + §4 #13 + §4 #25 — memory audit row builders.

use chrono::Utc;
use cyberos_email::audit::email_events::{
    dkim_key_rotated, hash16, message_bounced, message_quarantined, message_received, message_sent,
    spam_band,
};
use cyberos_email::types::{BounceKind, EmailMessage, MessageDirection, MessageStatus};
use uuid::Uuid;

fn sample(status: MessageStatus, score: Option<f32>) -> EmailMessage {
    EmailMessage {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        stalwart_message_id: 8472,
        thread_id: "<thread.001@cyberos>".into(),
        direction: MessageDirection::Inbound,
        from_address: "alice@example.com".into(),
        to_addresses: vec!["support@cyberskill.world".into()],
        cc_addresses: vec![],
        bcc_addresses: vec![],
        subject: Some("Hello".into()),
        received_at: Utc::now(),
        s3_body_key: "tid/8472/abc".into(),
        s3_body_kms_key_id: "alias/cyberos-email-vn-1-bodies".into(),
        body_sha256_hex: "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".into(),
        byte_size: 12_340,
        status,
        prior_message_id: None,
        spam_score: score,
        dkim_pass: Some(true),
        spf_pass: Some(true),
        dmarc_pass: Some(true),
        bimi_present: Some(false),
        created_at: Utc::now(),
    }
}

#[test]
fn received_row_carries_pii_hashes_not_raw_addresses() {
    let msg = sample(MessageStatus::Received, Some(0.5));
    let row = message_received(&msg);
    assert_eq!(row.kind, "email.message_received");
    assert_eq!(row.direction, Some("inbound"));
    // Borrow rather than move so `row` remains usable for serialisation below.
    let from = row
        .from_hash16
        .as_ref()
        .expect("from_hash16 set on received");
    let to = row.to_hash16.as_ref().expect("to_hash16 set on received");
    assert_eq!(from.len(), 16);
    assert_eq!(to.len(), 16);
    // Raw addresses MUST NOT appear anywhere on the row body.
    let json = serde_json::to_string(&row).unwrap();
    assert!(
        !json.contains("alice@example.com"),
        "raw from leaked into row"
    );
    assert!(
        !json.contains("support@cyberskill.world"),
        "raw to leaked into row"
    );
}

#[test]
fn quarantined_row_carries_spam_band() {
    let msg = sample(MessageStatus::Quarantined, Some(7.3));
    let row = message_quarantined(&msg);
    assert_eq!(row.kind, "email.message_quarantined");
    assert_eq!(row.spam_score_band, Some("7-10"));
    assert_eq!(row.status, Some("quarantined"));
}

#[test]
fn sent_row_kind_and_direction() {
    let msg = sample(MessageStatus::Sent, None);
    let row = message_sent(&msg);
    assert_eq!(row.kind, "email.message_sent");
    assert_eq!(row.direction, Some("outbound"));
    assert_eq!(row.status, Some("sent"));
}

#[test]
fn bounced_row_classifies_bounce_kind() {
    let tid = Uuid::new_v4();
    let mid = Uuid::new_v4();
    let row = message_bounced(tid, mid, BounceKind::Hard, Some("550 5.1.1"));
    assert_eq!(row.kind, "email.message_bounced");
    assert_eq!(row.bounce_kind, Some("hard"));
    assert_eq!(row.bounce_code.as_deref(), Some("550 5.1.1"));
    assert_eq!(row.message_id, Some(mid));
    assert_eq!(row.tenant_id, tid);
}

#[test]
fn dkim_rotated_row_has_both_old_and_new_keys() {
    let tid = Uuid::new_v4();
    let old = Uuid::new_v4();
    let new = Uuid::new_v4();
    let row = dkim_key_rotated(
        tid,
        Some(old),
        new,
        "cyberos",
        "rsa-2048",
        Some("admin@cyberos"),
    );
    assert_eq!(row.kind, "email.dkim_key_rotated");
    assert_eq!(row.old_key_id, Some(old));
    assert_eq!(row.new_key_id, Some(new));
    assert_eq!(row.dkim_selector.as_deref(), Some("cyberos"));
    assert_eq!(row.key_algorithm.as_deref(), Some("rsa-2048"));
    let by = row.rotated_by_subject_id_hash16.unwrap();
    assert_eq!(by.len(), 16);
}

#[test]
fn spam_band_classification_complete() {
    assert_eq!(spam_band(5.0), "5-7");
    assert_eq!(spam_band(6.99), "5-7");
    assert_eq!(spam_band(7.0), "7-10");
    assert_eq!(spam_band(10.0), "10+");
    assert_eq!(spam_band(20.0), "10+");
}

#[test]
fn hash16_is_case_insensitive_and_deterministic() {
    assert_eq!(hash16("alice@example.com"), hash16("ALICE@example.com  "));
    assert_eq!(hash16("alice@example.com").len(), 16);
    assert_ne!(hash16("alice@example.com"), hash16("bob@example.com"));
}

#[test]
fn body_sha256_hex_survives_into_audit_row() {
    let msg = sample(MessageStatus::Received, None);
    let row = message_received(&msg);
    assert_eq!(
        row.body_sha256_hex,
        Some("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9".into())
    );
}
