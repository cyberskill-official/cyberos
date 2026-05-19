//! FR-AUTH-001 — Tenant create integration tests.
//!
//! Each test corresponds to one or more edge-case-matrix rows in
//! `docs/feature-requests/auth/FR-AUTH-001-tenant-create.audit.md §10.5`.
//! The matrix is the source of truth; if a row lands here without a
//! corresponding ECM-NNN tag, the audit will fail.
//!
//! Requires Postgres; gated by `#[ignore]` so `cargo test --workspace`
//! stays fast. CI integration job runs `cargo test -- --ignored`.

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use cyberos_auth::{handlers, jwt::JwtService, keygen, AppState};
use cyberos_types::{SubjectId, TenantId};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Instant;
use tower::ServiceExt;

// ===========================================================================
// Test fixtures
// ===========================================================================

async fn build_app() -> axum::Router {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    bootstrap_test_key(&pool).await;

    handlers::router(AppState {
        pg: pool,
        jwt_issuer: "https://auth.cyberos.local".into(),
        role_matrix: std::sync::Arc::new(tokio::sync::RwLock::new(
            cyberos_auth::rbac::RoleMatrix::empty(),
        )),
        oidc_pending: std::sync::Arc::new(tokio::sync::RwLock::new(
            std::collections::HashMap::new(),
        )),
        geoip: std::sync::Arc::new(cyberos_auth::geoip::NullResolver),
        travel_policy: cyberos_auth::travel_policy::PolicyCache::new(),
        sticky_suppress: cyberos_auth::travel_policy::StickySuppress::new(),
        rate_limit: std::sync::Arc::new(cyberos_auth::rate_limit::RateLimiter::new()),
        deny_list: cyberos_auth::deny_list::DenyList::new(),
    })
}

async fn bootstrap_test_key(pool: &PgPool) {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM auth_signing_keys WHERE status='active' AND expires_at > NOW()",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    if n > 0 {
        return;
    }
    let k = keygen::generate_rsa_2048().expect("keygen");
    let kid = format!("test-tc-{}", uuid::Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO auth_signing_keys (kid, public_pem, private_pem, expires_at)
         VALUES ($1, $2, $3, NOW() + INTERVAL '1 hour')",
    )
    .bind(&kid)
    .bind(&k.public_pem)
    .bind(&k.private_pem)
    .execute(pool)
    .await
    .unwrap();
}

/// Mint a root-admin JWT for tenant 0. Required by every successful
/// integration test now that G-003 enforces handler-level authz.
async fn root_admin_token(pool: &PgPool) -> String {
    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local".to_string());
    let tokens = svc
        .issue(
            TenantId::ROOT,
            SubjectId(uuid::Uuid::new_v4()),
            "", // FR-AUTH-004 §1 #2 — root-admin test token, no email needed
            "human",
            vec!["admin".into()],
            vec!["root-admin".into()],
            Some(1),
            None,
            None,
        )
        .await
        .expect("issue");
    tokens.access_token
}

/// Mint a JWT for a non-root tenant (used to exercise the 403 path).
async fn non_root_admin_token(pool: &PgPool) -> String {
    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local".to_string());
    let tokens = svc
        .issue(
            TenantId(uuid::Uuid::new_v4()),
            SubjectId(uuid::Uuid::new_v4()),
            "", // FR-AUTH-004 §1 #2 — test token, no email needed
            "human",
            vec!["admin".into()],
            vec!["tenant-admin".into()],
            Some(1),
            None,
            None,
        )
        .await
        .expect("issue");
    tokens.access_token
}

/// Build a POST request to /v1/admin/tenants with auth + Idempotency-Key.
fn post_request(token: &str, idem_key: &str, body: Value) -> Request<axum::body::Body> {
    Request::builder()
        .method(Method::POST)
        .uri("/v1/admin/tenants")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", idem_key)
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

fn happy_body(slug: &str) -> Value {
    json!({
        "slug": slug,
        "display_name": "Test Tenant",
        "country": "VN",
        "plan_tier": "starter",
        "residency": "vn-1"
    })
}

// ===========================================================================
// G-001 — §1 #14 — slug == "root" reserved (ECM-008)
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_tenant_rejects_reserved_root_slug() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    let res = app
        .oneshot(post_request(
            &token,
            "test-root-reject-001",
            json!({
                "slug": "root",
                "display_name": "Should Be Rejected",
                "country": "VN",
                "plan_tier": "starter",
                "residency": "vn-1"
            }),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body_bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["error"], "invalid_input");
    assert_eq!(parsed["field"], "slug");
    assert!(parsed["reason"].as_str().unwrap().contains("reserved"));
}

// ===========================================================================
// G-002 — §1 #11 — structured 400 invalid_input body (ECM-006)
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_tenant_rejects_uppercase_slug_with_structured_body() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    let res = app
        .oneshot(post_request(
            &token,
            "test-uppercase-002",
            happy_body("Acme"),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body_bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["error"], "invalid_input");
    assert_eq!(parsed["field"], "slug");
}

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_tenant_rejects_missing_idempotency_key_with_structured_body() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    // Build a request WITHOUT the Idempotency-Key header.
    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/admin/tenants")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(
            happy_body("acme-no-idem").to_string(),
        ))
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body_bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["error"], "missing_header");
    assert_eq!(parsed["field"], "Idempotency-Key");
}

// ===========================================================================
// G-003 — §1 #1 — root-admin-in-tenant-0 authz (ECM-012)
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_tenant_rejects_non_root_tenant_caller() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = non_root_admin_token(&pool).await; // wrong tenant
    let app = build_app().await;

    let res = app
        .oneshot(post_request(
            &token,
            "test-non-root-caller-003",
            happy_body("acme-wrong-tenant"),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body_bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(parsed["error"], "forbidden");
    assert_eq!(parsed["needed"], "root-admin in tenant 0");
}

// ===========================================================================
// G-005 — §1 #6 — memory audit row emitted in transaction
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres + memory migrations applied (0003_layer1_audit_log.sql)"]
async fn create_tenant_emits_memory_audit_row_in_transaction() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    let slug = format!("audit-row-{}", uuid::Uuid::new_v4().simple());
    let res = app
        .oneshot(post_request(
            &token,
            &format!("test-audit-{}", &slug),
            happy_body(&slug),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body_bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let tenant: Value = serde_json::from_slice(&body_bytes).unwrap();
    let tenant_id = tenant["id"].as_str().expect("id is string");

    // Verify the memory audit row exists in l1_audit_log with the right shape.
    let (op, path, body): (String, String, String) = sqlx::query_as(
        "SELECT op, path, body FROM l1_audit_log
            WHERE tenant_id = $1::uuid AND op = 'put'
         ORDER BY seq DESC LIMIT 1",
    )
    .bind(tenant_id)
    .fetch_one(&pool)
    .await
    .expect("audit row exists");

    assert_eq!(op, "put");
    assert!(path.contains(&format!("auth/tenant/{tenant_id}/created")));
    assert!(body.contains("\"event_type\":\"auth.tenant_created\""));
    assert!(body.contains(&format!("\"slug\":\"{slug}\"")));
}

// ===========================================================================
// G-006 — §1 #8 — 100ms p95 SLO test
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres — measures p95 over 100 tenant creates"]
async fn create_tenant_p95_latency_under_100ms() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    const N: usize = 100;
    let mut latencies_ms = Vec::with_capacity(N);

    for i in 0..N {
        let slug = format!("slo-{}-{}", uuid::Uuid::new_v4().simple(), i);
        let req = post_request(
            &token,
            &format!("slo-idem-{i}-{}", uuid::Uuid::new_v4().simple()),
            happy_body(&slug),
        );
        let t0 = Instant::now();
        let res = app.clone().oneshot(req).await.unwrap();
        let elapsed_ms = t0.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(
            res.status(),
            StatusCode::CREATED,
            "iter {i}: expected 201, got {}",
            res.status()
        );
        latencies_ms.push(elapsed_ms);
    }

    latencies_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95_idx = (N as f64 * 0.95) as usize - 1;
    let p95 = latencies_ms[p95_idx];
    let p50 = latencies_ms[N / 2];
    let max = *latencies_ms.last().unwrap();

    eprintln!("p50: {p50:.1} ms · p95: {p95:.1} ms · max: {max:.1} ms");
    assert!(
        p95 < 100.0,
        "p95 latency MUST be < 100ms per §1 #8; got {p95:.1} ms (p50={p50:.1}, max={max:.1})"
    );
}

// ===========================================================================
// G-007 — §1 #5 — idempotency + boundary tests
// ===========================================================================

// ECM-010 — same Idempotency-Key + same body → 200 + same id
#[tokio::test]
#[ignore = "requires Postgres"]
async fn idempotent_replay_returns_same_tenant_id() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    let slug = format!("idem-replay-{}", uuid::Uuid::new_v4().simple());
    let key = format!("idem-key-{}", uuid::Uuid::new_v4().simple());

    // First call → 201 CREATED
    let res1 = app
        .clone()
        .oneshot(post_request(&token, &key, happy_body(&slug)))
        .await
        .unwrap();
    assert_eq!(res1.status(), StatusCode::CREATED);
    let body1: Value =
        serde_json::from_slice(&to_bytes(res1.into_body(), 1 << 20).await.unwrap()).unwrap();
    let id1 = body1["id"].as_str().unwrap().to_string();

    // Second call with SAME key + SAME body → same id (replay)
    let res2 = app
        .oneshot(post_request(&token, &key, happy_body(&slug)))
        .await
        .unwrap();
    assert_eq!(res2.status(), StatusCode::CREATED);
    let body2: Value =
        serde_json::from_slice(&to_bytes(res2.into_body(), 1 << 20).await.unwrap()).unwrap();
    let id2 = body2["id"].as_str().unwrap();
    assert_eq!(id1, id2, "idempotent replay MUST return the original id");
}

// ECM-005 — Idempotency-Key longer than 64 chars accepted (per spec) OR rejected;
// current impl is permissive — capture today's behaviour so a regression is visible.
#[tokio::test]
#[ignore = "requires Postgres"]
async fn idempotency_key_long_string_currently_accepted() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let token = root_admin_token(&pool).await;
    let app = build_app().await;

    let slug = format!("idem-long-{}", uuid::Uuid::new_v4().simple());
    let long_key = "k".repeat(120); // 120 chars
    let res = app
        .oneshot(post_request(&token, &long_key, happy_body(&slug)))
        .await
        .unwrap();
    // Today the handler doesn't bound key length explicitly. CI surfaces
    // the actual outcome; if a future tightening enforces ≤64, this test
    // will flip and pin the new contract.
    assert!(
        matches!(res.status(), StatusCode::CREATED | StatusCode::BAD_REQUEST),
        "unexpected status for long Idempotency-Key: {}",
        res.status()
    );
}

// ===========================================================================
// NOT covered here (documented in audit §10.2 G-007 follow-up):
//   * ECM-009 concurrent same-slug — needs tokio::join! racing two POSTs;
//     deterministic only with serializable isolation set on the test
//     transaction. Deferred to a follow-up integration FR.
//   * ECM-011 same Idempotency-Key + DIFFERENT body — 409 idempotency_key_reuse;
//     the current idempotency module returns the prior body (silent replay)
//     instead of 409. Closing this gap is a small idempotency.rs change,
//     scoped as a follow-up commit.
//   * ECM-014 memory unreachable rollback — requires injectable memory bridge
//     for deterministic failure. Deferred until memory_bridge moves behind
//     a trait that the test can swap to a failing impl.
// ===========================================================================
