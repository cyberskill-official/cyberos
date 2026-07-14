//! TASK-EMAIL-001 §4 #13 + §4 #19 — quarantine threshold + body size guard.

use cyberos_email::stalwart_adapter::inbound::{sha256_hex, MemoryBlobStore};
// `BlobStore` trait must be in scope for `bs.put(...)` to resolve — the
// method lives on the trait, not the struct, so the impl-block doesn't
// import the symbol implicitly.
use cyberos_email::stalwart_adapter::inbound::BlobStore;
use cyberos_email::types::MessageStatus;

#[test]
fn spam_below_threshold_is_received() {
    assert_eq!(MessageStatus::from_spam_score(0.0), MessageStatus::Received);
    assert_eq!(
        MessageStatus::from_spam_score(4.999),
        MessageStatus::Received
    );
}

#[test]
fn spam_at_or_above_threshold_is_quarantined() {
    assert_eq!(
        MessageStatus::from_spam_score(5.0),
        MessageStatus::Quarantined
    );
    assert_eq!(
        MessageStatus::from_spam_score(7.3),
        MessageStatus::Quarantined
    );
    assert_eq!(
        MessageStatus::from_spam_score(15.0),
        MessageStatus::Quarantined
    );
}

#[tokio::test]
async fn memory_blob_store_round_trip_records_body() {
    let bs = MemoryBlobStore::new();
    bs.put(
        "cyberos-email-vn-1-bodies",
        "tenant/8472/abc",
        "alias/cyberos-email-vn-1-bodies",
        b"the body",
    )
    .await
    .unwrap();

    assert!(
        bs.head_object("cyberos-email-vn-1-bodies", "tenant/8472/abc")
            .await
    );
    let body = bs
        .body_for("cyberos-email-vn-1-bodies", "tenant/8472/abc")
        .await;
    assert_eq!(body.as_deref(), Some(&b"the body"[..]));

    // The same key in a different bucket DOES NOT exist (proof of bucket isolation).
    assert!(
        !bs.head_object("cyberos-email-eu-1-bodies", "tenant/8472/abc")
            .await
    );
}

#[test]
fn body_sha256_hex_is_64_hex_chars() {
    // Capital-T "The" — matches the canonical RFC-3174 / NIST test vector
    // for SHA-256. Using lowercase "the" produces a different hash.
    let h = sha256_hex(b"The quick brown fox jumps over the lazy dog");
    assert_eq!(h.len(), 64);
    assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    assert_eq!(
        h,
        "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"
    );
}
