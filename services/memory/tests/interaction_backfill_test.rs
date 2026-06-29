//! FR-MEMORY-122 §5 — backfill integration tests.
//!
//! Proves the replay of recent chat history into `chat.message_created` interaction-events is idempotent,
//! keeps the original timestamp, and obeys the consent gate. Requires a Postgres that has the chat tables
//! (services/chat 0001 chat_core, 0006 direct, 0005 attachment), the memory `l1_audit_log` (0003), and the
//! `cyberos_app` role. Postgres-gated via `#[ignore]`:
//!   DATABASE_URL=... cargo test -p cyberos-memory --test interaction_backfill_test -- --ignored
//!
//! The gate is exercised with the FR-MEMORY-121 `AllowAll` / `DenyAll` stubs (not the real SQL gate, which
//! lives in cyberos-capture and would couple this test to that crate): `AllowAll` proves the idempotency +
//! original-time path; `DenyAll` proves consent skipping. The real SQL gate is covered by the auth + chat
//! capture tests.

use cyberos_memory::interaction::{backfill_chat, AllowAll, DenyAll};
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

/// Seed one group channel + `n` messages authored by `author`, all timestamped `days_ago` days in the past
/// so the test can assert the backfilled event keeps the original (not replay) time. Returns the channel id.
async fn seed_messages(pool: &PgPool, tenant: Uuid, author: Uuid, n: i32, days_ago: i64) -> Uuid {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    let channel: Uuid = sqlx::query_scalar(
        "INSERT INTO chat_channels (tenant_id, name, created_by, kind)
         VALUES ($1, 'general', $2, 'group') RETURNING id",
    )
    .bind(tenant)
    .bind(author)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    for i in 0..n {
        sqlx::query(
            "INSERT INTO chat_messages (tenant_id, channel_id, sender_subject_id, body, created_at)
             VALUES ($1, $2, $3, $4, now() - make_interval(days => $5))",
        )
        .bind(tenant)
        .bind(channel)
        .bind(author)
        .bind(format!("message {i}"))
        .bind(days_ago as i32)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();
    channel
}

async fn count_backfilled(pool: &PgPool, subject: Uuid) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE subject_id = $1
            AND body::jsonb -> 'payload' ->> 'event_type' = 'chat.message_created'",
    )
    .bind(subject)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires DATABASE_URL with chat + memory migrations applied"]
async fn backfill_is_idempotent_and_keeps_original_time() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let alice = Uuid::new_v4();
    seed_messages(&pool, tenant, alice, 5, 3).await;

    // AllowAll stands in for "alice has acknowledged" so every message is eligible.
    let gate = AllowAll;
    let r1 = backfill_chat(&pool, &pool, &gate, tenant, 30, true)
        .await
        .unwrap();
    let r2 = backfill_chat(&pool, &pool, &gate, tenant, 30, true)
        .await
        .unwrap();

    assert_eq!(r1.recorded, 5, "first run records all 5 messages");
    assert_eq!(
        r2.recorded, 0,
        "re-run records nothing (idempotent on the deterministic event_id / audit path)"
    );
    assert_eq!(
        r2.already_present, 5,
        "re-run sees all 5 as already present"
    );
    assert_eq!(
        count_backfilled(&pool, alice).await,
        5,
        "exactly one interaction-event per source message across both runs"
    );

    // The backfilled event sits at the original message time (~3 days ago), not replay time.
    let occurred_ns: i64 = sqlx::query_scalar(
        "SELECT (body::jsonb -> 'payload' ->> 'occurred_at_ns')::bigint FROM l1_audit_log
          WHERE subject_id = $1
            AND body::jsonb -> 'payload' ->> 'event_type' = 'chat.message_created'
          ORDER BY seq LIMIT 1",
    )
    .bind(alice)
    .fetch_one(&pool)
    .await
    .unwrap();
    let now_ns = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    let two_days_ns: i64 = 2 * 24 * 60 * 60 * 1_000_000_000;
    assert!(
        occurred_ns < now_ns - two_days_ns,
        "backfilled at the original message time (~3 days ago), not replay time"
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL with chat + memory migrations applied"]
async fn backfill_skips_unacknowledged_authors() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let bob = Uuid::new_v4();
    seed_messages(&pool, tenant, bob, 3, 3).await;

    // DenyAll stands in for "bob never acknowledged": every message is counted but none is written.
    let gate = DenyAll;
    let r = backfill_chat(&pool, &pool, &gate, tenant, 30, true)
        .await
        .unwrap();

    assert_eq!(r.seen, 3);
    assert_eq!(r.recorded, 0);
    assert_eq!(r.skipped_consent, 3, "all 3 skipped by the consent gate");
    assert_eq!(
        count_backfilled(&pool, bob).await,
        0,
        "nothing written for an unacknowledged author"
    );
}

#[tokio::test]
#[ignore = "requires DATABASE_URL with chat + memory migrations applied"]
async fn dry_run_writes_nothing_but_counts() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let carol = Uuid::new_v4();
    seed_messages(&pool, tenant, carol, 4, 1).await;

    let gate = AllowAll;
    let r = backfill_chat(&pool, &pool, &gate, tenant, 30, false)
        .await
        .unwrap();
    assert_eq!(r.seen, 4, "dry-run counts the candidates");
    assert_eq!(r.recorded, 0, "dry-run writes nothing");
    assert_eq!(count_backfilled(&pool, carol).await, 0);
}
