use cyberos_ten::plans::caps::{first_downgrade_violation, CapAxis, CurrentUsage};
use cyberos_ten::plans::tiers::PlanTier;

#[test]
fn api_calls_violation() {
    let usage = CurrentUsage {
        api_calls: 20_000,
        ..Default::default()
    };
    let (axis, cur, cap) = first_downgrade_violation(&usage, PlanTier::Starter).unwrap();
    assert_eq!(axis, CapAxis::ApiCalls);
    assert_eq!(cur, 20_000);
    assert_eq!(cap, 10_000);
}

#[test]
fn within_caps_ok() {
    let usage = CurrentUsage {
        seats: 2,
        api_calls: 100,
        ai_tokens: 50,
        storage_bytes: 1024,
    };
    assert!(first_downgrade_violation(&usage, PlanTier::Starter).is_none());
}
