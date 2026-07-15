//! HTTP-facing orchestration for TASK-EMAIL-009.

use crate::delivery_auth::DkimOutcome;
use crate::outbound::{
    compose_persisted, list_outbound, queue_persisted, record_delivery_status, unsuppress_address,
    ComposeRequest, OutboundMessageRow, PgOutboundError, SendStatus,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct ComposeHttpRequest {
    pub tenant_id: Uuid,
    pub sender_subject_id: Uuid,
    pub to: Vec<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub in_reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComposeHttpResponse {
    pub message: OutboundMessageRow,
    pub confirm_token: Uuid,
}

pub async fn compose(
    db: &PgPool,
    req: ComposeHttpRequest,
) -> Result<ComposeHttpResponse, PgOutboundError> {
    let confirm_token = Uuid::new_v4();
    let domain_req = ComposeRequest {
        tenant_id: req.tenant_id,
        sender_subject_id: req.sender_subject_id,
        to: req.to,
        cc: req.cc.unwrap_or_default(),
        bcc: req.bcc.unwrap_or_default(),
        subject: req.subject,
        body_text: req.body_text,
        in_reply_to: req.in_reply_to,
    };
    let message = compose_persisted(
        db,
        &domain_req,
        req.body_html.as_deref(),
        confirm_token,
        Utc::now(),
    )
    .await?;
    Ok(ComposeHttpResponse {
        message,
        confirm_token,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendHttpRequest {
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub confirm_token: Uuid,
    pub dkim_outcome: Option<DkimOutcome>,
    pub trace_id: Option<String>,
}

pub async fn send(
    db: &PgPool,
    req: SendHttpRequest,
) -> Result<OutboundMessageRow, PgOutboundError> {
    queue_persisted(
        db,
        req.tenant_id,
        req.message_id,
        req.confirm_token,
        req.dkim_outcome.unwrap_or(DkimOutcome::SignFailedNoKey),
        req.trace_id.as_deref(),
    )
    .await
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeliveryStatusRequest {
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub status: SendStatus,
    pub primary_recipient: Option<String>,
    pub trace_id: Option<String>,
}

pub async fn delivery_status(
    db: &PgPool,
    req: DeliveryStatusRequest,
) -> Result<OutboundMessageRow, PgOutboundError> {
    record_delivery_status(
        db,
        req.tenant_id,
        req.message_id,
        req.status,
        req.primary_recipient.as_deref(),
        req.trace_id.as_deref(),
    )
    .await
}

pub async fn list(
    db: &PgPool,
    tenant_id: Uuid,
    status: Option<SendStatus>,
    limit: Option<i64>,
) -> Result<Vec<OutboundMessageRow>, PgOutboundError> {
    list_outbound(db, tenant_id, status, limit.unwrap_or(50)).await
}

#[derive(Debug, Clone, Deserialize)]
pub struct UnsuppressRequest {
    pub tenant_id: Uuid,
    pub address: String,
}

pub async fn unsuppress(db: &PgPool, req: UnsuppressRequest) -> Result<(), PgOutboundError> {
    unsuppress_address(db, req.tenant_id, &req.address).await
}
