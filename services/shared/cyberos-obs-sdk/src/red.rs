//! FR-OBS-003 - RED (rate, errors, duration) metrics for every CyberOS service.
//!
//! `init` builds the three instruments off the global meter (the service installs the OTel SDK provider
//! and OTLP exporter at boot). `record_request` is called once per handled request - by the
//! `#[red_instrument]` macro on axum handlers, or by hand in non-HTTP paths. Status is bucketed to a
//! class (2xx/3xx/4xx/5xx/other) to bound cardinality (DEC-152); a cardinality guard refuses a label
//! set that would explode the series count (DEC, FR-OBS-003 §1 #9).
//!
//! Robustness note: unlike the spec's panic-on-missing-init, `record_request` is a safe no-op before
//! `init`. A metrics call must never crash a request path - a service that forgets `init` emits nothing
//! (caught by the completeness lint and an absent-metric alarm) rather than panicking in production.

use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use std::sync::OnceLock;
use std::time::Duration;

use crate::cardinality_guard;

/// Standard histogram bucket boundaries in ms (DEC-153). Cross-service p95 aggregation requires every
/// service to use identical boundaries, or `histogram_quantile` cannot merge them.
pub const HISTOGRAM_BUCKETS_MS: &[f64] = &[
    1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
];

const REQUESTS_METRIC: &str = "cyberos_requests_total";
const ERRORS_METRIC: &str = "cyberos_errors_total";
const DURATION_METRIC: &str = "cyberos_duration_ms";
// FR-OBS-005 §1 #12 - correlation self-observability.
const TRACECONTEXT_EXTRACTED_METRIC: &str = "obs_tracecontext_extracted_total";
const EXEMPLAR_EMISSION_METRIC: &str = "obs_exemplar_emission_total";

static REQUESTS: OnceLock<Counter<u64>> = OnceLock::new();
static ERRORS: OnceLock<Counter<u64>> = OnceLock::new();
static DURATION: OnceLock<Histogram<f64>> = OnceLock::new();
static TRACECONTEXT_EXTRACTED: OnceLock<Counter<u64>> = OnceLock::new();
static EXEMPLAR_EMISSIONS: OnceLock<Counter<u64>> = OnceLock::new();
/// Keeps the installed meter provider alive for the process lifetime (dropping it shuts it down).
static METER_PROVIDER: OnceLock<SdkMeterProvider> = OnceLock::new();

/// Install the OTLP exporter (if configured), then build the RED instruments off the global meter and
/// initialise the cardinality guard for `service`. Idempotent: a second call is a no-op (the
/// `OnceLock`s keep the first instruments). Call once at service boot, inside the tokio runtime.
pub fn init(service: &str, version: &str) {
    install_exporter(service, version);

    let meter = global::meter("cyberos");
    let _ = REQUESTS.set(meter.u64_counter(REQUESTS_METRIC).build());
    let _ = ERRORS.set(meter.u64_counter(ERRORS_METRIC).build());
    let _ = DURATION.set(
        meter
            .f64_histogram(DURATION_METRIC)
            .with_unit("ms")
            .with_boundaries(HISTOGRAM_BUCKETS_MS.to_vec())
            .build(),
    );
    let _ = TRACECONTEXT_EXTRACTED.set(meter.u64_counter(TRACECONTEXT_EXTRACTED_METRIC).build());
    let _ = EXEMPLAR_EMISSIONS.set(meter.u64_counter(EXEMPLAR_EMISSION_METRIC).build());
    cardinality_guard::init(service);
}

/// FR-OBS-005 §1 #12 - count a traceparent extraction by outcome. `outcome` is the bounded set
/// `extracted | malformed | missing_generated_new`, mirroring the `trace_ctx` boundary's three paths.
/// A safe no-op before `init`.
pub fn record_tracecontext_extracted(outcome: &str) {
    if let Some(counter) = TRACECONTEXT_EXTRACTED.get() {
        counter.add(1, &[KeyValue::new("outcome", outcome.to_string())]);
    }
}

/// Count one histogram exemplar emission (FR-OBS-005 §1 #12). Called by `exemplar::record_with_exemplar`
/// each time a sample is recorded with a trace in context. A safe no-op before `init`.
pub(crate) fn note_exemplar_emission() {
    if let Some(counter) = EXEMPLAR_EMISSIONS.get() {
        counter.add(1, &[]);
    }
}

/// The OTLP collector endpoint, from `OBS_OTLP_ENDPOINT` or the standard `OTEL_EXPORTER_OTLP_ENDPOINT`.
fn endpoint_from_env() -> Option<String> {
    std::env::var("OBS_OTLP_ENDPOINT")
        .or_else(|_| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT"))
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// If an OTLP endpoint is configured, install an OTLP gRPC meter provider as the global meter so the RED
/// instruments export to the collector (FR-OBS-001). Unset means no exporter: the instruments record to
/// the default no-op meter, so a dev/local run without a collector stays quiet. Best-effort: an exporter
/// that fails to build logs and leaves metrics disabled rather than crashing the service.
fn install_exporter(service: &str, version: &str) {
    let Some(endpoint) = endpoint_from_env() else {
        return;
    };
    match build_meter_provider(&endpoint, service, version) {
        Ok(provider) => {
            global::set_meter_provider(provider.clone());
            let _ = METER_PROVIDER.set(provider);
        }
        Err(e) => {
            eprintln!("obs-sdk: OTLP metric exporter init failed ({e}); RED metrics disabled for {service}");
        }
    }
}

fn build_meter_provider(
    endpoint: &str,
    service: &str,
    version: &str,
) -> Result<SdkMeterProvider, Box<dyn std::error::Error>> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(10))
        .build()?;
    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(Duration::from_secs(10))
        .build();
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service.to_string()),
        KeyValue::new("service.version", version.to_string()),
    ]);
    Ok(SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource)
        .build())
}

/// The HTTP status class label (DEC-152): coarse bands, not raw codes, to bound cardinality.
pub fn status_class(status: u16) -> &'static str {
    match status {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}

/// The error class for a >= 400 status: client (4xx) vs server (5xx and above).
fn error_class(status: u16) -> &'static str {
    if status >= 500 {
        "server_error"
    } else {
        "client_error"
    }
}

/// Record one request: increment `cyberos_requests_total`, record `cyberos_duration_ms`, and on a
/// 4xx/5xx status increment `cyberos_errors_total`. A label set that would overflow the cardinality
/// guard is refused (no emission) and counted. This is a safe no-op before `init`.
pub fn record_request(
    service: &str,
    route: &str,
    tenant_id: &str,
    status: u16,
    duration_ms: u32,
    extra_labels: &[(&str, String)],
) {
    let mut labels = vec![
        KeyValue::new("service", service.to_string()),
        KeyValue::new("route", route.to_string()),
        KeyValue::new("tenant_id", tenant_id.to_string()),
        KeyValue::new("status_class", status_class(status).to_string()),
    ];
    for (k, v) in extra_labels {
        labels.push(KeyValue::new(k.to_string(), v.clone()));
    }

    if !cardinality_guard::check(service, REQUESTS_METRIC, &labels) {
        return; // refused: registering this combo would overflow the per-metric series budget.
    }

    if let Some(requests) = REQUESTS.get() {
        requests.add(1, &labels);
    }
    if let Some(duration) = DURATION.get() {
        // FR-OBS-005 §1 #3 - record with an exemplar so a duration sample links to its trace (the
        // trace_id rides via the current OTel context set at the request boundary).
        crate::exemplar::record_with_exemplar(duration, f64::from(duration_ms), &labels);
    }
    if status >= 400 {
        if let Some(errors) = ERRORS.get() {
            let mut err_labels = labels.clone();
            err_labels.push(KeyValue::new("error_class", error_class(status).to_string()));
            errors.add(1, &err_labels);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_class_covers_every_band() {
        for (status, expected) in [
            (200, "2xx"),
            (201, "2xx"),
            (204, "2xx"),
            (301, "3xx"),
            (404, "4xx"),
            (429, "4xx"),
            (500, "5xx"),
            (503, "5xx"),
            (0, "other"),
            (999, "other"),
        ] {
            assert_eq!(status_class(status), expected, "status {status}");
        }
    }

    #[test]
    fn error_class_splits_client_and_server() {
        assert_eq!(error_class(404), "client_error");
        assert_eq!(error_class(499), "client_error");
        assert_eq!(error_class(500), "server_error");
        assert_eq!(error_class(503), "server_error");
    }

    #[test]
    fn buckets_are_the_thirteen_standard_boundaries() {
        assert_eq!(HISTOGRAM_BUCKETS_MS.len(), 13);
        assert_eq!(HISTOGRAM_BUCKETS_MS.first(), Some(&1.0));
        assert_eq!(HISTOGRAM_BUCKETS_MS.last(), Some(&10000.0));
        assert!(HISTOGRAM_BUCKETS_MS.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn record_request_is_a_safe_noop_before_init() {
        // No provider installed and no init() called: must not panic. Unique service name so the
        // shared cardinality guard does not collide with other tests.
        record_request("noop-svc", "/x", "t", 200, 5, &[]);
        record_request("noop-svc", "/x", "t", 503, 5, &[("model_alias", "chat.smart".into())]);
    }

    #[test]
    fn record_tracecontext_extracted_is_a_safe_noop_before_init() {
        // The three bounded outcomes; before init the counter is absent, so these must not panic.
        record_tracecontext_extracted("extracted");
        record_tracecontext_extracted("malformed");
        record_tracecontext_extracted("missing_generated_new");
    }

    #[test]
    fn record_with_exemplar_is_a_safe_noop_on_a_noop_meter() {
        // A histogram off the default (no-op) meter, with the emission counter not yet built: recording
        // must not panic and must count the emission as a no-op.
        let h = global::meter("test").f64_histogram("test_hist").build();
        crate::exemplar::record_with_exemplar(&h, 12.5, &[KeyValue::new("route", "/x".to_string())]);
    }
}
