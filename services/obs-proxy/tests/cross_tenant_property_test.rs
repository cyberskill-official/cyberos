//! TASK-OBS-002 §4 #13 / §5 - the cross-tenant invariant as a property test (mirrors TASK-AI-018).
//!
//! Pure / injection-level: a non-root tenant's forwarded query always carries ONLY that tenant's
//! `tenant_id` filter and never another tenant's, and any query that tries to supply its own
//! `tenant_id` is refused. 1000 random cases. (The response-level version - asserting tenant B's data
//! never appears in tenant A's response - lands with the mock backend once slice 4c's forwarder exists.)

use cyberos_obs_proxy::auth::Claims;
use cyberos_obs_proxy::error::Backend;
use cyberos_obs_proxy::proxy::{decide, Outcome};
use cyberos_obs_proxy::ProxyError;
use proptest::prelude::*;

fn claims(tenant: &str) -> Claims {
    Claims {
        sub: "subject".into(),
        tenant_id: tenant.into(),
        aud: vec![],
        exp: 0,
    }
}

/// Distinct, matcher-safe tenant ids (no quote / brace / backslash).
fn tenant() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9_:-]{2,12}"
}

/// Valid PromQL that carries no `tenant_id` label. Metric/label names are prefixed so a random ident
/// can never collide with a PromQL keyword (e.g. `on`, `bool`, `group_left`).
fn promql_query() -> impl Strategy<Value = String> {
    let metric = "metric_[a-z0-9_]{1,10}";
    prop_oneof![
        metric.prop_map(|m| m),
        metric.prop_map(|m| format!("rate({m}[5m])")),
        metric.prop_map(|m| format!("sum({m})")),
        (metric, "k_[a-z0-9]{1,5}", "[a-z0-9]{1,6}")
            .prop_map(|(m, k, v)| format!("{m}{{{k}=\"{v}\"}}")),
        (metric, metric).prop_map(|(a, b)| format!("sum(rate({a}[5m])) / sum(rate({b}[5m]))")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn forwarded_query_carries_only_the_callers_tenant(
        (a, b) in (tenant(), tenant()).prop_filter("distinct tenants", |(a, b)| a != b),
        q in promql_query(),
    ) {
        let (forwarded, outcome) = decide(&claims(&a), &q, Backend::Prometheus)
            .expect("a tenant query without a tenant_id label must inject, not error");
        prop_assert_eq!(outcome, Outcome::Proxied);
        prop_assert!(
            forwarded.contains(&format!("tenant_id=\"{a}\"")),
            "caller's tenant filter missing from: {forwarded}"
        );
        prop_assert!(
            !forwarded.contains(&format!("tenant_id=\"{b}\"")),
            "another tenant's id leaked into: {forwarded}"
        );
    }

    #[test]
    fn a_user_supplied_tenant_id_is_always_refused(
        a in tenant(),
        other in tenant(),
        metric in "metric_[a-z0-9_]{1,10}",
    ) {
        let q = format!("{metric}{{tenant_id=\"{other}\"}}");
        let err = decide(&claims(&a), &q, Backend::Prometheus)
            .expect_err("a user-supplied tenant_id must be refused");
        prop_assert!(matches!(err, ProxyError::UserSuppliedTenantId));
    }
}
