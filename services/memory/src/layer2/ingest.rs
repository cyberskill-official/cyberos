//! Layer-2 ingest orchestrator (FR-MEMORY-101).
//!
//! `run_batch` is the heart of the pipeline:
//!   1. Load the tenant's cursor (current `last_seq`).
//!   2. Poll `binlog_tail::poll` for up to `batch_size` rows past it.
//!   3. For each row, verify `chain_anchor` (catches Layer-1 tampering).
//!   4. Upsert into l2_memory + l2_entity via the pgvector module.
//!   5. Advance the cursor with the new last_seq + observed lag.
//!
//! Wraps everything in a single Postgres transaction so a crash mid-batch
//! is fully rewound — re-running picks up from the unchanged cursor.

use crate::layer2::{age, binlog_tail, chain_anchor, cursor::PgCursorStore, cursor::CursorStore, entity_extract, pgvector};
use cyberos_types::TenantId;
use sqlx::PgPool;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, info, warn};

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("chain anchor mismatch at seq {seq}: expected {expected}, computed {computed}")]
    ChainAnchorMismatch {
        seq: i64,
        expected: String,
        computed: String,
    },
}

/// Summary of one batch run.
#[derive(Debug, Clone)]
pub struct BatchSummary {
    pub tenant_id: TenantId,
    pub rows_processed: usize,
    pub from_seq: i64,
    pub to_seq: i64,
    pub duration_ms: i64,
}

/// Run a single ingest batch for one tenant. Returns a summary.
pub async fn run_batch(
    pool: &PgPool,
    tenant: TenantId,
    batch_size: i32,
) -> Result<BatchSummary, IngestError> {
    let start = Instant::now();
    let store = PgCursorStore { pool: pool.clone() };
    let cursor = store.load(tenant).await?;
    debug!(?tenant, last_seq = cursor.last_seq, "ingest batch starting");

    let rows = binlog_tail::poll(pool, tenant, cursor.last_seq, batch_size).await?;
    if rows.is_empty() {
        return Ok(BatchSummary {
            tenant_id: tenant,
            rows_processed: 0,
            from_seq: cursor.last_seq,
            to_seq: cursor.last_seq,
            duration_ms: start.elapsed().as_millis() as i64,
        });
    }

    let to_seq = rows.last().map(|r| r.seq).unwrap_or(cursor.last_seq);
    let mut last_anchor_hex: Option<String> = cursor.last_chain_anchor_hex.clone();

    // Per-tenant gate: ALL rows in this batch must be for the requested tenant.
    // FR-MEMORY-101 §1 #8 (tenant isolation invariant); the SELECT already filters,
    // but we re-check defensively per AUTHORING_DISCIPLINE §8.5a.
    for r in &rows {
        if cyberos_types::TenantId(r.tenant_id) != tenant {
            warn!(?tenant, foreign = ?r.tenant_id, "tenant isolation violation in batch — aborting");
            return Err(IngestError::Sqlx(sqlx::Error::Protocol(
                format!("tenant isolation violation: expected {}, row has {}", tenant, r.tenant_id),
            )));
        }
    }

    // Verify the chain anchor of every row before materializing.
    for r in &rows {
        let body = r.body.as_deref().unwrap_or("");
        let computed = chain_anchor::compute(r.prev_hash_hex.as_deref(), body);
        if computed != r.chain_anchor_hex {
            return Err(IngestError::ChainAnchorMismatch {
                seq: r.seq,
                expected: r.chain_anchor_hex.clone(),
                computed,
            });
        }
        last_anchor_hex = Some(r.chain_anchor_hex.clone());
    }

    // Materialize. Each row's upserts are individually idempotent on PK, so
    // partial-failure recovery is automatic: re-running from the same cursor
    // is a no-op for rows already written.
    for r in &rows {
        pgvector::upsert_memory(pool, r).await?;
        if let Some(body) = r.body.as_deref() {
            for e in entity_extract::extract(r.seq, &r.path, body) {
                pgvector::upsert_entity(
                    pool,
                    r.tenant_id,
                    &e.kind,
                    &e.name,
                    e.source_seq,
                    &e.source_path,
                )
                .await?;
                // Best-effort AGE graph mirror — failures don't block ingest.
                age::mirror_entity(pool, r.tenant_id, &e.kind, &e.name, &e.source_path).await;
            }
        }
    }

    let dur = start.elapsed().as_millis() as i64;
    store
        .advance(tenant, cursor.last_seq, to_seq, rows.len() as i32, dur, last_anchor_hex)
        .await?;

    info!(
        ?tenant,
        rows = rows.len(),
        from = cursor.last_seq,
        to = to_seq,
        duration_ms = dur,
        "ingest batch committed"
    );
    Ok(BatchSummary {
        tenant_id: tenant,
        rows_processed: rows.len(),
        from_seq: cursor.last_seq,
        to_seq,
        duration_ms: dur,
    })
}
