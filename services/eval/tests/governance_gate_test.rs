//! FR-EVAL-001 slice 1 - governance gate + access-control integration tests.
//!
//! Mirrors the auth / memory integration-test convention: these require a live Postgres with the eval
//! migration applied, so they are `#[ignore]` by default and gate on `EVAL_DATABASE_URL` (falling back to
//! `DATABASE_URL`). CI boots services/dev/docker-compose.yml and runs `--ignored`. Local:
//!   docker compose up -d   (in services/dev/)
//!   EVAL_DATABASE_URL=postgres://... cargo test -p cyberos-eval -- --ignored
//!
//! The test applies `migrations/0001_governance.sql` itself (idempotent CREATE ... IF NOT EXISTS), seeds
//! into a fresh random tenant so it does not collide with other data, and exercises the public functions:
//!   * the consent gate denies an un-acknowledged subject and allows an acknowledged one, and re-gates on
//!     a notice-version bump (clause 3, 17);
//!   * the access check denies a stranger and allows founder / self / manager-of (clause 7).

use cyberos_eval::{access, gate};
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("EVAL_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("EVAL_DATABASE_URL or DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    // Apply the governance migration (idempotent). Runs as the connecting role (superuser in dev), so the
    // append-only GRANTs and RLS DDL apply cleanly.
    sqlx::query(include_str!("../migrations/0001_governance.sql"))
        .execute(&pool)
        .await
        .expect("apply 0001_governance.sql");
    pool
}

/// Begin a transaction scoped to `tenant` for seeding (same GUC the service helper sets).
async fn seed_tx(pool: &PgPool, tenant: Uuid) -> sqlx::Transaction<'_, sqlx::Postgres> {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    tx
}

async fn publish_notice(pool: &PgPool, tenant: Uuid, version: i32, publisher: Uuid) -> Uuid {
    let mut tx = seed_tx(pool, tenant).await;
    // Flip any prior current notice for this tenant to non-current (mirrors clause-1 publish semantics).
    sqlx::query("UPDATE monitoring_notice SET is_current = FALSE WHERE tenant_id = $1 AND is_current")
        .bind(tenant)
        .execute(&mut *tx)
        .await
        .unwrap();
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO monitoring_notice
            (tenant_id, version, lang_en, lang_vi, lawful_basis, is_current, published_by)
         VALUES ($1, $2, 'We monitor work interactions.', 'Chung toi giam sat tuong tac cong viec.',
                 'legitimate_interest', TRUE, $3)
         RETURNING id",
    )
    .bind(tenant)
    .bind(version)
    .bind(publisher)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    id
}

async fn record_ack(pool: &PgPool, tenant: Uuid, subject: Uuid, notice_id: Uuid, version: i32) {
    let mut tx = seed_tx(pool, tenant).await;
    // Quiet operating mode: the default ack_source 'signed_contract' (HR-recorded clause) applies.
    sqlx::query(
        "INSERT INTO subject_acknowledgment
            (tenant_id, subject_id, notice_id, notice_version, recorded_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tenant)
    .bind(subject)
    .bind(notice_id)
    .bind(version)
    .bind(subject) // recorded_by = HR/admin; the subject themselves stands in for the test
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn grant(pool: &PgPool, tenant: Uuid, viewer: Uuid, target: Uuid, scope: &str, by: Uuid) {
    let mut tx = seed_tx(pool, tenant).await;
    sqlx::query(
        "INSERT INTO access_grant
            (tenant_id, viewer_subject_id, target_subject_id, scope, granted_by)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(tenant)
    .bind(viewer)
    .bind(target)
    .bind(scope)
    .bind(by)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn gate_denies_unacknowledged_and_allows_acknowledged() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();
    let admin = Uuid::new_v4();

    // No notice published yet ⇒ everyone gated (capture NOT allowed).
    assert!(
        !gate::is_capture_allowed(&pool, tenant, subject).await.unwrap(),
        "with no published notice the subject must be gated"
    );

    // Publish v1; the subject has not acknowledged ⇒ still gated, reason NoAck.
    let notice_v1 = publish_notice(&pool, tenant, 1, admin).await;
    assert_eq!(
        gate::gate_reason(&pool, tenant, subject).await.unwrap(),
        Some(gate::GateReason::NoAck),
        "un-acknowledged subject must be gated with reason NoAck"
    );
    assert!(!gate::is_capture_allowed(&pool, tenant, subject).await.unwrap());

    // Record the subject's acknowledgment of v1 ⇒ no longer gated, capture allowed.
    record_ack(&pool, tenant, subject, notice_v1, 1).await;
    assert_eq!(
        gate::gate_reason(&pool, tenant, subject).await.unwrap(),
        None,
        "acknowledged subject must not be gated"
    );
    assert!(
        gate::is_capture_allowed(&pool, tenant, subject).await.unwrap(),
        "acknowledged subject must be capture-allowed"
    );

    // Bump to v2 (clause 17 re-gating): the v1 ack is now stale ⇒ gated again, reason StaleAckVersion.
    publish_notice(&pool, tenant, 2, admin).await;
    assert_eq!(
        gate::gate_reason(&pool, tenant, subject).await.unwrap(),
        Some(gate::GateReason::StaleAckVersion),
        "a notice-version bump must re-gate the subject until they re-acknowledge"
    );
    assert!(!gate::is_capture_allowed(&pool, tenant, subject).await.unwrap());
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn access_denies_stranger_allows_founder_self_and_manager() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let manager = Uuid::new_v4();
    let stranger = Uuid::new_v4();
    let target = Uuid::new_v4();
    let granter = Uuid::new_v4();

    // Self: reading one's own record is always permitted (clause 7c), no grant row needed.
    assert!(
        access::can_read_evaluation(&pool, tenant, target, target).await.unwrap(),
        "a subject must be able to read their own record"
    );
    assert_eq!(
        access::may_read(&pool, tenant, target, target).await.unwrap(),
        Some(access::GrantKind::Self_)
    );

    // Stranger with no grant: denied by default.
    assert!(
        !access::can_read_evaluation(&pool, tenant, stranger, target).await.unwrap(),
        "a stranger with no grant must be denied"
    );
    assert_eq!(
        access::may_read(&pool, tenant, stranger, target).await.unwrap(),
        None
    );

    // Founder: a non-revoked founder-scope grant ⇒ may read anyone in the tenant (clause 7a).
    grant(&pool, tenant, founder, target, "founder", granter).await;
    assert_eq!(
        access::may_read(&pool, tenant, founder, target).await.unwrap(),
        Some(access::GrantKind::Founder)
    );
    assert!(access::can_read_evaluation(&pool, tenant, founder, target).await.unwrap());

    // Manager-of: a non-revoked manager_of grant for the exact pair ⇒ allowed (clause 7b).
    grant(&pool, tenant, manager, target, "manager_of", granter).await;
    assert_eq!(
        access::may_read(&pool, tenant, manager, target).await.unwrap(),
        Some(access::GrantKind::ManagerOf)
    );
    assert!(access::can_read_evaluation(&pool, tenant, manager, target).await.unwrap());

    // The manager_of grant is pair-specific: it must NOT let the manager read a different subject.
    let other_target = Uuid::new_v4();
    assert!(
        !access::can_read_evaluation(&pool, tenant, manager, other_target).await.unwrap(),
        "a manager_of grant must not generalise to other targets"
    );
}
