//! FR-MEMORY-123 §5 / AC #4, #5, #6 — rolling summaries: a summary covers an event range; a new event
//! supersedes the prior version; summaries-first then drill. Requires Postgres + pgvector; `#[ignore]`.

#[path = "brain_common.rs"]
mod common;

use common::{query, BrainTestEnv};
use cyberos_memory::brain::{self, HitSource};

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn rolling_summary_covers_event_range() {
    // AC #4: N events for one subject -> a brain_summary row whose covered_seq_range spans those events.
    let env = BrainTestEnv::new().await;
    let mut first = i64::MAX;
    let mut last = i64::MIN;
    for i in 0..6 {
        let ev = env
            .append_interaction_event(
                env.subject_alice(),
                "chat.message_created",
                &format!("standup note {i}"),
            )
            .await;
        first = first.min(ev.source_seq);
        last = last.max(ev.source_seq);
    }
    env.run_ingest_once().await;
    env.run_summarize_once(
        "subject",
        &env.subject_alice().to_string(),
        Some(env.subject_alice()),
    )
    .await;

    let row: (i64, i64, i64) = sqlx::query_as(
        "SELECT covered_seq_lo, covered_seq_hi, version FROM brain_summary
          WHERE tenant_id = $1 AND scope_kind = 'subject' AND scope_id = $2 AND superseded_by IS NULL",
    )
    .bind(env.tenant())
    .bind(env.subject_alice().to_string())
    .fetch_one(env.pool())
    .await
    .unwrap();
    assert!(
        row.0 <= first && row.1 >= last,
        "covered range must span the seeded events"
    );
    assert_eq!(row.2, 1, "first summary is version 1");

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn new_event_supersedes_prior_summary_version() {
    // AC #5: append an event into an already-summarised window -> a new version is written, the old row is
    // marked superseded_by, recall reads the new version only.
    let env = BrainTestEnv::new().await;
    for i in 0..5 {
        env.append_interaction_event(
            env.subject_alice(),
            "chat.message_created",
            &format!("note {i}"),
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

    // New event + re-ingest + re-summarise.
    env.append_interaction_event(
        env.subject_alice(),
        "chat.message_created",
        "a fresh later note",
    )
    .await;
    env.run_ingest_once().await;
    env.run_summarize_once(
        "subject",
        &env.subject_alice().to_string(),
        Some(env.subject_alice()),
    )
    .await;

    // Exactly one CURRENT version, and it is version 2; version 1 is superseded.
    let current_versions: Vec<(i64, Option<uuid::Uuid>)> = sqlx::query_as(
        "SELECT version, superseded_by FROM brain_summary
          WHERE tenant_id = $1 AND scope_kind = 'subject' AND scope_id = $2 ORDER BY version ASC",
    )
    .bind(env.tenant())
    .bind(env.subject_alice().to_string())
    .fetch_all(env.pool())
    .await
    .unwrap();
    assert_eq!(
        current_versions.len(),
        2,
        "both versions retained for audit"
    );
    assert!(current_versions[0].1.is_some(), "v1 is superseded_by v2");
    assert!(current_versions[1].1.is_none(), "v2 is the current version");

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn summaries_first_then_drill() {
    // AC #6: a recall with drill=false answers from brain_summary (source=summary); drill=true additionally
    // returns event-level hits (source=event).
    let env = BrainTestEnv::new().await;
    for i in 0..20 {
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

    // Summaries-first: at least one summary hit.
    let summary_only = brain::recall::recall(query("standup"), &caller, env.pool(), env.gw())
        .await
        .unwrap();
    assert!(
        summary_only
            .items
            .iter()
            .any(|h| matches!(h.source, HitSource::Summary)),
        "drill=false answers from summaries"
    );

    // Drill: at least one event hit.
    let mut q = query("standup note 3");
    q.drill = true;
    let drilled = brain::recall::recall(q, &caller, env.pool(), env.gw())
        .await
        .unwrap();
    assert!(
        drilled
            .items
            .iter()
            .any(|h| matches!(h.source, HitSource::Event)),
        "drill=true surfaces raw event hits"
    );

    env.cleanup().await;
}
