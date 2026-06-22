//! FR-OBS-002 §4 #5 / §8 - TraceQL injection acceptance tests (integration).

use cyberos_obs_proxy::inject::traceql;

#[test]
fn injects_resource_tenant_id_filter() {
    // §8 example, exact.
    let result = traceql::add_label(
        "{ service.name = \"ai-gateway\" }",
        "resource.tenant_id",
        "org:cyberskill",
    )
    .unwrap();
    assert_eq!(
        result,
        "{ service.name = \"ai-gateway\" && resource.tenant_id = \"org:cyberskill\" }"
    );
}

#[test]
fn detects_user_supplied_tenant_id() {
    assert!(
        traceql::has_label("{ resource.tenant_id = \"other\" }", "resource.tenant_id").unwrap()
    );
}
