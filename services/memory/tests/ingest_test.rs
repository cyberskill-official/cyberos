//! FR-MEMORY-101 — end-to-end Layer-2 ingest integration tests.
//!
//! Requires Postgres + the memory + auth migrations applied. CI integration
//! job boots services/dev/docker-compose.yml and runs `--ignored`.

use cyberos_memory::layer2::{binlog_tail, binlog_tail::L1Row, chain_anchor, ingest};
use cyberos_types::TenantId;
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    PgPool::connect(&url).await.expect("connect")
}

fn fresh_row(tenant: Uuid, prev_hex: Option<&str>, body: &str, path: &str) -> L1Row {
    let anchor = chain_anchor::compute(prev_hex, body);
    L1Row {
        seq: 0, // assigned by INSERT
        tenant_id: tenant,
        subject_id: None,
        op: "put".into(),
        path: path.into(),
        body: Some(body.into()),
        prev_hash_hex: prev_hex.map(String::from),
        chain_anchor_hex: anchor,
        ts_ns: chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
    }
}

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml first"]
async fn happy_path_ingests_two_rows_and_advances_cursor() {
    let pool = pool().await;
    let tenant_uuid = Uuid::new_v4();
    let tenant = TenantId(tenant_uuid);

    // Insert two rows directly into l1_audit_log.
    let r1 = fresh_row(tenant_uuid, None, "first row body", "memories/first.md");
    let r2_prev = r1.chain_anchor_hex.clone();
    let r2 = fresh_row(tenant_uuid, Some(&r2_prev), "second row body", "memories/second.md");
    binlog_tail::append(&pool, &r1).await.expect("append r1");
    binlog_tail::append(&pool, &r2).await.expect("append r2");

    // Run ingest.
    let summary = ingest::run_batch(&pool, tenant, 100).await.expect("ingest");
    assert_eq!(summary.rows_processed, 2);
    assert!(summary.to_seq > summary.from_seq);

    // l2_memory should now have 2 rows for this tenant.
    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM l2_memory WHERE tenant_id = $1")
        .bind(tenant_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 2, "expected 2 l2_memory rows materialised");

    // Cleanup
    sqlx::query("DELETE FROM l2_memory WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l2_entity WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l2_ingest_cursor WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l2_ingest_cursor_history WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l1_audit_log WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
}

#[tokio::test]
#[ignore = "requires Postgres"]
async fn tampered_chain_anchor_fails_with_mismatch_error() {
    let pool = pool().await;
    let tenant_uuid = Uuid::new_v4();
    let tenant = TenantId(tenant_uuid);

    // Insert a row whose stored chain_anchor LIES (doesn't match body).
    let bad = L1Row {
        seq: 0,
        tenant_id: tenant_uuid,
        subject_id: None,
        op: "put".into(),
        path: "memories/bad.md".into(),
        body: Some("honest body".into()),
        prev_hash_hex: None,
        chain_anchor_hex: "deadbeef".repeat(8), // 64 chars but wrong
        ts_ns: 0,
    };
    binlog_tail::append(&pool, &bad).await.unwrap();

    let res = ingest::run_batch(&pool, tenant, 100).await;
    assert!(
        matches!(res, Err(ingest::IngestError::ChainAnchorMismatch { .. })),
        "expected ChainAnchorMismatch; got {res:?}"
    );

    sqlx::query("DELETE FROM l1_audit_log WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
}

#[tokio::test]
#[ignore = "requires Postgres"]
async fn ingest_is_idempotent_under_replay() {
    let pool = pool().await;
    let tenant_uuid = Uuid::new_v4();
    let tenant = TenantId(tenant_uuid);

    let row = fresh_row(tenant_uuid, None, "idempotent body", "memories/idem.md");
    binlog_tail::append(&pool, &row).await.unwrap();

    let s1 = ingest::run_batch(&pool, tenant, 100).await.unwrap();
    let s2 = ingest::run_batch(&pool, tenant, 100).await.unwrap();
    assert_eq!(s1.rows_processed, 1);
    assert_eq!(s2.rows_processed, 0, "second run must be no-op (cursor already past)");

    let (n,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM l2_memory WHERE tenant_id = $1")
        .bind(tenant_uuid)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 1, "memory count must remain 1 after replay");

    sqlx::query("DELETE FROM l2_memory WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l2_entity WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l2_ingest_cursor WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l2_ingest_cursor_history WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
    sqlx::query("DELETE FROM l1_audit_log WHERE tenant_id = $1").bind(tenant_uuid).execute(&pool).await.ok();
}

#[tokio::test]
#[ignore = "requires Postgres"]
async fn tenant_isolation_a_cannot_see_b_rows() {
    let pool = pool().await;
    let a_uuid = Uuid::new_v4();
    let b_uuid = Uuid::new_v4();

    // Both tenants get one row.
    let ra = fresh_row(a_uuid, None, "alice memo", "memories/a.md");
    let rb = fresh_row(b_uuid, None, "bob memo", "memories/b.md");
    binlog_tail::append(&pool, &ra).await.unwrap();
    binlog_tail::append(&pool, &rb).await.unwrap();

    // Run ingest for A; B's row should NOT be touched.
    let s = ingest::run_batch(&pool, TenantId(a_uuid), 100).await.unwrap();
    assert_eq!(s.rows_processed, 1);

    let (a_count,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM l2_memory WHERE tenant_id = $1")
        .bind(a_uuid).fetch_one(&pool).await.unwrap();
    let (b_count,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM l2_memory WHERE tenant_id = $1")
        .bind(b_uuid).fetch_one(&pool).await.unwrap();
    assert_eq!(a_count, 1, "Alice's row must materialise");
    assert_eq!(b_count, 0, "Bob's row must remain untouched until Bob's ingest runs");

    // Cleanup
    for t in [a_uuid, b_uuid] {
        sqlx::query("DELETE FROM l2_memory WHERE tenant_id = $1").bind(t).execute(&pool).await.ok();
        sqlx::query("DELETE FROM l2_entity WHERE tenant_id = $1").bind(t).execute(&pool).await.ok();
        sqlx::query("DELETE FROM l2_ingest_cursor WHERE tenant_id = $1").bind(t).execute(&pool).await.ok();
        sqlx::query("DELETE FROM l2_ingest_cursor_history WHERE tenant_id = $1").bind(t).execute(&pool).await.ok();
        sqlx::query("DELETE FROM l1_audit_log WHERE tenant_id = $1").bind(t).execute(&pool).await.ok();
    }
}
