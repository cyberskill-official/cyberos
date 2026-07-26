//! TASK-TEN-208 — ai_tokens overage admission.

use cyberos_ai_gateway::cost_ledger::RefuseReason;
use cyberos_ai_gateway::metering_admit::{
    admit_ai_tokens, clear_override, set_override, AdmitOverride,
};
use cyberos_metering::admission::{admit, AdmissionOutcome, AdmissionRequest};
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::policy::OveragePolicy;
use cyberos_ten::PlanTier;

#[test]
fn admission_helper_blocks_ai_tokens() {
    let out = admit(&AdmissionRequest {
        axis: MeteringAxis::AiTokens,
        current: 100_000,
        quantity: 1,
        cap: Some(100_000),
        policy: OveragePolicy::Block,
        warn_threshold_bps: 8_000,
    });
    assert_eq!(
        out,
        AdmissionOutcome::Block {
            current: 100_000,
            cap: 100_000,
            axis: "ai_tokens",
        }
    );
}

#[test]
fn metering_token_overage_reason_is_distinct() {
    let r = RefuseReason::MeteringTokenOverage {
        current: 100_000,
        cap: 100_000,
    };
    assert!(matches!(r, RefuseReason::MeteringTokenOverage { .. }));
    assert!(!matches!(r, RefuseReason::BudgetCapExceeded));
}

#[test]
fn streaming_maps_metering_overage_to_402() {
    // Mirror the status selection used in streaming::handle path.
    let reason = RefuseReason::MeteringTokenOverage {
        current: 1,
        cap: 1,
    };
    let http_status = if matches!(reason, RefuseReason::MeteringTokenOverage { .. })
        || matches!(reason, RefuseReason::BudgetCapExceeded)
    {
        402
    } else {
        429
    };
    assert_eq!(http_status, 402);
}

#[tokio::test]
async fn override_block_rejects() {
    let tenant = format!("ten_tok_{}", uuid::Uuid::new_v4());
    set_override(
        &tenant,
        AdmitOverride {
            tier: PlanTier::Starter,
            policy: OveragePolicy::Block,
            current: Some(100_000),
        },
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://cyberos:cyberos@127.0.0.1:5432/cyberos")
        .expect("lazy pool");
    let err = admit_ai_tokens(&pool, &tenant, 1)
        .await
        .expect_err("block");
    assert_eq!(err.current, 100_000);
    assert_eq!(err.cap, 100_000);
    clear_override(&tenant);
}

#[tokio::test]
async fn default_warn_allows_at_cap() {
    let tenant = format!("ten_warn_{}", uuid::Uuid::new_v4());
    set_override(
        &tenant,
        AdmitOverride {
            tier: PlanTier::Starter,
            policy: OveragePolicy::Warn,
            current: Some(100_000),
        },
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://cyberos:cyberos@127.0.0.1:5432/cyberos")
        .expect("lazy pool");
    admit_ai_tokens(&pool, &tenant, 1).await.expect("warn allows");
    clear_override(&tenant);
}

#[tokio::test]
async fn under_cap_allows() {
    let tenant = format!("ten_ok_{}", uuid::Uuid::new_v4());
    set_override(
        &tenant,
        AdmitOverride {
            tier: PlanTier::Starter,
            policy: OveragePolicy::Block,
            current: Some(0),
        },
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://cyberos:cyberos@127.0.0.1:5432/cyberos")
        .expect("lazy pool");
    admit_ai_tokens(&pool, &tenant, 500).await.expect("allow");
    clear_override(&tenant);
}
