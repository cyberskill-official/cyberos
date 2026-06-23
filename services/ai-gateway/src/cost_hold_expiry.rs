//! FR-AI-004 — Cost-hold expiry cleanup job.
//!
//! Scans `cost_ledger_hold` for rows whose `state = 'held' AND expires_at < NOW()`
//! and transitions each to `state = 'expired'` with a chained memory audit row.
//!
//! See FR-AI-004 for normative behaviour and acceptance criteria.

use once_cell::sync::Lazy;
use prometheus::{
    register_counter_vec, register_gauge, register_histogram, CounterVec, Gauge, Histogram,
};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::memory_writer;

// ─── Constants ────────────────────────────────────────────────────────────────

const BATCH_SIZE: i64 = 500;

// ─── Metrics (FR-AI-004 §4 #10, #12) ─────────────────────────────────────────

static HOLDS_PROCESSED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_expiry_holds_processed_total",
        "Total holds processed by expiry job",
        &["result"]
    )
    .unwrap()
});

static TICK_DURATION: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "ai_expiry_tick_duration_seconds",
        "Duration of each expiry tick in seconds",
        vec![0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0]
    )
    .unwrap()
});

static TICKS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!("ai_expiry_ticks_total", "Total ticks executed", &["result"]).unwrap()
});

static CONSECUTIVE_FAILURES: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "ai_expiry_consecutive_failures",
        "Current consecutive failure count"
    )
    .unwrap()
});

static PENDING_GAUGE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "ai_expiry_holds_pending",
        "Number of expired holds waiting to be processed"
    )
    .unwrap()
});

// ─── Public types ─────────────────────────────────────────────────────────────

/// Report from a single tick execution.
#[derive(Debug, Clone)]
pub struct TickReport {
    pub holds_processed: u32,
    pub holds_succeeded: u32,
    pub holds_failed: u32,
    pub duration_ms: u32,
}

/// Tick-level error (DB unreachable, etc.). Per-hold failures roll up into TickReport.
#[derive(Debug, thiserror::Error)]
pub enum TickError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),
}

/// Per-hold processing result.
#[derive(Debug)]
enum HoldDisposition {
    Transitioned,
    AlreadyTransitioned,
}

/// Per-hold error.
#[derive(Debug, thiserror::Error)]
enum HoldError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("memory writer failed: {0}")]
    MemoryEmitFailed(String),
}

// ─── Internal row type ────────────────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
struct HoldRow {
    id: Uuid,
    tenant_id: String,
    estimated_usd: Decimal,
    expires_at: chrono::DateTime<chrono::Utc>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// FR-AI-004 — Run one expiry tick.
///
/// Scans for expired holds, processes each in its own transaction, emits
/// `ai.hold_expired` memory audit rows, and returns a summary report.
pub async fn run_tick(pool: &PgPool) -> Result<TickReport, TickError> {
    let started = std::time::Instant::now();
    let mut processed = 0u32;
    let mut succeeded = 0u32;
    let mut failed = 0u32;

    loop {
        // Fetch a batch of expired hold IDs.
        let ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT id FROM cost_ledger_hold \
             WHERE state = 'held' AND expires_at < NOW() \
             ORDER BY id ASC \
             LIMIT $1",
        )
        .bind(BATCH_SIZE)
        .fetch_all(pool)
        .await?;

        if ids.is_empty() {
            break;
        }

        // Update pending gauge.
        let pending_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM cost_ledger_hold \
             WHERE state = 'held' AND expires_at < NOW()",
        )
        .fetch_one(pool)
        .await
        .unwrap_or(0);
        PENDING_GAUGE.set(pending_count as f64);

        let batch_len = ids.len() as u32;
        for id in ids {
            processed += 1;
            match process_one_hold(pool, id).await {
                Ok(HoldDisposition::Transitioned) => {
                    succeeded += 1;
                    HOLDS_PROCESSED.with_label_values(&["succeeded"]).inc();
                }
                Ok(HoldDisposition::AlreadyTransitioned) => {
                    // Already processed (reconciled/expired/locked); skip.
                }
                Err(e) => {
                    failed += 1;
                    HOLDS_PROCESSED.with_label_values(&["failed"]).inc();
                    tracing::warn!(?id, ?e, "expiry_hold_failed");
                }
            }
        }

        // If partial batch, we're done.
        if batch_len < BATCH_SIZE as u32 {
            break;
        }
    }

    let duration_ms = started.elapsed().as_millis() as u32;
    TICK_DURATION.observe(duration_ms as f64 / 1000.0);

    if failed == 0 {
        TICKS_TOTAL.with_label_values(&["success"]).inc();
        CONSECUTIVE_FAILURES.set(0.0);
    } else {
        TICKS_TOTAL.with_label_values(&["partial_failure"]).inc();
    }

    Ok(TickReport {
        holds_processed: processed,
        holds_succeeded: succeeded,
        holds_failed: failed,
        duration_ms,
    })
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

async fn process_one_hold(pool: &PgPool, hold_id: Uuid) -> Result<HoldDisposition, HoldError> {
    let mut tx = pool.begin().await?;

    // Lock + load the hold. SKIP LOCKED means we bail if someone else took it.
    let hold: Option<HoldRow> = sqlx::query_as(
        "SELECT id, tenant_id, estimated_usd, expires_at \
         FROM cost_ledger_hold \
         WHERE id = $1 AND state = 'held' AND expires_at < NOW() \
         FOR UPDATE SKIP LOCKED",
    )
    .bind(hold_id)
    .fetch_optional(&mut *tx)
    .await?;

    let hold = match hold {
        Some(h) => h,
        None => return Ok(HoldDisposition::AlreadyTransitioned),
    };

    // Emit memory audit row INSIDE the transaction (audit-before-action).
    let emit_req = memory_writer::builders::hold_expired(
        &hold.tenant_id,
        hold.id,
        hold.expires_at,
        hold.estimated_usd,
    );
    memory_writer::emit(emit_req)
        .await
        .map_err(|e| HoldError::MemoryEmitFailed(e.to_string()))?;

    // Transition the hold.
    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'expired', refunded_at = NOW(), refund_reason = 'tick_expired' \
         WHERE id = $1",
    )
    .bind(hold.id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(HoldDisposition::Transitioned)
}
