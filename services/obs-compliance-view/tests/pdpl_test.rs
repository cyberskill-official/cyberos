mod support;

use cyberos_obs_compliance_view::views;
use support::{backend, claims, query, signer};

#[tokio::test]
async fn pdpl_view_returns_dsar_cross_border_and_consent_rows() {
    let memory = backend();
    let resp = views::build_view(
        memory.as_ref(),
        &signer(),
        views::pdpl::definition(),
        query(),
        claims("t1"),
    )
    .await
    .unwrap();

    let kinds: std::collections::BTreeSet<_> =
        resp.rows.iter().map(|row| row.kind.as_str()).collect();
    assert!(kinds.contains("dsar.export_completed"));
    assert!(kinds.contains("obs.langsmith_export_enabled"));
    assert_eq!(resp.summary["dsar_rows"], 1);
    assert_eq!(resp.summary["consent_rows"], 1);
}
