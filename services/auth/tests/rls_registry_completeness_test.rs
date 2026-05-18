//! FR-AUTH-003 §1 #7 — slice-1 audit-fix G-007.
//!
//! Asserts every entry in `cyberos_auth::rls::TENANT_SCOPED_TABLES` is:
//!   1. Present in `pg_tables` (schema=public)
//!   2. Has `rowsecurity = true` (except `auth_signing_keys` which is service-global)
//!   3. Has at least one policy in `pg_policies`
//!   4. Every such policy has BOTH `qual` (USING) and `with_check` populated
//!      (the latter is what stops silent wrong-tenant INSERTs; missing
//!      WITH CHECK is the silent-leak failure mode this gate exists to catch).
//!
//! This is the exact same check `rls::verify_rls_at_boot` runs, plus the
//! WITH CHECK assertion. It runs in CI via `.github/workflows/rls-property-gate.yml`
//! on every migration PR, so an operator who adds a table to the registry
//! without a CREATE POLICY migration (or with a USING-only policy) fails
//! the gate before merge.
//!
//! Requires Postgres + migrations applied. Run locally with:
//!   docker compose -f services/dev/docker-compose.yml up -d
//!   cd services/auth && sqlx migrate run --source migrations
//!   cargo test --test rls_registry_completeness_test -- --ignored

use cyberos_auth::rls::TENANT_SCOPED_TABLES;
use sqlx::PgPool;

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml + apply migrations first"]
async fn every_registered_table_has_rls_with_both_using_and_check_clauses() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var required");
    let pool = PgPool::connect(&url).await.expect("connect to Postgres");

    let mut failures: Vec<String> = Vec::new();

    for table in TENANT_SCOPED_TABLES {
        // ---- 1. Exists ---------------------------------------------------
        let row: Option<(bool,)> = sqlx::query_as(
            "SELECT rowsecurity FROM pg_tables WHERE schemaname='public' AND tablename=$1",
        )
        .bind(table)
        .fetch_optional(&pool)
        .await
        .expect("pg_tables query");

        let Some((rowsec,)) = row else {
            failures.push(format!(
                "{table}: registered in TENANT_SCOPED_TABLES but does not exist in pg_tables"
            ));
            continue;
        };

        // ---- 2. RLS on (auth_signing_keys is exempt — service-global) ----
        if *table == "auth_signing_keys" {
            continue;
        }
        if !rowsec {
            failures.push(format!(
                "{table}: registered but pg_tables.rowsecurity=false — \
                 missing ALTER TABLE … ENABLE ROW LEVEL SECURITY in a migration"
            ));
            continue;
        }

        // ---- 3. At least one policy --------------------------------------
        let (n,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*)::bigint FROM pg_policies WHERE schemaname='public' AND tablename=$1",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("pg_policies count query");

        if n == 0 {
            failures.push(format!(
                "{table}: RLS enabled but ZERO policies — silent allow-all if FORCE not also enabled"
            ));
            continue;
        }

        // ---- 4. Every policy has BOTH qual + with_check ------------------
        // pg_policies.qual = the USING clause; with_check = the WITH CHECK
        // clause. A missing with_check means INSERTs that target the wrong
        // tenant_id will be silently accepted — the canonical "smuggle into
        // someone else's tenant" bug. We MUST have both.
        let bad_rows: Vec<(String,)> = sqlx::query_as(
            "SELECT policyname FROM pg_policies
              WHERE schemaname='public' AND tablename=$1
                AND (qual IS NULL OR with_check IS NULL)",
        )
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("pg_policies qual/with_check query");

        for (policyname,) in bad_rows {
            failures.push(format!(
                "{table}.{policyname}: missing USING (qual) and/or WITH CHECK — \
                 BOTH are required. Without WITH CHECK, wrong-tenant INSERTs slip through."
            ));
        }
    }

    if !failures.is_empty() {
        panic!(
            "rls registry-completeness FAILED — {} violation(s):\n  - {}",
            failures.len(),
            failures.join("\n  - ")
        );
    }
}

/// Sanity: the registry itself is non-empty and reasonably sized. Catches
/// "someone wiped the const accidentally" before any DB query happens.
#[test]
fn registry_size_window() {
    let n = TENANT_SCOPED_TABLES.len();
    assert!(
        (12..=50).contains(&n),
        "TENANT_SCOPED_TABLES should have 12..=50 entries; got {n}. \
         If you intentionally pruned, update this window."
    );
}
