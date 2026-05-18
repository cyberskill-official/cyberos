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
use tower::ServiceExt;

// ---------------------------------------------------------------------------
// G-001 — §1 #14 — slug == "root" is reserved (ECM-008)
// ---------------------------------------------------------------------------
//
// Defence-in-depth: the handler MUST reject `slug=="root"` BEFORE any
// DB work. The DB UNIQUE constraint on `tenants.slug` also catches this,
// but the handler-level reject saves a round trip AND produces a
// structured `{error, field, reason}` body identifying which input was
// invalid (matches the §1 #11 error-body shape that G-002 closes).

#[tokio::test]
#[ignore = "requires Postgres — boot services/dev/docker-compose.yml first"]
async fn create_tenant_rejects_reserved_root_slug() {
    let app = build_app().await;
    let body = json!({
        "slug": "root",
        "display_name": "Should Be Rejected",
        "country": "VN",
        "plan_tier": "free",
        "residency": "vn-1"
    });

    let res = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/admin/tenants")
                .header("content-type", "application/json")
                .header("idempotency-key", "test-root-reject-001")
                .body(axum::body::Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(
        res.status(),
        StatusCode::BAD_REQUEST,
        "slug=='root' MUST return 400 (§1 #14), got {}",
        res.status()
    );

    let body_bytes = to_bytes(res.into_body(), 1 << 20).await.unwrap();
    let parsed: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Structured body per §1 #11 (G-002 closes the general case; G-001 establishes the shape for the reserved-slug path).
    assert_eq!(parsed["error"], "invalid_input", "error field should be 'invalid_input'");
    assert_eq!(parsed["field"], "slug",          "field should identify 'slug' as the failing input");
    let reason = parsed["reason"].as_str().expect("reason must be a string");
    assert!(
        reason.contains("root") && reason.contains("reserved"),
        "reason must explain the reservation; got: {reason}"
    );
}

// ---------------------------------------------------------------------------
// (Future tests land here as G-002..G-007 close. Each gap fill brings 1-4 new
// test functions, each tagged with the ECM-NNN row it covers.)
// ---------------------------------------------------------------------------

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

/// Helper for later tests that need an authenticated request. Mints a JWT for
/// the given subject + tenant; tests that need root-admin pass `Uuid::nil()`
/// + roles `vec!["root-admin"]`.
#[allow(dead_code)]
async fn issue_jwt(
    pool: &PgPool,
    tenant: TenantId,
    subject: SubjectId,
    roles: Vec<String>,
) -> String {
    let svc = JwtService::new(pool.clone(), "https://auth.cyberos.local");
    let tokens = svc
        .issue(tenant, subject, "human", roles, vec!["tenant-admin".into()], Some(1), None, None)
        .await
        .expect("issue");
    tokens.access_token
}
