use cyberos_ten::handlers::plan_change::{decide_plan_change, PlanChangeError, PlanChangeRequest};
use cyberos_ten::plans::caps::CurrentUsage;
use cyberos_ten::plans::tiers::{PlanChangeEffective, PlanTier};

fn base() -> PlanChangeRequest {
    PlanChangeRequest {
        tenant_id: "t1".into(),
        actor_id: "u1".into(),
        actor_is_founder: false,
        actor_is_tenant_admin: true,
        is_founder_tenant: false,
        current_tier: PlanTier::Starter,
        target_tier: PlanTier::Team,
        effective: PlanChangeEffective::Immediate,
        acknowledge_data_loss: false,
        current_usage: CurrentUsage::default(),
        days_remaining_in_period: 15,
        days_in_period: 30,
        reason: "upgrade".into(),
    }
}

#[test]
fn upgrade_is_immediate_with_proration() {
    let d = decide_plan_change(&base()).unwrap();
    assert_eq!(d.to_tier, PlanTier::Team);
    assert!(d.effective_at_is_immediate);
    assert_eq!(d.proration_amount_cents, 1450); // (2900-0)*15/30
}

#[test]
fn downgrade_defaults_to_next_period() {
    let mut req = base();
    req.current_tier = PlanTier::Team;
    req.target_tier = PlanTier::Starter;
    req.effective = PlanChangeEffective::NextPeriod;
    let d = decide_plan_change(&req).unwrap();
    assert!(!d.effective_at_is_immediate);
    assert_eq!(d.proration_amount_cents, 0);
}

#[test]
fn downgrade_violation_without_ack() {
    let mut req = base();
    req.current_tier = PlanTier::Team;
    req.target_tier = PlanTier::Starter;
    req.current_usage.seats = 10;
    let err = decide_plan_change(&req).unwrap_err();
    assert!(matches!(
        err,
        PlanChangeError::DowngradeViolation { axis: "seats", .. }
    ));
}
