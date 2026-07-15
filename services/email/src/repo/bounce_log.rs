//! TASK-EMAIL-001 §1 #17 — append-only bounce log writer.

use crate::errors::EmailResult;
use crate::types::{BounceEvent, BounceKind};
use sqlx::PgPool;
use uuid::Uuid;

pub struct NewBounce<'a> {
    pub tenant_id: Uuid,
    pub message_id: Uuid,
    pub bounce_kind: BounceKind,
    pub bounce_reason: &'a str,
    pub bounce_code: Option<&'a str>,
    pub remote_peer: Option<&'a str>,
}

/// Insert a bounce event. The SQL grant revokes UPDATE+DELETE so all
/// mutations are pure inserts.
pub async fn record(db: &PgPool, b: &NewBounce<'_>) -> EmailResult<BounceEvent> {
    let row: BounceEvent = sqlx::query_as(
        "INSERT INTO bounce_log
            (tenant_id, message_id, bounce_kind, bounce_reason, bounce_code, remote_peer)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *",
    )
    .bind(b.tenant_id)
    .bind(b.message_id)
    .bind(b.bounce_kind.as_str())
    .bind(b.bounce_reason)
    .bind(b.bounce_code)
    .bind(b.remote_peer)
    .fetch_one(db)
    .await?;
    Ok(row)
}

/// Compute the rolling-24h bounce rate for a tenant per §1 #17. The
/// alarm threshold is > 1%; OTel exporter consumes this.
pub async fn bounce_rate_24h(db: &PgPool, tenant_id: Uuid) -> EmailResult<f64> {
    let (bounce_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM bounce_log
         WHERE tenant_id = $1
           AND ts >= now() - INTERVAL '24 hours'",
    )
    .bind(tenant_id)
    .fetch_one(db)
    .await?;

    let (sent_count,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM message_metadata
         WHERE tenant_id = $1
           AND direction = 'outbound'::message_direction
           AND received_at >= now() - INTERVAL '24 hours'",
    )
    .bind(tenant_id)
    .fetch_one(db)
    .await?;

    if sent_count == 0 {
        return Ok(0.0);
    }
    Ok((bounce_count as f64) / (sent_count as f64))
}
