//! Per-tenant Layer-2 ingest cursor (DEC-073).
//!
//! Reads + writes `l2_ingest_cursor`. The ingest worker calls `load` at
//! startup for every tenant it owns, then `advance` after each successful
//! batch. On crash, the next start re-reads the same cursor and resumes
//! from `last_seq + 1`.

use cyberos_types::TenantId;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// A single tenant's ingest cursor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    /// Tenant this cursor belongs to.
    pub tenant_id: TenantId,
    /// Highest Layer-1 seq materialized into l2_memory for this tenant.
    pub last_seq: i64,
    /// Chain anchor of the row at `last_seq` (used for cross-check).
    pub last_chain_anchor_hex: Option<String>,
    /// Observed end-to-end lag in milliseconds for the last batch.
    /// Per §1 #5, the p95 floor target is 1,000 ms.
    pub last_lag_ms: i64,
}

/// Repository abstraction for the cursor. Backed by Postgres in prod;
/// in tests we can swap with an in-memory map.
#[async_trait::async_trait]
pub trait CursorStore: Send + Sync {
    /// Load the cursor for `tenant`. Returns `last_seq = 0` if the tenant
    /// has never had an ingest run.
    async fn load(&self, tenant: TenantId) -> Result<Cursor, sqlx::Error>;

    /// Advance the cursor. Records a history row for forensics.
    async fn advance(
        &self,
        tenant: TenantId,
        from_seq: i64,
        to_seq: i64,
        batch_rows: i32,
        batch_duration_ms: i64,
        new_chain_anchor_hex: Option<String>,
    ) -> Result<(), sqlx::Error>;
}

/// Postgres-backed implementation.
pub struct PgCursorStore {
    pub pool: PgPool,
}

#[async_trait::async_trait]
impl CursorStore for PgCursorStore {
    async fn load(&self, tenant: TenantId) -> Result<Cursor, sqlx::Error> {
        let row: Option<(i64, Option<String>, i64)> = sqlx::query_as(
            "SELECT last_seq, encode(last_chain_anchor, 'hex'), last_lag_ms
                 FROM l2_ingest_cursor
                WHERE tenant_id = $1",
        )
        .bind(tenant.as_uuid())
        .fetch_optional(&self.pool)
        .await?;

        Ok(match row {
            Some((seq, anchor, lag)) => Cursor {
                tenant_id: tenant,
                last_seq: seq,
                last_chain_anchor_hex: anchor,
                last_lag_ms: lag,
            },
            None => Cursor {
                tenant_id: tenant,
                last_seq: 0,
                last_chain_anchor_hex: None,
                last_lag_ms: 0,
            },
        })
    }

    async fn advance(
        &self,
        tenant: TenantId,
        from_seq: i64,
        to_seq: i64,
        batch_rows: i32,
        batch_duration_ms: i64,
        new_chain_anchor_hex: Option<String>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        sqlx::query(
            "INSERT INTO l2_ingest_cursor
                    (tenant_id, last_seq, last_chain_anchor, last_lag_ms, last_updated_at)
             VALUES ($1, $2, decode($3, 'hex'), $4, NOW())
             ON CONFLICT (tenant_id)
             DO UPDATE SET
                last_seq          = EXCLUDED.last_seq,
                last_chain_anchor = EXCLUDED.last_chain_anchor,
                last_lag_ms       = EXCLUDED.last_lag_ms,
                last_updated_at   = NOW()",
        )
        .bind(tenant.as_uuid())
        .bind(to_seq)
        .bind(new_chain_anchor_hex.as_deref())
        .bind(batch_duration_ms)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO l2_ingest_cursor_history
                    (tenant_id, from_seq, to_seq, batch_rows, batch_duration_ms)
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(tenant.as_uuid())
        .bind(from_seq)
        .bind(to_seq)
        .bind(batch_rows)
        .bind(batch_duration_ms)
        .execute(&mut *tx)
        .await?;

        tx.commit().await
    }
}
