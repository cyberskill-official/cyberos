//! FR-MEMORY-123 §5 / AC #7, #8 — hot/warm/cold tiering is idempotent; cold raw is retrievable on demand.
//! Requires Postgres + pgvector; `#[ignore]`.

#[path = "brain_common.rs"]
mod common;

use common::{days_ago_ns, BrainTestEnv};
use cyberos_memory::brain;

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn hot_warm_cold_tiering_is_idempotent() {
    // AC #7: seed events aged into each band (hot < 30d, warm 30-180d, cold > 180d); the tiering pass moves
    // them; re-running the pass is a no-op.
    let env = BrainTestEnv::new().await;
    // 2 hot (today), 2 warm (~90d), 2 cold (~365d).
    for i in 0..2 {
        env.append_interaction_event_at(
            env.subject_alice(),
            "chat.message_created",
            &format!("hot {i}"),
            days_ago_ns(1),
            None,
        )
        .await;
    }
    for i in 0..2 {
        env.append_interaction_event_at(
            env.subject_alice(),
            "chat.message_created",
            &format!("warm {i}"),
            days_ago_ns(90),
            None,
        )
        .await;
    }
    for i in 0..2 {
        env.append_interaction_event_at(
            env.subject_alice(),
            "chat.message_created",
            &format!("cold {i}"),
            days_ago_ns(365),
            None,
        )
        .await;
    }
    env.run_ingest_once().await;

    env.run_tiering_pass().await;
    let before = env.tier_counts().await;
    env.run_tiering_pass().await; // idempotent
    let after = env.tier_counts().await;

    assert_eq!(before, after, "re-running the tiering pass must be a no-op");
    assert!(after.hot >= 2, "recent events stay hot");
    assert!(after.warm >= 2, "30-180d events are warm");
    assert!(after.cold >= 2, "180d+ events are cold");

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn cold_raw_retrievable_on_demand_and_absent_from_hot_index() {
    // AC #8: a cold event's raw Layer-1 row is fetchable by audit_row_id, and its brain row is tier='cold'
    // (so it is not in the partial hot HNSW index).
    let env = BrainTestEnv::new().await;
    let ev = env
        .append_interaction_event_at(
            env.subject_alice(),
            "chat.message_created",
            "ancient decision",
            days_ago_ns(400),
            None,
        )
        .await;
    env.run_ingest_once().await;
    env.run_tiering_pass().await;

    // The brain row is cold.
    let row = env.embedding_row(ev.source_seq).await.expect("row exists");
    assert_eq!(row.0, "cold", "an event older than warm_max_age is cold");

    // The raw Layer-1 row is still retrievable on demand by audit_row_id.
    let raw = brain::provenance::fetch_raw_by_audit_row_id(env.pool(), &ev.audit_row_id)
        .await
        .unwrap();
    assert!(
        raw.is_some(),
        "cold raw stays in Layer 1, retrievable on demand"
    );

    env.cleanup().await;
}
