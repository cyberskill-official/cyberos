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

    // Insert two tenants under root context.
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tenants (id, slug, display_name) VALUES ($1, $2, 'Alice Inc')")
        .bind(alice_tenant).bind(format!("alice-{}", &alice_tenant.simple().to_string()[..6]))
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO tenants (id, slug, display_name) VALUES ($1, $2, 'Bob LLC')")
        .bind(bob_tenant).bind(format!("bob-{}", &bob_tenant.simple().to_string()[..6]))
        .execute(&pool).await.unwrap();

    // Each tenant gets one subject.
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(alice_tenant.to_string()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO subjects (tenant_id, handle, kind, password_hash) VALUES ($1, '@alice', 'human', 'bcrypt:fake')")
        .bind(alice_tenant).execute(&pool).await.unwrap();

    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(bob_tenant.to_string()).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO subjects (tenant_id, handle, kind, password_hash) VALUES ($1, '@bob', 'human', 'bcrypt:fake')")
        .bind(bob_tenant).execute(&pool).await.unwrap();

    // Now: switch back to Alice, count subjects. RLS MUST hide Bob.
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(alice_tenant.to_string()).execute(&pool).await.unwrap();
    let (alice_visible,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM subjects")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(alice_visible, 1, "Alice should see exactly 1 subject (herself) — RLS leak detected");

    // Symmetric check for Bob.
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(bob_tenant.to_string()).execute(&pool).await.unwrap();
    let (bob_visible,): (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM subjects")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(bob_visible, 1, "Bob should see exactly 1 subject (himself) — RLS leak detected");

    // Cleanup
    sqlx::query("SET LOCAL app.current_tenant_id = '00000000-0000-0000-0000-000000000000'")
        .execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM subjects WHERE tenant_id = ANY($1)")
        .bind(vec![alice_tenant, bob_tenant]).execute(&pool).await.unwrap();
    sqlx::query("DELETE FROM tenants WHERE id = ANY($1)")
        .bind(vec![alice_tenant, bob_tenant]).execute(&pool).await.unwrap();
}
