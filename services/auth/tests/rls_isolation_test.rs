//! FR-AUTH-003 — RLS isolation property test (skeleton).
//!
//! These tests require a live Postgres with migrations applied. CI runs them
//! against `services/dev/docker-compose.yml`. Local dev: `docker compose up -d`
//! in `services/dev/`, then `cargo test -p cyberos-auth -- --ignored`.

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml first"]
async fn cross_tenant_subject_select_returns_zero_rows() {
    use sqlx::PgPool;
    use uuid::Uuid;

    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");

    let alice_tenant = Uuid::new_v4();
    let bob_tenant = Uuid::new_v4();

    // Setup: Insert two tenants and their subjects in one transaction.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', '00000000-0000-0000-0000-000000000000', true)")
            .execute(&mut *tx)
            .await
            .unwrap();

        sqlx::query("INSERT INTO tenants (id, slug, display_name) VALUES ($1, $2, 'Alice Inc')")
            .bind(alice_tenant)
            .bind(format!("alice-{}", &alice_tenant.simple().to_string()[..6]))
            .execute(&mut *tx)
            .await
            .unwrap();

        sqlx::query("INSERT INTO tenants (id, slug, display_name) VALUES ($1, $2, 'Bob LLC')")
            .bind(bob_tenant)
            .bind(format!("bob-{}", &bob_tenant.simple().to_string()[..6]))
            .execute(&mut *tx)
            .await
            .unwrap();

        // Seed Alice's subject.
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(alice_tenant.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("INSERT INTO subjects (tenant_id, handle, kind, password_hash) VALUES ($1, '@alice', 'human', 'bcrypt:fake')")
            .bind(alice_tenant)
            .execute(&mut *tx)
            .await
            .unwrap();

        // Seed Bob's subject.
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(bob_tenant.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("INSERT INTO subjects (tenant_id, handle, kind, password_hash) VALUES ($1, '@bob', 'human', 'bcrypt:fake')")
            .bind(bob_tenant)
            .execute(&mut *tx)
            .await
            .unwrap();

        tx.commit().await.unwrap();
    }

    // Now: switch back to Alice, count subjects. RLS MUST hide Bob.
    let alice_visible = {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE cyberos_app")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(alice_tenant.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM subjects")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        count
    };
    assert_eq!(
        alice_visible, 1,
        "Alice should see exactly 1 subject (herself) — RLS leak detected"
    );

    // Symmetric check for Bob.
    let bob_visible = {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SET LOCAL ROLE cyberos_app")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(bob_tenant.to_string())
            .execute(&mut *tx)
            .await
            .unwrap();
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM subjects")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        count
    };
    assert_eq!(
        bob_visible, 1,
        "Bob should see exactly 1 subject (himself) — RLS leak detected"
    );

    // Cleanup: Remove inserted test data.
    {
        let mut tx = pool.begin().await.unwrap();
        sqlx::query("SELECT set_config('app.current_tenant_id', '00000000-0000-0000-0000-000000000000', true)")
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("DELETE FROM subjects WHERE tenant_id = ANY($1)")
            .bind(vec![alice_tenant, bob_tenant])
            .execute(&mut *tx)
            .await
            .unwrap();
        sqlx::query("DELETE FROM tenants WHERE id = ANY($1)")
            .bind(vec![alice_tenant, bob_tenant])
            .execute(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
}
