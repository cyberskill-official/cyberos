//! TASK-AI-022 §1 #1 — OTel SDK initialisation with OTLP gRPC exporter.

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace::TracerProvider, Resource};

/// Initialise the OTel SDK with an OTLP gRPC exporter.
///
/// # Errors
/// Returns `OtelInitError` if the exporter setup fails.
pub fn init_otel(endpoint: &str) -> Result<TracerProvider, OtelInitError> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| OtelInitError::Exporter(e.to_string()))?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(Resource::new(vec![
            KeyValue::new("service.name", "ai-gateway"),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ]))
        .build();

    global::set_tracer_provider(provider.clone());
    Ok(provider)
}

#[derive(Debug, thiserror::Error)]
pub enum OtelInitError {
    #[error("OTLP exporter setup failed: {0}")]
    Exporter(String),
    #[error("collector unreachable at {0} during boot health-check")]
    CollectorUnreachable(String),
}
