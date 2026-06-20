//! FR-OBS-003 - the axum RED middleware (ADR-OBS-003-001). One `.layer(...)` per service router
//! instruments every route: it reads the matched route template, times the request, reads the request's
//! tenant from a `TenantCtx`, and calls `red::record_request` on the way out. This is the per-service
//! instrumentation point that replaces the spec's per-handler proc-macro - one touch point, and it
//! cannot miss a route.
//!
//! Wiring, per service: at boot call `red::init("<service>", version)`, and on the router add
//! `.layer(axum::middleware::from_fn_with_state(RedState::new("<service>"), red_mw))`. The service's auth
//! middleware sets `TenantCtx` from its own claims so `tenant_id` is real; absent it (for example on
//! `/healthz`), the tenant label is `"unknown"`.

use std::time::Instant;

use axum::extract::{MatchedPath, Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::red;

/// Extension a service sets to declare the tenant for the current request. The service's auth middleware
/// inserts it (mapping from its own claims). Absent, the tenant label is `"unknown"`.
#[derive(Clone, Debug)]
pub struct TenantCtx(pub String);

/// Per-service state for the RED middleware: the service name carried into every metric.
#[derive(Clone, Debug)]
pub struct RedState {
    service: &'static str,
}

impl RedState {
    pub fn new(service: &'static str) -> Self {
        Self { service }
    }
}

/// The route and tenant labels for a request: the matched route template (or `"unmatched"`), and the
/// tenant from `TenantCtx` (or `"unknown"`). Pure, so it is unit-tested directly.
fn labels<'a>(matched_route: Option<&'a str>, tenant: Option<&'a str>) -> (&'a str, &'a str) {
    (
        matched_route.unwrap_or("unmatched"),
        tenant.unwrap_or("unknown"),
    )
}

/// The RED middleware. Wire it via
/// `axum::middleware::from_fn_with_state(RedState::new("<service>"), red_mw)`.
pub async fn red_mw(
    State(state): State<RedState>,
    matched: Option<MatchedPath>,
    req: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let route = matched.as_ref().map(|m| m.as_str().to_string());

    let response = next.run(req).await;

    // An inner layer (the service's auth middleware) may set TenantCtx on the response extensions.
    let tenant = response.extensions().get::<TenantCtx>().map(|t| t.0.clone());
    let (route_label, tenant_label) = labels(route.as_deref(), tenant.as_deref());
    let status = response.status().as_u16();
    let duration_ms = u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX);

    red::record_request(state.service, route_label, tenant_label, status, duration_ms, &[]);
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    #[test]
    fn labels_default_when_absent() {
        assert_eq!(labels(None, None), ("unmatched", "unknown"));
        assert_eq!(labels(Some("/v1/x"), Some("t1")), ("/v1/x", "t1"));
        assert_eq!(labels(Some("/v1/x"), None), ("/v1/x", "unknown"));
    }

    async fn ok() -> &'static str {
        "ok"
    }

    /// A handler that declares a tenant by inserting TenantCtx into the response.
    async fn ok_with_tenant() -> Response {
        let mut r = Response::new(Body::from("ok"));
        r.extensions_mut().insert(TenantCtx("org:test".into()));
        r
    }

    fn app() -> Router {
        Router::new()
            .route("/v1/ping", get(ok))
            .route("/v1/tenant", get(ok_with_tenant))
            .layer(axum::middleware::from_fn_with_state(
                RedState::new("test-svc"),
                red_mw,
            ))
    }

    #[tokio::test]
    async fn middleware_is_transparent_to_the_response() {
        let res = app()
            .oneshot(Request::builder().uri("/v1/ping").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }

    #[tokio::test]
    async fn middleware_runs_for_a_tenant_route_without_panicking() {
        let res = app()
            .oneshot(Request::builder().uri("/v1/tenant").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
    }

    #[tokio::test]
    async fn unmatched_route_still_passes_through() {
        let res = app()
            .oneshot(Request::builder().uri("/nope").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 404);
    }
}
