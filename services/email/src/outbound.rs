//! TASK-EMAIL-009 — outbound 1:1 send state machine.

use crate::audit::email_events::hash16;
use crate::delivery_auth::{sign_message, DkimMaterial, DkimOutcome};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sqlx::{FromRow, PgPool};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendStatus {
    Drafting,
    Queued,
    Sent,
    BouncedHard,
    BouncedSoft,
    Complaint,
    Suppressed,
}

impl SendStatus {
    pub const ALL: [Self; 7] = [
        Self::Drafting,
        Self::Queued,
        Self::Sent,
        Self::BouncedHard,
        Self::BouncedSoft,
        Self::Complaint,
        Self::Suppressed,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Drafting => "drafting",
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::BouncedHard => "bounced_hard",
            Self::BouncedSoft => "bounced_soft",
            Self::Complaint => "complaint",
            Self::Suppressed => "suppressed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressionReason {
    HardBounce,
    Complaint,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComposeRequest {
    pub tenant_id: Uuid,
    pub sender_subject_id: Uuid,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub in_reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftMessage {
    pub message_id: Uuid,
    pub tenant_id: Uuid,
    pub sender_subject_id: Uuid,
    pub recipients_hash16: Vec<String>,
    pub subject_sha256: String,
    pub body_sha256: String,
    pub status: SendStatus,
    pub confirm_token: Uuid,
    pub confirm_expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OutboundError {
    #[error("send requires a valid confirmation token")]
    ConfirmTokenInvalid,
    #[error("confirmation token expired")]
    ConfirmTokenExpired,
    #[error("recipient is suppressed: {0}")]
    RecipientSuppressed(String),
    #[error("rate limit exceeded")]
    RateLimitExceeded,
}

#[derive(Debug, thiserror::Error)]
pub enum PgOutboundError {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Policy(#[from] OutboundError),
    #[error("outbound message not found")]
    NotFound,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct OutboundMessageRow {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub sender_subject_id: Uuid,
    pub to_addresses: Vec<String>,
    pub cc_addresses: Vec<String>,
    pub bcc_addresses: Vec<String>,
    pub subject: String,
    pub body_text_sha256: String,
    pub body_html_sha256: Option<String>,
    pub in_reply_to: Option<String>,
    pub status: String,
    pub confirm_token_sha256: String,
    pub confirm_expires_at: DateTime<Utc>,
    pub dkim_outcome: Option<String>,
    pub queued_at: Option<DateTime<Utc>>,
    pub sent_at: Option<DateTime<Utc>>,
    pub last_bounce_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Default)]
pub struct SuppressionList {
    entries: HashMap<(Uuid, String), SuppressionReason>,
}

impl SuppressionList {
    pub fn suppress(&mut self, tenant_id: Uuid, address: &str, reason: SuppressionReason) {
        self.entries.insert((tenant_id, hash16(address)), reason);
    }

    pub fn is_suppressed(&self, tenant_id: Uuid, address: &str) -> bool {
        self.entries.contains_key(&(tenant_id, hash16(address)))
    }
}

#[derive(Default)]
pub struct SendRateLimiter {
    sends_by_hour: HashMap<(Uuid, i64), HashSet<Uuid>>,
}

impl SendRateLimiter {
    pub fn check_and_record(
        &mut self,
        sender_subject_id: Uuid,
        message_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<(), OutboundError> {
        let hour = now.timestamp() / 3600;
        let bucket = self
            .sends_by_hour
            .entry((sender_subject_id, hour))
            .or_default();
        if bucket.len() >= 100 && !bucket.contains(&message_id) {
            return Err(OutboundError::RateLimitExceeded);
        }
        bucket.insert(message_id);
        Ok(())
    }
}

pub fn compose(
    req: &ComposeRequest,
    suppressions: &SuppressionList,
    now: DateTime<Utc>,
) -> Result<DraftMessage, OutboundError> {
    for recipient in req.to.iter().chain(req.cc.iter()).chain(req.bcc.iter()) {
        if suppressions.is_suppressed(req.tenant_id, recipient) {
            return Err(OutboundError::RecipientSuppressed(hash16(recipient)));
        }
    }

    Ok(DraftMessage {
        message_id: Uuid::new_v4(),
        tenant_id: req.tenant_id,
        sender_subject_id: req.sender_subject_id,
        recipients_hash16: req
            .to
            .iter()
            .chain(req.cc.iter())
            .chain(req.bcc.iter())
            .map(|r| hash16(r))
            .collect(),
        subject_sha256: sha256_hex(&req.subject),
        body_sha256: sha256_hex(&req.body_text),
        status: SendStatus::Drafting,
        confirm_token: Uuid::new_v4(),
        confirm_expires_at: now + Duration::minutes(5),
    })
}

pub async fn compose_persisted(
    pool: &PgPool,
    req: &ComposeRequest,
    body_html: Option<&str>,
    confirm_token: Uuid,
    now: DateTime<Utc>,
) -> Result<OutboundMessageRow, PgOutboundError> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, req.tenant_id).await?;
    for recipient in req.to.iter().chain(req.cc.iter()).chain(req.bcc.iter()) {
        let suppressed: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM email_suppression_list
             WHERE tenant_id = $1 AND address_hash16 = $2 AND unsuppressed_at IS NULL",
        )
        .bind(req.tenant_id)
        .bind(hash16(recipient))
        .fetch_optional(&mut *tx)
        .await?;
        if suppressed.is_some() {
            return Err(OutboundError::RecipientSuppressed(hash16(recipient)).into());
        }
    }

    let row: OutboundMessageRow = sqlx::query_as(
        "INSERT INTO outbound_messages (
            tenant_id, sender_subject_id, to_addresses, cc_addresses, bcc_addresses,
            subject, body_text_sha256, body_html_sha256, in_reply_to,
            confirm_token_sha256, confirm_expires_at
         )
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
         RETURNING *",
    )
    .bind(req.tenant_id)
    .bind(req.sender_subject_id)
    .bind(&req.to)
    .bind(&req.cc)
    .bind(&req.bcc)
    .bind(&req.subject)
    .bind(sha256_hex(&req.body_text))
    .bind(body_html.map(sha256_hex))
    .bind(req.in_reply_to.as_deref())
    .bind(sha256_hex(&confirm_token.to_string()))
    .bind(now + Duration::minutes(5))
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(row)
}

pub async fn queue_persisted(
    pool: &PgPool,
    tenant_id: Uuid,
    message_id: Uuid,
    confirm_token: Uuid,
    dkim_outcome: DkimOutcome,
    trace_id: Option<&str>,
) -> Result<OutboundMessageRow, PgOutboundError> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let row: Option<OutboundMessageRow> =
        sqlx::query_as("SELECT * FROM outbound_messages WHERE tenant_id = $1 AND id = $2")
            .bind(tenant_id)
            .bind(message_id)
            .fetch_optional(&mut *tx)
            .await?;
    let row = row.ok_or(PgOutboundError::NotFound)?;
    if row.confirm_token_sha256 != sha256_hex(&confirm_token.to_string()) {
        return Err(OutboundError::ConfirmTokenInvalid.into());
    }
    if Utc::now() > row.confirm_expires_at {
        return Err(OutboundError::ConfirmTokenExpired.into());
    }
    let sent_count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM outbound_messages
         WHERE tenant_id = $1
           AND sender_subject_id = $2
           AND queued_at >= now() - interval '1 hour'",
    )
    .bind(tenant_id)
    .bind(row.sender_subject_id)
    .fetch_one(&mut *tx)
    .await?;
    if sent_count.0 >= 100 {
        return Err(OutboundError::RateLimitExceeded.into());
    }
    let updated: OutboundMessageRow = sqlx::query_as(
        "UPDATE outbound_messages
         SET status = 'queued', queued_at = now(), dkim_outcome = $3
         WHERE tenant_id = $1 AND id = $2
         RETURNING *",
    )
    .bind(tenant_id)
    .bind(message_id)
    .bind(dkim_outcome.as_str())
    .fetch_one(&mut *tx)
    .await?;
    insert_event_tx(
        &mut tx,
        tenant_id,
        message_id,
        "email.send_queued",
        serde_json::json!({"dkim_outcome": dkim_outcome.as_str()}),
        trace_id,
    )
    .await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn record_delivery_status(
    pool: &PgPool,
    tenant_id: Uuid,
    message_id: Uuid,
    status: SendStatus,
    primary_recipient: Option<&str>,
    trace_id: Option<&str>,
) -> Result<OutboundMessageRow, PgOutboundError> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let status_str = status.as_str();
    let updated: OutboundMessageRow = sqlx::query_as(
        "UPDATE outbound_messages
         SET status = $3,
             sent_at = CASE WHEN $3 = 'sent' THEN now() ELSE sent_at END,
             last_bounce_at = CASE WHEN $3 IN ('bounced_hard','bounced_soft','complaint') THEN now() ELSE last_bounce_at END
         WHERE tenant_id = $1 AND id = $2
         RETURNING *",
    )
    .bind(tenant_id)
    .bind(message_id)
    .bind(status_str)
    .fetch_one(&mut *tx)
    .await?;

    if matches!(status, SendStatus::BouncedHard | SendStatus::Complaint) {
        if let Some(recipient) = primary_recipient {
            let reason = if status == SendStatus::Complaint {
                "complaint"
            } else {
                "hard_bounce"
            };
            sqlx::query(
                "INSERT INTO email_suppression_list (
                    tenant_id, address_hash16, reason, source_message_id
                 )
                 VALUES ($1,$2,$3,$4)
                 ON CONFLICT (tenant_id, address_hash16) DO UPDATE SET
                    reason = EXCLUDED.reason,
                    source_message_id = EXCLUDED.source_message_id,
                    suppressed_at = now(),
                    unsuppressed_at = NULL",
            )
            .bind(tenant_id)
            .bind(hash16(recipient))
            .bind(reason)
            .bind(message_id)
            .execute(&mut *tx)
            .await?;
        }
    }
    let event_kind = match status {
        SendStatus::Sent => "email.send_delivered",
        SendStatus::BouncedHard | SendStatus::BouncedSoft => "email.send_bounced",
        SendStatus::Complaint => "email.send_complaint",
        SendStatus::Suppressed => "email.send_suppressed",
        SendStatus::Drafting | SendStatus::Queued => "email.send_queued",
    };
    insert_event_tx(
        &mut tx,
        tenant_id,
        message_id,
        event_kind,
        serde_json::json!({"status": status_str}),
        trace_id,
    )
    .await?;
    tx.commit().await?;
    Ok(updated)
}

pub async fn list_outbound(
    pool: &PgPool,
    tenant_id: Uuid,
    status: Option<SendStatus>,
    limit: i64,
) -> Result<Vec<OutboundMessageRow>, PgOutboundError> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    let rows: Vec<OutboundMessageRow> = sqlx::query_as(
        "SELECT * FROM outbound_messages
         WHERE tenant_id = $1
           AND ($2::text IS NULL OR status = $2)
         ORDER BY created_at DESC
         LIMIT $3",
    )
    .bind(tenant_id)
    .bind(status.map(|s| s.as_str()))
    .bind(limit.clamp(1, 500))
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(rows)
}

pub async fn unsuppress_address(
    pool: &PgPool,
    tenant_id: Uuid,
    address: &str,
) -> Result<(), PgOutboundError> {
    let mut tx = pool.begin().await?;
    set_tenant(&mut tx, tenant_id).await?;
    sqlx::query(
        "UPDATE email_suppression_list
         SET unsuppressed_at = now()
         WHERE tenant_id = $1 AND address_hash16 = $2 AND unsuppressed_at IS NULL",
    )
    .bind(tenant_id)
    .bind(hash16(address))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

pub fn queue_send(
    draft: &mut DraftMessage,
    confirm_token: Uuid,
    now: DateTime<Utc>,
    limiter: &mut SendRateLimiter,
    dkim_key: Option<&DkimMaterial>,
) -> Result<DkimOutcome, OutboundError> {
    if draft.confirm_token != confirm_token {
        return Err(OutboundError::ConfirmTokenInvalid);
    }
    if now > draft.confirm_expires_at {
        return Err(OutboundError::ConfirmTokenExpired);
    }
    limiter.check_and_record(draft.sender_subject_id, draft.message_id, now)?;
    let signed = sign_message(&draft.body_sha256, dkim_key);
    draft.status = SendStatus::Queued;
    Ok(signed.outcome)
}

pub fn handle_bounce(
    draft: &mut DraftMessage,
    hard: bool,
    primary_recipient: &str,
    suppressions: &mut SuppressionList,
) {
    if hard {
        draft.status = SendStatus::BouncedHard;
        suppressions.suppress(
            draft.tenant_id,
            primary_recipient,
            SuppressionReason::HardBounce,
        );
    } else {
        draft.status = SendStatus::BouncedSoft;
    }
}

pub fn handle_complaint(
    draft: &mut DraftMessage,
    primary_recipient: &str,
    suppressions: &mut SuppressionList,
) {
    draft.status = SendStatus::Complaint;
    suppressions.suppress(
        draft.tenant_id,
        primary_recipient,
        SuppressionReason::Complaint,
    );
}

#[derive(Debug, Clone, Serialize)]
pub struct OutboundAuditRow {
    pub kind: &'static str,
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub status: &'static str,
    pub trace_id: Option<String>,
}

pub fn audit_row(
    kind: &'static str,
    draft: &DraftMessage,
    trace_id: Option<&str>,
) -> OutboundAuditRow {
    OutboundAuditRow {
        kind,
        tenant_id: draft.tenant_id,
        message_id: draft.message_id,
        status: draft.status.as_str(),
        trace_id: trace_id.map(str::to_owned),
    }
}

fn sha256_hex(input: &str) -> String {
    let digest = sha2::Sha256::digest(input.as_bytes());
    let mut out = String::with_capacity(64);
    for b in digest {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

async fn insert_event_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    message_id: Uuid,
    event_kind: &str,
    payload: serde_json::Value,
    trace_id: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO outbound_delivery_events (tenant_id, message_id, event_kind, payload, trace_id)
         VALUES ($1,$2,$3,$4,$5)",
    )
    .bind(tenant_id)
    .bind(message_id)
    .bind(event_kind)
    .bind(payload)
    .bind(trace_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn set_tenant(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delivery_auth::DkimKeyKind;

    fn req() -> ComposeRequest {
        ComposeRequest {
            tenant_id: Uuid::new_v4(),
            sender_subject_id: Uuid::new_v4(),
            to: vec!["customer@example.com".into()],
            cc: vec![],
            bcc: vec![],
            subject: "Hello".into(),
            body_text: "Body".into(),
            in_reply_to: None,
        }
    }

    fn dkim_key(tenant_id: Uuid) -> DkimMaterial {
        DkimMaterial {
            tenant_id,
            selector: "cyberos".into(),
            domain: "example.com".into(),
            key_kind: DkimKeyKind::Ed25519,
            public_dns_txt: "v=DKIM1; p=abc".into(),
            signing_secret: Some("secret".into()),
        }
    }

    #[test]
    fn send_status_cardinality_is_seven() {
        assert_eq!(SendStatus::ALL.len(), 7);
    }

    #[test]
    fn compose_returns_five_minute_confirm_token() {
        let now = Utc::now();
        let draft = compose(&req(), &SuppressionList::default(), now).unwrap();
        assert_eq!(draft.status, SendStatus::Drafting);
        assert_eq!(draft.confirm_expires_at, now + Duration::minutes(5));
    }

    #[test]
    fn send_requires_valid_confirm_token() {
        let now = Utc::now();
        let mut draft = compose(&req(), &SuppressionList::default(), now).unwrap();
        let mut limiter = SendRateLimiter::default();
        let err = queue_send(&mut draft, Uuid::new_v4(), now, &mut limiter, None).unwrap_err();
        assert_eq!(err, OutboundError::ConfirmTokenInvalid);
    }

    #[test]
    fn valid_confirm_queues_and_dkim_signs() {
        let now = Utc::now();
        let mut draft = compose(&req(), &SuppressionList::default(), now).unwrap();
        let token = draft.confirm_token;
        let key = dkim_key(draft.tenant_id);
        let mut limiter = SendRateLimiter::default();
        let outcome = queue_send(&mut draft, token, now, &mut limiter, Some(&key)).unwrap();
        assert_eq!(draft.status, SendStatus::Queued);
        assert_eq!(outcome, DkimOutcome::SignedEd25519);
    }

    #[test]
    fn suppressed_recipient_blocks_compose() {
        let req = req();
        let mut suppressions = SuppressionList::default();
        suppressions.suppress(
            req.tenant_id,
            "customer@example.com",
            SuppressionReason::Manual,
        );
        let err = compose(&req, &suppressions, Utc::now()).unwrap_err();
        assert!(matches!(err, OutboundError::RecipientSuppressed(_)));
    }

    #[test]
    fn hard_bounce_and_complaint_suppress_recipient() {
        let req = req();
        let mut suppressions = SuppressionList::default();
        let mut draft = compose(&req, &suppressions, Utc::now()).unwrap();
        handle_bounce(&mut draft, true, "customer@example.com", &mut suppressions);
        assert_eq!(draft.status, SendStatus::BouncedHard);
        assert!(suppressions.is_suppressed(req.tenant_id, "customer@example.com"));

        let mut draft = compose(
            &ComposeRequest {
                to: vec!["other@example.com".into()],
                ..req.clone()
            },
            &suppressions,
            Utc::now(),
        )
        .unwrap();
        handle_complaint(&mut draft, "other@example.com", &mut suppressions);
        assert_eq!(draft.status, SendStatus::Complaint);
        assert!(suppressions.is_suppressed(req.tenant_id, "other@example.com"));
    }

    #[test]
    fn rate_limit_blocks_101st_distinct_message() {
        let now = Utc::now();
        let sender = Uuid::new_v4();
        let mut limiter = SendRateLimiter::default();
        for _ in 0..100 {
            limiter
                .check_and_record(sender, Uuid::new_v4(), now)
                .unwrap();
        }
        let err = limiter
            .check_and_record(sender, Uuid::new_v4(), now)
            .unwrap_err();
        assert_eq!(err, OutboundError::RateLimitExceeded);
    }
}
