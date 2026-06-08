//! FR-AI-001 — Cost-ledger type definitions.

use rust_decimal::Decimal;
use uuid::Uuid;

/// Outcome of a successful precheck.
#[derive(Debug, Clone)]
pub enum PrecheckOutcome {
    /// Request is within budget; hold created.
    Allow {
        hold_id: Uuid,
        estimated_usd: Decimal,
        ttl_seconds: u32,
    },
    /// Request refused (budget cap, persona, provider, etc.).
    Refuse {
        reason: RefuseReason,
        current_spent_usd: Decimal,
        cap_usd: Decimal,
    },
}

/// Reason for refusal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefuseReason {
    /// estimated + current > cap.
    BudgetCapExceeded,
    /// Tenant suspended (reserved for slice 2).
    TenantSuspended,
    /// Resolved model has no cost-table entry.
    ProviderUnavailable,
    /// Idempotency key failed validation.
    InvalidIdempotencyKey,
    /// agent_persona not in allowed_personas.
    PersonaNotAllowed,
    /// Tenant policy requires ZDR but alias resolved to a non-ZDR model.
    ZdrViolation,
}

/// Error taxonomy for precheck.
#[derive(Debug, thiserror::Error)]
pub enum PrecheckError {
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("memory writer failed: {stderr}")]
    MemoryWriterFailed { stderr: String },

    #[error("cost estimate failed: {reason}")]
    CostEstimateFailed { reason: String },
}

/// Row shape for the `cost_ledger` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CostLedgerRow {
    pub id: Uuid,
    pub tenant_id: String,
    pub period: chrono::NaiveDate,
    pub spent_usd: Decimal,
    pub monthly_cap_usd: Decimal,
}

/// Row shape for the `cost_ledger_hold` table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CostLedgerHoldRow {
    pub id: Uuid,
    pub tenant_id: String,
    pub idempotency_key: String,
    pub estimated_usd: Decimal,
    pub resolved_provider: String,
    pub resolved_model: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub state: String,
}

/// Request shape for precheck.
#[derive(Debug, Clone)]
pub struct ChatCompleteRequest {
    pub tenant_id: String,
    pub agent_persona: String,
    pub model_alias: String,
    pub prompt_tokens: u32,
    pub expected_completion_tokens: u32,
    pub idempotency_key: String,
}

/// HOLD_TTL_SECONDS — 60s per FR-AI-001 §1 #5.
pub const HOLD_TTL_SECONDS: u32 = 60;

/// Maximum idempotency key length per FR-AI-001 §1 #10.
pub const IDEMPOTENCY_KEY_MAX_LEN: usize = 64;
