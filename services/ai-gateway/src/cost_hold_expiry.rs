//! FR-AI-004 — Cost-hold expiry cleanup job.
//!
//! Scans `cost_ledger_hold` for rows whose `state = 'held' AND expires_at < NOW()`
//! and transitions each to `state = 'expired'` with a chained memory audit row.
//!
//! See FR-AI-004 for normative behaviour and acceptance criteria.

use once_cell::sync::Lazy;
use prometheus::{
    register_counter, register_counter_vec, register_gauge, register_histogram, Counter,
    CounterVec, Gauge, Histogram,
};
use rand::RngCore;
use rust_decimal::Decimal;
use sqlx::PgPool;
use std::sync::Mutex;
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

static HOLDS_SUCCEEDED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "ai_expiry_holds_succeeded_total",
        "Total holds successfully expired by expiry job"
    )
    .unwrap()
});

static HOLDS_FAILED: Lazy<Counter> = Lazy::new(|| {
    register_counter!(
        "ai_expiry_holds_failed_total",
        "Total holds that failed expiry processing"
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

static CLEANUP_PENDING_GAUGE: Lazy<Gauge> = Lazy::new(|| {
    register_gauge!(
        "cleanup_holds_pending_gauge",
        "Expired holds pending cleanup, for operator dashboards"
    )
    .unwrap()
});

static EXPIRY_EVENT_LOG: Lazy<Mutex<Vec<ExpiryEventRecord>>> = Lazy::new(|| Mutex::new(Vec::new()));
const MAX_EXPIRY_EVENT_LOG: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiryEventRecord {
    pub hold_id: Uuid,
    pub event: &'static str,
}

pub fn clear_expiry_event_log() {
    EXPIRY_EVENT_LOG.lock().unwrap().clear();
}

pub fn expiry_event_log_snapshot() -> Vec<ExpiryEventRecord> {
    EXPIRY_EVENT_LOG.lock().unwrap().clone()
}

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
    let tick_id = new_tick_id();
    let mut processed = 0u32;
    let mut succeeded = 0u32;
    let mut failed = 0u32;
    let mut expired_hold_ids = Vec::new();

    // Fetch one bounded batch of expired hold IDs.
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "SELECT id FROM cost_ledger_hold \
         WHERE state = 'held' AND expires_at < NOW() \
         ORDER BY id ASC \
         LIMIT $1 \
         FOR UPDATE SKIP LOCKED",
    )
    .bind(BATCH_SIZE)
    .fetch_all(pool)
    .await?;

    // Update pending gauge before the tick for operators watching backlog pressure.
    let pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cost_ledger_hold \
         WHERE state = 'held' AND expires_at < NOW()",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(0);
    PENDING_GAUGE.set(pending_count as f64);
    CLEANUP_PENDING_GAUGE.set(pending_count as f64);

    for id in ids {
        match process_one_hold(pool, id, &tick_id).await {
            Ok(HoldDisposition::Transitioned) => {
                processed += 1;
                succeeded += 1;
                expired_hold_ids.push(id);
                HOLDS_PROCESSED.with_label_values(&["succeeded"]).inc();
                HOLDS_SUCCEEDED.inc();
            }
            Ok(HoldDisposition::AlreadyTransitioned) => {
                // Already processed, reconciled, expired, or locked; do not count as processed.
            }
            Err(e) => {
                processed += 1;
                failed += 1;
                HOLDS_PROCESSED.with_label_values(&["failed"]).inc();
                HOLDS_FAILED.inc();
                tracing::warn!(?id, ?e, "expiry_hold_failed");
            }
        }
    }

    if !expired_hold_ids.is_empty() {
        let emit_req = memory_writer::builders::cleanup_run_completed(
            &tick_id,
            &expired_hold_ids,
            succeeded,
            failed,
        );
        if let Err(e) = memory_writer::emit(emit_req).await {
            tracing::warn!(?e, tick_id, "cleanup_run_completed_emit_failed");
        }
    }

    // Publish the remaining pending backlog after this tick's bounded work.
    let remaining_pending_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM cost_ledger_hold \
         WHERE state = 'held' AND expires_at < NOW()",
    )
    .fetch_one(pool)
    .await
    .unwrap_or(pending_count);
    PENDING_GAUGE.set(remaining_pending_count as f64);
    CLEANUP_PENDING_GAUGE.set(remaining_pending_count as f64);

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

async fn process_one_hold(
    pool: &PgPool,
    hold_id: Uuid,
    tick_id: &str,
) -> Result<HoldDisposition, HoldError> {
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

    let started_req =
        memory_writer::builders::hold_expired_started(&hold.tenant_id, hold.id, tick_id);
    memory_writer::emit(started_req)
        .await
        .map_err(|e| HoldError::MemoryEmitFailed(e.to_string()))?;
    record_expiry_event(hold.id, "hold_expired_started");

    if env_hold_id_matches("CYBEROS_AI_EXPIRY_FAIL_HOLD_ID", hold.id) {
        return Err(HoldError::MemoryEmitFailed(
            "injected memory writer failure".to_string(),
        ));
    }

    // Emit memory audit row before the state transition.
    let emit_req = memory_writer::builders::hold_expired(
        &hold.tenant_id,
        hold.id,
        hold.expires_at,
        hold.estimated_usd,
        tick_id,
    );
    memory_writer::emit(emit_req)
        .await
        .map_err(|e| HoldError::MemoryEmitFailed(e.to_string()))?;

    if env_hold_id_matches("CYBEROS_AI_EXPIRY_EXIT_AFTER_HOLD_EMIT", hold.id) {
        std::process::exit(42);
    }

    // Transition the hold.
    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'expired', refunded_at = NOW(), refund_reason = 'tick_expired' \
         WHERE id = $1",
    )
    .bind(hold.id)
    .execute(&mut *tx)
    .await?;
    record_expiry_event(hold.id, "sql_update");

    tx.commit().await?;
    record_expiry_event(hold.id, "commit");

    let completed_req = memory_writer::builders::hold_expired_completed(
        &hold.tenant_id,
        hold.id,
        tick_id,
        "tick_expired",
    );
    memory_writer::emit(completed_req)
        .await
        .map_err(|e| HoldError::MemoryEmitFailed(e.to_string()))?;
    record_expiry_event(hold.id, "hold_expired_completed");

    Ok(HoldDisposition::Transitioned)
}

fn record_expiry_event(hold_id: Uuid, event: &'static str) {
    tracing::info!(%hold_id, event, "cost_hold_expiry_event");
    let mut log = EXPIRY_EVENT_LOG.lock().unwrap();
    if log.len() >= MAX_EXPIRY_EVENT_LOG {
        log.remove(0);
    }
    log.push(ExpiryEventRecord { hold_id, event });
}

fn env_hold_id_matches(name: &str, hold_id: Uuid) -> bool {
    matches!(std::env::var(name), Ok(value) if value == hold_id.to_string())
}

fn new_tick_id() -> String {
    const ENCODING: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

    let now_ms = chrono::Utc::now().timestamp_millis().max(0) as u64;
    let mut bytes = [0u8; 16];
    bytes[..6].copy_from_slice(&now_ms.to_be_bytes()[2..]);
    rand::thread_rng().fill_bytes(&mut bytes[6..]);

    let mut value = u128::from_be_bytes(bytes);
    let mut out = [b'0'; 26];
    for slot in out.iter_mut().rev() {
        *slot = ENCODING[(value & 0x1f) as usize];
        value >>= 5;
    }
    String::from_utf8(out.to_vec()).expect("ULID alphabet is valid UTF-8")
}
