//! FR-EVAL-001 slice 2 - governance HTTP endpoint integration tests.
//!
//! Mirrors the auth HTTP test harness (`services/auth/tests/admin_subject_create_test.rs`): build the
//! real `AppState` + `router`, mint a CyberOS token (HS256 here, the test/local verifier), drive the
//! router with `tower::ServiceExt::oneshot`, and assert on the JSON response and the data effects. Like
//! the auth + slice-1 suites these require a live Postgres with the eval migrations applied, so they are
//! `#[ignore]` by default and gate on `EVAL_DATABASE_URL` (falling back to `DATABASE_URL`). Local:
//!   docker compose up -d        (in services/dev/)
//!   EVAL_DATABASE_URL=postgres://... cargo test -p cyberos-eval -- --ignored
//!
//! The test applies `migrations/0001_governance.sql` + `migrations/0002_subject_request.sql` itself
//! (idempotent CREATE ... IF NOT EXISTS), seeds into a fresh random tenant, and exercises the surface:
//!   * publish a notice, then GET returns it;
//!   * record an ack -> the gate flips for that subject (is_capture_allowed becomes true);
//!   * grant access -> can_read_evaluation passes for the grantee;
//!   * GET /v1/eval/me returns ONLY the caller's own data;
//!   * a non-founder is rejected (403) from POST /v1/eval/notice.

use std::sync::Arc;

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use cyberos_eval::{access, auth::Authenticator, gate, router, AppState};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &[u8] = b"eval-endpoints-test-secret";

async fn pool() -> PgPool {
    let url = std::env::var("EVAL_DATABASE_URL")
        .or_else(|_| std::env::var("DATABASE_URL"))
        .expect("EVAL_DATABASE_URL or DATABASE_URL env var");
    let pool = PgPool::connect(&url).await.expect("connect");
    sqlx::query(include_str!("../migrations/0001_governance.sql"))
        .execute(&pool)
        .await
        .expect("apply 0001_governance.sql");
    sqlx::query(include_str!("../migrations/0002_subject_request.sql"))
        .execute(&pool)
        .await
        .expect("apply 0002_subject_request.sql");
    pool
}

/// Build the router over an `AppState` whose verifier is the HS256 test secret (no audit pool: governance
/// events log, which is the test convention).
fn app(pool: PgPool) -> axum::Router {
    let state = AppState {
        pool,
        audit_pool: None,
        authenticator: Arc::new(Authenticator::from_hs256_secret(SECRET)),
        version: "test",
    };
    router(state)
}

/// Mint an HS256 CyberOS token for `subject` in `tenant` with `roles`.
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

fn post(uri: &str, token: &str, body: Value) -> Request<axum::body::Body> {
    Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(body.to_string()))
        .unwrap()
}

fn get(uri: &str, token: &str) -> Request<axum::body::Body> {
    Request::builder()
        .method(Method::GET)
        .uri(uri)
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn body_json(res: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(res.into_body(), 1 << 20).await.unwrap()).unwrap()
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn publish_notice_then_get_returns_it() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let ftoken = token(founder, tenant, &["founder"]);

    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/notice",
            &ftoken,
            json!({
                "lang_en": "We monitor work interactions.",
                "lang_vi": "Chung toi giam sat tuong tac cong viec.",
                "lawful_basis": "Decree 13/2023/ND-CP + Labor Code 45/2019/QH14"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let published = body_json(res).await;
    assert_eq!(published["version"], 1);
    assert_eq!(published["is_current"], true);

    let res = app(pool.clone()).oneshot(get("/v1/eval/notice", &ftoken)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let current = body_json(res).await;
    assert_eq!(current["version"], 1);
    assert_eq!(current["lang_en"], "We monitor work interactions.");
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn ack_record_flips_capture_allowed_for_subject() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let subject = Uuid::new_v4();
    let ftoken = token(founder, tenant, &["founder"]);

    // Publish a notice so there is a current version to acknowledge.
    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/notice",
            &ftoken,
            json!({"lang_en": "v1", "lang_vi": "v1", "lawful_basis": "li"}),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // Before the ack: the subject is gated (capture NOT allowed).
    assert!(
        !gate::is_capture_allowed(&pool, tenant, subject).await.unwrap(),
        "subject must be gated before acknowledgment"
    );

    // Record the ack (founder acting as HR; ack_source defaults to 'signed_contract').
    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/ack",
            &ftoken,
            json!({ "subject_id": subject }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let ack = body_json(res).await;
    assert_eq!(ack["ack_source"], "signed_contract");
    assert_eq!(ack["capture_allowed"], true);

    // After the ack: the gate is lifted (capture allowed), proven via slice 1's predicate.
    assert!(
        gate::is_capture_allowed(&pool, tenant, subject).await.unwrap(),
        "recording the ack must flip is_capture_allowed to true"
    );
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn access_grant_then_can_read_evaluation_passes() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let manager = Uuid::new_v4();
    let report = Uuid::new_v4();
    let ftoken = token(founder, tenant, &["founder"]);

    // Before any grant: the manager is denied reading the report (slice 1 deny-by-default).
    assert!(
        !access::can_read_evaluation(&pool, tenant, manager, report).await.unwrap(),
        "manager must be denied before a grant exists"
    );

    // Founder grants manager_of(manager -> report).
    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/access",
            &ftoken,
            json!({
                "viewer_subject_id": manager,
                "target_subject_id": report,
                "scope": "manager_of"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let grant = body_json(res).await;
    let grant_id = grant["id"].as_str().unwrap().to_string();

    // After the grant: slice 1's access check passes for that pair.
    assert!(
        access::can_read_evaluation(&pool, tenant, manager, report).await.unwrap(),
        "an active manager_of grant must let the manager read the report"
    );
    assert_eq!(
        access::may_read(&pool, tenant, manager, report).await.unwrap(),
        Some(access::GrantKind::ManagerOf)
    );

    // Revoke removes access again (clause 8).
    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/access/revoke",
            &ftoken,
            json!({ "grant_id": grant_id }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
    assert!(
        !access::can_read_evaluation(&pool, tenant, manager, report).await.unwrap(),
        "after revoke the manager must be denied again"
    );
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn get_me_returns_only_the_callers_own_data() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let founder = Uuid::new_v4();
    let alice = Uuid::new_v4();
    let bob = Uuid::new_v4();
    let ftoken = token(founder, tenant, &["founder"]);
    let atoken = token(alice, tenant, &["tenant-member"]);

    // Publish a notice, then HR-record an ack for BOTH alice and bob.
    app(pool.clone())
        .oneshot(post(
            "/v1/eval/notice",
            &ftoken,
            json!({"lang_en": "v1", "lang_vi": "v1", "lawful_basis": "li"}),
        ))
        .await
        .unwrap();
    for s in [alice, bob] {
        let res = app(pool.clone())
            .oneshot(post("/v1/eval/ack", &ftoken, json!({ "subject_id": s })))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::CREATED);
    }
    // Grant a manager_of over bob (a grant ABOUT bob, not about alice).
    app(pool.clone())
        .oneshot(post(
            "/v1/eval/access",
            &ftoken,
            json!({"viewer_subject_id": founder, "target_subject_id": bob, "scope": "manager_of"}),
        ))
        .await
        .unwrap();

    // Alice reads /me: she sees her OWN ack and no grant about her, and never bob's data.
    let res = app(pool.clone()).oneshot(get("/v1/eval/me", &atoken)).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let me = body_json(res).await;
    assert_eq!(me["subject_id"], alice.to_string());
    assert_eq!(me["capture_allowed"], true);
    let acks = me["acknowledgments"].as_array().unwrap();
    assert_eq!(acks.len(), 1, "alice must see exactly her own ack");
    // The grant about bob must NOT appear in alice's record.
    let grants = me["access_grants_about_me"].as_array().unwrap();
    assert!(
        grants.is_empty(),
        "alice's record must not include grants about another subject (bob)"
    );
    // Belt-and-braces: bob's id must not appear anywhere in alice's record.
    assert!(
        !me.to_string().contains(&bob.to_string()),
        "alice's /me must never leak another subject's id"
    );
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn non_founder_is_rejected_from_publish_notice() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let member = Uuid::new_v4();
    let mtoken = token(member, tenant, &["tenant-member"]);

    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/notice",
            &mtoken,
            json!({"lang_en": "x", "lang_vi": "x", "lawful_basis": "li"}),
        ))
        .await
        .unwrap();
    assert_eq!(
        res.status(),
        StatusCode::FORBIDDEN,
        "a non-founder must be forbidden from publishing a notice"
    );

    // A missing token is a 401 (the auth boundary), distinct from the 403 above.
    let res = app(pool.clone())
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/eval/notice")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(
                    json!({"lang_en": "x", "lang_vi": "x", "lawful_basis": "li"}).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
}
