//! FR-EVAL-001 clause 6 - retention/erasure sweeper integration tests, plus the clause-18 governance-status
//! shape.
//!
//! Mirrors `governance_gate_test.rs`: requires a live Postgres with the eval migration applied, so the tests
//! are `#[ignore]` by default and gate on `EVAL_DATABASE_URL` (falling back to `DATABASE_URL`). Local:
//!   docker compose up -d   (in services/dev/)
//!   EVAL_DATABASE_URL=postgres://... cargo test -p cyberos-eval --test retention_sweep_test -- --ignored
//!
//! The sweep erases the DERIVED (L2 / BRAIN) projections, which live in the memory module's Postgres. So a
//! sweep test needs those tables present. Rather than `include_str!` the memory migrations (which carry
//! unguarded GRANTs + a pgvector dependency that need the full dev compose), these tests create MINIMAL
//! stand-ins for `l2_memory`, `brain_event_embedding`, and `l1_audit_log` carrying exactly the columns the
//! sweep references (`tenant_id`, the category-bearing column, the age column, and `subject_id` where
//! applicable). The stand-ins mirror the real column names + types so the sweep SQL runs verbatim; the test
//! exercises the real `retention::run_retention_sweep`, not a re-implementation. In CI (the full dev compose)
//! the real tables already exist; `CREATE TABLE IF NOT EXISTS` is a no-op then and the sweep hits the real
//! schema.
//!
//! What they prove (FR §4 #10, #11; §1 #18):
//!   * the sweep deletes an L2 row past `retain_days` but keeps a fresher one (clause 6);
//!   * the sweep NEVER deletes an `l1_audit_log` row - the count is unchanged (clause 6, 11);
//!   * the sweep is idempotent (a second run erases zero more);
//!   * a tenant with no policy sweeps nothing;
//!   * the governance status reflects an unpublished vs published notice correctly (clause 18).

use std::sync::Arc;

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use cyberos_eval::{auth::Authenticator, retention, router, AppState};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &[u8] = b"eval-retention-test-secret";

async fn pool() -> PgPool {
    let url = std::env::var("EVAL_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("EVAL_DATABASE_URL or DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    // Real eval migrations (retention_policy + data_category live here).
    sqlx::query(include_str!("../migrations/0001_governance.sql"))
        .execute(&pool)
        .await
        .expect("apply 0001_governance.sql");
    sqlx::query(include_str!("../migrations/0002_subject_request.sql"))
        .execute(&pool)
        .await
        .expect("apply 0002_subject_request.sql");
    // Minimal stand-ins for the derived tables the sweep erases + the L1 chain it must NEVER touch. These
    // mirror the real column names/types (services/memory/migrations) for exactly the columns the sweep
    // references; in the full dev compose the real tables already exist and these are no-ops.
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS l2_memory (
            tenant_id   UUID NOT NULL,
            seq         BIGINT NOT NULL DEFAULT 0,
            path        TEXT NOT NULL DEFAULT '',
            body        TEXT NOT NULL DEFAULT '',
            frontmatter JSONB NOT NULL DEFAULT '{}'::jsonb,
            ingested_at TIMESTAMPTZ NOT NULL DEFAULT now()
         )",
    )
    .execute(&pool)
    .await
    .expect("create l2_memory stand-in");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS brain_event_embedding (
            tenant_id  UUID NOT NULL,
            source_seq BIGINT NOT NULL DEFAULT 0,
            subject_id UUID NOT NULL,
            kind       TEXT NOT NULL,
            ts_ns      BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
         )",
    )
    .execute(&pool)
    .await
    .expect("create brain_event_embedding stand-in");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS brain_summary (
            id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id  UUID NOT NULL,
            scope_kind TEXT NOT NULL,
            scope_id   TEXT NOT NULL DEFAULT '',
            subject_id UUID,
            digest     TEXT NOT NULL DEFAULT '',
            created_at TIMESTAMPTZ NOT NULL DEFAULT now()
         )",
    )
    .execute(&pool)
    .await
    .expect("create brain_summary stand-in");
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS l1_audit_log (
            seq              BIGSERIAL PRIMARY KEY,
            tenant_id        UUID NOT NULL,
            subject_id       UUID,
            op               TEXT NOT NULL DEFAULT 'put',
            path             TEXT NOT NULL DEFAULT '',
            body             TEXT,
            prev_hash_hex    TEXT,
            chain_anchor_hex TEXT NOT NULL DEFAULT '',
            ts_ns            BIGINT NOT NULL DEFAULT 0,
            ingested_at      TIMESTAMPTZ NOT NULL DEFAULT now()
         )",
    )
    .execute(&pool)
    .await
    .expect("create l1_audit_log stand-in");
    pool
}

async fn seed_tx(pool: &PgPool, tenant: Uuid) -> sqlx::Transaction<'_, sqlx::Postgres> {
    let mut tx = pool.begin().await.unwrap();
    // Set BOTH GUCs so the seed writes pass RLS on the eval-keyed tables (app.current_tenant_id) AND the
    // brain-keyed tables (app.tenant_id) - exactly what the sweep sets.
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("SELECT set_config('app.tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await
        .unwrap();
    tx
}

/// Register a category + a retention policy of `retain_days` for `tenant`, returning nothing. Uses the real
/// tables so the sweep's policy JOIN finds them.
async fn set_policy(pool: &PgPool, tenant: Uuid, category: &str, retain_days: i32, by: Uuid) {
    let mut tx = seed_tx(pool, tenant).await;
    let (cat_id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO data_category (tenant_id, name, purpose, lawful_basis, created_by)
         VALUES ($1, $2, 'retention test', 'legitimate_interest', $3)
         RETURNING id",
    )
    .bind(tenant)
    .bind(category)
    .bind(by)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO retention_policy (tenant_id, data_category_id, retain_days, basis, updated_by)
         VALUES ($1, $2, $3, 'legitimate_interest', $4)",
    )
    .bind(tenant)
    .bind(cat_id)
    .bind(retain_days)
    .bind(by)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// Seed an l2_memory row tagged with `category` and aged `days_ago` days.
async fn seed_l2(pool: &PgPool, tenant: Uuid, category: &str, days_ago: i64) {
    let mut tx = seed_tx(pool, tenant).await;
    sqlx::query(
        "INSERT INTO l2_memory (tenant_id, seq, path, body, frontmatter, ingested_at)
         VALUES ($1, $2, $3, 'body', jsonb_build_object('eval_category', $4::text),
                 now() - make_interval(days => $5))",
    )
    .bind(tenant)
    .bind(rand_seq())
    .bind(format!("eval/{category}"))
    .bind(category)
    .bind(days_ago as i32)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// Seed a brain_event_embedding row for `subject` tagged `kind = category` aged `days_ago` days.
async fn seed_brain_event(
    pool: &PgPool,
    tenant: Uuid,
    subject: Uuid,
    category: &str,
    days_ago: i64,
) {
    let mut tx = seed_tx(pool, tenant).await;
    sqlx::query(
        "INSERT INTO brain_event_embedding (tenant_id, source_seq, subject_id, kind, created_at)
         VALUES ($1, $2, $3, $4, now() - make_interval(days => $5))",
    )
    .bind(tenant)
    .bind(rand_seq())
    .bind(subject)
    .bind(category)
    .bind(days_ago as i32)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// Seed one l1_audit_log row so we can prove the sweep never deletes from the chain.
async fn seed_l1(pool: &PgPool, tenant: Uuid) {
    let mut tx = seed_tx(pool, tenant).await;
    sqlx::query(
        "INSERT INTO l1_audit_log (tenant_id, subject_id, op, path, chain_anchor_hex, ts_ns)
         VALUES ($1, $1, 'put', 'eval/seed', 'deadbeef', 0)",
    )
    .bind(tenant)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn count(pool: &PgPool, tenant: Uuid, table: &str) -> i64 {
    let mut tx = seed_tx(pool, tenant).await;
    let sql = format!("SELECT COUNT(*) FROM {table} WHERE tenant_id = $1");
    let n: i64 = sqlx::query_scalar(&sql)
        .bind(tenant)
        .fetch_one(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    n
}

fn rand_seq() -> i64 {
    // A throwaway monotone-ish seq per row; uniqueness is not required by the stand-in (no PK on seq).
    (Uuid::new_v4().as_u128() & 0x7fff_ffff) as i64
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn sweep_erases_past_retention_l2_but_never_l1() {
    let p = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let subject = Uuid::new_v4();

    set_policy(&p, tenant, "chat_message", 30, founder).await;
    seed_l2(&p, tenant, "chat_message", 40).await; // past retention (40 > 30)
    seed_l2(&p, tenant, "chat_message", 5).await; // fresh (5 < 30)
    seed_brain_event(&p, tenant, subject, "chat_message", 40).await; // past retention, subject-attributed
    seed_l1(&p, tenant).await; // the chain row that must survive

    let l1_before = count(&p, tenant, "l1_audit_log").await;
    assert_eq!(count(&p, tenant, "l2_memory").await, 2);
    assert_eq!(count(&p, tenant, "brain_event_embedding").await, 1);

    // Run the REAL sweep. Single-DB topology: policy + derived + audit all on the same pool.
    let report = retention::run_retention_sweep(&p, &p, Some(&p))
        .await
        .unwrap();

    // The 40-day l2 row + the 40-day brain row are erased; the 5-day l2 row is kept.
    assert_eq!(
        count(&p, tenant, "l2_memory").await,
        1,
        "the fresh l2 row must be kept; only the past-retention one erased"
    );
    assert_eq!(
        count(&p, tenant, "brain_event_embedding").await,
        0,
        "the past-retention brain row must be erased"
    );
    // The report counts both erased rows, the one category, and the one subject (from the brain row).
    assert_eq!(report.rows_erased, 2);
    assert_eq!(report.categories_swept, 1);
    assert_eq!(report.subjects_erased, 1);

    // THE load-bearing invariant: l1_audit_log only GREW (by the eval.retention_swept + eval.subject_erased
    // rows the sweep appended) - it was NEVER deleted from.
    let l1_after = count(&p, tenant, "l1_audit_log").await;
    assert!(
        l1_after >= l1_before,
        "l1_audit_log must never shrink: before={l1_before} after={l1_after}"
    );

    // Idempotent: a second sweep with no new past-retention rows erases nothing more.
    let again = retention::run_retention_sweep(&p, &p, Some(&p))
        .await
        .unwrap();
    assert_eq!(again.rows_erased, 0, "a second sweep must be a no-op");
    assert_eq!(count(&p, tenant, "l2_memory").await, 1);
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn sweep_with_no_policy_erases_nothing() {
    let p = pool().await;
    let tenant = Uuid::new_v4();
    // No policy for this tenant. Seed an ancient row; it must survive because nothing is retained-by-policy.
    seed_l2(&p, tenant, "module_usage", 999).await;
    let report = retention::run_retention_sweep(&p, &p, Some(&p))
        .await
        .unwrap();
    // The report is global, but THIS tenant's row must be untouched.
    assert_eq!(
        count(&p, tenant, "l2_memory").await,
        1,
        "a tenant with no policy must keep its data (no policy ⇒ no deletion)"
    );
    // Sanity: this tenant contributed nothing (it had no policy). (Other tenants in a shared DB may have, so
    // we assert on the tenant-scoped row count, not the global report.)
    let _ = report;
}

// --- clause 18 governance status -------------------------------------------------------------------------

fn app(pool: PgPool) -> axum::Router {
    let state = AppState {
        pool,
        audit_pool: None,
        authenticator: Arc::new(Authenticator::from_hs256_secret(SECRET)),
        version: "test",
    };
    router(state)
}

fn token(subject: Uuid, tenant: Uuid, roles: &[&str]) -> String {
    #[derive(serde::Serialize)]
    struct C {
        sub: String,
        tenant_id: String,
        roles: Vec<String>,
        exp: i64,
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let c = C {
        sub: subject.to_string(),
        tenant_id: tenant.to_string(),
        roles: roles.iter().map(|s| s.to_string()).collect(),
        exp: now + 3600,
    };
    encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET)).unwrap()
}

fn get(uri: &str, token: &str) -> Request<axum::body::Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap()
}

fn post(uri: &str, token: &str, body: Value) -> Request<axum::body::Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

async fn body_json(res: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap()
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn governance_status_reflects_unpublished_then_published() {
    let p = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let subject = Uuid::new_v4();
    let ftoken = token(founder, tenant, &["founder"]);

    // Before any notice: status shows no current version, zero acknowledged.
    let res = app(p.clone())
        .oneshot(get("/v1/eval/governance/status", &ftoken))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let s = body_json(res).await;
    assert!(
        s["current_notice_version"].is_null(),
        "no notice published ⇒ current_notice_version is null"
    );
    assert_eq!(s["acknowledged_current"], 0);

    // Publish a notice, register a category + retention, ack the subject.
    app(p.clone())
        .oneshot(post(
            "/v1/eval/notice",
            &ftoken,
            json!({"lang_en": "v1", "lang_vi": "v1", "lawful_basis": "li"}),
        ))
        .await
        .unwrap();
    app(p.clone())
        .oneshot(post(
            "/v1/eval/categories",
            &ftoken,
            json!({"name": "chat_message", "purpose": "perf", "lawful_basis": "legitimate_interest"}),
        ))
        .await
        .unwrap();
    app(p.clone())
        .oneshot(post(
            "/v1/eval/retention",
            &ftoken,
            json!({"category": "chat_message", "retain_days": 90}),
        ))
        .await
        .unwrap();
    app(p.clone())
        .oneshot(post(
            "/v1/eval/ack",
            &ftoken,
            json!({"subject_id": subject}),
        ))
        .await
        .unwrap();

    // After: current version 1, one acknowledged subject, the category present with its retention.
    let res = app(p.clone())
        .oneshot(get("/v1/eval/governance/status", &ftoken))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let s = body_json(res).await;
    assert_eq!(s["current_notice_version"], 1);
    assert_eq!(s["acknowledged_current"], 1, "the acked subject must count");
    assert_eq!(s["stale_ack_subjects"], 0);
    let cats = s["categories"].as_array().unwrap();
    let chat = cats
        .iter()
        .find(|c| c["name"] == "chat_message")
        .expect("the registered category must appear");
    assert_eq!(chat["lawful_basis"], "legitimate_interest");
    assert_eq!(
        chat["retain_days"], 90,
        "the retention policy must show in status"
    );
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn governance_status_forbidden_for_non_founder_non_manager() {
    let p = pool().await;
    let tenant = Uuid::new_v4();
    let member = Uuid::new_v4();
    let mtoken = token(member, tenant, &["tenant-member"]);
    let res = app(p.clone())
        .oneshot(get("/v1/eval/governance/status", &mtoken))
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "a plain member (not founder, no manager_of grant) must be forbidden from governance status"
    );
}
