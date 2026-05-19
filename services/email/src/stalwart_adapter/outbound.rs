//! FR-EMAIL-001 §3.6 — outbound adapter.
//!
//! Slice 1 records the outbound message metadata; the actual SMTP submit
//! is performed by Stalwart's outbound queue when the JWT bridge plugin
//! (FR-EMAIL-002) lands. This module exposes the shape so callers can
//! prepare the metadata + emit the memory audit row.

use crate::audit::email_events;
use crate::errors::{EmailError, EmailResult};
use crate::repo::messages::{self, NewMessage};
use crate::residency;
use crate::types::{EmailMessage, MessageDirection, MessageStatus};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use uuid::Uuid;

use super::inbound::{sha256_hex, BlobStore};

#[derive(Debug, Clone, Deserialize)]
pub struct OutboundRequest {
    pub tenant_id: Uuid,
    pub thread_id: String,
    pub from_address: String,
    pub to_addresses: Vec<String>,
    #[serde(default)]
    pub cc_addresses: Vec<String>,
    #[serde(default)]
    pub bcc_addresses: Vec<String>,
    pub subject: Option<String>,
    pub body_bytes: Vec<u8>,
    /// Tenant DKIM selector to sign with (defaults to "cyberos").
    #[serde(default = "default_selector")]
    pub dkim_selector: String,
}

fn default_selector() -> String {
    "cyberos".to_owned()
}

pub async fn on_outbound(
    db: &sqlx::PgPool,
    blob: &dyn BlobStore,
    req: OutboundRequest,
) -> EmailResult<(EmailMessage, email_events::EmailAuditRow)> {
    if req.body_bytes.is_empty() || req.body_bytes.len() > 26_214_400 {
        return Err(EmailError::BodyTooLarge(req.body_bytes.len()));
    }

    // §1 #15 — outbound MUST be DKIM-signed. Verify the tenant has an
    // active DKIM key for the selector before allowing the submission.
    let active_key: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM dkim_keys WHERE tenant_id = $1 AND dkim_selector = $2 AND status = 'active'",
    )
    .bind(req.tenant_id)
    .bind(&req.dkim_selector)
    .fetch_optional(db)
    .await?;

    if active_key.is_none() {
        return Err(EmailError::DkimKeyNotFound(
            req.tenant_id,
            req.dkim_selector,
        ));
    }

    let body_sha256_hex = sha256_hex(&req.body_bytes);

    let binding = residency::resolve(req.tenant_id, db).await?;
    let stalwart_id: i64 = next_stalwart_id();
    let s3_key = format!(
        "{tid}/{sid}/{sha}",
        tid = req.tenant_id,
        sid = stalwart_id,
        sha = body_sha256_hex
    );
    residency::assert_residency_match(req.tenant_id, &binding, &binding.bucket)?;
    blob.put(
        &binding.bucket,
        &s3_key,
        &binding.kms_key_id,
        &req.body_bytes,
    )
    .await?;

    let now: DateTime<Utc> = Utc::now();

    let subject_normalised = messages::normalise_subject(req.subject.as_deref());
    messages::upsert_thread(
        db,
        req.tenant_id,
        &req.thread_id,
        subject_normalised.as_deref(),
        now,
        std::slice::from_ref(&req.from_address),
    )
    .await?;

    let msg = messages::insert_message(
        db,
        &NewMessage {
            tenant_id: req.tenant_id,
            stalwart_message_id: stalwart_id,
            thread_id: &req.thread_id,
            direction: MessageDirection::Outbound,
            from_address: &req.from_address,
            to_addresses: &req.to_addresses,
            cc_addresses: &req.cc_addresses,
            bcc_addresses: &req.bcc_addresses,
            subject: req.subject.as_deref(),
            received_at: now,
            s3_body_key: &s3_key,
            s3_body_kms_key_id: &binding.kms_key_id,
            body_sha256_hex: &body_sha256_hex,
            byte_size: req.body_bytes.len() as i64,
            status: MessageStatus::Sent,
            prior_message_id: None,
            spam_score: None,
            dkim_pass: Some(true),
            spf_pass: None,
            dmarc_pass: None,
            bimi_present: None,
        },
    )
    .await?;

    let audit_row = email_events::message_sent(&msg);
    Ok((msg, audit_row))
}

/// Slice-1 stub for the Stalwart message id. Real wiring delegates to
/// Stalwart's submission API which returns the canonical id.
fn next_stalwart_id() -> i64 {
    use std::sync::atomic::{AtomicI64, Ordering};
    static COUNTER: AtomicI64 = AtomicI64::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
