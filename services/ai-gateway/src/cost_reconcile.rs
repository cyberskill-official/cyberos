//! FR-AI-002 — Post-call cost reconciliation.
//!
//! Settles holds created by `cost_ledger::precheck()` (FR-AI-001). On success,
//! records actual spend; on provider error, refunds the hold; on cancel with
//! partial stream, charges only what was delivered.
//!
//! See FR-AI-002 for normative behaviour and acceptance criteria.

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram, CounterVec, Histogram};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::sync::Mutex;
use uuid::Uuid;

use crate::cost_table;
use crate::memory_writer;
use crate::otel::{attributes as otel_attributes, spans as otel_spans};

// ─── Metrics (FR-AI-002 §4 #14) ──────────────────────────────────────────────

static RECONCILE_CALLS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_reconcile_calls_total",
        "Reconcile outcomes by result",
        &["outcome"]
    )
    .unwrap()
});

static RECONCILE_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "ai_gateway_reconcile_latency_ms",
        "Reconcile call latency in milliseconds",
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 80.0, 150.0]
    )
    .unwrap()
});

static HOLDS_RECONCILED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_holds_reconciled_total",
        "Hold rows reconciled",
        &["tenant_id"]
    )
    .unwrap()
});

static HOLDS_REFUNDED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_holds_refunded_total",
        "Hold rows refunded by reason",
        &["reason"]
    )
    .unwrap()
});

static SPEND_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_spend_usd_total",
        "Total spend USD by tenant and period",
        &["tenant_id", "period"]
    )
    .unwrap()
});

static RECONCILE_EVENT_LOG: Lazy<Mutex<Vec<ReconcileEventRecord>>> =
    Lazy::new(|| Mutex::new(Vec::new()));
const MAX_RECONCILE_EVENT_LOG: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReconcileEventRecord {
    pub hold_id: Uuid,
    pub event: &'static str,
}

pub fn clear_reconcile_event_log() {
    RECONCILE_EVENT_LOG.lock().unwrap().clear();
}

pub fn reconcile_event_log_snapshot() -> Vec<ReconcileEventRecord> {
    RECONCILE_EVENT_LOG.lock().unwrap().clone()
}

// ─── Public types ─────────────────────────────────────────────────────────────

/// What actually happened on the provider call.
#[derive(Debug, Clone)]
pub enum CallOutcome {
    /// Provider returned a successful response.
    Success {
        usage: ProviderUsage,
        latency_ms: u32,
        cache_state: CacheState,
        provider_request_id: String,
    },
    /// Provider returned an error (4xx/5xx).
    ProviderError {
        http_status: u16,
        retryable: bool,
        provider_error_message: String,
    },
    /// Client disconnected mid-stream.
    Cancelled {
        partial_usage: Option<ProviderUsage>,
        reason: CancelReason,
    },
}

/// Token counts from a provider response.
#[derive(Debug, Clone)]
pub struct ProviderUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
}

/// Cache state of the provider response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheState {
    Hit,
    Miss,
    Partial,
}

impl CacheState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hit => "hit",
            Self::Miss => "miss",
            Self::Partial => "partial",
        }
    }
}

/// Reason for cancellation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CancelReason {
    ClientDisconnect,
    TimeoutBeforeFirstToken,
    TimeoutMidStream,
    GatewayShutdown,
}

/// Outcome of a successful reconcile.
#[derive(Debug, Clone)]
pub enum ReconcileOutcome {
    /// Hold settled; actual spend recorded.
    Reconciled {
        actual_usd: Decimal,
        new_spent_total_usd: Decimal,
        warn_crossed: bool,
    },
    /// Hold released; no spend recorded.
    Refunded {
        hold_estimated_usd: Decimal,
        reason: RefundReason,
    },
}

/// Why a hold was refunded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefundReason {
    ProviderError { http_status: u16 },
    ProviderUnreachable,
}

/// Error taxonomy for reconcile.
#[derive(Debug, thiserror::Error)]
pub enum ReconcileError {
    #[error("hold not found: {0}")]
    HoldNotFound(Uuid),

    #[error("hold already finalised (state={current_state})")]
    AlreadyFinalised {
        current_state: String,
        original_outcome: ReconcileOutcome,
    },

    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("memory writer failed: {stderr}")]
    MemoryWriterFailed { stderr: String },

    #[error("cost table missing entry for {provider}/{model}")]
    CostTableMissing { provider: String, model: String },
}

// ─── Internal row type ────────────────────────────────────────────────────────

#[derive(Debug, Clone, sqlx::FromRow)]
struct HoldRow {
    id: Uuid,
    tenant_id: String,
    idempotency_key: String,
    estimated_usd: Decimal,
    agent_persona: String,
    model_alias: String,
    resolved_provider: String,
    resolved_model: String,
    state: String,
    actual_usd: Option<Decimal>,
    refund_reason: Option<String>,
    warn_crossed: Option<bool>,
    warn_emitted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// FR-AI-002 — Post-call cost reconciliation.
///
/// Runs synchronously after every LLM-provider call. Given the `hold_id` from
/// `precheck()` and the actual `CallOutcome`, settles the hold:
/// - Success → compute actual cost, update ledger, emit `ai.invocation`
/// - ProviderError → refund hold, emit `ai.invocation_failed`
/// - Cancelled with partial → charge partial, emit `ai.invocation` with `cancelled: true`
/// - Cancelled with no partial → refund, emit `ai.invocation_failed`
///
/// Idempotent: calling on an already-finalised hold returns `AlreadyFinalised`.
pub async fn reconcile(
    hold_id: Uuid,
    outcome: CallOutcome,
    pool: &PgPool,
) -> Result<ReconcileOutcome, ReconcileError> {
    let mut span = otel_spans::start_reconcile_span(&hold_id.to_string());
    let result = reconcile_inner(hold_id, outcome, pool).await;
    match &result {
        Ok(ReconcileOutcome::Reconciled { actual_usd, .. }) => {
            span.set_str(otel_attributes::OUTCOME, "allow");
            span.set_str(otel_attributes::ACTUAL_USD, actual_usd.to_string());
            span.end_ok();
        }
        Ok(ReconcileOutcome::Refunded { .. }) => {
            span.set_str(otel_attributes::OUTCOME, "refuse");
            span.end_error("refunded");
        }
        Err(err) => {
            span.set_str(otel_attributes::OUTCOME, "error");
            span.end_error(reconcile_error_label(err));
        }
    }
    result
}

async fn reconcile_inner(
    hold_id: Uuid,
    outcome: CallOutcome,
    pool: &PgPool,
) -> Result<ReconcileOutcome, ReconcileError> {
    let started = std::time::Instant::now();

    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 1. Lock the hold row.
    let hold = sqlx::query_as::<_, HoldRow>(
        "SELECT id, tenant_id, idempotency_key, estimated_usd, agent_persona, model_alias, \
         resolved_provider, resolved_model, state, actual_usd, refund_reason, warn_crossed, \
         (SELECT warn_emitted_at FROM cost_ledger \
          WHERE tenant_id = cost_ledger_hold.tenant_id \
            AND period = date_trunc('month', NOW())::date) as warn_emitted_at \
         FROM cost_ledger_hold WHERE id = $1 FOR UPDATE",
    )
    .bind(hold_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(ReconcileError::HoldNotFound(hold_id))?;

    // 2. Idempotency check — reconstruct original outcome from persisted state.
    if hold.state != "held" {
        let original_outcome = reconstruct_outcome(&hold);
        RECONCILE_CALLS
            .with_label_values(&["already_finalised"])
            .inc();
        return Err(ReconcileError::AlreadyFinalised {
            current_state: hold.state.clone(),
            original_outcome,
        });
    }

    emit_reconcile_started(&mut tx, &hold, outcome_kind(&outcome)).await?;
    record_reconcile_event(hold.id, "reconcile_started");

    // 3. Branch by outcome.
    let result = match apply_outcome(&mut tx, &hold, hold_id, outcome).await {
        Ok(result) => result,
        Err(err) => {
            drop(tx);
            emit_reconcile_failed_best_effort(&hold, &err.to_string()).await;
            record_reconcile_event(hold.id, "reconcile_failed");
            return Err(err);
        }
    };

    // 4. Commit — hold transition + ledger update + final audit row are durable together.
    if let Err(err) = tx.commit().await {
        let err = ReconcileError::DbError(err);
        emit_reconcile_failed_best_effort(&hold, &err.to_string()).await;
        record_reconcile_event(hold.id, "reconcile_failed");
        return Err(err);
    }
    record_reconcile_event(hold.id, "commit");

    emit_reconcile_completed(&hold, &result).await?;
    record_reconcile_event(hold.id, "reconcile_completed");

    record_reconcile_metrics(&hold, &result);

    let elapsed_ms = started.elapsed().as_millis() as f64;
    RECONCILE_LATENCY.observe(elapsed_ms);

    Ok(result)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn reconcile_error_label(error: &ReconcileError) -> &'static str {
    match error {
        ReconcileError::DbError(_) => "db_error",
        ReconcileError::HoldNotFound(_) => "hold_not_found",
        ReconcileError::AlreadyFinalised { .. } => "already_finalised",
        ReconcileError::CostTableMissing { .. } => "cost_table_missing",
        ReconcileError::MemoryWriterFailed { .. } => "memory_writer_failed",
    }
}

async fn apply_outcome(
    tx: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
    hold_id: Uuid,
    outcome: CallOutcome,
) -> Result<ReconcileOutcome, ReconcileError> {
    match outcome {
        CallOutcome::Success {
            usage,
            latency_ms,
            cache_state,
            provider_request_id,
        } => {
            let actual_usd = compute_actual_cost(hold, &usage)?;
            let (new_spent, warn_crossed) =
                apply_success(tx, hold, actual_usd, &provider_request_id).await?;

            let emit_req = memory_writer::builders::invocation(
                &hold.tenant_id,
                &hold.agent_persona,
                &hold.model_alias,
                &hold.resolved_provider,
                &hold.resolved_model,
                usage.prompt_tokens,
                usage.completion_tokens,
                actual_usd,
                hold_id,
                latency_ms,
                cache_state.as_str(),
                &provider_request_id,
            );
            emit_audit(tx, emit_req).await?;

            Ok(ReconcileOutcome::Reconciled {
                actual_usd,
                new_spent_total_usd: new_spent,
                warn_crossed,
            })
        }
        CallOutcome::ProviderError {
            http_status,
            retryable,
            provider_error_message,
        } => {
            apply_refund(tx, hold, RefundReason::ProviderError { http_status }).await?;

            let emit_req = memory_writer::builders::invocation_failed(
                &hold.tenant_id,
                &hold.agent_persona,
                &hold.model_alias,
                &hold.resolved_provider,
                &hold.resolved_model,
                http_status,
                retryable,
                &provider_error_message,
                hold_id,
                hold.estimated_usd,
            );
            emit_audit(tx, emit_req).await?;

            Ok(ReconcileOutcome::Refunded {
                hold_estimated_usd: hold.estimated_usd,
                reason: RefundReason::ProviderError { http_status },
            })
        }
        CallOutcome::Cancelled {
            partial_usage: Some(usage),
            ..
        } => {
            let actual_usd = compute_actual_cost(hold, &usage)?;
            let actual_usd = actual_usd.max(Decimal::new(1, 4)); // AC #12: floor at 0.0001.
            let (new_spent, warn_crossed) = apply_success(tx, hold, actual_usd, "").await?;

            let mut emit_req = memory_writer::builders::invocation(
                &hold.tenant_id,
                &hold.agent_persona,
                &hold.model_alias,
                &hold.resolved_provider,
                &hold.resolved_model,
                usage.prompt_tokens,
                usage.completion_tokens,
                actual_usd,
                hold_id,
                0,
                CacheState::Partial.as_str(),
                "",
            );
            emit_req
                .extra
                .as_object_mut()
                .unwrap()
                .insert("cancelled".to_string(), serde_json::json!(true));
            emit_audit(tx, emit_req).await?;

            Ok(ReconcileOutcome::Reconciled {
                actual_usd,
                new_spent_total_usd: new_spent,
                warn_crossed,
            })
        }
        CallOutcome::Cancelled {
            partial_usage: None,
            ..
        } => {
            apply_refund(tx, hold, RefundReason::ProviderUnreachable).await?;

            let emit_req = memory_writer::builders::invocation_failed(
                &hold.tenant_id,
                &hold.agent_persona,
                &hold.model_alias,
                &hold.resolved_provider,
                &hold.resolved_model,
                0,
                false,
                "cancelled_before_first_token",
                hold_id,
                hold.estimated_usd,
            );
            emit_audit(tx, emit_req).await?;

            Ok(ReconcileOutcome::Refunded {
                hold_estimated_usd: hold.estimated_usd,
                reason: RefundReason::ProviderUnreachable,
            })
        }
    }
}

async fn emit_reconcile_started(
    tx: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
    outcome_kind: &str,
) -> Result<(), ReconcileError> {
    let emit_req = memory_writer::builders::reconcile_started(
        &hold.tenant_id,
        &hold.agent_persona,
        &hold.model_alias,
        &hold.resolved_provider,
        &hold.resolved_model,
        hold.id,
        outcome_kind,
    );
    emit_audit(tx, emit_req).await
}

async fn emit_reconcile_completed(
    hold: &HoldRow,
    result: &ReconcileOutcome,
) -> Result<(), ReconcileError> {
    let (outcome, actual, spent, warn, refund_reason): (
        &str,
        Option<Decimal>,
        Option<Decimal>,
        bool,
        Option<String>,
    ) = match result {
        ReconcileOutcome::Reconciled {
            actual_usd,
            new_spent_total_usd,
            warn_crossed,
        } => (
            "reconciled",
            Some(*actual_usd),
            Some(*new_spent_total_usd),
            *warn_crossed,
            None,
        ),
        ReconcileOutcome::Refunded { reason, .. } => (
            "refunded",
            None,
            None,
            false,
            Some(refund_reason_tag(reason)),
        ),
    };
    let emit_req = memory_writer::builders::reconcile_completed(
        &hold.tenant_id,
        &hold.agent_persona,
        &hold.model_alias,
        &hold.resolved_provider,
        &hold.resolved_model,
        hold.id,
        outcome,
        actual,
        spent,
        warn,
        refund_reason.as_deref(),
    );
    emit_audit_after_commit(emit_req).await
}

async fn emit_reconcile_failed_best_effort(hold: &HoldRow, error: &str) {
    let emit_req = memory_writer::builders::reconcile_failed(
        &hold.tenant_id,
        &hold.agent_persona,
        &hold.model_alias,
        &hold.resolved_provider,
        &hold.resolved_model,
        hold.id,
        error,
    );
    let _ = emit_audit_after_commit(emit_req).await;
}

fn record_reconcile_metrics(hold: &HoldRow, result: &ReconcileOutcome) {
    match result {
        ReconcileOutcome::Reconciled { actual_usd, .. } => {
            RECONCILE_CALLS.with_label_values(&["reconciled"]).inc();
            HOLDS_RECONCILED.with_label_values(&[&hold.tenant_id]).inc();
            SPEND_TOTAL
                .with_label_values(&[&hold.tenant_id, "current"])
                .inc_by(actual_usd.to_string().parse::<f64>().unwrap_or(0.0));
        }
        ReconcileOutcome::Refunded { reason, .. } => {
            let reason_tag = refund_reason_tag(reason);
            RECONCILE_CALLS.with_label_values(&["refunded"]).inc();
            HOLDS_REFUNDED
                .with_label_values(&[reason_tag.as_str()])
                .inc();
        }
    }
}

fn compute_actual_cost(hold: &HoldRow, usage: &ProviderUsage) -> Result<Decimal, ReconcileError> {
    let provider_kind = parse_provider_kind(&hold.resolved_provider).ok_or_else(|| {
        ReconcileError::CostTableMissing {
            provider: hold.resolved_provider.clone(),
            model: hold.resolved_model.clone(),
        }
    })?;
    let rate = cost_table::lookup(&provider_kind, &hold.resolved_model).ok_or(
        ReconcileError::CostTableMissing {
            provider: hold.resolved_provider.clone(),
            model: hold.resolved_model.clone(),
        },
    )?;
    let per_1k = Decimal::from(1000u32);
    let prompt_cost = (Decimal::from(usage.prompt_tokens) / per_1k) * rate.input_per_1k_usd;
    let completion_cost =
        (Decimal::from(usage.completion_tokens) / per_1k) * rate.output_per_1k_usd;
    Ok(prompt_cost + completion_cost)
}

fn record_reconcile_event(hold_id: Uuid, event: &'static str) {
    tracing::info!(%hold_id, event, "cost_reconcile_event");
    let mut log = RECONCILE_EVENT_LOG.lock().unwrap();
    if log.len() >= MAX_RECONCILE_EVENT_LOG {
        log.remove(0);
    }
    log.push(ReconcileEventRecord { hold_id, event });
}

fn outcome_kind(outcome: &CallOutcome) -> &'static str {
    match outcome {
        CallOutcome::Success { .. } => "success",
        CallOutcome::ProviderError { .. } => "provider_error",
        CallOutcome::Cancelled {
            partial_usage: Some(_),
            ..
        } => "cancelled_partial",
        CallOutcome::Cancelled {
            partial_usage: None,
            ..
        } => "cancelled_no_stream",
    }
}

fn refund_reason_tag(reason: &RefundReason) -> String {
    match reason {
        RefundReason::ProviderError { http_status } => format!("provider_error_{http_status}"),
        RefundReason::ProviderUnreachable => "provider_unreachable".to_string(),
    }
}

fn parse_provider_kind(s: &str) -> Option<crate::policy::ProviderKind> {
    match s {
        "bedrock" => Some(crate::policy::ProviderKind::Bedrock),
        "anthropic" => Some(crate::policy::ProviderKind::Anthropic),
        "openai" => Some(crate::policy::ProviderKind::Openai),
        "vertex" => Some(crate::policy::ProviderKind::Vertex),
        "bge" => Some(crate::policy::ProviderKind::Bge),
        _ => None,
    }
}

async fn apply_success(
    tx: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
    actual_usd: Decimal,
    provider_request_id: &str,
) -> Result<(Decimal, bool), ReconcileError> {
    // Update ledger spend + check warn threshold in one shot.
    let row = sqlx::query_as::<_, (Decimal, bool)>(
        "WITH prior AS ( \
             SELECT tenant_id, period, spent_usd, monthly_cap_usd, warn_emitted_at, \
                    (spent_usd < monthly_cap_usd * 0.8 \
                     AND spent_usd + $1 >= monthly_cap_usd * 0.8 \
                     AND warn_emitted_at IS NULL) AS crossed \
             FROM cost_ledger \
             WHERE tenant_id = $2 \
               AND period = date_trunc('month', NOW())::date \
             FOR UPDATE \
         ), updated AS ( \
             UPDATE cost_ledger ledger \
             SET spent_usd = ledger.spent_usd + $1, \
                 warn_emitted_at = CASE \
                   WHEN prior.crossed THEN NOW() \
                   ELSE ledger.warn_emitted_at \
                 END \
             FROM prior \
             WHERE ledger.tenant_id = prior.tenant_id \
               AND ledger.period = prior.period \
             RETURNING ledger.spent_usd, prior.crossed \
         ) \
         SELECT spent_usd, crossed FROM updated",
    )
    .bind(actual_usd)
    .bind(&hold.tenant_id)
    .fetch_one(&mut **tx)
    .await?;

    if row.1 {
        tracing::info!(
            tenant_id = %hold.tenant_id,
            hold_id = %hold.id,
            event = "cap_crossed_after_reconcile"
        );
    }

    // Transition hold to reconciled.
    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'reconciled', actual_usd = $1, reconciled_at = NOW(), \
             provider_request_id = $2, warn_crossed = $3 \
         WHERE id = $4",
    )
    .bind(actual_usd)
    .bind(provider_request_id)
    .bind(row.1)
    .bind(hold.id)
    .execute(&mut **tx)
    .await?;
    record_reconcile_event(hold.id, "sql_update");

    Ok(row)
}

async fn apply_refund(
    tx: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
    reason: RefundReason,
) -> Result<(), ReconcileError> {
    let reason_str = refund_reason_tag(&reason);

    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'refunded', refunded_at = NOW(), refund_reason = $1 \
         WHERE id = $2",
    )
    .bind(&reason_str)
    .bind(hold.id)
    .execute(&mut **tx)
    .await?;
    record_reconcile_event(hold.id, "sql_update");

    Ok(())
}

async fn emit_audit(
    _tx: &mut Transaction<'_, Postgres>,
    req: memory_writer::MemoryEmit,
) -> Result<(), ReconcileError> {
    // Emit via memory writer. If this fails, the transaction will roll back.
    memory_writer::emit(req)
        .await
        .map_err(|e| ReconcileError::MemoryWriterFailed {
            stderr: e.to_string(),
        })?;
    Ok(())
}

async fn emit_audit_after_commit(req: memory_writer::MemoryEmit) -> Result<(), ReconcileError> {
    memory_writer::emit(req)
        .await
        .map_err(|e| ReconcileError::MemoryWriterFailed {
            stderr: e.to_string(),
        })?;
    Ok(())
}

fn reconstruct_outcome(hold: &HoldRow) -> ReconcileOutcome {
    match hold.state.as_str() {
        "reconciled" => ReconcileOutcome::Reconciled {
            actual_usd: hold.actual_usd.unwrap_or(Decimal::ZERO),
            new_spent_total_usd: Decimal::ZERO, // reconstructed without re-reading ledger
            warn_crossed: hold.warn_crossed.unwrap_or(false),
        },
        "refunded" => {
            let reason = if hold
                .refund_reason
                .as_deref()
                .unwrap_or("")
                .starts_with("provider_error_")
            {
                let status = hold
                    .refund_reason
                    .as_deref()
                    .unwrap_or("provider_error_0")
                    .trim_start_matches("provider_error_")
                    .parse::<u16>()
                    .unwrap_or(0);
                RefundReason::ProviderError {
                    http_status: status,
                }
            } else {
                RefundReason::ProviderUnreachable
            };
            ReconcileOutcome::Refunded {
                hold_estimated_usd: hold.estimated_usd,
                reason,
            }
        }
        _ => {
            // expired or unknown — treat as refund for reconstruction purposes.
            ReconcileOutcome::Refunded {
                hold_estimated_usd: hold.estimated_usd,
                reason: RefundReason::ProviderUnreachable,
            }
        }
    }
}
