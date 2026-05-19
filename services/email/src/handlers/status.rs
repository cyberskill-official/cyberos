//! FR-EMAIL-001 §1 #19 — health + per-message status + list endpoints.

use crate::errors::EmailResult;
use crate::types::EmailMessage;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub stalwart_status: &'static str,
    pub last_message_received_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_message_sent_at: Option<chrono::DateTime<chrono::Utc>>,
    pub postgres_status: &'static str,
    pub s3_status: &'static str,
    pub registered_tenants: i64,
}

pub async fn healthz(db: &PgPool) -> EmailResult<HealthResponse> {
    let postgres_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(db)
        .await
        .is_ok();

    let last_received: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT max(received_at) FROM message_metadata WHERE direction = 'inbound'::message_direction",
    )
    .fetch_optional(db)
    .await?
    .flatten();
    let last_sent: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT max(received_at) FROM message_metadata WHERE direction = 'outbound'::message_direction",
    )
    .fetch_optional(db)
    .await?
    .flatten();
    let registered: i64 = sqlx::query_scalar("SELECT count(*) FROM tenant_residency")
        .fetch_one(db)
        .await
        .unwrap_or(0);

    Ok(HealthResponse {
        stalwart_status: "external",      // wired at slice 2 via FR-EMAIL-002
        last_message_received_at: last_received,
        last_message_sent_at: last_sent,
        postgres_status: if postgres_ok { "ok" } else { "degraded" },
        s3_status: "external",            // wired by the deployment env
        registered_tenants: registered,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageStatusResponse {
    pub id: Uuid,
    pub status: String,
    pub dkim_pass: Option<bool>,
    pub spf_pass: Option<bool>,
    pub dmarc_pass: Option<bool>,
    pub bimi_present: Option<bool>,
    pub bounces: Vec<crate::types::BounceEvent>,
}

pub async fn message_status(db: &PgPool, id: Uuid) -> EmailResult<Option<MessageStatusResponse>> {
    let msg: Option<EmailMessage> = sqlx::query_as("SELECT * FROM message_metadata WHERE id = $1")
        .bind(id)
        .fetch_optional(db)
        .await?;

    let Some(msg) = msg else { return Ok(None) };

    let bounces: Vec<crate::types::BounceEvent> =
        sqlx::query_as("SELECT * FROM bounce_log WHERE message_id = $1 ORDER BY ts ASC")
            .bind(id)
            .fetch_all(db)
            .await?;

    Ok(Some(MessageStatusResponse {
        id: msg.id,
        status: match msg.status {
            crate::types::MessageStatus::Received => "received".into(),
            crate::types::MessageStatus::Quarantined => "quarantined".into(),
            crate::types::MessageStatus::Delivered => "delivered".into(),
            crate::types::MessageStatus::Sent => "sent".into(),
            crate::types::MessageStatus::Bounced => "bounced".into(),
            crate::types::MessageStatus::Dropped => "dropped".into(),
        },
        dkim_pass: msg.dkim_pass,
        spf_pass: msg.spf_pass,
        dmarc_pass: msg.dmarc_pass,
        bimi_present: msg.bimi_present,
        bounces,
    }))
}
