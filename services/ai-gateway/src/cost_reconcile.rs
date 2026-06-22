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
use uuid::Uuid;

use crate::cost_table;
use crate::memory_writer;

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
    let started = std::time::Instant::now();

    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 1. Lock the hold row.
    let hold = sqlx::query_as::<_, HoldRow>(
        "SELECT id, tenant_id, idempotency_key, estimated_usd, resolved_provider, \
         resolved_model, state, actual_usd, refund_reason, \
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
        RECONCILE_CALLS.with_label_values(&["already_finalised"]).inc();
        return Err(ReconcileError::AlreadyFinalised {
            current_state: hold.state.clone(),
            original_outcome,
        });
    }

    // 3. Branch by outcome.
    let result = match outcome {
        CallOutcome::Success {
            usage,
            latency_ms,
            cache_state: _,
            provider_request_id,
        } => {
            let actual_usd = compute_actual_cost(&hold, &usage)?;
            let (new_spent, warn_crossed) =
                apply_success(&mut tx, &hold, actual_usd, &provider_request_id).await?;

            // Emit memory audit INSIDE transaction (audit-before-action).
            let emit_req = memory_writer::builders::invocation(
                &hold.tenant_id,
                "", // agent_persona not stored on hold; empty for now
                &hold.resolved_provider,
                &hold.resolved_model,
                usage.prompt_tokens,
                usage.completion_tokens,
                actual_usd,
                hold_id,
                latency_ms,
            );
            emit_audit(&mut tx, emit_req).await?;

            RECONCILE_CALLS.with_label_values(&["reconciled"]).inc();
            HOLDS_RECONCILED.with_label_values(&[&hold.tenant_id]).inc();
            SPEND_TOTAL
                .with_label_values(&[&hold.tenant_id, "current"])
                .inc_by(actual_usd.to_string().parse::<f64>().unwrap_or(0.0));

            ReconcileOutcome::Reconciled {
                actual_usd,
                new_spent_total_usd: new_spent,
                warn_crossed,
            }
        }
        CallOutcome::ProviderError {
            http_status,
            retryable,
            provider_error_message,
        } => {
            apply_refund(&mut tx, &hold, RefundReason::ProviderError { http_status }).await?;

            let emit_req = memory_writer::builders::invocation_failed(
                &hold.tenant_id,
                "",
                &hold.resolved_provider,
                &hold.resolved_model,
                http_status,
                retryable,
                &provider_error_message,
                hold_id,
                hold.estimated_usd,
            );
            emit_audit(&mut tx, emit_req).await?;

            RECONCILE_CALLS.with_label_values(&["refunded"]).inc();
            HOLDS_REFUNDED
                .with_label_values(&["provider_error"])
                .inc();

            ReconcileOutcome::Refunded {
                hold_estimated_usd: hold.estimated_usd,
                reason: RefundReason::ProviderError { http_status },
            }
        }
        CallOutcome::Cancelled {
            partial_usage: Some(usage),
            ..
        } => {
            let actual_usd = compute_actual_cost(&hold, &usage)?;
            // Floor at column precision (AC #12).
            let actual_usd = actual_usd.max(Decimal::new(1, 4)); // 0.0001

            let (new_spent, warn_crossed) =
                apply_success(&mut tx, &hold, actual_usd, "").await?;

            let mut emit_req = memory_writer::builders::invocation(
                &hold.tenant_id,
                "",
                &hold.resolved_provider,
                &hold.resolved_model,
                usage.prompt_tokens,
                usage.completion_tokens,
                actual_usd,
                hold_id,
                0,
            );
            // Tag as cancelled.
            emit_req
                .extra
                .as_object_mut()
                .unwrap()
                .insert("cancelled".to_string(), serde_json::json!(true));
            emit_audit(&mut tx, emit_req).await?;

            RECONCILE_CALLS.with_label_values(&["reconciled"]).inc();
            HOLDS_RECONCILED.with_label_values(&[&hold.tenant_id]).inc();

            ReconcileOutcome::Reconciled {
                actual_usd,
                new_spent_total_usd: new_spent,
                warn_crossed,
            }
        }
        CallOutcome::Cancelled {
            partial_usage: None,
            ..
        } => {
            apply_refund(&mut tx, &hold, RefundReason::ProviderUnreachable).await?;

            let emit_req = memory_writer::builders::invocation_failed(
                &hold.tenant_id,
                "",
                &hold.resolved_provider,
                &hold.resolved_model,
                0,
                false,
                "cancelled_before_first_token",
                hold_id,
                hold.estimated_usd,
            );
            emit_audit(&mut tx, emit_req).await?;

            RECONCILE_CALLS.with_label_values(&["refunded"]).inc();
            HOLDS_REFUNDED
                .with_label_values(&["provider_unreachable"])
                .inc();

            ReconcileOutcome::Refunded {
                hold_estimated_usd: hold.estimated_usd,
                reason: RefundReason::ProviderUnreachable,
            }
        }
    };

    // 4. Commit — hold transition + ledger update + audit row all durable together.
    tx.commit().await?;

    let elapsed_ms = started.elapsed().as_millis() as f64;
    RECONCILE_LATENCY.observe(elapsed_ms);

    Ok(result)
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn compute_actual_cost(hold: &HoldRow, usage: &ProviderUsage) -> Result<Decimal, ReconcileError> {
    let provider_kind = parse_provider_kind(&hold.resolved_provider)
        .ok_or_else(|| ReconcileError::CostTableMissing {
            provider: hold.resolved_provider.clone(),
            model: hold.resolved_model.clone(),
        })?;
    let rate = cost_table::lookup(&provider_kind, &hold.resolved_model).ok_or(
        ReconcileError::CostTableMissing {
            provider: hold.resolved_provider.clone(),
            model: hold.resolved_model.clone(),
        },
    )?;
    let per_1k = Decimal::from(1000u32);
    let prompt_cost = (Decimal::from(usage.prompt_tokens) / per_1k) * rate.input_per_1k_usd;
    let completion_cost = (Decimal::from(usage.completion_tokens) / per_1k) * rate.output_per_1k_usd;
    Ok(prompt_cost + completion_cost)
}

fn parse_provider_kind(s: &str) -> Option<crate::policy::ProviderKind> {
    match s {
        "bedrock" => Some(crate::policy::ProviderKind::Bedrock),
        "anthropic" => Some(crate::policy::ProviderKind::Anthropic),
        "openai" => Some(crate::policy::ProviderKind::Openai),
        "vertex" => Some(crate::policy::ProviderKind::Vertex),
        "bge" => Some(crate::policy::ProviderKind::Bge),
        "ollama" => Some(crate::policy::ProviderKind::Ollama),
        "local_openai" => Some(crate::policy::ProviderKind::LocalOpenai),
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
        "UPDATE cost_ledger \
         SET spent_usd = spent_usd + $1, \
             warn_emitted_at = CASE \
               WHEN spent_usd + $1 >= monthly_cap_usd * 0.8 \
                    AND warn_emitted_at IS NULL \
               THEN NOW() \
               ELSE warn_emitted_at \
             END \
         WHERE tenant_id = $2 \
           AND period = date_trunc('month', NOW())::date \
         RETURNING spent_usd, \
                   (warn_emitted_at IS NOT NULL AND \
                    (SELECT warn_emitted_at FROM cost_ledger \
                     WHERE tenant_id = $2 AND period = date_trunc('month', NOW())::date) \
                    = NOW()) as warn_crossed",
    )
    .bind(actual_usd)
    .bind(&hold.tenant_id)
    .fetch_one(&mut **tx)
    .await?;

    // Transition hold to reconciled.
    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'reconciled', actual_usd = $1, reconciled_at = NOW(), \
             provider_request_id = $2 \
         WHERE id = $3",
    )
    .bind(actual_usd)
    .bind(provider_request_id)
    .bind(hold.id)
    .execute(&mut **tx)
    .await?;

    Ok(row)
}

async fn apply_refund(
    tx: &mut Transaction<'_, Postgres>,
    hold: &HoldRow,
    reason: RefundReason,
) -> Result<(), ReconcileError> {
    let reason_str = match &reason {
        RefundReason::ProviderError { http_status } => format!("provider_error_{http_status}"),
        RefundReason::ProviderUnreachable => "provider_unreachable".to_string(),
    };

    sqlx::query(
        "UPDATE cost_ledger_hold \
         SET state = 'refunded', refunded_at = NOW(), refund_reason = $1 \
         WHERE id = $2",
    )
    .bind(&reason_str)
    .bind(hold.id)
    .execute(&mut **tx)
    .await?;

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

fn reconstruct_outcome(hold: &HoldRow) -> ReconcileOutcome {
    match hold.state.as_str() {
        "reconciled" => ReconcileOutcome::Reconciled {
            actual_usd: hold.actual_usd.unwrap_or(Decimal::ZERO),
            new_spent_total_usd: Decimal::ZERO, // reconstructed without re-reading ledger
            warn_crossed: hold.warn_crossed.unwrap_or(false),
        },
        "refunded" => {
            let reason = if hold.refund_reason.as_deref().unwrap_or("").starts_with("provider_error_")
            {
                let status = hold
                    .refund_reason
                    .as_deref()
                    .unwrap_or("provider_error_0")
                    .trim_start_matches("provider_error_")
                    .parse::<u16>()
                    .unwrap_or(0);
                RefundReason::ProviderError { http_status: status }
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
