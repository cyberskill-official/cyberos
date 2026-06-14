//! Logging helpers for carrying OBS context through structured logs.

use tracing::{Event, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

use crate::red;

/// `tracing-subscriber` layer that records log-enrichment coverage.
#[derive(Clone, Debug)]
pub struct ObsContextLayer {
    service: String,
}

impl ObsContextLayer {
    /// Create a log-enrichment coverage layer for one service.
    pub fn new(service: impl Into<String>) -> Self {
        Self {
            service: service.into(),
        }
    }
}

impl<S> Layer<S> for ObsContextLayer
where
    S: Subscriber,
{
    fn on_event(&self, _event: &Event<'_>, _ctx: Context<'_, S>) {
        red::record_log_enrichment(&self.service);
    }
}

/// Build the canonical request span whose fields are inherited by request logs.
pub fn request_span(
    service: &str,
    route: &str,
    tenant_id: &str,
    trace_id: &str,
    span_id: &str,
) -> tracing::Span {
    tracing::info_span!(
        "obs.request",
        service = %service,
        route = %route,
        tenant_id = %tenant_id,
        trace_id = %trace_id,
        span_id = %span_id,
    )
}
