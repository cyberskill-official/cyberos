//! The request lifecycle (FR-OBS-002 §3), independent of the HTTP framework.
//!
//! `handle` authenticates, detects the backend, finds the `query` parameter, refuses a user-supplied
//! tenant (sev-1 audit), passes a root-admin query through unfiltered, otherwise injects the tenant
//! filter into the query while preserving every other parameter (start / end / step / time), forwards,
//! and emits the audit row. It is generic over a `Forwarder` so the whole flow is testable with a
//! recording forwarder; the reqwest forwarder + axum router are the production shell (main.rs).

use crate::audit::{self, AuditSink};
use crate::auth::Authenticator;
use crate::error::{Backend, ProxyError};
use crate::proxy::{decide, detect_backend, Outcome};

/// Forwards a request to a backend - GET `<base><path>` with `params` as the query string - and
/// returns the response body.
pub trait Forwarder {
    fn forward(
        &self,
        backend: Backend,
        path: &str,
        params: &[(String, String)],
    ) -> impl std::future::Future<Output = Result<String, ProxyError>>;
}

/// Handle one query request end to end. `raw_query` is the URL query string (GET) or the form body
/// (POST) - both are `application/x-www-form-urlencoded`. Returns the backend body, or an error the
/// HTTP shell maps to a status (AuthFailed -> 401, UserSuppliedTenantId -> 400, UnsupportedPath -> 404,
/// ParseFailed -> 400, BackendUnreachable -> 503).
pub async fn handle<F: Forwarder>(
    auth: &Authenticator,
    forwarder: &F,
    sink: &dyn AuditSink,
    token: Option<&str>,
    path: &str,
    raw_query: &str,
    request_id: &str,
) -> Result<String, ProxyError> {
    let token = token.ok_or_else(|| ProxyError::AuthFailed("missing bearer token".into()))?;
    let claims = auth.verify(token)?;
    let backend = detect_backend(path).ok_or_else(|| ProxyError::UnsupportedPath(path.to_string()))?;

    let mut params: Vec<(String, String)> = form_urlencoded::parse(raw_query.as_bytes())
        .into_owned()
        .collect();
    let query_idx = params.iter().position(|(k, _)| k == "query");

    let outcome;
    let audited_query;
    match query_idx {
        Some(i) => {
            let user_query = params[i].1.clone();
            match decide(&claims, &user_query, backend) {
                Ok((injected, oc)) => {
                    params[i].1 = injected;
                    outcome = oc;
                    audited_query = user_query;
                }
                Err(ProxyError::UserSuppliedTenantId) => {
                    sink.emit(&audit::cross_tenant_query_attempt(
                        &claims.tenant_id,
                        &user_query,
                        request_id,
                    ));
                    return Err(ProxyError::UserSuppliedTenantId);
                }
                Err(other) => return Err(other),
            }
        }
        // No `query` parameter (e.g. /api/v1/labels, /series). Forwarded as-is; per-tenant scoping of
        // the label/series endpoints is a documented follow-up beyond the query path (FR §1 #13).
        None => {
            outcome = Outcome::Proxied;
            audited_query = raw_query.to_string();
        }
    }

    let body = forwarder.forward(backend, path, &params).await?;

    let outcome_str = match outcome {
        Outcome::Proxied => "proxied",
        Outcome::RootAdminUnfiltered => "root_admin_unfiltered",
    };
    sink.emit(&audit::query_proxied(
        &claims.tenant_id,
        &claims.sub,
        backend,
        &audited_query,
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

    /// (backend, path, forwarded params) captured by the recording forwarder.
    type Recorded = (Backend, String, Vec<(String, String)>);

    struct Recording {
        last: Mutex<Option<Recorded>>,
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
        fn last(&self) -> Option<Recorded> {
            self.last.lock().unwrap().clone()
        }
        /// The forwarded value of the `query` parameter, if any.
        fn forwarded_query(&self) -> Option<String> {
            self.last().and_then(|(_, _, params)| {
                params.into_iter().find(|(k, _)| k == "query").map(|(_, v)| v)
            })
        }
    }
    impl Forwarder for Recording {
        async fn forward(
            &self,
            backend: Backend,
            path: &str,
            params: &[(String, String)],
        ) -> Result<String, ProxyError> {
            if self.fail {
                return Err(ProxyError::BackendUnreachable("connection refused".into()));
            }
            *self.last.lock().unwrap() = Some((backend, path.to_string(), params.to_vec()));
            Ok("OK".into())
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
        let e = handle(&auth(), &r, &s, None, "/api/v1/query", "query=rate(foo[5m])", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::AuthFailed(_)));
    }

    #[tokio::test]
    async fn invalid_token_is_unauthorized() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let e = handle(&auth(), &r, &s, Some("bad.jwt"), "/api/v1/query", "query=up", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::AuthFailed(_)));
    }

    #[tokio::test]
    async fn unknown_path_is_unsupported() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let e = handle(&auth(), &r, &s, Some(&jwt("T")), "/nope", "query=up", "id1")
            .await
            .unwrap_err();
        assert!(matches!(e, ProxyError::UnsupportedPath(_)));
    }

    #[tokio::test]
    async fn regular_tenant_query_injected_forwarded_audited_preserving_params() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        let body = handle(
            &auth(),
            &r,
            &s,
            Some(&jwt("T")),
            "/api/v1/query_range",
            "query=rate(foo[5m])&start=1&end=2&step=15",
            "id1",
        )
        .await
        .unwrap();
        assert_eq!(body, "OK");
        let (be, path, params) = r.last().unwrap();
        assert_eq!(be, Backend::Prometheus);
        assert_eq!(path, "/api/v1/query_range");
        assert!(r.forwarded_query().unwrap().contains("tenant_id=\"T\""));
        // sibling params preserved
        assert!(params.iter().any(|(k, v)| k == "start" && v == "1"));
        assert!(params.iter().any(|(k, v)| k == "step" && v == "15"));
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
            "query=foo{tenant_id=\"other\"}",
            "id1",
        )
        .await
        .unwrap_err();
        assert!(matches!(e, ProxyError::UserSuppliedTenantId));
        assert!(r.last().is_none());
        assert!(s.latest("obs.cross_tenant_query_attempt").is_some());
    }

    #[tokio::test]
    async fn root_admin_query_forwarded_unfiltered() {
        let (r, s) = (Recording::new(), RecordingSink::default());
        handle(
            &auth(),
            &r,
            &s,
            Some(&jwt(NIL_TENANT)),
            "/api/v1/query",
            "query=rate(foo[5m])",
            "id1",
        )
        .await
        .unwrap();
        assert!(!r.forwarded_query().unwrap().contains("tenant_id"));
        assert_eq!(
            s.latest("obs.query_proxied").unwrap().payload["outcome"],
            "root_admin_unfiltered"
        );
    }

    #[tokio::test]
    async fn backend_unreachable_propagates() {
        let (r, s) = (Recording::failing(), RecordingSink::default());
        let e = handle(
            &auth(),
            &r,
            &s,
            Some(&jwt("T")),
            "/loki/api/v1/query",
            "query={service=\"x\"}",
            "id1",
        )
        .await
        .unwrap_err();
        assert!(matches!(e, ProxyError::BackendUnreachable(_)));
    }
}
