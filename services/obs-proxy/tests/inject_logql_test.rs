//! TASK-OBS-002 §5 - LogQL injection acceptance tests (integration, against the public crate API).

use cyberos_obs_proxy::inject::logql;
use cyberos_obs_proxy::ProxyError;

#[test]
fn injects_logql_simple_selector() {
    let result = logql::add_label("{service=\"x\"}", "tenant_id", "T").unwrap();
    assert_eq!(result, "{service=\"x\",tenant_id=\"T\"}");
}

#[test]
fn preserves_pipe_stages() {
    let result = logql::add_label(
        "{service=\"x\"} | json | line_format \"...\"",
        "tenant_id",
        "T",
    )
    .unwrap();
    assert!(result.starts_with("{service=\"x\",tenant_id=\"T\"}"));
    assert!(result.contains("| json"));
    assert!(result.contains("| line_format"));
}

#[test]
fn rejects_query_that_already_supplies_tenant_id() {
    // TASK-OBS-002 §1 #4 / §4 #6 - a user-supplied tenant_id is a bypass attempt and is refused.
    let err = logql::add_label("{tenant_id=\"other\"}", "tenant_id", "T")
        .expect_err("must refuse a user-supplied tenant_id");
    assert!(matches!(err, ProxyError::ParseFailed { .. }));
    assert!(logql::has_label("{tenant_id=\"other\"}", "tenant_id").unwrap());
}
