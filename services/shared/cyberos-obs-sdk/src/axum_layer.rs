//! Axum/tower RED instrumentation layer.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::extract::MatchedPath;
use http::{HeaderMap, Request, Response};
use tower::{Layer, Service};

use crate::red;

/// Tower layer that records RED metrics for every routed axum request.
#[derive(Clone, Debug)]
pub struct RedLayer {
    service: String,
    extra_labels: Vec<(String, String)>,
}

impl RedLayer {
    /// Create a RED layer for one service.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
            extra_labels: Vec::new(),
        }
    }

    /// Add a static custom dimension.
    pub fn with_extra_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_labels.push((key.into(), value.into()));
        self
    }
}

impl<S> Layer<S> for RedLayer {
    type Service = RedService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RedService {
            inner,
            service: self.service.clone(),
            extra_labels: self.extra_labels.clone(),
        }
    }
}

/// Service produced by [`RedLayer`].
#[derive(Clone, Debug)]
pub struct RedService<S> {
    inner: S,
    service: String,
    extra_labels: Vec<(String, String)>,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for RedService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
    ReqBody: Send + 'static,
    ResBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let started = Instant::now();
        let route = route_label(&req);
        let tenant_id = tenant_id(req.headers());
        let service = self.service.clone();
        let extra_labels = self.extra_labels.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            let result = future.await;
            let status = result
                .as_ref()
                .map(|response| response.status().as_u16())
                .unwrap_or(500);
            let duration_ms = started.elapsed().as_millis().min(u32::MAX as u128) as u32;
            let extra: Vec<(&str, String)> = extra_labels
                .iter()
                .map(|(key, value)| (key.as_str(), value.clone()))
                .collect();
            red::record_request(&service, &route, &tenant_id, status, duration_ms, &extra);
            result
        })
    }
}

fn route_label<B>(req: &Request<B>) -> String {
    req.extensions()
        .get::<MatchedPath>()
        .map(|path| path.as_str().to_string())
        .unwrap_or_else(|| req.uri().path().to_string())
}

fn tenant_id(headers: &HeaderMap) -> String {
    for name in ["x-cyberos-tenant-id", "x-tenant-id", "cyberos-tenant-id"] {
        if let Some(value) = headers.get(name).and_then(|value| value.to_str().ok()) {
            if !value.trim().is_empty() {
                return value.to_string();
            }
        }
    }
    "unknown".to_string()
}
