//! TASK-TEN-203 — admin REST for tenant plan show + change.
//!
//! Routes (JWT via admin router `verify_jwt`):
//!   GET  /v1/admin/tenants/:tenant_id/plan
//!   POST /v1/admin/tenants/:tenant_id/plan
//!
//! Decision logic lives in `cyberos_ten::handlers::decide_plan_change`; this
//! module owns authz, usage snapshot, and the history+UPDATE TX.

use axum::{
    extract::{Json as JsonInput, Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{Datelike, Utc};
use cyberos_ten::handlers::{
    decide_plan_change, PlanChangeError, PlanChangeRequest, PlanShow,
};
use cyberos_ten::plans::caps::CurrentUsage;
use cyberos_ten::plans::tiers::{PlanChangeEffective, PlanTier};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::jwt::Claims;
use crate::AppState;

const ROLE_TENANT_ADMIN: &str = "tenant-admin";
const ROLE_SECURITY_ADMIN: &str = "security-admin";
const ROLE_FOUNDER: &str = "founder";
const ROLE_ROOT_ADMIN: &str = "root-admin";

#[derive(Debug, Serialize)]
pub struct PlanView {
    pub tenant_id: String,
    pub tier: PlanTier,
    pub caps: CapsView,
    pub is_founder_tenant: bool,
    pub effective_since: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CapsView {
    pub seats: Option<u64>,
    pub api_calls_per_month: Option<u64>,
    pub ai_tokens_per_month: Option<u64>,
    pub storage_bytes: Option<u64>,
}

impl From<cyberos_ten::plans::caps::TierCaps> for CapsView {
    fn from(c: cyberos_ten::plans::caps::TierCaps) -> Self {
        Self {
            seats: c.seats,
            api_calls_per_month: c.api_calls_per_month,
            ai_tokens_per_month: c.ai_tokens_per_month,
            storage_bytes: c.storage_bytes,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PlanChangeBody {
    pub target_tier: PlanTier,
    #[serde(default = "default_effective")]
    pub effective: PlanChangeEffective,
    #[serde(default)]
    pub acknowledge_data_loss: bool,
    #[serde(default)]
    pub reason: String,
}

fn default_effective() -> PlanChangeEffective {
    PlanChangeEffective::Immediate
}

pub async fn get_plan(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<PlanView>, (StatusCode, Json<Value>)> {
    require_plan_reader(&claims, tenant_id)?;
    set_tenant(&state, tenant_id).await?;

    let row: Option<(String, bool, chrono::DateTime<Utc>)> = sqlx::query_as(
        "SELECT plan_tier::text, is_founder_tenant, plan_effective_since
         FROM tenants WHERE id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&state.pg)
    .await
    .map_err(internal)?;

    let (tier_s, is_founder, effective_since) = row.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tenant_not_found"})),
        )
    })?;
    let tier = PlanTier::parse(&tier_s).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "invalid_plan_tier_in_db", "value": tier_s})),
        )
    })?;
    let show = PlanShow::new(tenant_id.to_string(), tier, is_founder);
    Ok(Json(PlanView {
        tenant_id: show.tenant_id,
        tier: show.tier,
        caps: show.caps.into(),
        is_founder_tenant: show.is_founder_tenant,
        effective_since,
    }))
}

pub async fn post_plan(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(tenant_id): Path<Uuid>,
    JsonInput(body): JsonInput<PlanChangeBody>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let (actor_is_founder, actor_is_tenant_admin) = require_plan_writer(&claims, tenant_id)?;
    let actor_id = parse_actor(&claims)?;
    set_tenant(&state, tenant_id).await?;

    let row: Option<(String, bool)> = sqlx::query_as(
        "SELECT plan_tier::text, is_founder_tenant FROM tenants WHERE id = $1",
    )
    .bind(tenant_id)
    .fetch_optional(&state.pg)
    .await
    .map_err(internal)?;
    let (tier_s, is_founder_tenant) = row.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "tenant_not_found"})),
        )
    })?;
    let current_tier = PlanTier::parse(&tier_s).ok_or_else(|| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "invalid_plan_tier_in_db", "value": tier_s})),
        )
    })?;

    let seats = seat_count(&state, tenant_id).await?;
    let (days_remaining, days_in_period) = period_days();

    let req = PlanChangeRequest {
        tenant_id: tenant_id.to_string(),
        actor_id: actor_id.to_string(),
        actor_is_founder,
        actor_is_tenant_admin,
        is_founder_tenant,
        current_tier,
        target_tier: body.target_tier,
        effective: body.effective,
        acknowledge_data_loss: body.acknowledge_data_loss,
        current_usage: CurrentUsage {
            seats,
            api_calls: 0,
            ai_tokens: 0,
            storage_bytes: 0,
        },
        days_remaining_in_period: days_remaining,
        days_in_period,
        reason: body.reason.clone(),
    };

    let decision = match decide_plan_change(&req) {
        Ok(d) => d,
        Err(e) => return Err(map_plan_error(e)),
    };

    let effective_at = if decision.effective_at_is_immediate {
        Utc::now()
    } else {
        end_of_month_utc()
    };
    let effective_enum = if decision.effective_at_is_immediate {
        PlanChangeEffective::Immediate
    } else {
        PlanChangeEffective::NextPeriod
    };

    let mut tx = state.pg.begin().await.map_err(internal)?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(internal)?;

    sqlx::query(
        "INSERT INTO tenant_plan_history
            (tenant_id, from_tier, to_tier, actor_id, effective_at,
             proration_amount_cents, reason, effective)
         VALUES ($1, $2::plan_tier, $3::plan_tier, $4, $5, $6, $7, $8::plan_change_effective)",
    )
    .bind(tenant_id)
    .bind(decision.from_tier.as_str())
    .bind(decision.to_tier.as_str())
    .bind(actor_id)
    .bind(effective_at)
    .bind(decision.proration_amount_cents)
    .bind(&body.reason)
    .bind(match effective_enum {
        PlanChangeEffective::Immediate => "immediate",
        PlanChangeEffective::NextPeriod => "next_period",
        PlanChangeEffective::DeferBillingOnly => "defer_billing_only",
    })
    .execute(&mut *tx)
    .await
    .map_err(internal)?;

    // Only persist immediate tier flips here; deferred downgrades keep current
    // tier until period-close (TASK-TEN-004 residual). History still records intent.
    if decision.effective_at_is_immediate {
        sqlx::query("UPDATE tenants SET plan_tier = $2::plan_tier, updated_at = NOW() WHERE id = $1")
            .bind(tenant_id)
            .bind(decision.to_tier.as_str())
            .execute(&mut *tx)
            .await
            .map_err(internal)?;
    }

    tx.commit().await.map_err(internal)?;

    Ok(Json(json!({
        "status": "ok",
        "from_tier": decision.from_tier,
        "to_tier": decision.to_tier,
        "effective_at_is_immediate": decision.effective_at_is_immediate,
        "proration_amount_cents": decision.proration_amount_cents,
        "sev2_immediate_downgrade": decision.sev2_immediate_downgrade,
        "effective_at": effective_at,
    })))
}

/// Map library errors to HTTP (TASK-TEN-203 AC #3–#5).
pub fn map_plan_error(err: PlanChangeError) -> (StatusCode, Json<Value>) {
    match err {
        PlanChangeError::Forbidden => (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "forbidden"})),
        ),
        PlanChangeError::FounderImmutable => (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "founder_tenant_plan_immutable"})),
        ),
        PlanChangeError::SameTier => (
            StatusCode::CONFLICT,
            Json(json!({"error": "no_change"})),
        ),
        PlanChangeError::DowngradeViolation {
            axis,
            current,
            target_cap,
        } => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "downgrade_violation",
                "axis": axis,
                "current": current,
                "target_cap": target_cap,
            })),
        ),
    }
}

pub fn validate_plan_tier_str(s: &str) -> Result<(), (StatusCode, Json<Value>)> {
    if PlanTier::parse(s).is_some() {
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_input",
                "field": "plan_tier",
                "reason": "must be one of starter|team|enterprise (sandbox removed)"
            })),
        ))
    }
}

fn require_plan_reader(
    claims: &Claims,
    tenant_id: Uuid,
) -> Result<(), (StatusCode, Json<Value>)> {
    if is_founder(claims) {
        return Ok(());
    }
    if claims.tenant_id != tenant_id.to_string() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "wrong_tenant"})),
        ));
    }
    if claims
        .roles
        .iter()
        .any(|r| r == ROLE_TENANT_ADMIN || r == ROLE_SECURITY_ADMIN)
    {
        return Ok(());
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(json!({"error": "role_required", "role": ROLE_TENANT_ADMIN})),
    ))
}

fn require_plan_writer(
    claims: &Claims,
    tenant_id: Uuid,
) -> Result<(bool, bool), (StatusCode, Json<Value>)> {
    let founder = is_founder(claims);
    if founder {
        return Ok((true, true));
    }
    if claims.tenant_id != tenant_id.to_string() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(json!({"error": "wrong_tenant"})),
        ));
    }
    let tenant_admin = claims
        .roles
        .iter()
        .any(|r| r == ROLE_TENANT_ADMIN || r == ROLE_SECURITY_ADMIN);
    if tenant_admin {
        return Ok((false, true));
    }
    Err((
        StatusCode::FORBIDDEN,
        Json(json!({"error": "role_required", "role": ROLE_TENANT_ADMIN})),
    ))
}

fn is_founder(claims: &Claims) -> bool {
    claims.roles.iter().any(|r| r == ROLE_FOUNDER)
        || (claims.tenant_id == Uuid::nil().to_string()
            && claims.roles.iter().any(|r| r == ROLE_ROOT_ADMIN))
}

fn parse_actor(claims: &Claims) -> Result<Uuid, (StatusCode, Json<Value>)> {
    Uuid::parse_str(&claims.sub).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("bad subject in JWT: {e}")})),
        )
    })
}

async fn set_tenant(state: &AppState, tenant_id: Uuid) -> Result<(), (StatusCode, Json<Value>)> {
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&state.pg)
        .await
        .map_err(internal)
        .map(|_| ())
}

async fn seat_count(state: &AppState, tenant_id: Uuid) -> Result<u64, (StatusCode, Json<Value>)> {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*)::bigint FROM subjects
         WHERE tenant_id = $1 AND status = 'active'",
    )
    .bind(tenant_id)
    .fetch_one(&state.pg)
    .await
    .map_err(internal)?;
    Ok(n.max(0) as u64)
}

fn period_days() -> (u32, u32) {
    let now = Utc::now().date_naive();
    let days_in_period = days_in_month(now.year(), now.month());
    let day = now.day();
    let days_remaining = days_in_period.saturating_sub(day).saturating_add(1);
    (days_remaining, days_in_period)
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_next = chrono::NaiveDate::from_ymd_opt(ny, nm, 1).expect("valid month");
    first_next
        .pred_opt()
        .map(|d| d.day())
        .unwrap_or(30)
}

fn end_of_month_utc() -> chrono::DateTime<Utc> {
    let now = Utc::now().date_naive();
    let dim = days_in_month(now.year(), now.month());
    let last = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), dim).unwrap_or(now);
    last.and_hms_opt(23, 59, 59)
        .map(|n| chrono::DateTime::<Utc>::from_naive_utc_and_offset(n, Utc))
        .unwrap_or_else(Utc::now)
}

fn internal<E: std::fmt::Display>(e: E) -> (StatusCode, Json<Value>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"error": format!("internal: {e}")})),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cyberos_ten::handlers::PlanChangeError;

    #[test]
    fn map_same_tier_is_409_no_change() {
        let (st, body) = map_plan_error(PlanChangeError::SameTier);
        assert_eq!(st, StatusCode::CONFLICT);
        assert_eq!(body.0["error"], "no_change");
    }

    #[test]
    fn map_founder_is_403() {
        let (st, body) = map_plan_error(PlanChangeError::FounderImmutable);
        assert_eq!(st, StatusCode::FORBIDDEN);
        assert_eq!(body.0["error"], "founder_tenant_plan_immutable");
    }

    #[test]
    fn map_downgrade_violation() {
        let (st, body) = map_plan_error(PlanChangeError::DowngradeViolation {
            axis: "seats",
            current: 10,
            target_cap: 3,
        });
        assert_eq!(st, StatusCode::CONFLICT);
        assert_eq!(body.0["error"], "downgrade_violation");
        assert_eq!(body.0["axis"], "seats");
    }

    #[test]
    fn reject_sandbox_plan_tier() {
        assert!(validate_plan_tier_str("sandbox").is_err());
        assert!(validate_plan_tier_str("starter").is_ok());
    }
}
