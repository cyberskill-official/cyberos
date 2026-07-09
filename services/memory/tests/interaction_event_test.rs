//! FR-MEMORY-121 — emit integration tests (§4 AC 5, 6, 8, 9, 10, 13, 19).
//!
//! Mirrors the memory integration-test convention (tests/ingest_test.rs): these require a live Postgres
//! with the memory migrations applied, so they are `#[ignore]` by default and gate on `DATABASE_URL`. CI
//! boots services/dev/docker-compose.yml and runs `--ignored`. Local:
//!   docker compose up -d            (in services/dev/)
//!   DATABASE_URL=postgres://... cargo test -p cyberos-memory --test interaction_event_test -- --ignored
//!
//! Unlike ingest_test (which assumes migrations are pre-applied), these also apply migrations 0003-0005
//! idempotently, because 0005 is new and may not yet be in a given dev DB. Each test seeds into a fresh
//! random tenant and deletes its rows at the end. The emit path is exercised through the public API
//! (`cyberos_memory::interaction::emit`) with an in-test allow/deny gate — the real FR-EVAL-001-backed
//! gate is wired by FR-MEMORY-122, out of scope here.

use cyberos_memory::interaction::{
    emit, AllowAll, ContentRef, DenyAll, EmitOutcome, EventClass, InteractionEvent, Module,
    SkipReason, SourceChannel, TargetRef,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Connect + apply the audit-log schema (migrations 0003, 0004, 0005), idempotently. 0004's `ADD COLUMN`
/// is not `IF NOT EXISTS`, so re-application is tolerated by swallowing "already exists" errors — the
/// success condition is that the columns are present, however they got there. `DATABASE_URL` is the memory
/// convention; `MEMORY_DATABASE_URL` is honoured as an override if set.
async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("MEMORY_DATABASE_URL"))
        .expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    apply_lenient(
        &pool,
        include_str!("../migrations/0003_layer1_audit_log.sql"),
    )
    .await;
    apply_lenient(&pool, include_str!("../migrations/0004_l1_event_type.sql")).await;
    apply_lenient(
        &pool,
        include_str!("../migrations/0005_interaction_event.sql"),
    )
    .await;
    pool
}

/// Delete every l1_audit_log row for a test tenant — keeps the shared dev DB clean and avoids the dedup
/// unique index (l1_iev_event_id_uq) colliding across runs.
async fn cleanup(pool: &PgPool, tenant: Uuid) {
    sqlx::query("DELETE FROM l1_audit_log WHERE tenant_id = $1")
        .bind(tenant)
        .execute(pool)
        .await
        .ok();
}

/// Apply a migration, swallowing "already exists" so re-runs against a migrated dev DB are a no-op.
async fn apply_lenient(pool: &PgPool, sql: &str) {
    if let Err(e) = sqlx::raw_sql(sql).execute(pool).await {
        let msg = e.to_string();
        if !msg.contains("already exists") {
            panic!("migration failed: {msg}");
        }
    }
}

fn event(tenant: Uuid, subject: Option<Uuid>, verb: &str, class: EventClass) -> InteractionEvent {
    InteractionEvent {
        schema_version: cyberos_memory::interaction::SCHEMA_VERSION,
        event_id: Uuid::now_v7(),
        tenant_id: tenant,
        subject_id: subject,
        occurred_at_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        module: Module::Chat,
        event_type: verb.to_string(),
        event_class: class,
        target_ref: TargetRef::None,
        content_ref: ContentRef::None,
        session_id: None,
        trace_id: None,
        source_channel: SourceChannel::Web,
        attributes: serde_json::Map::new(),
    }
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn valid_event_chains_into_audit_log_as_put() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();

    let ev = InteractionEvent::builder(Module::Chat, "chat.message_created", EventClass::Content)
        .tenant(tenant)
        .subject(subject)
        .occurred_now()
        .target(TargetRef::Message { id: "msg-1".into() })
        .content(ContentRef::pointer("chat_messages", "msg-1"))
        .source(SourceChannel::Web)
        .build()
        .unwrap();

    let out = emit(&pool, &ev, &AllowAll).await.unwrap();
    let seq = match out {
        EmitOutcome::Recorded { seq } => seq,
        other => panic!("expected Recorded, got {other:?}"),
    };

    let (op, event_type, anchor): (String, String, String) =
        sqlx::query_as("SELECT op, event_type, chain_anchor_hex FROM l1_audit_log WHERE seq = $1")
            .bind(seq)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(event_type, "memory.interaction_event");
    assert_eq!(op, "put", "Content class chains as put");
    assert_eq!(anchor.len(), 64, "chain anchor is 64-hex SHA-256");

    // The generated columns reach into the payload (migration 0005).
    let (iev_module, iev_verb, iev_class): (Option<String>, Option<String>, Option<String>) =
        sqlx::query_as(
            "SELECT iev_module, iev_event_type, iev_event_class FROM l1_audit_log WHERE seq = $1",
        )
        .bind(seq)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(iev_module.as_deref(), Some("chat"));
    assert_eq!(iev_verb.as_deref(), Some("chat.message_created"));
    assert_eq!(iev_class.as_deref(), Some("content"));
    cleanup(&pool, tenant).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn read_class_records_as_view() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();

    let ev = InteractionEvent::builder(Module::Proj, "proj.document_opened", EventClass::Read)
        .tenant(tenant)
        .subject(subject)
        .occurred_now()
        .target(TargetRef::Document { id: "doc-1".into() })
        .content(ContentRef::None)
        .source(SourceChannel::Web)
        .build()
        .unwrap();

    let EmitOutcome::Recorded { seq } = emit(&pool, &ev, &AllowAll).await.unwrap() else {
        panic!("expected Recorded");
    };
    let op: String = sqlx::query_scalar("SELECT op FROM l1_audit_log WHERE seq = $1")
        .bind(seq)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(op, "view", "Read class chains as view");
    cleanup(&pool, tenant).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn consent_gate_blocks_unacknowledged_subject_no_row_written() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();

    let ev = event(
        tenant,
        Some(subject),
        "chat.message_created",
        EventClass::Content,
    );
    // DenyAll = the default-deny stub: a subject without consent is skipped, NO row written.
    let out = emit(&pool, &ev, &DenyAll).await.unwrap();
    assert!(
        matches!(
            out,
            EmitOutcome::Skipped {
                reason: SkipReason::ConsentNotAcknowledged
            }
        ),
        "deny-gate must skip with ConsentNotAcknowledged"
    );

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE tenant_id = $1 AND event_type = 'memory.interaction_event'",
    )
    .bind(tenant)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 0, "no row may be written before consent");
    cleanup(&pool, tenant).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn acknowledged_subject_records() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();
    let ev = event(
        tenant,
        Some(subject),
        "chat.message_created",
        EventClass::Content,
    );
    // AllowAll stands in for an acknowledged subject (the real ack lives in FR-EVAL-001 / FR-MEMORY-122).
    assert!(matches!(
        emit(&pool, &ev, &AllowAll).await.unwrap(),
        EmitOutcome::Recorded { .. }
    ));
    cleanup(&pool, tenant).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn system_actor_is_exempt_from_gate() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    // subject_id = None => system actor; the gate is never consulted. Even DenyAll must not block it.
    let ev = event(tenant, None, "cuo.dream_tick", EventClass::Activity);
    let ev = InteractionEvent {
        module: Module::Cuo,
        ..ev
    };
    assert!(
        matches!(
            emit(&pool, &ev, &DenyAll).await.unwrap(),
            EmitOutcome::Recorded { .. }
        ),
        "a system actor (subject_id=None) skips the gate and records even under DenyAll"
    );
    cleanup(&pool, tenant).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn replay_same_event_id_is_idempotent() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();
    let ev = event(
        tenant,
        Some(subject),
        "chat.message_created",
        EventClass::Content,
    );

    let _ = emit(&pool, &ev, &AllowAll).await.unwrap();
    // Re-emit the SAME event_id: the unique partial index (l1_iev_event_id_uq) rejects the duplicate, so
    // the chain never double-counts. Best-effort: a Db error is the expected, swallowed outcome.
    let second = emit(&pool, &ev, &AllowAll).await;
    assert!(
        second.is_err(),
        "the duplicate event_id must be rejected by the unique index"
    );

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE tenant_id = $1 AND iev_event_id = $2",
    )
    .bind(tenant)
    .bind(ev.event_id.to_string())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1, "exactly one row for a replayed event_id");
    cleanup(&pool, tenant).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn recorded_anchor_reverifies_like_memory_reconcile() {
    use sha2::{Digest, Sha256};
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let subject = Uuid::new_v4();
    let ev = event(
        tenant,
        Some(subject),
        "chat.message_created",
        EventClass::Content,
    );
    let EmitOutcome::Recorded { seq } = emit(&pool, &ev, &AllowAll).await.unwrap() else {
        panic!("expected Recorded");
    };

    // Pull body + prev + stored anchor and recompute SHA-256(prev || body) exactly as memory's reconcile
    // does (genesis row => prev is NULL). This proves the interaction-event row verifies under the same
    // invariant as every other chain row (§1 #5).
    let (body, prev, stored): (Option<String>, Option<String>, String) = sqlx::query_as(
        "SELECT body, prev_hash_hex, chain_anchor_hex FROM l1_audit_log WHERE seq = $1",
    )
    .bind(seq)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut h = Sha256::new();
    if let Some(p) = prev.as_deref() {
        h.update(p.as_bytes());
    }
    h.update(body.as_deref().unwrap_or("").as_bytes());
    let recomputed: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(recomputed, stored, "interaction-event anchor must reverify");
    cleanup(&pool, tenant).await;
}
