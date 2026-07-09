//! MEM-005 (report R9, F7) — the recall confidence floor uses the REAL top summary cosine similarity, not a
//! hardcoded 1.0. Before the fix, `best_summary` was set to `1.0` whenever ANY summary matched, so the
//! floor -> drill decision could never fire on quality grounds. Requires Postgres + pgvector; `#[ignore]`.

#[path = "brain_common.rs"]
mod common;

use common::{query, BrainTestEnv};
use cyberos_memory::brain;

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn best_summary_is_real_similarity_not_hardcoded_one() {
    let env = BrainTestEnv::new().await;
    for i in 0..6 {
        env.append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            &format!("standup note {i}"),
        )
        .await;
    }
    env.run_ingest_once().await;
    env.run_summarize_once(
        "subject",
        &env.subject_alice().to_string(),
        Some(env.subject_alice()),
    )
    .await;

    let caller = env.caller_entitled_to(&[env.subject_alice()]).await;

    // A query whose text differs from the (verb-frequency) digest -> cosine similarity STRICTLY below 1.0.
    // Before the fix, best_summary was hardcoded to 1.0 for any match, so this assertion would fail; the
    // regression is that best_summary must be a real similarity derived from the summary embedding.
    let mut q = query("quarterly revenue projections unrelated topic");
    q.drill = false;
    q.explain = true;
    let res = brain::recall::recall(q, &caller, env.pool(), env.gw())
        .await
        .unwrap();

    let explain = res.explain.expect("explain requested");
    let best = explain
        .get("best_summary")
        .and_then(|v| v.as_f64())
        .expect("best_summary present in explain envelope");
    assert!(
        (0.0..1.0).contains(&best),
        "best_summary must be the real top cosine similarity in [0,1), not a hardcoded 1.0 (got {best})"
    );

    env.cleanup().await;
}
