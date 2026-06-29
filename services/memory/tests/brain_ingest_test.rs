//! FR-MEMORY-123 §5 / AC #1, #3 — brain ingest: event ingested -> embedding visible; idempotent UPSERT;
//! cursor resume. Requires Postgres + pgvector; `#[ignore]` by default (boot services/dev/docker-compose.yml,
//! `CREATE EXTENSION vector`, then `cargo test -p cyberos-memory --test brain_ingest_test -- --ignored`).

#[path = "brain_common.rs"]
mod common;

use common::BrainTestEnv;

#[tokio::test]
#[ignore = "requires Postgres + pgvector — boot services/dev/docker-compose.yml first"]
async fn event_ingested_then_embedding_visible() {
    // AC #1: append a FR-MEMORY-121 event; after ingest a brain_event_embedding row exists with the event's
    // audit_row_id and tier='hot'.
    let env = BrainTestEnv::new().await;
    let ev = env
        .append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            "shipped the proj sync",
        )
        .await;
    env.run_ingest_once().await;

    let row = env
        .embedding_row(ev.source_seq)
        .await
        .expect("embedding row exists");
    assert_eq!(row.0, "hot", "freshly ingested event is hot");
    assert_eq!(
        row.1, ev.audit_row_id,
        "row carries the event's provenance audit_row_id"
    );
    assert_eq!(
        row.2.as_deref(),
        Some("complete"),
        "embedded via the stub gateway"
    );

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn idempotent_ingest_replay_produces_one_row() {
    // AC #3: re-processing the same source_seq (simulated restart-mid-batch) -> exactly one row; the cursor
    // resumes without re-embedding earlier rows.
    let env = BrainTestEnv::new().await;
    let ev = env
        .append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            "idempotent body",
        )
        .await;

    env.run_ingest_once().await;
    // Run again: the cursor is past this seq, and even if it weren't the UPSERT is idempotent.
    env.run_ingest_once().await;

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM brain_event_embedding WHERE tenant_id = $1 AND source_seq = $2",
    )
    .bind(env.tenant())
    .bind(ev.source_seq)
    .fetch_one(env.pool())
    .await
    .unwrap();
    assert_eq!(n, 1, "replay must not duplicate the embedding row");

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn cursor_resumes_and_picks_up_new_events() {
    // AC #1/#3: after an ingest, appending more events and re-ingesting picks up ONLY the new ones (the
    // cursor resumed). The total embedding count equals the total events appended.
    let env = BrainTestEnv::new().await;
    env.append_interaction_event(env.subject_alice(), "chat.message_created", "first")
        .await;
    env.append_interaction_event(env.subject_alice(), "chat.message_created", "second")
        .await;
    env.run_ingest_once().await;
    assert_eq!(env.embedding_count().await, 2);

    // Append two more, re-ingest: the cursor resumes from where it left off.
    env.append_interaction_event(env.subject_alice(), "chat.message_created", "third")
        .await;
    env.append_interaction_event(env.subject_alice(), "chat.message_created", "fourth")
        .await;
    env.run_ingest_once().await;
    assert_eq!(
        env.embedding_count().await,
        4,
        "cursor resumed and ingested the new events"
    );

    env.cleanup().await;
}
