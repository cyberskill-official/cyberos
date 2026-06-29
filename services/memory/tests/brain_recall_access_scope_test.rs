//! FR-MEMORY-123 §5 / AC #9, #10, #11 — recall is access-scoped: tenant RLS PLUS the FR-EVAL-001 per-subject
//! predicate. A semantically-closest neighbour the caller may not see is EXCLUDED (not deranked); an unknown
//! subject denies by default. Requires Postgres + pgvector; `#[ignore]` by default.

#[path = "brain_common.rs"]
mod common;

use common::{query, BrainTestEnv};
use cyberos_memory::brain::{self, HitSource};

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn recall_excludes_closest_neighbour_outside_access_scope() {
    // AC #10: the single closest cosine neighbour belongs to a subject (bob) the caller may NOT see. With
    // drill, the raw event is the top neighbour of the exact-match query; recall must EXCLUDE it.
    let env = BrainTestEnv::new().await;

    // Bob's event is the exact-match for the query text (its embedding will be closest).
    env.append_interaction_event(
        env.subject_bob(),
        "chat.message_created",
        "exact query text match",
    )
    .await;
    // Alice has an unrelated event so the caller (entitled to alice) has something legitimately visible.
    env.append_interaction_event(
        env.subject_alice(),
        "chat.message_created",
        "totally different note",
    )
    .await;
    env.run_ingest_once().await;

    // Caller is entitled to ALICE only — NOT bob.
    let caller = env.caller_entitled_to(&[env.subject_alice()]).await;
    let mut q = query("exact query text match");
    q.drill = true; // force raw-event search so bob's exact-match event is a candidate
    let res = brain::recall::recall(q, &caller, env.pool(), env.gw())
        .await
        .unwrap();

    assert!(
        res.items.iter().all(|h| h.subject_id != env.subject_bob()),
        "bob's event must be EXCLUDED (not returned), even as the closest neighbour"
    );

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn deny_by_default_on_unknown_subject() {
    // AC #11: a subject with NO FR-EVAL-001 entitlement returns 0 hits for that subject. The caller is
    // entitled to nobody; an event for an unknown subject must not surface.
    let env = BrainTestEnv::new().await;
    let stranger = uuid::Uuid::new_v4();
    env.append_interaction_event(stranger, "chat.message_created", "stranger says hello")
        .await;
    env.run_ingest_once().await;

    // Caller entitled to no subjects at all (only their own self path, which is a different subject).
    let caller = env.caller_entitled_to(&[]).await;
    let mut q = query("stranger says hello");
    q.drill = true;
    let res = brain::recall::recall(q, &caller, env.pool(), env.gw())
        .await
        .unwrap();
    assert!(
        res.items.is_empty(),
        "deny-by-default: an unentitled caller sees no hits for an unknown subject"
    );

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn founder_sees_any_subject_self_sees_own() {
    // The positive access paths: a founder caller sees a hit for any subject; a subject sees their own.
    let env = BrainTestEnv::new().await;
    env.append_interaction_event(
        env.subject_bob(),
        "chat.message_created",
        "bob ships the launch",
    )
    .await;
    env.run_ingest_once().await;

    // Founder may see bob.
    let founder = env.founder_caller().await;
    let mut q = query("bob ships the launch");
    q.drill = true;
    let res = brain::recall::recall(q, &founder, env.pool(), env.gw())
        .await
        .unwrap();
    assert!(
        res.items
            .iter()
            .any(|h| h.subject_id == env.subject_bob() && matches!(h.source, HitSource::Event)),
        "founder must see bob's event"
    );

    // Bob himself (self path) may see his own event with no grant.
    let bob_self = brain::Caller {
        tenant_id: env.tenant(),
        viewer_subject_id: env.subject_bob(),
    };
    let mut q2 = query("bob ships the launch");
    q2.drill = true;
    let res2 = brain::recall::recall(q2, &bob_self, env.pool(), env.gw())
        .await
        .unwrap();
    assert!(
        res2.items.iter().any(|h| h.subject_id == env.subject_bob()),
        "bob sees his own record via the self path"
    );

    env.cleanup().await;
}

#[tokio::test]
#[ignore = "requires Postgres + pgvector"]
async fn tenant_rls_excludes_other_tenants_rows() {
    // AC #9: a caller in tenant A never receives tenant B's hits (RLS at the DB). We seed B's event under a
    // second env (separate tenant) and confirm A's recall, even as founder, returns nothing of B's.
    let a = BrainTestEnv::new().await;
    let b = BrainTestEnv::new().await;
    b.append_interaction_event(
        b.subject_alice(),
        "chat.message_created",
        "tenant B secret note",
    )
    .await;
    b.run_ingest_once().await;

    let a_founder = a.founder_caller().await;
    let mut q = query("tenant B secret note");
    q.drill = true;
    let res = brain::recall::recall(q, &a_founder, a.pool(), a.gw())
        .await
        .unwrap();
    assert!(
        res.items.is_empty(),
        "tenant A recall must not see tenant B rows (RLS)"
    );

    a.cleanup().await;
    b.cleanup().await;
}
