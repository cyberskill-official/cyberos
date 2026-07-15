//! TASK-AI-022 §1 #2 — W3C TraceContext extraction (incoming) + injection (outgoing).

use opentelemetry::propagation::{Extractor, Injector, TextMapPropagator};
use opentelemetry_sdk::propagation::TraceContextPropagator;

/// Adapter to extract trace context from HTTP headers.
#[derive(Debug)]
pub struct HeaderExtractor<'a>(pub &'a http::HeaderMap);

impl<'a> Extractor for HeaderExtractor<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

/// Adapter to inject trace context into HTTP headers.
#[derive(Debug)]
pub struct HeaderInjector<'a>(pub &'a mut http::HeaderMap);

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(name), Ok(val)) = (
            http::HeaderName::from_bytes(key.as_bytes()),
            http::HeaderValue::from_str(&value),
        ) {
            self.0.insert(name, val);
        }
    }
}

/// Extract W3C TraceContext from incoming HTTP headers.
pub fn extract_context_from_headers(headers: &http::HeaderMap) -> opentelemetry::Context {
    let propagator = TraceContextPropagator::new();
    propagator.extract(&HeaderExtractor(headers))
}

/// Inject W3C TraceContext into outgoing HTTP headers.
pub fn inject_context_into_headers(ctx: &opentelemetry::Context, headers: &mut http::HeaderMap) {
    let propagator = TraceContextPropagator::new();
    propagator.inject_context(ctx, &mut HeaderInjector(headers));
}
