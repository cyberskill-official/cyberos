//! FR-AI-001 — Cost-ledger pre-call check.
//!
//! Synchronous pre-call budget gate. Every AI provider call routes through
//! `precheck()` first. On `Allow`, a hold is created; on `Refuse`, the caller
//! gets a 402/403/503 with structured error.
//!
//! See FR-AI-001 for normative behaviour and acceptance criteria.

pub mod types;

pub use types::{
    ChatCompleteRequest, CostLedgerHoldRow, CostLedgerRow, PrecheckError, PrecheckOutcome,
    RefuseReason, HOLD_TTL_SECONDS, IDEMPOTENCY_KEY_MAX_LEN,
};

use once_cell::sync::Lazy;
use prometheus::{register_counter_vec, register_histogram, CounterVec, Histogram};
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::alias;
use crate::cost_table;
use crate::memory_writer;
use crate::otel::{attributes as otel_attributes, spans as otel_spans};
use crate::policy::TenantPolicy;
use crate::residency;
use crate::zdr;

// ─── Metrics (FR-AI-001 §1 #14) ──────────────────────────────────────────────

static PRECHECK_CALLS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_precheck_calls_total",
        "Precheck outcomes by result",
        &["outcome"]
    )
    .unwrap()
});

static PRECHECK_LATENCY: Lazy<Histogram> = Lazy::new(|| {
    register_histogram!(
        "ai_gateway_precheck_latency_ms",
        "Precheck call latency in milliseconds",
        vec![1.0, 5.0, 10.0, 25.0, 50.0, 100.0]
    )
    .unwrap()
});

static HOLDS_CREATED: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_holds_created_total",
        "Hold rows created",
        &["tenant_id"]
    )
    .unwrap()
});

static BUDGET_WARNS: Lazy<CounterVec> = Lazy::new(|| {
    register_counter_vec!(
        "ai_gateway_budget_warns_total",
        "Budget warn threshold crossings",
        &["tenant_id"]
    )
    .unwrap()
});

// ─── Public API ───────────────────────────────────────────────────────────────

/// FR-AI-001 — Pre-call cost gate.
///
/// Runs synchronously before any LLM-provider call. On success, returns
/// `PrecheckOutcome::Allow` with a hold_id. On budget/Persona/Provider failure,
/// returns `PrecheckOutcome::Refuse`.
///
/// The cap-check and hold-creation execute inside a single Postgres transaction
/// with `FOR UPDATE` row locking to prevent concurrent cap-races (FR-AI-001 §1 #12).
pub async fn precheck(
    req: &ChatCompleteRequest,
    pool: &PgPool,
    policy: &TenantPolicy,
) -> Result<PrecheckOutcome, PrecheckError> {
    let mut span = otel_spans::start_precheck_span(
        &req.tenant_id,
        &req.agent_persona,
        &req.model_alias,
        &req.idempotency_key,
    );
    let result = precheck_inner(req, pool, policy).await;
    match &result {
        Ok(PrecheckOutcome::Allow { estimated_usd, .. }) => {
            span.set_str(otel_attributes::OUTCOME, "allow");
            span.set_str(otel_attributes::ESTIMATED_USD, estimated_usd.to_string());
            span.end_ok();
        }
        Ok(PrecheckOutcome::Refuse { reason, .. }) => {
            span.set_str(otel_attributes::OUTCOME, "refuse");
            span.end_error(precheck_refuse_label(reason));
        }
        Err(err) => {
            span.set_str(otel_attributes::OUTCOME, "error");
            span.end_error(precheck_error_label(err));
        }
    }
    result
}

async fn precheck_inner(
    req: &ChatCompleteRequest,
    pool: &PgPool,
    policy: &TenantPolicy,
) -> Result<PrecheckOutcome, PrecheckError> {
    let started = std::time::Instant::now();

    // 0. Validate idempotency key (§1 #10)
    if validate_idempotency_key(&req.idempotency_key).is_err() {
        PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
        return Ok(PrecheckOutcome::Refuse {
            reason: RefuseReason::InvalidIdempotencyKey,
            current_spent_usd: Decimal::ZERO,
            cap_usd: policy.ai_policy.monthly_cap_usd,
        });
    }

    // 0b. Persona pinning check (§1 #13)
    if let Some(allowed) = &policy.ai_policy.allowed_personas {
        if !allowed.contains(&req.agent_persona) {
            PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
            return Ok(PrecheckOutcome::Refuse {
                reason: RefuseReason::PersonaNotAllowed,
                current_spent_usd: Decimal::ZERO,
                cap_usd: policy.ai_policy.monthly_cap_usd,
            });
        }
    }

    // 1. Resolve provider + model from alias via FR-AI-006
    let resolved = match alias::resolve(&req.model_alias, policy) {
        Ok(resolved) => resolved,
        Err(alias::AliasError::ResolvedModelMissingCostEntry { .. }) => {
            PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
            return Ok(PrecheckOutcome::Refuse {
                reason: RefuseReason::ProviderUnavailable,
                current_spent_usd: Decimal::ZERO,
                cap_usd: policy.ai_policy.monthly_cap_usd,
            });
        }
        Err(alias::AliasError::ZdrViolation {
            resolved_provider,
            resolved_model,
            attestation,
        }) => {
            let attestation_present = attestation.is_some();
            zdr::record_violation(&req.tenant_id);
            memory_writer::emit(memory_writer::builders::zdr_violation(
                &req.tenant_id,
                &req.agent_persona,
                &req.model_alias,
                resolved_provider.as_metric_label(),
                &resolved_model,
                true,
                attestation_present,
                &req.idempotency_key,
            ))
            .await
            .map_err(|e| PrecheckError::MemoryWriterFailed {
                stderr: e.to_string(),
            })?;
            PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
            return Ok(PrecheckOutcome::Refuse {
                reason: RefuseReason::ZdrViolation,
                current_spent_usd: Decimal::ZERO,
                cap_usd: policy.ai_policy.monthly_cap_usd,
            });
        }
        Err(alias::AliasError::ResidencyViolation {
            resolved_region,
            policy_residency,
            attempted_alias,
            vn1_no_provider,
        }) => {
            memory_writer::emit(memory_writer::builders::residency_violation(
                &req.tenant_id,
                &req.agent_persona,
                &attempted_alias,
                residency::residency_label(policy_residency),
                resolved_region.as_deref(),
                vn1_no_provider,
                &req.idempotency_key,
            ))
            .await
            .map_err(|e| PrecheckError::MemoryWriterFailed {
                stderr: e.to_string(),
            })?;
            PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
            return Ok(PrecheckOutcome::Refuse {
                reason: RefuseReason::ResidencyViolation,
                current_spent_usd: Decimal::ZERO,
                cap_usd: policy.ai_policy.monthly_cap_usd,
            });
        }
        Err(e) => {
            PRECHECK_CALLS.with_label_values(&["error"]).inc();
            return Err(PrecheckError::CostEstimateFailed {
                reason: format!("alias resolution failed: {e}"),
            });
        }
    };

    // 2. Estimate cost from cost table (FR-AI-007)
    let cost_rate = match cost_table::lookup(&resolved.provider_kind, &resolved.model) {
        Some(cost_rate) => cost_rate,
        None => {
            PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
            return Ok(PrecheckOutcome::Refuse {
                reason: RefuseReason::ProviderUnavailable,
                current_spent_usd: Decimal::ZERO,
                cap_usd: policy.ai_policy.monthly_cap_usd,
            });
        }
    };

    let estimated_usd = estimate_cost(
        req.prompt_tokens,
        req.expected_completion_tokens,
        &cost_rate,
    );

    // 3. Open transaction (§1 #12 — cap check + hold insert serialised per tenant)
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;

    // 3a. Upsert cost_ledger row + lock it with FOR UPDATE
    let current = sqlx::query_as::<_, CostLedgerRow>(
        "INSERT INTO cost_ledger (tenant_id, period, spent_usd, monthly_cap_usd) \
         VALUES ($1, date_trunc('month', NOW())::date, 0, $2) \
         ON CONFLICT (tenant_id, period) DO UPDATE \
         SET tenant_id = EXCLUDED.tenant_id \
         RETURNING id, tenant_id, period, spent_usd, monthly_cap_usd",
    )
    .bind(&req.tenant_id)
    .bind(policy.ai_policy.monthly_cap_usd)
    .fetch_one(&mut *tx)
    .await?;

    // 3b. Explicit FOR UPDATE lock on the ledger row
    sqlx::query(
        "SELECT 1 FROM cost_ledger WHERE tenant_id = $1 AND period = date_trunc('month', NOW())::date FOR UPDATE",
    )
    .bind(&req.tenant_id)
    .execute(&mut *tx)
    .await?;

    // 4. Cap check (boundary inclusive — spent + estimated == cap is permitted)
    if current.spent_usd + estimated_usd > policy.ai_policy.monthly_cap_usd {
        tx.rollback().await.ok();
        PRECHECK_CALLS.with_label_values(&["refuse"]).inc();
        return Ok(PrecheckOutcome::Refuse {
            reason: RefuseReason::BudgetCapExceeded,
            current_spent_usd: current.spent_usd,
            cap_usd: policy.ai_policy.monthly_cap_usd,
        });
    }

    // 4b. Warn threshold check (§1 #4)
    let warn_at = policy.ai_policy.monthly_cap_usd
        * Decimal::try_from(policy.ai_policy.warn_threshold).unwrap_or(Decimal::new(8, 1));
    if current.spent_usd >= warn_at {
        BUDGET_WARNS.with_label_values(&[&req.tenant_id]).inc();
    }

    // 5. Insert hold (idempotent via UNIQUE on (tenant_id, idempotency_key))
    let hold_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO cost_ledger_hold \
         (tenant_id, idempotency_key, estimated_usd, agent_persona, model_alias, \
          resolved_provider, resolved_model, expires_at, state) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, NOW() + INTERVAL '60 seconds', 'held') \
         ON CONFLICT (tenant_id, idempotency_key) DO UPDATE \
         SET state = cost_ledger_hold.state \
         RETURNING id",
    )
    .bind(&req.tenant_id)
    .bind(&req.idempotency_key)
    .bind(estimated_usd)
    .bind(&req.agent_persona)
    .bind(&req.model_alias)
    .bind(resolved.provider_kind.as_metric_label())
    .bind(&resolved.model)
    .fetch_one(&mut *tx)
    .await?;

    // 6. Emit memory audit row BEFORE commit (audit-before-action invariant — §1 #6)
    let emit_result = memory_writer::emit(memory_writer::builders::precheck(
        &req.tenant_id,
        &req.agent_persona,
        &req.model_alias,
        resolved.provider_kind.as_metric_label(),
        &resolved.model,
        estimated_usd,
        current.spent_usd,
        &req.idempotency_key,
    ))
    .await;

    if let Err(e) = emit_result {
        // Memory failure → rollback hold; refuse the call
        tx.rollback().await.ok();
        PRECHECK_CALLS.with_label_values(&["error"]).inc();
        return Err(PrecheckError::MemoryWriterFailed {
            stderr: e.to_string(),
        });
    }

    // 7. Commit — hold + ledger lock now durable
    tx.commit().await?;

    // 8. Post-commit metrics (non-critical, fire-and-forget)
    let elapsed_ms = started.elapsed().as_millis() as f64;
    PRECHECK_CALLS.with_label_values(&["allow"]).inc();
    PRECHECK_LATENCY.observe(elapsed_ms);
    HOLDS_CREATED.with_label_values(&[&req.tenant_id]).inc();

    Ok(PrecheckOutcome::Allow {
        hold_id,
        estimated_usd,
        ttl_seconds: HOLD_TTL_SECONDS,
    })
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn precheck_refuse_label(reason: &RefuseReason) -> &'static str {
    match reason {
        RefuseReason::BudgetCapExceeded => "budget_cap_exceeded",
        RefuseReason::TenantSuspended => "tenant_suspended",
        RefuseReason::ProviderUnavailable => "provider_unavailable",
        RefuseReason::InvalidIdempotencyKey => "invalid_idempotency_key",
        RefuseReason::PersonaNotAllowed => "persona_not_allowed",
        RefuseReason::ZdrViolation => "zdr_violation",
        RefuseReason::ResidencyViolation => "residency_violation",
    }
}

fn precheck_error_label(error: &PrecheckError) -> &'static str {
    match error {
        PrecheckError::DbError(_) => "db_error",
        PrecheckError::MemoryWriterFailed { .. } => "memory_writer_failed",
        PrecheckError::CostEstimateFailed { .. } => "cost_estimate_failed",
    }
}

fn validate_idempotency_key(key: &str) -> Result<(), String> {
    if key.is_empty() || key.len() > IDEMPOTENCY_KEY_MAX_LEN {
        return Err("invalid_idempotency_key: length must be 1..=64".into());
    }
    if !key.chars().all(|c| c.is_ascii() && !c.is_ascii_control()) {
        return Err("invalid_idempotency_key: charset must be ASCII printable".into());
    }
    Ok(())
}

fn estimate_cost(
    prompt_tokens: u32,
    expected_completion_tokens: u32,
    rate: &cost_table::CostRate,
) -> Decimal {
    let per_1k = Decimal::from(1000u32);
    let prompt_cost = (Decimal::from(prompt_tokens) / per_1k) * rate.input_per_1k_usd;
    let completion_cost =
        (Decimal::from(expected_completion_tokens) / per_1k) * rate.output_per_1k_usd;
    prompt_cost + completion_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_key_validation_matches_printable_ascii_contract() {
        assert!(validate_idempotency_key("key with space").is_ok());
        assert!(validate_idempotency_key("visible-ASCII_123").is_ok());
        assert!(validate_idempotency_key("").is_err());
        assert!(validate_idempotency_key(&"a".repeat(IDEMPOTENCY_KEY_MAX_LEN + 1)).is_err());
        assert!(validate_idempotency_key("key\x00with-control").is_err());
        assert!(validate_idempotency_key("unicode-✓").is_err());
    }
}
