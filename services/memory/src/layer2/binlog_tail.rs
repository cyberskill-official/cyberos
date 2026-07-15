//! Polls Layer-1's append-only audit log for new rows.
//!
//! Strategy (TASK-MEMORY-101 §1 #5 — 1s p95 lag floor):
//!   * Per-tenant cursor → `SELECT * FROM l1_audit_log WHERE tenant_id=$1
//!     AND seq > $cursor ORDER BY seq ASC LIMIT $batch_size`.
//!   * 200ms poll interval (configurable via `MEMORY_TAIL_POLL_MS`).
//!   * On batch boundary, advance the cursor in `l2_ingest_cursor` (cursor.rs).
//!
//! The simple polling design is intentional — Postgres LISTEN/NOTIFY would
//! cut lag but requires a long-lived connection per tenant; polling keeps
//! ops simple while still satisfying the 1s p95 lag target on tenants with
//! < 10k writes/min.

use cyberos_types::TenantId;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;

/// A single Layer-1 row as observed during a tail poll.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L1Row {
    pub seq: i64,
    pub tenant_id: uuid::Uuid,
    pub subject_id: Option<uuid::Uuid>,
    pub op: String,
    pub path: String,
    pub body: Option<String>,
    pub prev_hash_hex: Option<String>,
    pub chain_anchor_hex: String,
    pub ts_ns: i64,
}

/// Default poll interval — overridable via `MEMORY_TAIL_POLL_MS` env var.
pub fn default_poll_interval() -> Duration {
    let ms: u64 = std::env::var("MEMORY_TAIL_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200);
    Duration::from_millis(ms)
}

/// Pull the next batch of rows for `tenant` strictly after `after_seq`.
/// Returns up to `batch_size` rows in seq-ascending order.
pub async fn poll(
    pool: &PgPool,
    tenant: TenantId,
    after_seq: i64,
    batch_size: i32,
) -> Result<Vec<L1Row>, sqlx::Error> {
    let rows: Vec<(
        i64,
        uuid::Uuid,
        Option<uuid::Uuid>,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        i64,
    )> = sqlx::query_as(
        "SELECT seq, tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns
             FROM l1_audit_log
            WHERE tenant_id = $1 AND seq > $2
         ORDER BY seq ASC
            LIMIT $3",
    )
    .bind(tenant.as_uuid())
    .bind(after_seq)
    .bind(batch_size)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(
            |(
                seq,
                tenant_id,
                subject_id,
                op,
                path,
                body,
                prev_hash_hex,
                chain_anchor_hex,
                ts_ns,
            )| L1Row {
                seq,
                tenant_id,
                subject_id,
                op,
                path,
                body,
                prev_hash_hex,
                chain_anchor_hex,
                ts_ns,
            },
        )
        .collect())
}

/// Append an L1 row (used by the memory-sync daemon when it lands in the
/// Cloud-memory path, and by tests). Returns the assigned seq.
pub async fn append(pool: &PgPool, row: &L1Row) -> Result<i64, sqlx::Error> {
    let (seq,): (i64,) = sqlx::query_as(
        "INSERT INTO l1_audit_log
                (tenant_id, subject_id, op, path, body, prev_hash_hex, chain_anchor_hex, ts_ns)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING seq",
    )
    .bind(row.tenant_id)
    .bind(row.subject_id)
    .bind(&row.op)
    .bind(&row.path)
    .bind(row.body.as_deref())
    .bind(row.prev_hash_hex.as_deref())
    .bind(&row.chain_anchor_hex)
    .bind(row.ts_ns)
    .fetch_one(pool)
    .await?;
    Ok(seq)
}
