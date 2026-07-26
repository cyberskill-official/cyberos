//! TASK-TEN-203 — plan-change HTTP error mapping + decide wiring (unit).
//! Postgres/HTTP path is covered when DATABASE_URL is set (`#[ignore]`).

use axum::http::StatusCode;
use cyberos_auth::plan_admin::{map_plan_error, validate_plan_tier_str};
use cyberos_ten::handlers::{decide_plan_change, PlanChangeError, PlanChangeRequest};
use cyberos_ten::plans::caps::CurrentUsage;
use cyberos_ten::plans::tiers::{PlanChangeEffective, PlanTier};

#[test]
fn decide_upgrade_wired() {
    let req = PlanChangeRequest {
        tenant_id: "t".into(),
        actor_id: "a".into(),
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
        reason: "upgrade for seats".into(),
    };
    let d = decide_plan_change(&req).expect("upgrade");
    assert!(d.effective_at_is_immediate);
    assert!(d.proration_amount_cents > 0);
}

#[test]
fn http_maps_library_errors() {
    let (st, body) = map_plan_error(PlanChangeError::SameTier);
    assert_eq!(st, StatusCode::CONFLICT);
    assert_eq!(body.0["error"], "no_change");

    let (st, body) = map_plan_error(PlanChangeError::FounderImmutable);
    assert_eq!(st, StatusCode::FORBIDDEN);
    assert_eq!(body.0["error"], "founder_tenant_plan_immutable");
}

#[test]
fn sandbox_rejected_at_api() {
    assert!(validate_plan_tier_str("sandbox").is_err());
    assert!(validate_plan_tier_str("enterprise").is_ok());
}

#[tokio::test]
#[ignore = "requires DATABASE_URL + migrated auth DB"]
async fn p0301_bare_update_fails() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = sqlx::PgPool::connect(&url).await.expect("connect");
    let tid = uuid::Uuid::new_v4();
    sqlx::query(
        "INSERT INTO tenants (id, slug, display_name, country, plan_tier, status, residency)
         VALUES ($1, $2, 'Plan Trigger', 'VN', 'starter', 'active', 'sg-1')",
    )
    .bind(tid)
    .bind(format!("plan-trig-{}", &tid.to_string()[..8]))
    .execute(&pool)
    .await
    .expect("insert tenant");

    let mut tx = pool.begin().await.unwrap();
    let err = sqlx::query("UPDATE tenants SET plan_tier = 'team' WHERE id = $1")
        .bind(tid)
        .execute(&mut *tx)
        .await
        .expect_err("bare update must fail P0301");
    let msg = err.to_string();
    assert!(
        msg.contains("P0301") || msg.contains("plan_tier"),
        "unexpected err: {msg}"
    );
}
