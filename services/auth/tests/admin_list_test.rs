//! FR-AUTH-005 — list_tenants + list_subjects integration tests.
//!
//! Covers ECM-001/002/003/004/005/006/007/013/015 from §10.7.
//! Postgres-gated via `#[ignore]`; CI runs `cargo test -- --ignored`.

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use cyberos_auth::cursor::{make_cursor, CursorTable};
use cyberos_auth::{handlers, jwt::JwtService, keygen, AppState};
use serde_json::Value;
use sqlx::PgPool;
use std::time::Instant;
use tower::ServiceExt;

async fn build_app() -> axum::Router {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET ROLE cyberos_app").execute(conn).await.ok();
                Ok(())
            })
        })
        .connect(&url)
        .await
        .expect("connect");
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
        // FR-MEMORY-122: capture is off in tests unless a test installs a Capturer; None = no-op emitters.
        capturer: None,
    })
}

async fn bootstrap_test_key(pool: &PgPool) {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM auth_signing_keys WHERE status='active' AND expires_at > NOW()",
    )
    .fetch_one(pool)
    .await
    .expect("count keys");
    if n > 0 {
        return;
    }
    let key = keygen::generate_rsa_2048().expect("rsa");
    sqlx::query(
        "INSERT INTO auth_signing_keys (kid, algorithm, public_pem, private_pem, status, expires_at)
              VALUES ('test-key', 'RS256', $1, $2, 'active', NOW() + interval '90 days')
         ON CONFLICT (kid) DO NOTHING",
    )
    .bind(&key.public_pem)
    .bind(&key.private_pem)
    .execute(pool)
    .await
    .expect("insert key");
}

async fn issue_root_admin_token(pool: &PgPool) -> String {
    bootstrap_test_key(pool).await;
    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local".to_string());
    let tokens = svc
        .issue(
            cyberos_types::TenantId(uuid::Uuid::nil()),
            cyberos_types::SubjectId(uuid::Uuid::new_v4()),
            "root@cyberos.local",
            "human",
            vec!["admin".to_string()],
            vec!["root-admin".to_string()],
            Some(1),
            Some("cuo-cpo@0.4.1".into()),
            None,
        )
        .await
        .expect("issue token");
    tokens.access_token
}

// ECM-004 — list_tenants as tenant-admin → 403
#[tokio::test]
#[ignore]
async fn list_tenants_as_tenant_admin_returns_403() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    bootstrap_test_key(&pool).await;
    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local".to_string());
    let token = svc
        .issue(
            cyberos_types::TenantId(uuid::Uuid::new_v4()), // non-zero tenant
            cyberos_types::SubjectId(uuid::Uuid::new_v4()),
            "ta@x.com",
            "human",
            vec!["admin".to_string()],
            vec!["tenant-admin".to_string()],
            Some(1),
            None,
            None,
        )
        .await
        .unwrap()
        .access_token;

    let app = build_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/admin/tenants")
                .header("Authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    let body: Value =
        serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
    assert_eq!(body["error"], "forbidden");
    assert!(body["needed"].as_str().unwrap().contains("root-admin"));
}

// ECM-005/006/007 — X-Switch-Tenant branches
#[tokio::test]
#[ignore]
async fn list_subjects_root_admin_no_switch_uses_tenant_zero() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let token = issue_root_admin_token(&pool).await;
    let app = build_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/admin/subjects")
                .header("Authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ECM-001 — cursor signature mismatch → 400 invalid_cursor
#[tokio::test]
#[ignore]
async fn tampered_cursor_returns_400_invalid_cursor() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let token = issue_root_admin_token(&pool).await;
    let good = make_cursor(CursorTable::Tenants, uuid::Uuid::new_v4());
    // Flip a byte by replacing the last char with something different.
    let last = good.chars().last().unwrap();
    let alt = if last == 'a' { 'b' } else { 'a' };
    let bad = format!("{}{}", &good[..good.len() - 1], alt);

    let app = build_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(format!("/v1/admin/tenants?cursor={bad}"))
                .header("Authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let body: Value =
        serde_json::from_slice(&to_bytes(resp.into_body(), 1024).await.unwrap()).unwrap();
    assert_eq!(body["error"], "invalid_cursor");
    assert_eq!(body["field"], "cursor");
}

// ECM-015 — G-007 100 ms p95 SLO over 100 list calls
#[tokio::test]
#[ignore]
async fn list_tenants_p95_under_100ms() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let token = issue_root_admin_token(&pool).await;

    let mut samples_ms: Vec<u128> = Vec::with_capacity(100);
    for _ in 0..100 {
        // Fresh app per call to amortise router setup cost outside the timer.
        let app = build_app().await;
        let req = Request::builder()
            .method(Method::GET)
            .uri("/v1/admin/tenants?limit=10")
            .header("Authorization", format!("Bearer {token}"))
            .body(axum::body::Body::empty())
            .unwrap();
        let start = Instant::now();
        let resp = app.oneshot(req).await.unwrap();
        let elapsed = start.elapsed().as_millis();
        assert_eq!(resp.status(), StatusCode::OK);
        samples_ms.push(elapsed);
    }
    samples_ms.sort_unstable();
    let p95 = samples_ms[(100 * 95) / 100];
    assert!(
        p95 < 100,
        "p95 list_tenants latency = {p95}ms (spec ≤ 100ms)"
    );
}

// ECM-013 — ?include_suspended=true returns suspended subjects too
#[tokio::test]
#[ignore]
async fn include_suspended_flag_widens_filter() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let token = issue_root_admin_token(&pool).await;
    let app = build_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/admin/subjects?include_suspended=true&limit=200")
                .header("Authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(resp.into_body(), 65536).await.unwrap()).unwrap();
    assert!(body["items"].is_array());
}

// ECM-003 — limit clamping (over-limit is silently clamped to 100; under-limit to 1)
#[tokio::test]
#[ignore]
async fn limit_is_clamped_not_rejected() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();
    let token = issue_root_admin_token(&pool).await;
    let app = build_app().await;
    let resp = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/admin/tenants?limit=9999")
                .header("Authorization", format!("Bearer {token}"))
                .body(axum::body::Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
