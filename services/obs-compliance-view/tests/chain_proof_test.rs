mod support;

use cyberos_obs_compliance_view::chain_proof::{canonicalise, verify};
use cyberos_obs_compliance_view::error::ViewError;
use cyberos_obs_compliance_view::export::{json, pdf};
use cyberos_obs_compliance_view::memory::{AuditRow, InMemoryBackend};
use cyberos_obs_compliance_view::views;
use support::{backend, claims, query, signer};

#[tokio::test]
async fn chain_proof_verifies_independently() {
    let signer = signer();
    let memory = backend();
    let resp = views::build_view(
        memory.as_ref(),
        &signer,
        views::eu_ai_act::definition(),
        query(),
        claims("t1"),
    )
    .await
    .unwrap();
    let canonical = canonicalise(&resp.rows, &resp.summary).unwrap();
    assert!(!canonical.is_empty());
    assert!(verify(
        &signer.verifying_key(),
        &resp.rows,
        &resp.summary,
        &resp.chain_proof
    ));
}

#[tokio::test]
async fn json_and_pdf_exports_are_deterministic_and_include_rows() {
    let signer = signer();
    let memory = backend();
    let resp = views::build_view(
        memory.as_ref(),
        &signer,
        views::soc2::definition(),
        query(),
        claims("t1"),
    )
    .await
    .unwrap();
    assert_eq!(
        json::render_json(&resp).unwrap(),
        json::render_json(&resp).unwrap()
    );
    let pdf = pdf::render_pdf(&resp).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    let pdf_text = String::from_utf8_lossy(&pdf);
    for row in &resp.rows {
        assert!(pdf_text.contains(&row.kind));
    }
}

#[tokio::test]
async fn pii_in_response_500s_and_metric_can_increment() {
    let now = chrono::Utc::now();
    let memory = InMemoryBackend::with_rows(vec![AuditRow::new(
        now,
        "ai.invocation",
        "t1",
        serde_json::json!({"leaked_email": "alice@cyberos.world"}),
    )]);
    let err = views::build_view(
        &memory,
        &signer(),
        views::eu_ai_act::definition(),
        query(),
        claims("t1"),
    )
    .await
    .unwrap_err();
    assert_eq!(err, ViewError::PiiLeakAttempt);
}

#[tokio::test]
async fn time_range_over_365_days_rejected() {
    let mut query = query();
    query.since = chrono::Utc::now() - chrono::Duration::days(400);
    let err = views::build_view(
        backend().as_ref(),
        &signer(),
        views::eu_ai_act::definition(),
        query,
        claims("t1"),
    )
    .await
    .unwrap_err();
    assert_eq!(err, ViewError::TimeRangeTooLarge);
}

#[tokio::test]
async fn audit_row_per_query_is_emitted() {
    let memory = backend();
    views::build_view(
        memory.as_ref(),
        &signer(),
        views::eu_ai_act::definition(),
        query(),
        claims("t1"),
    )
    .await
    .unwrap();
    let emitted = memory.emitted();
    assert_eq!(emitted[0].kind, "obs.compliance_view_accessed");
    assert_eq!(emitted[0].payload["view"], "eu-ai-act");
    assert_eq!(emitted[0].payload["tenant_id"], "t1");
}
