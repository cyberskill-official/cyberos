//! TASK-TEN-207 — overage admission unit surface.

use cyberos_auth::metering_admit::{admit_api_call, clear_override, set_override, AdmitOverride};
use cyberos_auth::metering_emit::{emit_api_call, recorded_len};
use cyberos_metering::admission::{admit, AdmissionOutcome, AdmissionRequest};
use cyberos_metering::axes::MeteringAxis;
use cyberos_metering::policy::OveragePolicy;
use cyberos_ten::PlanTier;

#[test]
fn admission_helper_block_shape() {
    let out = admit(&AdmissionRequest {
        axis: MeteringAxis::ApiCalls,
        current: 10_000,
        quantity: 1,
        cap: Some(10_000),
        policy: OveragePolicy::Block,
        warn_threshold_bps: 8_000,
    });
    assert_eq!(
        out,
        AdmissionOutcome::Block {
            current: 10_000,
            cap: 10_000,
            axis: "api_calls",
        }
    );
}

#[test]
fn default_warn_does_not_block_at_cap() {
    let out = admit(&AdmissionRequest {
        axis: MeteringAxis::ApiCalls,
        current: 10_000,
        quantity: 1,
        cap: Some(10_000),
        policy: OveragePolicy::Warn,
        warn_threshold_bps: 8_000,
    });
    assert!(matches!(out, AdmissionOutcome::Warn { .. }));
}

#[tokio::test]
async fn override_block_rejects_without_emit() {
    // No live pool needed when override supplies current+policy+tier.
    let tenant = format!("ten_overage_{}", uuid::Uuid::new_v4());
    set_override(
        &tenant,
        AdmitOverride {
            tier: PlanTier::Starter,
            policy: OveragePolicy::Block,
            current: Some(10_000),
        },
    );
    let before = recorded_len();
    // Fake pool: admit_api_call still needs PgPool — use connect_lazy.
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://cyberos:cyberos@127.0.0.1:5432/cyberos")
        .expect("lazy pool");
    let err = admit_api_call(&pool, &tenant).await.expect_err("should block");
    match err {
        cyberos_auth::metering_admit::AdmitError::Blocked { current, cap } => {
            assert_eq!(current, 10_000);
            assert_eq!(cap, 10_000);
        }
    }
    assert_eq!(recorded_len(), before);
    // Emit is only on success path — calling emit manually still works, but 402 path does not.
    let _ = emit_api_call(&tenant, &format!("k-{}", uuid::Uuid::new_v4()), "GET", "/x");
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
    admit_api_call(&pool, &tenant).await.expect("allow");
    clear_override(&tenant);
}

#[tokio::test]
async fn enterprise_unlimited_api_allows() {
    let tenant = format!("ten_ent_{}", uuid::Uuid::new_v4());
    set_override(
        &tenant,
        AdmitOverride {
            tier: PlanTier::Enterprise,
            policy: OveragePolicy::Block,
            current: Some(9_999_999),
        },
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://cyberos:cyberos@127.0.0.1:5432/cyberos")
        .expect("lazy pool");
    admit_api_call(&pool, &tenant).await.expect("unlimited");
    clear_override(&tenant);
}
