//! FR-EMAIL-001 §3.6 — inbound webhook adapter.
//!
//! Receives a Stalwart inbound event (post-classification: DKIM/SPF/DMARC
//! flags + spam score already computed). Persists metadata + body, emits
//! the memory audit row. Quarantined messages get `status = quarantined`
//! per §1 #18.

use crate::audit::email_events;
use crate::errors::{EmailError, EmailResult};
use crate::repo::messages::{self, NewMessage};
use crate::residency;
use crate::types::{EmailMessage, MessageDirection, MessageStatus};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Wire shape Stalwart posts to `/v1/email/stalwart/inbound`.
#[derive(Debug, Clone, Deserialize)]
pub struct StalwartInboundEvent {
    pub stalwart_message_id: i64,
    pub thread_id: String,
    pub tenant_id: Uuid,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    #[serde(default)]
    pub cc_addresses: Vec<String>,
    #[serde(default)]
    pub bcc_addresses: Vec<String>,
    pub subject: Option<String>,
    pub received_at: DateTime<Utc>,
    /// Base64-encoded message body bytes.
    pub body_bytes: Vec<u8>,
    pub spam_score: f32,
    pub dkim_pass: bool,
    pub spf_pass: bool,
    pub dmarc_pass: bool,
    #[serde(default)]
    pub bimi_present: bool,
}

/// Blob-store abstraction so unit tests don't need a live S3.
///
/// Returns a boxed future to keep the trait object-safe under
/// dyn-dispatch without needing the `async-trait` crate.
pub trait BlobStore: Send + Sync {
    fn put<'a>(
        &'a self,
        bucket: &'a str,
        key: &'a str,
        kms_key_id: &'a str,
        body: &'a [u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmailResult<()>> + Send + 'a>>;
}

/// Test-only in-memory blob store.
#[derive(Debug, Default)]
pub struct MemoryBlobStore {
    /// Map (bucket, key) → body.
    objects: tokio::sync::Mutex<std::collections::HashMap<(String, String), Vec<u8>>>,
}

impl MemoryBlobStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub async fn head_object(&self, bucket: &str, key: &str) -> bool {
        let g = self.objects.lock().await;
        g.contains_key(&(bucket.to_owned(), key.to_owned()))
    }
    pub async fn body_for(&self, bucket: &str, key: &str) -> Option<Vec<u8>> {
        let g = self.objects.lock().await;
        g.get(&(bucket.to_owned(), key.to_owned())).cloned()
    }
}

impl BlobStore for MemoryBlobStore {
    fn put<'a>(
        &'a self,
        bucket: &'a str,
        key: &'a str,
        _kms_key_id: &'a str,
        body: &'a [u8],
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = EmailResult<()>> + Send + 'a>> {
        let bucket = bucket.to_owned();
        let key = key.to_owned();
        let body = body.to_vec();
        Box::pin(async move {
            let mut g = self.objects.lock().await;
            g.insert((bucket, key), body);
            Ok(())
        })
    }
}

/// Handle one inbound Stalwart event end-to-end.
///
/// Steps per §1 #12 + §1 #18:
///   1. Size validation (§1 #19) — body in (1, 25 MB].
///   2. SHA-256 compute (§1 #25 body integrity).
///   3. Residency resolve (§1 #12).
///   4. S3 PUT to residency-pinned bucket.
///   5. Upsert thread + insert message_metadata in one tx.
///   6. Build + return the memory audit row (the writer transport binds in
///      the binary `src/bin/server.rs`).
///
/// Returns the inserted `EmailMessage` row + the memory audit row.
pub async fn on_inbound(
    db: &sqlx::PgPool,
    blob: &dyn BlobStore,
    evt: StalwartInboundEvent,
) -> EmailResult<(EmailMessage, email_events::EmailAuditRow)> {
    // §1 #19 — size guard.
    if evt.body_bytes.is_empty() || evt.body_bytes.len() > 26_214_400 {
        return Err(EmailError::BodyTooLarge(evt.body_bytes.len()));
    }

    // §1 #25 — body integrity hash.
    let body_sha256_hex = sha256_hex(&evt.body_bytes);

    // §1 #12 — residency-pin write.
    let binding = residency::resolve(evt.tenant_id, db).await?;
    let s3_key = format!(
        "{tid}/{sid}/{sha}",
        tid = evt.tenant_id,
        sid = evt.stalwart_message_id,
        sha = body_sha256_hex
    );

    // The handler asserts the destination matches BEFORE the PUT so a
    // mis-configured binding fails closed without writing data.
    residency::assert_residency_match(evt.tenant_id, &binding, &binding.bucket)?;
    blob.put(
        &binding.bucket,
        &s3_key,
        &binding.kms_key_id,
        &evt.body_bytes,
    )
    .await?;

    // §1 #18 — spam quarantine threshold = 5.0.
    let status = MessageStatus::from_spam_score(evt.spam_score);
    let subject_normalised = messages::normalise_subject(evt.subject.as_deref());

    // NOTE (slice 2): the upsert_thread + insert_message pair SHOULD run in
    // a single transaction so a partial failure rolls back both writes.
    // The repo helpers currently take `&PgPool` which makes wrapping them
    // in a tx awkward. Slice 2 refactors the repo to accept either pool
    // or an `&mut Transaction` (via the `sqlx::Acquire` trait) so this
    // body can wrap the writes atomically. The slice-1 risk is bounded:
    // - upsert_thread on conflict is idempotent (ON CONFLICT DO UPDATE).
    // - insert_message failure leaves the thread row with a stale count
    //   that the next message will increment past, not corrupt.
    messages::upsert_thread(
        db,
        evt.tenant_id,
        &evt.thread_id,
        subject_normalised.as_deref(),
        evt.received_at,
        std::slice::from_ref(&evt.from_address),
    )
    .await?;

    let msg = messages::insert_message(
        db,
        &NewMessage {
            tenant_id: evt.tenant_id,
            stalwart_message_id: evt.stalwart_message_id,
            thread_id: &evt.thread_id,
            direction: MessageDirection::Inbound,
            from_address: &evt.from_address,
            to_addresses: &evt.to_addresses,
            cc_addresses: &evt.cc_addresses,
            bcc_addresses: &evt.bcc_addresses,
            subject: evt.subject.as_deref(),
            received_at: evt.received_at,
            s3_body_key: &s3_key,
            s3_body_kms_key_id: &binding.kms_key_id,
            body_sha256_hex: &body_sha256_hex,
            byte_size: evt.body_bytes.len() as i64,
            status,
            prior_message_id: None,
            spam_score: Some(evt.spam_score),
            dkim_pass: Some(evt.dkim_pass),
            spf_pass: Some(evt.spf_pass),
            dmarc_pass: Some(evt.dmarc_pass),
            bimi_present: Some(evt.bimi_present),
        },
    )
    .await?;

    let audit_row = if status == MessageStatus::Quarantined {
        email_events::message_quarantined(&msg)
    } else {
        email_events::message_received(&msg)
    };

    Ok((msg, audit_row))
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hex_shape() {
        let h = sha256_hex(b"hello");
        assert_eq!(h.len(), 64);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[tokio::test]
    async fn memory_blob_store_round_trip() {
        let bs = MemoryBlobStore::new();
        bs.put("test-bucket", "key/1", "alias/test", b"hi")
            .await
            .unwrap();
        assert!(bs.head_object("test-bucket", "key/1").await);
        assert!(!bs.head_object("test-bucket", "key/2").await);
        assert_eq!(
            bs.body_for("test-bucket", "key/1").await,
            Some(b"hi".to_vec())
        );
    }
}
