//! TASK-AUTH-003 §1 #6 — slice-1 audit-fix G-006.
//!
//! Property test: 100 random tenant pairs × a mix of SELECT/INSERT/UPDATE/
//! DELETE operations. For each pair (A, B), after setting
//! `app.current_tenant_id = A`, the test asserts:
//!
//!   - SELECT on any tenant-scoped table returns ZERO rows belonging to B.
//!   - INSERT on a tenant-scoped table with `tenant_id = B` raises
//!     `42501 insufficient_privilege` (the WITH CHECK rejection).
//!   - UPDATE that tries to mutate a B-owned row is a no-op (RLS filters
//!     B's rows out of A's SELECT-then-UPDATE scope; affected = 0).
//!   - DELETE on a B-owned row is a no-op for the same reason.
//!
//! We use `subjects` as the canonical exemplar table because every other
//! tenant-scoped table follows the same shape and the same policy
//! template, and exercising one table thoroughly is sufficient to catch
//! the policy-template-regression failure mode. (The
//! `rls_registry_completeness_test` covers the "every-table-has-WITH-CHECK"
//! axis independently.)
//!
//! Requires Postgres + migrations applied. Run locally with:
//!   docker compose -f services/dev/docker-compose.yml up -d
//!   cd services/auth && sqlx migrate run --source migrations
//!   cargo test --test rls_property_test -- --ignored

use cyberos_auth::rls;
use sqlx::PgPool;
use uuid::Uuid;

const NIL_TENANT: &str = "00000000-0000-0000-0000-000000000000";

macro_rules! with_tenant {
    ($pool:expr, $tenant:expr, $tx:ident, $body:expr) => {{
        let mut $tx = $pool.begin().await.expect("begin tx");
        sqlx::query("SET LOCAL ROLE cyberos_app")
            .execute(&mut *$tx)
            .await
            .expect("set role");
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind($tenant.to_string())
            .execute(&mut *$tx)
            .await
            .expect("set GUC");
        $body
    }};
}

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml + apply migrations first"]
async fn property_test_100_tenant_pairs_no_cross_tenant_leak() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var required");
    let pool = PgPool::connect(&url).await.expect("connect");

    // Sanity: boot-time check passes before we run the property test.
    // If THIS fails, the registry/migrations are broken and the rest of
    // the test is meaningless.
    rls::verify_rls_at_boot(&pool)
        .await
        .expect("verify_rls_at_boot should pass — registry/migrations mismatch");

    // Create 200 tenants up-front; we'll pair them off.
    let tenants: Vec<Uuid> = (0..200).map(|_| Uuid::new_v4()).collect();

    // Use root tenant context for setup.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(NIL_TENANT)
            .execute(&mut *tx)
            .await
            .unwrap();
        for t in &tenants {
            let slug = format!("ten-{}", &t.simple().to_string()[..12]);
            sqlx::query("INSERT INTO tenants (id, slug, display_name) VALUES ($1, $2, $3)")
                .bind(t)
                .bind(&slug)
                .bind(format!("Tenant {slug}"))
                .execute(&mut *tx)
                .await
                .expect("insert tenant");

            // Seed exactly one subject per tenant.
            sqlx::query(
                "INSERT INTO subjects (tenant_id, handle, kind, password_hash)
                 VALUES ($1, $2, 'human', 'bcrypt:fake-for-test')",
            )
            .bind(t)
            .bind(format!("@u-{}", &t.simple().to_string()[..8]))
            .execute(&mut *tx)
            .await
            .expect("insert subject");
        }
        tx.commit().await.unwrap();
    }

    let mut failures: Vec<String> = Vec::new();

    // 100 pairs — adjacent indices, no repeats.
    for i in 0..100 {
        let a = tenants[i * 2];
        let b = tenants[i * 2 + 1];

        // ─── 1. SELECT — A must not see B's subjects ─────────────────────
        let (visible_to_a,): (i64,) = with_tenant!(pool, a, tx, {
            let r = sqlx::query_as::<_, (i64,)>(
                "SELECT COUNT(*)::bigint FROM subjects WHERE tenant_id = $1",
            )
            .bind(b)
            .fetch_one(&mut *tx)
            .await
            .expect("count query");
            tx.commit().await.unwrap();
            r
        });
        if visible_to_a != 0 {
            failures.push(format!(
                "pair {i}: tenant {a} could SELECT {visible_to_a} row(s) belonging to {b} — RLS leak"
            ));
        }

        // ─── 2. INSERT — A trying to write with tenant_id=B must 42501 ──
        let insert_result = with_tenant!(pool, a, tx, {
            let r = sqlx::query(
                "INSERT INTO subjects (tenant_id, handle, kind, password_hash)
                 VALUES ($1, $2, 'human', 'bcrypt:smuggled')",
            )
            .bind(b)
            .bind(format!("@smuggle-{}", &b.simple().to_string()[..6]))
            .execute(&mut *tx)
            .await;
            // Don't commit either way; we only want the error code.
            r
        });
        match insert_result {
            Ok(_) => failures.push(format!(
                "pair {i}: smuggle INSERT (A={a} writing tenant_id={b}) SUCCEEDED — WITH CHECK missing or broken"
            )),
            Err(e) => {
                if rls::map_pg_error(&e).is_none() {
                    failures.push(format!(
                        "pair {i}: smuggle INSERT failed but with non-42501 error: {e}"
                    ));
                }
            }
        }

        // ─── 3. UPDATE — A targeting a B-owned row must affect 0 rows ───
        let updated = with_tenant!(pool, a, tx, {
            let r = sqlx::query(
                "UPDATE subjects SET password_hash = 'bcrypt:overwritten' WHERE tenant_id = $1",
            )
            .bind(b)
            .execute(&mut *tx)
            .await
            .expect("update query");
            tx.commit().await.unwrap();
            r.rows_affected()
        });
        if updated != 0 {
            failures.push(format!(
                "pair {i}: UPDATE from A={a} touched {updated} row(s) belonging to {b} — RLS leak"
            ));
        }

        // ─── 4. DELETE — A trying to remove B's row must affect 0 rows ──
        let deleted = with_tenant!(pool, a, tx, {
            let r = sqlx::query("DELETE FROM subjects WHERE tenant_id = $1")
                .bind(b)
                .execute(&mut *tx)
                .await
                .expect("delete query");
            tx.commit().await.unwrap();
            r.rows_affected()
        });
        if deleted != 0 {
            failures.push(format!(
                "pair {i}: DELETE from A={a} removed {deleted} row(s) belonging to {b} — RLS leak"
            ));
        }
    }

    // Cleanup — use root tenant context.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(NIL_TENANT)
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("DELETE FROM subjects WHERE tenant_id = ANY($1)")
            .bind(&tenants[..])
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("DELETE FROM tenants WHERE id = ANY($1)")
            .bind(&tenants[..])
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    if !failures.is_empty() {
        panic!(
            "rls property test FAILED — {} violation(s) across 100 pairs:\n  - {}",
            failures.len(),
            failures.join("\n  - ")
        );
    }
}

/// Focused single-pair test that explicitly verifies the WITH CHECK 42501
/// path round-trips through `rls::map_pg_error()` to a 403 body. This
/// covers the "happy path of the error-mapper" case the unit tests
/// couldn't construct without a real backend.
#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml + apply migrations first"]
async fn with_check_rejects_wrong_tenant_insert_via_map_pg_error() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var required");
    let pool = PgPool::connect(&url).await.expect("connect");

    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();

    // Setup under root.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(NIL_TENANT)
            .execute(&mut *tx)
            .await
            .unwrap();
        for (t, slug) in [(alice, "alice"), (bob, "bob")] {
            let slug = format!("{slug}-{}", &t.simple().to_string()[..6]);
            sqlx::query("INSERT INTO tenants (id, slug, display_name) VALUES ($1, $2, $3)")
                .bind(t)
                .bind(&slug)
                .bind(format!("Tenant {slug}"))
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
    }

    // Switch to Alice; try to insert a subject with Bob's tenant_id.
    let err = {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE cyberos_app")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(alice.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let r = sqlx::query(
            "INSERT INTO subjects (tenant_id, handle, kind, password_hash)
             VALUES ($1, '@smuggled', 'human', 'bcrypt:fake')",
        )
        .bind(bob)
        .execute(&mut *tx)
        .await;
        // Don't bother committing.
        r.expect_err("INSERT with wrong tenant_id MUST fail with 42501")
    };

    let mapped = rls::map_pg_error(&err);
    assert!(
        mapped.is_some(),
        "map_pg_error must return Some(_) for a 42501 error; got None for: {err}"
    );
    let (status, body) = mapped.unwrap();
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    let body_str = serde_json::to_string(&body.0).unwrap();
    assert!(
        body_str.contains("rls_check_violation"),
        "body should mention rls_check_violation; got: {body_str}"
    );

    // Cleanup
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(NIL_TENANT)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("DELETE FROM tenants WHERE id = ANY($1)")
        .bind(vec![alice, bob])
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}
