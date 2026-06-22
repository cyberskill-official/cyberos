//! The proxy decision logic (FR-OBS-002 §3) as a pure function, separate from the HTTP I/O.
//!
//! `decide` is the security core: given verified claims, the user's query, and the target backend, it
//! returns the query to forward plus the audit outcome - or an error the router maps to a status. The
//! axum router, reqwest forwarding, and JWKS boot fetch (slice 4b) wrap this; keeping it pure makes
//! the tenant-isolation behaviour exhaustively testable without a network.

use crate::auth::Claims;
use crate::error::{Backend, ProxyError};
use crate::inject;

/// The audit outcome of a proxied query (FR §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Proxied,
    RootAdminUnfiltered,
}

/// Map a request path to its backend (FR §1 #13). Returns `None` for paths the proxy does not serve.
pub fn detect_backend(path: &str) -> Option<Backend> {
    if path.starts_with("/loki/") {
        Some(Backend::Loki)
    } else if path.starts_with("/tempo/")
        || path.starts_with("/api/search")
        || path.starts_with("/api/traces")
    {
        Some(Backend::Tempo)
    } else if path.starts_with("/api/v1/") || path.starts_with("/api/labels") {
        Some(Backend::Prometheus)
    } else {
        None
    }
}

/// The reserved label per backend: Tempo namespaces by `resource.tenant_id`, Loki/Prometheus by
/// `tenant_id`.
fn reserved_label(backend: Backend) -> &'static str {
    match backend {
        Backend::Tempo => "resource.tenant_id",
        _ => "tenant_id",
    }
}

/// True if the user's query already supplies the reserved tenant label - a bypass attempt (FR §1 #4).
pub fn has_user_supplied_tenant_id(query: &str, backend: Backend) -> Result<bool, ProxyError> {
    match backend {
        Backend::Prometheus => inject::promql::has_label(query, "tenant_id"),
        Backend::Loki => inject::logql::has_label(query, "tenant_id"),
        Backend::Tempo => inject::traceql::has_label(query, "resource.tenant_id"),
    }
}

/// Decide what to forward (FR §3):
/// - a user-supplied tenant label is refused (`UserSuppliedTenantId` -> 400 + sev-1 audit);
/// - a root-admin (nil-UUID tenant) query is forwarded unchanged (`RootAdminUnfiltered`);
/// - otherwise the tenant label is AST-injected into every selector.
pub fn decide(
    claims: &Claims,
    query: &str,
    backend: Backend,
) -> Result<(String, Outcome), ProxyError> {
    if has_user_supplied_tenant_id(query, backend)? {
        return Err(ProxyError::UserSuppliedTenantId);
    }
    if claims.is_root_admin() {
        return Ok((query.to_string(), Outcome::RootAdminUnfiltered));
    }
    let label = reserved_label(backend);
    let injected = match backend {
        Backend::Prometheus => inject::promql::add_label(query, label, &claims.tenant_id)?,
        Backend::Loki => inject::logql::add_label(query, label, &claims.tenant_id)?,
        Backend::Tempo => inject::traceql::add_label(query, label, &claims.tenant_id)?,
    };
    Ok((injected, Outcome::Proxied))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::NIL_TENANT;

    fn claims(tenant: &str) -> Claims {
        Claims {
            sub: "subject-1".into(),
            tenant_id: tenant.into(),
            aud: vec![],
            exp: 0,
        }
    }

    #[test]
    fn detect_backend_maps_paths() {
        assert_eq!(
            detect_backend("/api/v1/query?query=x"),
            Some(Backend::Prometheus)
        );
        assert_eq!(detect_backend("/loki/api/v1/query"), Some(Backend::Loki));
        assert_eq!(detect_backend("/api/search?q=x"), Some(Backend::Tempo));
        assert_eq!(
            detect_backend("/tempo/api/traces/abc"),
            Some(Backend::Tempo)
        );
        assert_eq!(detect_backend("/nope"), None);
    }

    #[test]
    fn injects_for_a_regular_tenant() {
        let (q, o) = decide(
            &claims("org:cyberskill"),
            "rate(foo[5m])",
            Backend::Prometheus,
        )
        .unwrap();
        assert!(q.contains("tenant_id=\"org:cyberskill\""));
        assert_eq!(o, Outcome::Proxied);
    }

    #[test]
    fn refuses_user_supplied_tenant_id() {
        let e = decide(
            &claims("T"),
            "foo{tenant_id=\"other\"}",
            Backend::Prometheus,
        )
        .unwrap_err();
        assert!(matches!(e, ProxyError::UserSuppliedTenantId));
    }

    #[test]
    fn root_admin_query_is_unfiltered() {
        let (q, o) = decide(&claims(NIL_TENANT), "rate(foo[5m])", Backend::Prometheus).unwrap();
        assert!(!q.contains("tenant_id"));
        assert_eq!(o, Outcome::RootAdminUnfiltered);
    }

    #[test]
    fn injects_for_loki_and_tempo() {
        let (lq, _) = decide(&claims("T"), "{service=\"x\"}", Backend::Loki).unwrap();
        assert!(lq.contains("tenant_id=\"T\""));
        let (tq, _) = decide(&claims("T"), "{ service.name = \"x\" }", Backend::Tempo).unwrap();
        assert!(tq.contains("resource.tenant_id = \"T\""));
    }
}
