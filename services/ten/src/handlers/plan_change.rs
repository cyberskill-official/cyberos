//! Plan-change decision logic (TASK-TEN-002 §1 #4–#8).

use crate::plans::caps::{first_downgrade_violation, proration_cents, CapAxis, CurrentUsage};
use crate::plans::tiers::{PlanChangeEffective, PlanTier};

/// Incoming plan-change request body.
#[derive(Debug, Clone)]
pub struct PlanChangeRequest {
    pub tenant_id: String,
    pub actor_id: String,
    pub actor_is_founder: bool,
    pub actor_is_tenant_admin: bool,
    pub is_founder_tenant: bool,
    pub current_tier: PlanTier,
    pub target_tier: PlanTier,
    pub effective: PlanChangeEffective,
    pub acknowledge_data_loss: bool,
    pub current_usage: CurrentUsage,
    pub days_remaining_in_period: u32,
    pub days_in_period: u32,
    pub reason: String,
}

/// Outcome of a successful plan-change decision (ready to persist in one TX).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanChangeDecision {
    pub from_tier: PlanTier,
    pub to_tier: PlanTier,
    pub effective_at_is_immediate: bool,
    pub proration_amount_cents: i64,
    pub sev2_immediate_downgrade: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlanChangeError {
    #[error("forbidden")]
    Forbidden,
    #[error("founder_tenant_plan_immutable")]
    FounderImmutable,
    #[error("same_tier")]
    SameTier,
    #[error("downgrade_violation axis={axis} current={current} target_cap={target_cap}")]
    DowngradeViolation {
        axis: &'static str,
        current: u64,
        target_cap: u64,
    },
}

/// Pure decision: role checks, founder lock, downgrade violation, proration.
pub fn decide_plan_change(req: &PlanChangeRequest) -> Result<PlanChangeDecision, PlanChangeError> {
    if req.is_founder_tenant && !req.actor_is_founder {
        return Err(PlanChangeError::FounderImmutable);
    }
    if !(req.actor_is_founder || req.actor_is_tenant_admin) {
        return Err(PlanChangeError::Forbidden);
    }
    if req.current_tier == req.target_tier {
        return Err(PlanChangeError::SameTier);
    }

    let is_downgrade = PlanTier::is_downgrade(req.current_tier, req.target_tier);
    if is_downgrade {
        if let Some((axis, current, cap)) =
            first_downgrade_violation(&req.current_usage, req.target_tier)
        {
            if !req.acknowledge_data_loss {
                return Err(PlanChangeError::DowngradeViolation {
                    axis: axis.as_str(),
                    current,
                    target_cap: cap,
                });
            }
            let _ = CapAxis::Seats; // keep axis type linked for future resolution rules
        }
    }

    let effective = if is_downgrade {
        match req.effective {
            PlanChangeEffective::Immediate => PlanChangeEffective::Immediate,
            _ => PlanChangeEffective::NextPeriod,
        }
    } else {
        // Upgrades are immediate (DEC-773).
        PlanChangeEffective::Immediate
    };

    let immediate = matches!(effective, PlanChangeEffective::Immediate);
    let sev2_immediate_downgrade = is_downgrade && immediate;
    let proration = if PlanTier::is_upgrade(req.current_tier, req.target_tier) {
        proration_cents(
            req.current_tier,
            req.target_tier,
            req.days_remaining_in_period,
            req.days_in_period,
        )
    } else {
        0
    };

    Ok(PlanChangeDecision {
        from_tier: req.current_tier,
        to_tier: req.target_tier,
        effective_at_is_immediate: immediate,
        proration_amount_cents: proration,
        sev2_immediate_downgrade,
    })
}
