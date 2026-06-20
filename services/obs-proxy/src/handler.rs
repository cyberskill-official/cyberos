//! The request lifecycle (FR-OBS-002 §3), independent of the HTTP framework.
//!
//! `handle` ties the pieces together: authenticate the bearer token, detect the backend from the path,
//! refuse a user-supplied tenant label (sev-1 audit), pass a root-admin query through unfiltered,
//! otherwise inject the tenant filter, forward to the backend, and emit the audit row. It is generic
//! over a `Forwarder` so the whole flow is testable with a recording forwarder - the reqwest forwarder,
//! the axum router, and the JWKS boot fetch are the thin production shell (slice 4c).

use crate::audit::{self, AuditSink};
use crate::auth::Authenticator;
use crate::error::{Backend, ProxyError};
use crate::proxy::{decide, detect_backend, Outcome};

/// Forwards an already-injected query to a backend and returns the response body.
pub trait Forwarder {
    fn forward(
        &self,
        backend: Backend,
        query: &str,
    ) -> impl std::future::Future<Output = Result<String, ProxyError>>;
}

/// Handle one query request end to end. Returns the backend response body, or an error the HTTP shell
/// maps to a status (AuthFailed -> 401, UserSuppliedTenantId -> 400, UnsupportedPath -> 404,
/// ParseFailed -> 400, BackendUnreachable -> 503).
pub async fn handle<F: Forwarder>(
    auth: &Authenticator,
    forwarder: &F,
    sink: &dyn AuditSink,
    token: Option<&str>,
    path: &str,
    query: &str,
    request_id: &str,
) -> Result<String, ProxyError> {
    let token = token.ok_or_else(|| ProxyError::AuthFailed("missing bearer token".into()))?;
    let claims = auth.verify(token)?;
    let backend = detect_backend(path).ok_or_else(|| ProxyError::UnsupportedPath(path.to_string()))?;

    let (forward_query, outcome) = match decide(&claims, query, backend) {
        Ok(decision) => decision,
        Err(ProxyError::UserSuppliedTenantId) => {
            // sev-1: a caller tried to supply their own tenant_id label.
            sink.emit(&audit::cross_tenant_query_attempt(
                &claims.tenant_id,
                query,
                request_id,
            ));
            return Err(ProxyError::UserSuppliedTenantId);
        }
        Err(other) => return Err(other),
    };

    let body = forwarder.forward(backend, &forward_query).await?;

    let outcome_str = match outcome {
        Outcome::Proxied => "proxied",
        Outcome::RootAdminUnfiltered => "root_admin_unfiltered",
    };
    sink.emit(&audit::query_proxied(
        &claims.tenant_id,
        &claims.sub,
        backend,
        query,
        outcome_str,
        request_id,
    ));
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::RecordingSink;
    use crate::auth::{Claims, NIL_TENANT};
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use std::sync::Mutex;

    const SECRET: &[u8] = b"test-secret";

    /// Records the last forwarded (backend, query); can simulate a down backend.
    struct Recording {
        last: Mutex<Option<(Backend, String)>>,
        fail: bool,
    }
    impl Recording {
        fn new() -> Self {
            Self {
                last: Mutex::new(None),
                fail: false,
            }
        }
        fn failing() -> Self {
            Self {
                last: Mutex::new(None),
                fail: true,
            }
        }
        fn last(&self) -> Option<(Backend, String)> {
            self.last.lock().unwrap().clone()
        }
    }
    impl Forwarder for Recording {
        async fn forward(&self, backend: Backend, query: &str) -> Result<String, ProxyError> {
            if self.fail {
                return Err(ProxyError::BackendUnreachable("connection refused".into()));
            }
            *self.last.lock().unwrap() = Some((backend, query.to_string()));
            Ok(format!("OK:{query}"))
        }
    }

    fn auth() -> Authenticator {
        Authenticator::from_hs256_secret(SECRET)
    }
    fn future() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + 3600
    }
    fn jwt(tenant: &str) -> String {
        let c = Claims {
            sub: "sub-1".into(),
            tenant_id: tenant.into(),
            aud: vec![],
            exp: future(),
        };
        encode(
            &Header::new(Algorithm::HS256),
            &c,
            &EncodingKey::from_secret(SECRET),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn missing_token_is_unauthorized() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let e = handle(&auth(), &r, &s, None, "/api/v1/query", "rate(foo[5m])", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::AuthFailed(_)));
    }

    #[tokio::test]
    async fn invalid_token_is_unauthorized() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let e = handle(&auth(), &r, &s, Some("bad.jwt"), "/api/v1/query", "rate(foo[5m])", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::AuthFailed(_)));
    }

    #[tokio::test]
    async fn unknown_path_is_unsupported() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let e = handle(&auth(), &r, &s, Some(&jwt("T")), "/nope", "x", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::UnsupportedPath(_)));
    }

    #[tokio::test]
    async fn regular_tenant_query_injected_forwarded_audited() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let body = handle(&auth(), &r, &s, Some(&jwt("T")), "/api/v1/query", "rate(foo[5m])", "id1")
            .await
            .unwrap();
        let (be, fwd) = r.last().unwrap();
        assert_eq!(be, Backend::Prometheus);
        assert!(fwd.contains("tenant_id=\"T\""));
        assert!(body.contains("tenant_id=\"T\""));
        let row = s.latest("obs.query_proxied").unwrap();
        assert_eq!(row.payload["tenant_id"], "T");
        assert_eq!(row.payload["outcome"], "proxied");
    }

    #[tokio::test]
    async fn user_supplied_tenant_refused_and_sev1_audited_not_forwarded() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let e = handle(
            &auth(),
            &r,
            &s,
            Some(&jwt("T")),
            "/api/v1/query",
            "foo{tenant_id=\"other\"}",
            "id1",
        )
        .await
        .unwrap_err();
        assert!(matches!(e, ProxyError::UserSuppliedTenantId));
        assert!(r.last().is_none()); // never reached the backend
        assert!(s.latest("obs.cross_tenant_query_attempt").is_some());
    }

    #[tokio::test]
    async fn root_admin_query_forwarded_unfiltered() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let _ = handle(
            &auth(),
            &r,
            &s,
            Some(&jwt(NIL_TENANT)),
            "/api/v1/query",
            "rate(foo[5m])",
            "id1",
        )
        .await
        .unwrap();
        let (_, fwd) = r.last().unwrap();
        assert!(!fwd.contains("tenant_id"));
        assert_eq!(
            s.latest("obs.query_proxied").unwrap().payload["outcome"],
            "root_admin_unfiltered"
        );
    }

    #[tokio::test]
    async fn backend_unreachable_propagates() {
        let (r, s) = (Recording::failing(), RecordingSink::default());
        let e = handle(&auth(), &r, &s, Some(&jwt("T")), "/loki/api/v1/query", "{service=\"x\"}", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::BackendUnreachable(_)));
    }
}
