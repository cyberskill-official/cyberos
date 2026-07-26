//! TASK-TEN-207 — api_calls overage admission before handler work.

use std::sync::{Mutex, OnceLock};

use chrono::Utc;
use cyberos_metering::admission::{admit, AdmissionOutcome, AdmissionRequest};
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::policy::OveragePolicy;
use cyberos_metering::usage::utc_month_start;
use cyberos_ten::{caps_for, PlanTier};
use serde_json::json;
use sqlx::PgPool;
use tracing::warn;

use crate::metering_emit;

/// Testable overrides: (tenant_id → (tier, policy, optional current override)).
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

/// Install a per-tenant override (tests). Clears previous for that tenant.
pub fn set_override(tenant_id: &str, o: AdmitOverride) {
    overrides()
        .lock()
        .expect("overrides lock")
        .insert(tenant_id.to_string(), o);
}

pub fn clear_override(tenant_id: &str) {
    overrides().lock().expect("overrides lock").remove(tenant_id);
}

/// Default production policy (DEC-710): warn — does not 402.
pub fn default_policy() -> OveragePolicy {
    OveragePolicy::Warn
}

#[derive(Debug)]
pub enum AdmitError {
    Blocked { current: u64, cap: u64 },
}

impl AdmitError {
    pub fn into_response(self) -> axum::response::Response {
        use axum::response::IntoResponse;
        match self {
            AdmitError::Blocked { current, cap } => (
                axum::http::StatusCode::PAYMENT_REQUIRED,
                axum::Json(json!({
                    "error": "overage_blocked",
                    "axis": "api_calls",
                    "current": current,
                    "cap": cap,
                })),
            )
                .into_response(),
        }
    }
}

/// Admit one prospective api_call for `tenant_id`.
pub async fn admit_api_call(pool: &PgPool, tenant_id: &str) -> Result<(), AdmitError> {
    let (tier, policy, current_override) = resolve_context(pool, tenant_id).await;
    let caps = caps_for(tier);
    let current = match current_override {
        Some(c) => c,
        None => current_api_calls(tenant_id),
    };
    let outcome = admit(&AdmissionRequest {
        axis: MeteringAxis::ApiCalls,
        current,
        quantity: 1,
        cap: caps.api_calls_per_month,
        policy,
        warn_threshold_bps: 8_000,
    });
    match outcome {
        AdmissionOutcome::Allow | AdmissionOutcome::Warn { .. } => Ok(()),
        AdmissionOutcome::Block { current, cap, .. } => Err(AdmitError::Blocked { current, cap }),
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

fn current_api_calls(tenant_id: &str) -> u64 {
    let start = utc_month_start(Utc::now());
    metering_emit::api_calls_sum(tenant_id, start)
}
