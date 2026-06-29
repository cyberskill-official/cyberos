//! FR-MEMORY-122 §5 — AUTH capture integration tests.
//!
//! Proves the AUTH capture path end-to-end against a live Postgres that has BOTH the memory `l1_audit_log`
//! table (the destination) and the eval governance tables (`monitoring_notice` + `subject_acknowledgment`,
//! the consent gate's source). The three cases:
//!   * an acknowledged subject's sign-in emits exactly one `auth.signed_in` interaction-event;
//!   * an unacknowledged subject's sign-in emits NOTHING (the consent gate held);
//!   * capture against a dead pool does not panic / propagate (best-effort) — the emit is swallowed.
//!
//! Postgres-gated via `#[ignore]` like the other auth integration tests; run with:
//!   DATABASE_URL=... cargo test -p cyberos-auth --test capture_signin_test -- --ignored
//! The DB must have migrations from services/memory (0003 l1_audit_log) and services/eval (0001 governance)
//! applied, and the `cyberos_app` role present (the gate + audit writer run under it).

use cyberos_capture::Capturer;
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE cyberos_app").execute(conn).await.ok();
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect")
}

/// Publish a current monitoring notice for `tenant` (version 1), returning its id. Sets the tenant GUC so
/// the RLS WITH CHECK passes. Idempotent enough for a test: flips any prior current row first.
async fn publish_notice(pool: &PgPool, tenant: Uuid) -> Uuid {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE monitoring_notice SET is_current = FALSE WHERE is_current")
        .execute(&mut *tx)
        .await
        .unwrap();
    let id: Uuid = sqlx::query_scalar(
        "INSERT INTO monitoring_notice
            (tenant_id, version, lang_en, lang_vi, lawful_basis, is_current, published_by)
         VALUES ($1, 1, 'en', 'vi', 'legitimate_interest', TRUE, $2)
         RETURNING id",
    )
    .bind(tenant)
    .bind(Uuid::nil())
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    id
}

/// Record `subject`'s acknowledgment of the current notice version (the signed-contract source).
async fn acknowledge(pool: &PgPool, tenant: Uuid, subject: Uuid, notice_id: Uuid) {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO subject_acknowledgment
            (tenant_id, subject_id, notice_id, notice_version, ack_source, recorded_by)
         VALUES ($1, $2, $3, 1, 'signed_contract', $4)
         ON CONFLICT (tenant_id, subject_id, notice_version) DO NOTHING",
    )
    .bind(tenant)
    .bind(subject)
    .bind(notice_id)
    .bind(Uuid::nil())
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn count_signed_in(pool: &PgPool, subject: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE event_type = 'memory.interaction_event'
            AND subject_id = $1
            AND body::jsonb -> 'payload' ->> 'event_type' = 'auth.signed_in'",
    )
    .bind(subject)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires DATABASE_URL with memory + eval migrations applied"]
async fn signin_emits_interaction_event_for_acknowledged_subject() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let alice = Uuid::new_v4();
    let notice = publish_notice(&pool, tenant).await;
    acknowledge(&pool, tenant, alice, notice).await;

    let cap = Capturer::new(pool.clone());
    cyberos_auth::capture::emit_signed_in(
        Some(&cap),
        tenant,
        alice,
        "jti-abc123",
        "password",
        cyberos_capture::SourceChannel::Web,
        "9f86d081884c7d65",
        None,
    )
    .await;

    assert_eq!(
        count_signed_in(&pool, alice).await,
        1,
        "an acknowledged subject's sign-in must record exactly one auth.signed_in event"
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL with memory + eval migrations applied"]
async fn signin_for_unacknowledged_subject_captures_nothing() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let bob = Uuid::new_v4();
    // Publish a notice but DO NOT acknowledge for bob — the gate must deny.
    let _notice = publish_notice(&pool, tenant).await;

    let cap = Capturer::new(pool.clone());
    cyberos_auth::capture::emit_signed_in(
        Some(&cap),
        tenant,
        bob,
        "jti-def456",
        "password",
        cyberos_capture::SourceChannel::Web,
        "9f86d081884c7d65",
        None,
    )
    .await;

    assert_eq!(
        count_signed_in(&pool, bob).await,
        0,
        "an unacknowledged subject must produce zero rows (consent gate held)"
    );
}

#[tokio::test]
async fn capture_against_a_dead_pool_does_not_panic() {
    // A closed pool stands in for "audit DB down": the emit must be swallowed, not panic or propagate. This
    // is the best-effort guarantee that a capture failure never breaks sign-in. No DB needed.
    let dead = PgPool::connect_lazy("postgres://invalid:invalid@127.0.0.1:1/none").unwrap();
    let cap = Capturer::new(dead);
    // Must simply return; the absence of a panic is the assertion.
    cyberos_auth::capture::emit_signed_in(
        Some(&cap),
        Uuid::new_v4(),
        Uuid::new_v4(),
        "jti-x",
        "password",
        cyberos_capture::SourceChannel::Web,
        "deadbeefdeadbeef",
        None,
    )
    .await;
}
