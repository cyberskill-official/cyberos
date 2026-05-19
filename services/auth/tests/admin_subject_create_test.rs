//! FR-AUTH-002 — Subject create integration tests (slice-3, G-014 + G-007).
//!
//! Mirrors the pattern from `admin_tenant_create_test.rs`. Each test corresponds
//! to one or more edge-case-matrix rows in
//! `docs/feature-requests/auth/FR-AUTH-002-subject-create.audit.md`.
//!
//! Requires Postgres; gated by `#[ignore]`. CI integration job runs
//! `cargo test -- --ignored` against the docker-compose stack.

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use cyberos_auth::{handlers, jwt::JwtService, keygen, AppState};
use cyberos_types::{SubjectId, TenantId};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::time::Instant;
use tower::ServiceExt;

// ===========================================================================
// Fixtures
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
    let kid = format!("test-sc-{}", uuid::Uuid::new_v4().simple());
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

/// Mint a tenant-admin JWT scoped to a fresh test tenant. Returns
/// `(access_token, tenant_uuid)` so tests can assert against the same
/// tenant they're authenticated for.
async fn tenant_admin_token(pool: &PgPool) -> (String, uuid::Uuid) {
    let tenant_uuid = uuid::Uuid::new_v4();
    // Seed the tenant directly (the FR-AUTH-001 endpoint requires
    // root-admin-in-tenant-0; for this test we go around it).
    let _ = sqlx::query(
        "INSERT INTO tenants (id, slug, display_name, country, plan_tier, status, residency)
              VALUES ($1, $2, 'Test Tenant', 'VN', 'free', 'active', 'vn-1')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(tenant_uuid)
    .bind(format!("test-{}", tenant_uuid.simple()))
    .execute(pool)
    .await;

    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local".to_string());
    let tokens = svc
        .issue(
            TenantId(tenant_uuid),
            SubjectId(uuid::Uuid::new_v4()),
            "",     // FR-AUTH-004 §1 #2 — test token, no email needed
            "human",
            vec!["admin".into()],
            vec!["tenant-admin".into()],
            Some(1),
            None,
            None,
        )
        .await
        .expect("issue");
    (tokens.access_token, tenant_uuid)
}

/// Build a POST request with auth + idempotency-key + X-Forwarded-Proto: https.
fn post_subject(token: &str, idem_key: &str, body: Value) -> Request<axum::body::Body> {
    Request::builder()
        .method(Method::POST)
        .uri("/v1/admin/subjects")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", idem_key)
        .header("x-forwarded-proto", "https")
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

fn happy_subject_body(handle: &str, email: &str) -> Value {
    json!({
        "handle": handle,
        "display_name": "Test Subject",
        "email": email,
        "kind": "human",
        "password": "Tx9!mZ@qVnL3pR2k",  // strong: 16 chars · 4 classes · not common
        "roles": ["tenant-member"]
    })
}

// ===========================================================================
// G-014 — happy path (ECM-001 baseline)
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_subject_happy_path_returns_201_with_clean_body() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, _) = tenant_admin_token(&pool).await;
    let app = build_app().await;

    let handle = format!("user-{}", uuid::Uuid::new_v4().simple());
    let res = app
        .oneshot(post_subject(
            &token,
            &format!("happy-{}", &handle),
            happy_subject_body(&handle, "alice@example.com"),
        ))
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::CREATED);
    let body: Value =
        serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap();
    // §1 #8 — response shape contract: NEVER password hash, NEVER plaintext password
    let body_str = body.to_string();
    assert!(!body_str.to_lowercase().contains("password"), "response leaked password field");
    assert!(!body_str.contains("$2"), "response leaked bcrypt hash");
    assert_eq!(body["handle"], handle);
    assert_eq!(body["email"], "alice@example.com");
    assert_eq!(body["kind"], "human");
    assert_eq!(body["status"], "active");
}

// ===========================================================================
// G-008 — HTTPS-required (§1 #11) — covers ECM-SECURITY row
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_subject_rejects_http_request() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, _) = tenant_admin_token(&pool).await;
    let app = build_app().await;
    // Belt-and-braces: ensure the test env-var isn't set.
    std::env::remove_var("AUTH_TEST_ALLOW_HTTP");

    let req = Request::builder()
        .method(Method::POST)
        .uri("/v1/admin/subjects")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .header("idempotency-key", "test-http-reject")
        // No X-Forwarded-Proto — should be rejected.
        .body(axum::body::Body::from(
            happy_subject_body("http-user", "alice@example.com").to_string(),
        ))
        .unwrap();
    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let body: Value =
        serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap();
    assert_eq!(body["error"], "https_required");
}

// ===========================================================================
// G-002 weak password (ECM-PASSWORD-WEAK)
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_subject_rejects_weak_password_with_multiple_reasons() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, _) = tenant_admin_token(&pool).await;
    let app = build_app().await;

    let mut body = happy_subject_body("weak-user", "alice@example.com");
    body["password"] = json!("short"); // 5 chars, lowercase only
    let res = app
        .oneshot(post_subject(&token, "test-weak-pw", body))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let parsed: Value =
        serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap();
    assert_eq!(parsed["error"], "weak_password");
    let reasons = parsed["reasons"].as_array().expect("reasons array");
    assert!(reasons.iter().any(|r| r == "too_short"));
    // Multiple reasons in one response per §1 #4
    assert!(reasons.len() >= 2);
}

// ===========================================================================
// G-003 unknown role
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_subject_rejects_unknown_role() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, _) = tenant_admin_token(&pool).await;
    let app = build_app().await;

    let mut body = happy_subject_body("typo-user", "alice@example.com");
    body["roles"] = json!(["tenant-superadmin"]); // not in slice-1 allow-list
    let res = app
        .oneshot(post_subject(&token, "test-bad-role", body))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    let parsed: Value =
        serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap();
    assert_eq!(parsed["error"], "unknown_role");
    assert_eq!(parsed["role"], "tenant-superadmin");
}

// ===========================================================================
// G-004 idempotent replay returns same id
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_subject_idempotent_replay_returns_same_id() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, _) = tenant_admin_token(&pool).await;
    let app = build_app().await;

    let handle = format!("idem-{}", uuid::Uuid::new_v4().simple());
    let key = format!("idem-key-{}", uuid::Uuid::new_v4().simple());
    let body = happy_subject_body(&handle, &format!("{handle}@example.com"));

    let r1 = app.clone().oneshot(post_subject(&token, &key, body.clone())).await.unwrap();
    assert_eq!(r1.status(), StatusCode::CREATED);
    let b1: Value =
        serde_json::from_slice(&to_bytes(r1.into_body(), 1 << 20).await.unwrap()).unwrap();
    let id1 = b1["id"].as_str().unwrap().to_string();

    let r2 = app.oneshot(post_subject(&token, &key, body)).await.unwrap();
    let b2: Value =
        serde_json::from_slice(&to_bytes(r2.into_body(), 1 << 20).await.unwrap()).unwrap();
    assert_eq!(b2["id"], id1, "idempotent replay MUST return the same id");
}

// ===========================================================================
// G-005 memory audit row schema after success
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres + memory migrations applied"]
async fn create_subject_emits_memory_audit_row() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, tenant_uuid) = tenant_admin_token(&pool).await;
    let app = build_app().await;

    let handle = format!("audit-{}", uuid::Uuid::new_v4().simple());
    let email = format!("{handle}@example.com");
    let res = app
        .oneshot(post_subject(
            &token,
            &format!("audit-key-{}", &handle),
            happy_subject_body(&handle, &email),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let body: Value =
        serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap();
    let subject_id = body["id"].as_str().unwrap();

    // Find the auth.subject_created row in l1_audit_log
    let (op, path, audit_body): (String, String, String) = sqlx::query_as(
        "SELECT op, path, body FROM l1_audit_log
            WHERE tenant_id = $1 AND path LIKE 'auth/subject/%'
         ORDER BY seq DESC LIMIT 1",
    )
    .bind(tenant_uuid)
    .fetch_one(&pool)
    .await
    .expect("audit row exists");

    assert_eq!(op, "put");
    assert!(path.contains(&format!("auth/subject/{subject_id}/created")));
    assert!(audit_body.contains("\"event_type\":\"auth.subject_created\""));
    // §1 #7 privacy contract: no plaintext password / no full email in audit body
    assert!(
        !audit_body.contains(&email),
        "audit row MUST NOT contain plaintext email"
    );
    assert!(!audit_body.to_lowercase().contains("\"password\""));
    // email_hash16 IS present (privacy-safe identifier)
    let expected_hash = cyberos_auth::memory_bridge::email_hash16(&email);
    assert!(audit_body.contains(&expected_hash));
}

// ===========================================================================
// G-007 — §1 #10 — 200ms p95 SLO test
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres — measures p95 over 100 subject creates (bcrypt cost 12)"]
async fn create_subject_p95_latency_under_200ms() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let (token, _) = tenant_admin_token(&pool).await;
    let app = build_app().await;

    const N: usize = 100;
    let mut latencies_ms = Vec::with_capacity(N);

    for i in 0..N {
        let handle = format!("slo-{}-{}", uuid::Uuid::new_v4().simple(), i);
        let email = format!("{handle}@example.com");
        let req = post_subject(
            &token,
            &format!("slo-{}-{i}", uuid::Uuid::new_v4().simple()),
            happy_subject_body(&handle, &email),
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
    let p95 = latencies_ms[(N as f64 * 0.95) as usize - 1];
    let p50 = latencies_ms[N / 2];
    let max = *latencies_ms.last().unwrap();
    eprintln!("p50: {p50:.1} ms · p95: {p95:.1} ms · max: {max:.1} ms");

    // §1 #10 SLO: p95 < 200ms. bcrypt cost 12 is ~150ms by itself; remaining
    // 50ms budget covers HIBP API + DB + validation + audit. If a CI runner
    // is slower than a typical prod cell, this test may flake — track CI
    // flake rate and tune cost downward only via explicit FR amendment.
    assert!(
        p95 < 200.0,
        "p95 latency MUST be < 200ms per §1 #10; got {p95:.1} ms (p50={p50:.1}, max={max:.1})"
    );
}

// ===========================================================================
// G-012 cross-tenant guard — caller without tenant-admin role
// ===========================================================================

#[tokio::test]
#[ignore = "requires Postgres"]
async fn create_subject_without_tenant_admin_role_returns_403() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = PgPool::connect(&url).await.unwrap();
    bootstrap_test_key(&pool).await;
    let app = build_app().await;

    // Mint a token with tenant-MEMBER (not admin) role
    let tenant_uuid = uuid::Uuid::new_v4();
    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local".to_string());
    let tokens = svc
        .issue(
            TenantId(tenant_uuid),
            SubjectId(uuid::Uuid::new_v4()),
            "",     // FR-AUTH-004 §1 #2 — test token, no email needed
            "human",
            vec!["admin".into()],
            vec!["tenant-member".into()],
            Some(1),
            None,
            None,
        )
        .await
        .expect("issue");

    let res = app
        .oneshot(post_subject(
            &tokens.access_token,
            "test-non-admin",
            happy_subject_body("non-admin", "x@example.com"),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let body: Value =
        serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap();
    assert_eq!(body["error"], "forbidden");
}
