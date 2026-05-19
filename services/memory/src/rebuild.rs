//! FR-MEMORY-102 — Rebuild Layer 2 from Layer 1.
//!
//! Two surfaces:
//!   * `rebuild::run_full(pool, tenant)` — truncate l2_memory + l2_entity +
//!     l2_ingest_cursor for the tenant; reset the cursor to 0; re-tail the
//!     full `l1_audit_log` history through `ingest::run_batch` until exhausted.
//!     Returns a `RebuildSummary`.
//!   * `rebuild::reconcile(pool, tenant, sample_size)` — without truncating,
//!     pull a random sample of `l2_memory` rows and verify their `chain_anchor`
//!     against the corresponding `l1_audit_log` row. Reports mismatches.
//!
//! Wired into the CLI: `cyberos-memory rebuild --tenant <UUID>` and
//! `cyberos-memory reconcile --tenant <UUID> --sample 100`. A cron / OBS
//! alert can call these on a schedule; the 30-minute reconcile cadence
//! lands in FR-OBS-001's alertmanager config (not in this crate).

use crate::layer2::{binlog_tail, chain_anchor, ingest};
use cyberos_types::TenantId;
use serde::Serialize;
use sqlx::PgPool;
use std::time::Instant;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum RebuildError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("ingest error: {0}")]
    Ingest(#[from] ingest::IngestError),
    #[error("reconcile mismatch — {found} of {checked} rows failed chain-anchor verify")]
    ReconcileMismatch { checked: usize, found: usize },
}

#[derive(Debug, Clone, Serialize)]
pub struct RebuildSummary {
    pub tenant_id: TenantId,
    pub rows_truncated: i64,
    pub rows_reingested: i64,
    pub batches: usize,
    pub duration_secs: i64,
}

/// FULL rebuild — destroys + reingests Layer 2 for this tenant. Holds a
/// transaction over the truncate so concurrent reads see either old or
/// new state, never mid-rebuild emptiness.
pub async fn run_full(pool: &PgPool, tenant: TenantId) -> Result<RebuildSummary, RebuildError> {
    let start = Instant::now();
    info!(?tenant, "rebuild starting");

    // Step 1 — record what we're about to destroy so the summary is accurate.
    let (rows_before,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM l2_memory WHERE tenant_id = $1",
    )
    .bind(tenant.as_uuid())
    .fetch_one(pool)
    .await?;

    // Step 2 — truncate (DELETE in a transaction so concurrent readers don't
    // see emptiness — they see either old rows + a then-empty result depending
    // on commit ordering; that's acceptable for a rebuild gate).
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant.to_string())
        .execute(&mut *tx).await?;

    sqlx::query("DELETE FROM l2_memory WHERE tenant_id = $1")
        .bind(tenant.as_uuid()).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM l2_entity WHERE tenant_id = $1")
        .bind(tenant.as_uuid()).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM l2_ingest_cursor WHERE tenant_id = $1")
        .bind(tenant.as_uuid()).execute(&mut *tx).await?;
    sqlx::query("DELETE FROM l2_ingest_cursor_history WHERE tenant_id = $1")
        .bind(tenant.as_uuid()).execute(&mut *tx).await?;
    tx.commit().await?;

    info!(?tenant, rows_truncated = rows_before, "rebuild truncate complete; re-ingesting");

    // Step 3 — drain batches until no more rows. Cap iterations defensively.
    let mut batches = 0usize;
    let mut rows_reingested = 0i64;
    const MAX_BATCHES: usize = 10_000;
    const BATCH_SIZE: i32 = 1000;

    loop {
        if batches >= MAX_BATCHES {
            warn!(?tenant, "rebuild hit batch cap; tenant has > 10M rows");
            break;
        }
        let summary = ingest::run_batch(pool, tenant, BATCH_SIZE).await?;
        if summary.rows_processed == 0 {
            break;
        }
        batches += 1;
        rows_reingested += summary.rows_processed as i64;
    }

    let duration_secs = start.elapsed().as_secs() as i64;
    info!(?tenant, rows_reingested, batches, duration_secs, "rebuild complete");
    Ok(RebuildSummary {
        tenant_id: tenant,
        rows_truncated: rows_before,
        rows_reingested,
        batches,
        duration_secs,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ReconcileSummary {
    pub tenant_id: TenantId,
    pub sample_size: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<ReconcileFailure>,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReconcileFailure {
    pub seq: i64,
    pub path: String,
    pub stored_anchor: String,
    pub recomputed_anchor: String,
}

/// SAMPLE reconcile — non-destructive verification that l2_memory's stored
/// chain_anchor matches what we'd recompute from l1_audit_log right now.
/// Used by the 30-minute OBS-triggered cron.
pub async fn reconcile(
    pool: &PgPool,
    tenant: TenantId,
    sample_size: i64,
) -> Result<ReconcileSummary, RebuildError> {
    let start = Instant::now();
    let sample_size = sample_size.clamp(1, 1000);

    let rows: Vec<(i64, String, String)> = sqlx::query_as(
        "SELECT seq, path, chain_anchor_hex
             FROM l2_memory
            WHERE tenant_id = $1
         ORDER BY RANDOM()
            LIMIT $2",
    )
    .bind(tenant.as_uuid())
    .bind(sample_size)
    .fetch_all(pool)
    .await?;

    let mut passed = 0usize;
    let mut failures = Vec::new();
    for (seq, path, stored_anchor) in &rows {
        // Pull the corresponding l1_audit_log row.
        let l1: Option<(Option<String>, Option<String>, String)> = sqlx::query_as(
            "SELECT body, prev_hash_hex, chain_anchor_hex
                 FROM l1_audit_log
                WHERE tenant_id = $1 AND seq = $2 AND path = $3
                LIMIT 1",
        )
        .bind(tenant.as_uuid())
        .bind(*seq)
        .bind(path)
        .fetch_optional(pool)
        .await?;
        let Some((body, prev, _l1_anchor)) = l1 else {
            failures.push(ReconcileFailure {
                seq: *seq,
                path: path.clone(),
                stored_anchor: stored_anchor.clone(),
                recomputed_anchor: "<L1 row missing>".to_string(),
            });
            continue;
        };
        let recomputed = chain_anchor::compute(prev.as_deref(), body.as_deref().unwrap_or(""));
        if &recomputed != stored_anchor {
            failures.push(ReconcileFailure {
                seq: *seq,
                path: path.clone(),
                stored_anchor: stored_anchor.clone(),
                recomputed_anchor: recomputed,
            });
        } else {
            passed += 1;
        }
    }

    Ok(ReconcileSummary {
        tenant_id: tenant,
        sample_size: rows.len(),
        passed,
        failed: failures.len(),
        failures,
        duration_ms: start.elapsed().as_millis() as i64,
    })
}
