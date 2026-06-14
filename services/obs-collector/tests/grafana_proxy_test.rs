use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, HeaderValue};
use cyberos_obs_collector::grafana_proxy::{
    inject_query, Backend, BackendUrls, JwtVerifier, ProxyClaims, ProxyError, ProxyState,
    QueryOutcome,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::Serialize;

const SECRET: &str = "test-grafana-proxy-secret";

#[derive(Debug, Serialize)]
struct TestClaims<'a> {
    sub: &'a str,
    tenant_id: &'a str,
    exp: usize,
    roles: Vec<&'a str>,
}

fn state() -> ProxyState {
    ProxyState::new(
        BackendUrls {
            prometheus: "http://prometheus:9090".into(),
            loki: "http://loki:3100".into(),
            tempo: "http://tempo:3200".into(),
        },
        JwtVerifier::hs256(SECRET),
    )
}

fn token(tenant_id: &str, subject: &str, roles: Vec<&str>, exp: usize) -> String {
    encode(
        &Header::default(),
        &TestClaims {
            sub: subject,
            tenant_id,
            exp,
            roles,
        },
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap()
}

fn headers_with(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    headers
}

fn claims(tenant_id: &str) -> ProxyClaims {
    ProxyClaims {
        sub: "subject-a".into(),
        tenant_id: tenant_id.into(),
        exp: future_exp(),
        roles: Vec::new(),
        iss: None,
        aud: None,
    }
}

fn future_exp() -> usize {
    4_102_444_800
}

#[test]
fn promql_injection_covers_complex_queries() {
    let rewritten = inject_query(
        Backend::Prometheus,
        "sum(rate(foo[5m])) / sum(rate(bar[5m]))",
        "tenant-a",
    )
    .unwrap();
    assert_eq!(rewritten.matches(r#"tenant_id="tenant-a""#).count(), 2);
}

#[test]
fn promql_rejects_only_tenant_label_matchers_not_string_arguments() {
    let rewritten = inject_query(
        Backend::Prometheus,
        r#"label_replace(foo, "dst", "$1", "tenant_id", "(.*)")"#,
        "tenant-a",
    )
    .unwrap();
    assert!(rewritten.contains(r#"foo{tenant_id="tenant-a"}"#));
    assert!(rewritten.contains(r#""tenant_id""#));
}

#[test]
fn logql_and_traceql_injection_preserve_query_shape() {
    let logql = inject_query(
        Backend::Loki,
        r#"{service="api"} | json | line_format "{{.message}}""#,
        "tenant-a",
    )
    .unwrap();
    assert!(logql.starts_with(r#"{service="api",tenant_id="tenant-a"}"#));
    assert!(logql.contains("| json"));

    let traceql = inject_query(
        Backend::Tempo,
        r#"{ service.name = "ai-gateway" }"#,
        "tenant-a",
    )
    .unwrap();
    assert_eq!(
        traceql,
        r#"{ service.name = "ai-gateway" && resource.tenant_id = "tenant-a" }"#
    );
}

#[test]
fn missing_invalid_and_expired_jwt_are_unauthorized() {
    let proxy = state();
    let missing = HeaderMap::new();
    assert!(matches!(
        proxy.verify_headers(&missing).unwrap_err(),
        ProxyError::MissingBearer
    ));

    let mut invalid = HeaderMap::new();
    invalid.insert(AUTHORIZATION, HeaderValue::from_static("Bearer not-a-jwt"));
    assert!(matches!(
        proxy.verify_headers(&invalid).unwrap_err(),
        ProxyError::AuthFailed(_)
    ));

    let expired = token("tenant-a", "subject-a", Vec::new(), 1);
    assert!(matches!(
        proxy.verify_headers(&headers_with(&expired)).unwrap_err(),
        ProxyError::AuthFailed(_)
    ));
}

#[test]
fn valid_jwt_extracts_tenant_claim() {
    let proxy = state();
    let jwt = token("tenant-a", "subject-a", Vec::new(), future_exp());
    let verified = proxy.verify_headers(&headers_with(&jwt)).unwrap();
    assert_eq!(verified.tenant_id, "tenant-a");
    assert_eq!(verified.sub, "subject-a");
}

#[test]
fn user_supplied_tenant_label_returns_400_shape_and_sev1_audit() {
    let proxy = state();
    let err = proxy
        .process_query(
            Backend::Prometheus,
            r#"http_requests_total{tenant_id="other"}"#,
            &claims("tenant-a"),
            "req-1",
        )
        .unwrap_err();
    assert!(matches!(err, ProxyError::UserSuppliedTenantId));

    let events = proxy.audit_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "obs.cross_tenant_query_attempt");
    assert_eq!(events[0].severity.as_deref(), Some("sev-1"));
    assert_eq!(events[0].payload["caller_tenant_id"], "tenant-a");
    assert_eq!(events[0].payload["attempted_label_value"], "other");
}

#[test]
fn every_successful_query_emits_privacy_preserving_audit_row() {
    let proxy = state();
    let rewrite = proxy
        .process_query(
            Backend::Prometheus,
            "rate(foo[5m])",
            &claims("tenant-a"),
            "req-2",
        )
        .unwrap();
    assert_eq!(rewrite.outcome, QueryOutcome::Proxied);
    assert!(rewrite.rewritten_query.contains(r#"tenant_id="tenant-a""#));

    let events = proxy.audit_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "obs.query_proxied");
    assert_eq!(events[0].payload["tenant_id"], "tenant-a");
    assert_eq!(events[0].payload["backend"], "prometheus");
    assert_eq!(events[0].payload["outcome"], "proxied");
    assert_eq!(
        events[0].payload["query_sha256"].as_str().unwrap().len(),
        64
    );
}

#[test]
fn root_admin_query_is_forwarded_unfiltered_and_audited() {
    let proxy = state();
    let root = ProxyClaims {
        sub: "root-subject".into(),
        tenant_id: "00000000-0000-0000-0000-000000000000".into(),
        exp: future_exp(),
        roles: vec!["root-admin".into()],
        iss: None,
        aud: None,
    };
    let rewrite = proxy
        .process_query(Backend::Prometheus, "rate(foo[5m])", &root, "req-3")
        .unwrap();
    assert_eq!(rewrite.rewritten_query, "rate(foo[5m])");
    assert_eq!(rewrite.outcome, QueryOutcome::RootAdminUnfiltered);

    let events = proxy.audit_events();
    assert_eq!(events[0].kind, "obs.query_proxied");
    assert_eq!(events[0].payload["outcome"], "root_admin_unfiltered");
    assert_eq!(events[0].severity.as_deref(), Some("sev-2"));
}

#[test]
fn property_style_queries_never_inject_another_tenant() {
    let tenants = [("tenant-a", "tenant-b"), ("tenant-b", "tenant-a")];
    for i in 0..1000 {
        let metric = format!("metric_{i}");
        let query = format!(r#"sum(rate({metric}{{service="svc-{i}"}}[5m]))"#);
        for (caller, other) in tenants {
            let rewritten = inject_query(Backend::Prometheus, &query, caller).unwrap();
            assert!(rewritten.contains(&format!(r#"tenant_id="{caller}""#)));
            assert!(!rewritten.contains(&format!(r#"tenant_id="{other}""#)));
        }
    }
}

#[test]
fn injection_overhead_stays_under_five_ms_for_typical_queries() {
    let proxy = state();
    let mut latencies = Vec::new();
    for i in 0..200 {
        let rewrite = proxy
            .process_query(
                Backend::Prometheus,
                &format!("sum(rate(metric_{i}[5m]))"),
                &claims("tenant-a"),
                &format!("req-latency-{i}"),
            )
            .unwrap();
        latencies.push(rewrite.injection_latency_ms);
    }
    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
    assert!(p95 < 5.0, "p95 injection overhead was {p95}ms");
}
