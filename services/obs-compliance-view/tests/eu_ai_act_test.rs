mod support;

use cyberos_obs_compliance_view::views;
use support::{backend, claims, query, signer};

#[tokio::test]
async fn eu_ai_act_view_returns_relevant_rows() {
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

    let kinds: std::collections::BTreeSet<_> =
        resp.rows.iter().map(|row| row.kind.as_str()).collect();
    assert!(kinds.contains("ai.invocation"));
    assert!(kinds.contains("ai.persona_loaded"));
    assert!(!kinds.contains("auth.token_issued"));
    assert_eq!(resp.summary["total_calls"], 1);
    assert_eq!(resp.summary["unique_personas"], 1);
}
