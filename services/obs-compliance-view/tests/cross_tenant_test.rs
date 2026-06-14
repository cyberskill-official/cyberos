mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use cyberos_obs_compliance_view::auth::{issue_local_auditor_token, AuthConfig};
use cyberos_obs_compliance_view::error::ViewError;
use cyberos_obs_compliance_view::views;
use cyberos_obs_compliance_view::app;
use support::{app_state, backend, claims, query, signer, token, SECRET};
use tower::ServiceExt;

#[tokio::test]
async fn tenant_id_query_param_rejected() {
    let memory = backend();
    let mut query = query();
    query.tenant_id = Some("t2".to_string());
    let err = views::build_view(
        memory.as_ref(),
        &signer(),
        views::eu_ai_act::definition(),
        query,
        claims("t1"),
    )
    .await
    .unwrap_err();
    assert_eq!(err, ViewError::TenantIdInQuery);
}

#[tokio::test]
async fn tenant_scope_excludes_other_tenant_rows() {
    let memory = backend();
    let resp = views::build_view(
        memory.as_ref(),
        &signer(),
        views::eu_ai_act::definition(),
        query(),
        claims("t1"),
    )
    .await
    .unwrap();
    assert!(resp.rows.iter().all(|row| row.tenant_id == "t1"));
}

#[tokio::test]
async fn jwt_missing_or_without_auditor_role_is_rejected() {
    let app = app(app_state(backend()));
    let uri = "/eu-ai-act/?since=2026-01-01T00:00:00Z&until=2026-01-31T00:00:00Z";

    let response = app
        .clone()
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let non_auditor = issue_local_auditor_token(
        SECRET,
        "t1",
        "subject-1",
        vec!["tenant-admin".to_string()],
        30,
    );
    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .header("authorization", format!("Bearer {non_auditor}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn per_engagement_ttl_supported() {
    let token = token(vec!["external_auditor".to_string()]);
    let claims = cyberos_obs_compliance_view::auth::verify_authorization(
        Some(&format!("Bearer {token}")),
        &AuthConfig::local(SECRET),
    )
    .unwrap();
    assert!(claims.supports_engagement_ttl());
}
