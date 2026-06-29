//! FR-MEMORY-123 §5 / AC #12, #13, #14 — provenance on every hit; chain_anchor mismatch drops the hit;
//! the audit chain is read-only over an ingest + summarise + tier cycle. Requires Postgres + pgvector;
//! `#[ignore]`.

#[path = "brain_common.rs"]
mod common;

use common::{query, BrainTestEnv};
use cyberos_memory::brain;

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn provenance_points_back_to_audit_rows() {
    // AC #12: each event hit cites exactly one audit_row_id — the row it derived from.
    let env = BrainTestEnv::new().await;
    let ev = env
        .append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            "decision recorded",
        )
        .await;
    env.run_ingest_once().await;

    let caller = env.caller_entitled_to(&[env.subject_alice()]).await;
    let mut q = query("decision recorded");
    q.drill = true;
    let res = brain::recall::recall(q, &caller, env.pool(), env.gw())
        .await
        .unwrap();

    let hit = res.items.first().expect("a hit");
    assert!(
        hit.provenance.audit_row_ids.contains(&ev.audit_row_id),
        "the hit must cite its exact source audit row"
    );
    assert!(hit.provenance.chain_verified, "a clean chain verifies");

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn chain_anchor_mismatch_drops_hit() {
    // AC #13: corrupt the Layer-1 row under a hit; recall drops it (the read-time anchor recompute no longer
    // matches what the row advertises).
    let env = BrainTestEnv::new().await;
    let ev = env
        .append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            "verifiable body",
        )
        .await;
    env.run_ingest_once().await;

    // Tamper: change the body so recompute(prev || new_body) != stored chain_anchor_hex.
    env.corrupt_layer1_row(ev.source_seq, "TAMPERED BODY").await;

    let caller = env.caller_entitled_to(&[env.subject_alice()]).await;
    let mut q = query("verifiable body");
    q.drill = true;
    let res = brain::recall::recall(q, &caller, env.pool(), env.gw())
        .await
        .unwrap();
    // The tampered event's hit must not appear: its read-time anchor recompute fails, so it is dropped.
    assert!(
        !res.items.iter().any(|h| h.audit_row_id == ev.audit_row_id),
        "the tampered event's hit is dropped (chain_anchor mismatch)"
    );

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn worker_never_writes_audit_chain() {
    // AC #14: over a full ingest + summarise + tier cycle, the chain HEAD is unchanged (the brain is
    // read-only over Layer 1).
    let env = BrainTestEnv::new().await;
    for i in 0..6 {
        env.append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            &format!("entry {i}"),
        )
        .await;
    }
    let head_after_seed = env.audit_head().await;

    env.run_ingest_once().await;
    env.run_summarize_once(
        "subject",
        &env.subject_alice().to_string(),
        Some(env.subject_alice()),
    )
    .await;
    env.run_tiering_pass().await;

    let head_after_brain = env.audit_head().await;
    assert_eq!(
        head_after_seed, head_after_brain,
        "the brain must not write, delete, or mutate the chain HEAD"
    );

    env.cleanup().await;
}
