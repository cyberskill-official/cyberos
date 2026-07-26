use cyberos_ten::handlers::plan_change::{decide_plan_change, PlanChangeError, PlanChangeRequest};
use cyberos_ten::plans::caps::CurrentUsage;
use cyberos_ten::plans::tiers::{PlanChangeEffective, PlanTier};

#[test]
fn founder_tenant_immutable_for_non_founder() {
    let req = PlanChangeRequest {
        tenant_id: "founder".into(),
        actor_id: "admin".into(),
        actor_is_founder: false,
        actor_is_tenant_admin: true,
        is_founder_tenant: true,
        current_tier: PlanTier::Enterprise,
        target_tier: PlanTier::Team,
        effective: PlanChangeEffective::Immediate,
        acknowledge_data_loss: true,
        current_usage: CurrentUsage::default(),
        days_remaining_in_period: 10,
        days_in_period: 30,
        reason: "no".into(),
    };
    assert_eq!(
        decide_plan_change(&req).unwrap_err(),
        PlanChangeError::FounderImmutable
    );
}

#[test]
fn founder_actor_may_change_founder_tenant() {
    let req = PlanChangeRequest {
        tenant_id: "founder".into(),
        actor_id: "founder-op".into(),
        actor_is_founder: true,
        actor_is_tenant_admin: true,
        is_founder_tenant: true,
        current_tier: PlanTier::Enterprise,
        target_tier: PlanTier::Enterprise, // same → SameTier
        effective: PlanChangeEffective::Immediate,
        acknowledge_data_loss: false,
        current_usage: CurrentUsage::default(),
        days_remaining_in_period: 10,
        days_in_period: 30,
        reason: "noop".into(),
    };
    assert_eq!(decide_plan_change(&req).unwrap_err(), PlanChangeError::SameTier);
}
