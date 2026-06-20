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
use std::sync::OnceLock;

use crate::cardinality_guard;

/// Standard histogram bucket boundaries in ms (DEC-153). Cross-service p95 aggregation requires every
/// service to use identical boundaries, or `histogram_quantile` cannot merge them.
pub const HISTOGRAM_BUCKETS_MS: &[f64] = &[
    1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
];

const REQUESTS_METRIC: &str = "cyberos_requests_total";
const ERRORS_METRIC: &str = "cyberos_errors_total";
const DURATION_METRIC: &str = "cyberos_duration_ms";

static REQUESTS: OnceLock<Counter<u64>> = OnceLock::new();
static ERRORS: OnceLock<Counter<u64>> = OnceLock::new();
static DURATION: OnceLock<Histogram<f64>> = OnceLock::new();

/// Build the RED instruments off the global meter and initialise the cardinality guard for `service`.
/// Idempotent: a second call is a no-op (the `OnceLock`s keep the first instruments).
pub fn init(service: &str, _version: &str) {
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
    cardinality_guard::init(service);
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
        duration.record(f64::from(duration_ms), &labels);
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
}
