//! FR-MEMORY-121 §1 #14 / §4 AC 16 — tenant isolation for interaction-events.
//!
//! Interaction-events ARE `l1_audit_log` rows, so they inherit that table's tenant scoping — there is no
//! second table and therefore no second policy to drift. On `l1_audit_log` (migration 0003) isolation is
//! enforced on the read path: every memory reader (`binlog_tail::poll`, `rebuild::reconcile`, the
//! FR-APP-005 viewer) filters `WHERE tenant_id = $1`, and the interaction-event partial indexes
//! (migration 0005) are likewise tenant-leading. This test proves tenant A's interaction rows never
//! surface in a tenant-B-scoped read.
//!
//! `#[ignore]` + `MEMORY_DATABASE_URL`/`DATABASE_URL`, same harness as interaction_event_test.

use cyberos_memory::interaction::{
    emit, AllowAll, EmitOutcome, EventClass, InteractionEvent, Module, SourceChannel,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("MEMORY_DATABASE_URL"))
        .expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    apply_lenient(&pool, include_str!("../migrations/0003_layer1_audit_log.sql")).await;
    apply_lenient(&pool, include_str!("../migrations/0004_l1_event_type.sql")).await;
    apply_lenient(&pool, include_str!("../migrations/0005_interaction_event.sql")).await;
    pool
}

async fn apply_lenient(pool: &PgPool, sql: &str) {
    if let Err(e) = sqlx::query(sql).execute(pool).await {
        let msg = e.to_string();
        if !msg.contains("already exists") {
            panic!("migration failed: {msg}");
        }
    }
}

async fn cleanup(pool: &PgPool, tenant: Uuid) {
    sqlx::query("DELETE FROM l1_audit_log WHERE tenant_id = $1")
        .bind(tenant)
        .execute(pool)
        .await
        .ok();
}

fn ev(tenant: Uuid, subject: Uuid) -> InteractionEvent {
    InteractionEvent {
        schema_version: cyberos_memory::interaction::SCHEMA_VERSION,
        event_id: Uuid::now_v7(),
        tenant_id: tenant,
        subject_id: Some(subject),
        occurred_at_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
        module: Module::Chat,
        event_type: "chat.message_created".to_string(),
        event_class: EventClass::Content,
        target_ref: cyberos_memory::interaction::TargetRef::None,
        content_ref: cyberos_memory::interaction::ContentRef::None,
        session_id: None,
        trace_id: None,
        source_channel: SourceChannel::Web,
        attributes: serde_json::Map::new(),
    }
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn tenant_a_rows_invisible_to_tenant_b_read() {
    let pool = pool().await;
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let subject_a = Uuid::new_v4();

    // Record one interaction-event for tenant A.
    let EmitOutcome::Recorded { seq } = emit(&pool, &ev(tenant_a, subject_a), &AllowAll)
        .await
        .unwrap()
    else {
        panic!("expected Recorded");
    };

    // A tenant-B-scoped read (the read path every memory consumer uses) must not see tenant A's row.
    let visible_to_b: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE tenant_id = $1 AND seq = $2",
    )
    .bind(tenant_b)
    .bind(seq)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(visible_to_b, 0, "tenant B must not see tenant A's interaction-event");

    // The same read scoped to tenant A sees exactly the one row — isolation does not hide it from its owner.
    let visible_to_a: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE tenant_id = $1 AND seq = $2 AND event_type = 'memory.interaction_event'",
    )
    .bind(tenant_a)
    .bind(seq)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(visible_to_a, 1, "tenant A must see its own interaction-event");
    cleanup(&pool, tenant_a).await;
    cleanup(&pool, tenant_b).await;
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn per_subject_index_scan_is_tenant_scoped() {
    let pool = pool().await;
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();
    let shared_subject = Uuid::new_v4(); // same subject id used under both tenants (worst case)

    let _ = emit(&pool, &ev(tenant_a, shared_subject), &AllowAll).await.unwrap();
    let _ = emit(&pool, &ev(tenant_b, shared_subject), &AllowAll).await.unwrap();

    // The l1_iev_subject_idx scan (tenant_id, subject_id, ts_ns) for tenant A + the shared subject must
    // return only tenant A's row, even though tenant B has a row for the same subject id.
    let n_a: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM l1_audit_log
          WHERE tenant_id = $1 AND subject_id = $2 AND event_type = 'memory.interaction_event'",
    )
    .bind(tenant_a)
    .bind(shared_subject)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n_a, 1, "the subject-scoped scan must not bleed across tenants");
    cleanup(&pool, tenant_a).await;
    cleanup(&pool, tenant_b).await;
}
