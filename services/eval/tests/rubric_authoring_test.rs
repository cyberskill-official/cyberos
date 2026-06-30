//! FR-EVAL-002 - rubric authoring integration tests (§1 #2 #3 #4 #5, §4 #2 #3 #4 #5 #6).
//!
//! Mirrors `endpoints_test.rs`: build the real `AppState` + `router`, mint an HS256 CyberOS token (the
//! test/local verifier), drive the router with `tower::ServiceExt::oneshot`, and assert on the JSON and the
//! data effects. Requires a live Postgres with the eval migrations applied, so it is `#[ignore]` by default
//! and gates on `EVAL_DATABASE_URL` (falling back to `DATABASE_URL`). Local:
//!   docker compose up -d        (in services/dev/)
//!   EVAL_DATABASE_URL=postgres://... cargo test -p cyberos-eval -- --ignored
//!
//! Applies 0001 + 0002 + 0003 itself (idempotent CREATE ... IF NOT EXISTS), seeds into a fresh random
//! tenant, and exercises the authoring surface: create a rubric, open a draft version, and prove the
//! write-time validation gates (uncited / source_doc-outside-the-three / obligation-kind / check-shape /
//! missing-vi), plus the happy path.

use std::sync::Arc;

use axum::body::to_bytes;
use axum::http::{Method, Request, StatusCode};
use cyberos_eval::{auth::Authenticator, router, AppState};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const SECRET: &[u8] = b"eval-rubric-authoring-test-secret";

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
    sqlx::query(include_str!("../migrations/0003_rubric.sql"))
        .execute(&pool)
        .await
        .expect("apply 0003_rubric.sql");
    pool
}

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

/// A well-formed obligation item body (the baseline the negative cases mutate one field of).
fn good_item() -> Value {
    json!({
        "source_doc": "nda_ip",
        "clause_ref": "art.2(a)",
        "source_quote_vi": "Bên Nhận Thông Tin cam kết chỉ sử dụng Thông Tin Bảo Mật...",
        "source_quote_en": "RECEIVING PARTY undertakes to use the Confidential Information...",
        "item_kind": "obligation",
        "obligation_kind": "confidentiality",
        "check_type": "evidence_presence",
        "check_params": {},
        "weight": 10.0,
        "title_vi": "Bảo mật thông tin",
        "title_en": "Confidentiality"
    })
}

/// Create a rubric + open a draft version as the founder; returns (rubric_id, version_id).
async fn new_draft_version(pool: &PgPool, ftoken: &str) -> (String, String) {
    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/rubrics",
            ftoken,
            json!({ "name": "CyberSkill employment rubric" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let rubric = body_json(res).await;
    let rid = rubric["id"].as_str().unwrap().to_string();

    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions"),
            ftoken,
            json!({}),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let version = body_json(res).await;
    assert_eq!(version["state"], "draft");
    assert_eq!(version["version_no"], 1);
    let vid = version["id"].as_str().unwrap().to_string();
    (rid, vid)
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn well_formed_item_is_accepted() {
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let ftoken = token(Uuid::new_v4(), tenant, &["founder"]);
    let (rid, vid) = new_draft_version(&pool, &ftoken).await;

    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            good_item(),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let item = body_json(res).await;
    assert_eq!(item["source_doc"], "nda_ip");
    assert_eq!(item["clause_ref"], "art.2(a)");
    assert_eq!(item["authored_by"], "human");
    assert_eq!(item["needs_clause_ref"], false);
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn uncited_item_is_rejected() {
    // §1 #2 / AC #2 - an empty clause_ref is rejected 422 with rubric_item_uncited.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let ftoken = token(Uuid::new_v4(), tenant, &["founder"]);
    let (rid, vid) = new_draft_version(&pool, &ftoken).await;

    let mut body = good_item();
    body["clause_ref"] = json!("");
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            body,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let err = body_json(res).await;
    assert_eq!(err, json!("rubric_item_uncited"));
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn source_doc_outside_the_three_is_rejected() {
    // §1 #2 / AC #3 - source_doc='handbook' violates the CHECK; one of the three is accepted (covered by
    // well_formed_item_is_accepted). The CHECK violation surfaces as a 500 (a DB constraint error), which is
    // the documented outcome - the value never reaches a row.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let ftoken = token(Uuid::new_v4(), tenant, &["founder"]);
    let (rid, vid) = new_draft_version(&pool, &ftoken).await;

    // A source_doc that is not one of the three enum variants fails deserialization at the API boundary
    // (the body cannot be parsed into the closed SourceDoc enum) -> 422 from axum's JSON extractor.
    let mut body = good_item();
    body["source_doc"] = json!("handbook");
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            body,
        ))
        .await
        .unwrap();
    assert!(
        res.status() == StatusCode::UNPROCESSABLE_ENTITY || res.status() == StatusCode::BAD_REQUEST,
        "a source_doc outside the three documents must be rejected, got {}",
        res.status()
    );
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn obligation_without_obligation_kind_is_rejected() {
    // §1 #3 / AC #4.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let ftoken = token(Uuid::new_v4(), tenant, &["founder"]);
    let (rid, vid) = new_draft_version(&pool, &ftoken).await;

    let mut body = good_item();
    body.as_object_mut().unwrap().remove("obligation_kind");
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            body,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let err = body_json(res).await;
    assert_eq!(err, json!("rubric_item_obligation_kind_required"));
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn threshold_numeric_requires_full_check_shape() {
    // §1 #4 / AC #5 - a complete {metric, operator, target} KPI is accepted; {} is rejected 422.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let ftoken = token(Uuid::new_v4(), tenant, &["founder"]);
    let (rid, vid) = new_draft_version(&pool, &ftoken).await;

    let kpi_ok = json!({
        "source_doc": "total_rewards",
        "clause_ref": "art.2(a)",
        "item_kind": "kpi",
        "check_type": "threshold_numeric",
        "check_params": {"metric": "on_time_delivery", "operator": ">=", "target": 0.9},
        "weight": 5,
        "title_vi": "Giao hàng đúng hạn",
        "title_en": "On-time delivery"
    });
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            kpi_ok,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let kpi_bad = json!({
        "source_doc": "total_rewards",
        "clause_ref": "art.2(a)",
        "item_kind": "kpi",
        "check_type": "threshold_numeric",
        "check_params": {},
        "weight": 5,
        "title_vi": "Giao hàng đúng hạn"
    });
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            kpi_bad,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    let err = body_json(res).await;
    assert_eq!(err, json!("rubric_item_check_params_invalid"));
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn missing_vietnamese_title_is_rejected() {
    // §1 #5 / AC #6 - title_vi="" is rejected; title_en absent is allowed (proven below).
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let ftoken = token(Uuid::new_v4(), tenant, &["founder"]);
    let (rid, vid) = new_draft_version(&pool, &ftoken).await;

    let mut body = good_item();
    body["title_vi"] = json!("");
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            body,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body_json(res).await, json!("rubric_item_missing_vi"));

    // title_en absent is fine - the Vietnamese is the required, legally-operative text.
    let mut body = good_item();
    body.as_object_mut().unwrap().remove("title_en");
    let res = app(pool.clone())
        .oneshot(post(
            &format!("/v1/eval/rubrics/{rid}/versions/{vid}/items"),
            &ftoken,
            body,
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}

#[tokio::test]
#[ignore = "requires Postgres - boot services/dev/docker-compose.yml first"]
async fn non_admin_cannot_author() {
    // §1 #10 / AC #14 - a caller without the rubric-admin grant is forbidden from authoring.
    let pool = pool().await;
    let tenant = Uuid::new_v4();
    let mtoken = token(Uuid::new_v4(), tenant, &["tenant-member"]);

    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/rubrics",
            &mtoken,
            json!({ "name": "rogue rubric" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::FORBIDDEN);

    // A designated rubric-admin (non-founder) MAY author.
    let atoken = token(Uuid::new_v4(), tenant, &["rubric-admin"]);
    let res = app(pool.clone())
        .oneshot(post(
            "/v1/eval/rubrics",
            &atoken,
            json!({ "name": "admin rubric" }),
        ))
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
}
