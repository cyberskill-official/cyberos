//! TASK-TEN-208 — ai_tokens overage admission at cost_ledger precheck.

use std::sync::{Mutex, OnceLock};

use chrono::Utc;
use cyberos_metering::admission::{admit, AdmissionOutcome, AdmissionRequest};
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::policy::OveragePolicy;
use cyberos_metering::usage::utc_month_start;
use cyberos_ten::{caps_for, PlanTier};
use sqlx::PgPool;
use tracing::warn;

use crate::metering_emit;

static OVERRIDES: OnceLock<Mutex<std::collections::HashMap<String, AdmitOverride>>> =
    OnceLock::new();

#[derive(Debug, Clone)]
pub struct AdmitOverride {
    pub tier: PlanTier,
    pub policy: OveragePolicy,
    pub current: Option<u64>,
}

fn overrides() -> &'static Mutex<std::collections::HashMap<String, AdmitOverride>> {
    OVERRIDES.get_or_init(|| Mutex::new(std::collections::HashMap::new()))
}

pub fn set_override(tenant_id: &str, o: AdmitOverride) {
    overrides()
        .lock()
        .expect("overrides lock")
        .insert(tenant_id.to_string(), o);
}

pub fn clear_override(tenant_id: &str) {
    overrides().lock().expect("overrides lock").remove(tenant_id);
}

pub fn default_policy() -> OveragePolicy {
    OveragePolicy::Warn
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenOverage {
    pub current: u64,
    pub cap: u64,
}

/// Admit estimated tokens for `tenant_id`. `quantity` = prompt + expected completion.
pub async fn admit_ai_tokens(
    pool: &PgPool,
    tenant_id: &str,
    quantity: u64,
) -> Result<(), TokenOverage> {
    if quantity == 0 {
        return Ok(());
    }
    let (tier, policy, current_override) = resolve_context(pool, tenant_id).await;
    let caps = caps_for(tier);
    let current = match current_override {
        Some(c) => c,
        None => current_ai_tokens(tenant_id),
    };
    let outcome = admit(&AdmissionRequest {
        axis: MeteringAxis::AiTokens,
        current,
        quantity,
        cap: caps.ai_tokens_per_month,
        policy,
        warn_threshold_bps: 8_000,
    });
    match outcome {
        AdmissionOutcome::Allow | AdmissionOutcome::Warn { .. } => Ok(()),
        AdmissionOutcome::Block { current, cap, .. } => Err(TokenOverage { current, cap }),
    }
}

async fn resolve_context(
    pool: &PgPool,
    tenant_id: &str,
) -> (PlanTier, OveragePolicy, Option<u64>) {
    if let Ok(map) = overrides().lock() {
        if let Some(o) = map.get(tenant_id) {
            return (o.tier, o.policy, o.current);
        }
    }
    let tier = match load_plan_tier(pool, tenant_id).await {
        Some(t) => t,
        None => {
            warn!(tenant_id, "metering admit: plan_tier missing; treat as starter");
            PlanTier::Starter
        }
    };
    (tier, default_policy(), None)
}

async fn load_plan_tier(pool: &PgPool, tenant_id: &str) -> Option<PlanTier> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT plan_tier::text FROM tenants WHERE id = $1")
            .bind(tenant_id)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten();
    row.and_then(|(s,)| PlanTier::parse(&s))
}

fn current_ai_tokens(tenant_id: &str) -> u64 {
    let start = utc_month_start(Utc::now());
    metering_emit::ai_tokens_sum(tenant_id, start)
}
