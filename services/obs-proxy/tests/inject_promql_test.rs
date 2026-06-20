//! FR-OBS-002 §5 - PromQL injection acceptance tests (integration, against the public crate API).

use cyberos_obs_proxy::inject::promql;
use cyberos_obs_proxy::ProxyError;

#[test]
fn injects_into_simple_selector() {
    let result = promql::add_label("rate(foo[5m])", "tenant_id", "T").unwrap();
    assert!(result.contains("tenant_id=\"T\""));
    assert!(result.contains("foo"));
}

#[test]
fn injects_into_every_selector_in_complex_query() {
    let result =
        promql::add_label("sum(rate(foo[5m])) / sum(rate(bar[5m]))", "tenant_id", "T").unwrap();
    assert_eq!(result.matches("tenant_id=\"T\"").count(), 2);
}

#[test]
fn detects_user_supplied_tenant_id() {
    assert!(promql::has_label("foo{tenant_id=\"other\"}", "tenant_id").unwrap());
}

#[test]
fn malformed_promql_returns_parse_error() {
    let err = promql::add_label("rate(foo[bad", "tenant_id", "T")
        .expect_err("expected a parse failure, not a panic");
    assert!(matches!(err, ProxyError::ParseFailed { .. }));
}
