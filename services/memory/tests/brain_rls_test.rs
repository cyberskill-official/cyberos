//! MEM-002 (report R74, F16) — the brain-table RLS policies are FAIL-CLOSED.
//!
//! Migration 0009 removed the two bypass arms (unset-GUC and nil-uuid) from every brain-table policy. This
//! test proves the guarantee at the DB: a reader with NO `app.tenant_id` set sees ZERO rows (not all rows),
//! a reader scoped to the wrong tenant sees zero, and only a reader scoped to the owning tenant sees the row.
//!
//! Why a probe role: the dev `cyberos` user is a Postgres SUPERUSER, and superusers bypass RLS entirely, so
//! asserting fail-closed as `cyberos` would prove nothing. The test creates a throwaway NOSUPERUSER,
//! NOBYPASSRLS role, `SET ROLE`s to it, and runs the assertions there — the same posture a real
//! least-privilege service role has. FORCE RLS (asserted by 0009) additionally covers the table-owner case.
//!
//! `#[ignore]` + `DATABASE_URL` / `MEMORY_DATABASE_URL`, same harness as the other DB-backed brain tests:
//!   DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos \
//!     cargo test -p cyberos-memory --test brain_rls_test -- --ignored

use sqlx::{PgPool, Row};
use uuid::Uuid;

const PROBE_ROLE: &str = "mem_rls_probe";

async fn pool() -> PgPool {
    let url = std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("MEMORY_DATABASE_URL"))
        .expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    for sql in [
        include_str!("../migrations/0006_brain_event_embeddings.sql"),
        include_str!("../migrations/0007_brain_summaries.sql"),
        include_str!("../migrations/0008_brain_tier_cursor.sql"),
        include_str!("../migrations/0009_rls_fail_closed.sql"),
    ] {
        apply_lenient(&pool, sql).await;
    }
    pool
}

async fn apply_lenient(pool: &PgPool, sql: &str) {
    // Migration files are multi-statement, so use the simple query protocol (`raw_sql`) rather than
    // `query` (extended protocol, single statement only). The 0006-0009 migrations are all idempotent
    // (CREATE ... IF NOT EXISTS / DROP POLICY IF EXISTS), so a re-run against an already-migrated dev DB is
    // a no-op; the "already exists" swallow stays as belt-and-suspenders.
    if let Err(e) = sqlx::raw_sql(sql).execute(pool).await {
        let msg = e.to_string();
        if !msg.contains("already exists") {
            panic!("migration failed: {msg}");
        }
    }
}

async fn count(conn: &mut sqlx::PgConnection, table: &str) -> i64 {
    let row = sqlx::query(&format!("SELECT count(*) AS n FROM {table}"))
        .fetch_one(&mut *conn)
        .await
        .unwrap_or_else(|e| panic!("count {table}: {e}"));
    row.get::<i64, _>("n")
}

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml first"]
async fn brain_tables_are_fail_closed_without_tenant_guc() {
    let pool = pool().await;
    let tenant_a = Uuid::new_v4();
    let tenant_b = Uuid::new_v4();

    let mut conn = pool.acquire().await.expect("acquire");

    // --- clean any prior probe state + rows (idempotent re-run) ---
    sqlx::query(
        "DO $$ BEGIN
           IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'mem_rls_probe') THEN
             EXECUTE 'DROP OWNED BY mem_rls_probe';
             EXECUTE 'DROP ROLE mem_rls_probe';
           END IF;
         END $$;",
    )
    .execute(&mut *conn)
    .await
    .expect("drop stale probe role");
    for t in [tenant_a, tenant_b] {
        for table in [
            "brain_event_embedding",
            "brain_summary",
            "brain_ingest_cursor",
            "brain_tier_watermark",
        ] {
            sqlx::query(&format!("DELETE FROM {table} WHERE tenant_id = $1"))
                .bind(t)
                .execute(&mut *conn)
                .await
                .ok();
        }
    }

    // --- seed one row per table for tenant A (as superuser: RLS bypassed on write) ---
    sqlx::query(
        "INSERT INTO brain_event_embedding
           (tenant_id, source_seq, audit_row_id, subject_id, kind, ts_ns, chain_anchor)
         VALUES ($1, 1, 'l1:probe:1', $1, 'chat.message_created', 1, decode('00','hex'))",
    )
    .bind(tenant_a)
    .execute(&mut *conn)
    .await
    .expect("seed event");
    sqlx::query(
        "INSERT INTO brain_summary
           (tenant_id, scope_kind, scope_id, window_start_ns, window_end_ns, covered_seq_lo, covered_seq_hi, digest)
         VALUES ($1, 'subject', $1::text, 0, 1, 1, 1, 'probe digest')",
    )
    .bind(tenant_a)
    .execute(&mut *conn)
    .await
    .expect("seed summary");
    sqlx::query("INSERT INTO brain_ingest_cursor (tenant_id, last_source_seq) VALUES ($1, 1)")
        .bind(tenant_a)
        .execute(&mut *conn)
        .await
        .expect("seed cursor");
    sqlx::query("INSERT INTO brain_tier_watermark (tenant_id, last_tiered_ts_ns) VALUES ($1, 1)")
        .bind(tenant_a)
        .execute(&mut *conn)
        .await
        .expect("seed watermark");

    // --- create the non-superuser probe role + grants ---
    sqlx::query("CREATE ROLE mem_rls_probe NOSUPERUSER NOBYPASSRLS NOLOGIN")
        .execute(&mut *conn)
        .await
        .expect("create probe role");
    sqlx::query("GRANT USAGE ON SCHEMA public TO mem_rls_probe")
        .execute(&mut *conn)
        .await
        .expect("grant usage");
    sqlx::query(
        "GRANT SELECT ON brain_event_embedding, brain_summary, brain_ingest_cursor, brain_tier_watermark
         TO mem_rls_probe",
    )
    .execute(&mut *conn)
    .await
    .expect("grant select");

    // --- switch to the probe role; RLS now applies ---
    sqlx::query(&format!("SET ROLE {PROBE_ROLE}"))
        .execute(&mut *conn)
        .await
        .expect("set role");

    // 1) No app.tenant_id set -> fail-closed -> zero rows on every table (this is the F16 hole, now closed).
    for table in [
        "brain_event_embedding",
        "brain_summary",
        "brain_ingest_cursor",
        "brain_tier_watermark",
    ] {
        assert_eq!(
            count(&mut conn, table).await,
            0,
            "{table}: an unset app.tenant_id must read ZERO rows (fail-closed)"
        );
    }

    // 2) Scoped to the WRONG tenant -> still zero (cross-tenant invisible).
    sqlx::query("SELECT set_config('app.tenant_id', $1, false)")
        .bind(tenant_b.to_string())
        .execute(&mut *conn)
        .await
        .expect("set tenant b");
    assert_eq!(
        count(&mut conn, "brain_event_embedding").await,
        0,
        "tenant B must not see tenant A's row"
    );

    // 3) Scoped to the OWNING tenant -> the seeded row is visible (the normal path still works).
    sqlx::query("SELECT set_config('app.tenant_id', $1, false)")
        .bind(tenant_a.to_string())
        .execute(&mut *conn)
        .await
        .expect("set tenant a");
    assert_eq!(
        count(&mut conn, "brain_event_embedding").await,
        1,
        "tenant A must see its own row when scoped correctly"
    );

    // --- restore + clean up (RESET ALL clears role + the session GUC so the pooled conn returns clean) ---
    sqlx::query("RESET ALL").execute(&mut *conn).await.ok();
    for t in [tenant_a, tenant_b] {
        for table in [
            "brain_event_embedding",
            "brain_summary",
            "brain_ingest_cursor",
            "brain_tier_watermark",
        ] {
            sqlx::query(&format!("DELETE FROM {table} WHERE tenant_id = $1"))
                .bind(t)
                .execute(&mut *conn)
                .await
                .ok();
        }
    }
    sqlx::query("DROP OWNED BY mem_rls_probe")
        .execute(&mut *conn)
        .await
        .ok();
    sqlx::query("DROP ROLE mem_rls_probe")
        .execute(&mut *conn)
        .await
        .ok();
}
