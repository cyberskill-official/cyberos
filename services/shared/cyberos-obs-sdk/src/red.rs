//! RED request metrics.

use std::collections::BTreeMap;
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{LazyLock, Mutex, OnceLock};
use std::time::{Duration, Instant};

use opentelemetry::metrics::{Counter, Histogram};
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::metrics::{PeriodicReader, SdkMeterProvider};
use opentelemetry_sdk::{runtime, Resource};
use thiserror::Error;
use tonic::metadata::{MetadataMap, MetadataValue};

use crate::cardinality_guard;
use crate::exemplar::HistogramExemplar;
use crate::tracecontext;

pub use crate::axum_layer::RedLayer;

/// RED request counter metric.
pub const REQUESTS_TOTAL: &str = "cyberos_requests_total";
/// RED error counter metric.
pub const ERRORS_TOTAL: &str = "cyberos_errors_total";
/// RED duration histogram metric.
pub const DURATION_MS: &str = "cyberos_duration_ms";
/// SDK self-metric for calls.
pub const SDK_RECORD_CALLS_TOTAL: &str = "obs_sdk_record_calls_total";
/// SDK self-metric for recording latency.
pub const SDK_RECORD_LATENCY_NS: &str = "obs_sdk_record_latency_ns";
/// SDK self-metric for cardinality refusals.
pub const SDK_CARDINALITY_BLOCKED_TOTAL: &str = "obs_sdk_cardinality_blocked_total";
/// TraceContext extraction self-metric.
pub const TRACECONTEXT_EXTRACTED_TOTAL: &str = "obs_tracecontext_extracted_total";
/// Log-enrichment coverage self-metric.
pub const LOG_ENRICHMENT_TOTAL: &str = "obs_log_enrichment_total";
/// Exemplar emission self-metric.
pub const EXEMPLAR_EMISSION_TOTAL: &str = "obs_exemplar_emission_total";

/// Standard RED duration histogram buckets in milliseconds.
pub const STANDARD_BUCKETS_MS: &[f64] = &[
    1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10000.0,
];

/// Target hot-path budget from FR-OBS-003.
pub const RECORD_OVERHEAD_TARGET_NS: u64 = 100;

static SDK: OnceLock<SdkState> = OnceLock::new();
static CAPTURE_FOR_TESTS: AtomicBool = AtomicBool::new(false);
static SNAPSHOT: LazyLock<Mutex<MetricSnapshot>> =
    LazyLock::new(|| Mutex::new(MetricSnapshot::default()));

#[derive(Debug)]
struct SdkState {
    requests: Counter<u64>,
    errors: Counter<u64>,
    duration: Histogram<f64>,
    record_calls: Counter<u64>,
    record_latency: Histogram<u64>,
    cardinality_blocked: Counter<u64>,
    tracecontext_extracted: Counter<u64>,
    log_enrichment: Counter<u64>,
    exemplar_emission: Counter<u64>,
    _provider: SdkMeterProvider,
}

/// Errors returned by [`init`].
#[derive(Debug, Error)]
pub enum InitError {
    /// OTLP exporter setup failed.
    #[error("OTLP metrics exporter setup failed: {0}")]
    Exporter(String),
    /// Bearer-token metadata was malformed.
    #[error("OTLP metrics metadata setup failed: {0}")]
    Metadata(String),
}

/// Metric label used by the SDK and tests.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Label {
    /// Label key.
    pub key: String,
    /// Label value.
    pub value: String,
}

impl Label {
    /// Construct a label.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Outcome returned by [`record_request`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordOutcome {
    /// True when the request and duration metrics were accepted.
    pub recorded: bool,
    /// First metric refused by the cardinality guard, if any.
    pub blocked_metric: Option<String>,
    /// Derived status class.
    pub status_class: &'static str,
    /// Derived error class for 4xx/5xx statuses.
    pub error_class: Option<&'static str>,
}

/// In-process metric snapshot used by deterministic tests.
#[derive(Clone, Debug, Default)]
pub struct MetricSnapshot {
    counters: BTreeMap<SeriesKey, u64>,
    histograms: BTreeMap<SeriesKey, Vec<u64>>,
    exemplars: BTreeMap<SeriesKey, Vec<HistogramExemplar>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct SeriesKey {
    metric: String,
    labels: Vec<(String, String)>,
}

impl MetricSnapshot {
    /// Read a captured counter value.
    pub fn counter_value(&self, metric: &str, labels: &[(&str, &str)]) -> u64 {
        let key = SeriesKey::new(metric, labels);
        self.counters.get(&key).copied().unwrap_or(0)
    }

    /// Read a captured histogram bucket value.
    pub fn histogram_bucket_value(&self, metric: &str, labels: &[(&str, &str)], le: f64) -> u64 {
        let key = SeriesKey::new(metric, labels);
        let Some(values) = self.histograms.get(&key) else {
            return 0;
        };
        let Some(index) = STANDARD_BUCKETS_MS
            .iter()
            .position(|bucket| (*bucket - le).abs() < f64::EPSILON)
        else {
            return 0;
        };
        values.get(index).copied().unwrap_or(0)
    }

    /// Read captured histogram exemplars for an exact metric series.
    pub fn histogram_exemplars(
        &self,
        metric: &str,
        labels: &[(&str, &str)],
    ) -> Vec<HistogramExemplar> {
        let key = SeriesKey::new(metric, labels);
        self.exemplars.get(&key).cloned().unwrap_or_default()
    }
}

impl SeriesKey {
    fn from_labels(metric: &str, labels: &[Label]) -> Self {
        let mut pairs: Vec<_> = labels
            .iter()
            .map(|label| (label.key.clone(), label.value.clone()))
            .collect();
        pairs.sort_unstable();
        Self {
            metric: metric.to_string(),
            labels: pairs,
        }
    }

    fn new(metric: &str, labels: &[(&str, &str)]) -> Self {
        let mut pairs: Vec<_> = labels
            .iter()
            .map(|(key, value)| ((*key).to_string(), (*value).to_string()))
            .collect();
        pairs.sort_unstable();
        Self {
            metric: metric.to_string(),
            labels: pairs,
        }
    }
}

/// Initialise the SDK's OTel meter provider and instruments.
pub fn init(service_name: &str, version: &str) -> Result<(), InitError> {
    if SDK.get().is_some() {
        tracing::info!(service = service_name, version, "obs_sdk_initialized");
        return Ok(());
    }

    let provider = build_provider(service_name, version)?;
    global::set_meter_provider(provider.clone());
    let meter = global::meter("cyberos.obs_sdk");
    let state = SdkState {
        requests: meter.u64_counter(REQUESTS_TOTAL).build(),
        errors: meter.u64_counter(ERRORS_TOTAL).build(),
        duration: meter
            .f64_histogram(DURATION_MS)
            .with_unit("ms")
            .with_boundaries(STANDARD_BUCKETS_MS.to_vec())
            .build(),
        record_calls: meter.u64_counter(SDK_RECORD_CALLS_TOTAL).build(),
        record_latency: meter
            .u64_histogram(SDK_RECORD_LATENCY_NS)
            .with_unit("ns")
            .build(),
        cardinality_blocked: meter.u64_counter(SDK_CARDINALITY_BLOCKED_TOTAL).build(),
        tracecontext_extracted: meter.u64_counter(TRACECONTEXT_EXTRACTED_TOTAL).build(),
        log_enrichment: meter.u64_counter(LOG_ENRICHMENT_TOTAL).build(),
        exemplar_emission: meter.u64_counter(EXEMPLAR_EMISSION_TOTAL).build(),
        _provider: provider,
    };
    let _ = SDK.set(state);
    tracing::info!(service = service_name, version, "obs_sdk_initialized");
    Ok(())
}

/// Record one request's RED metrics.
pub fn record_request(
    service: &str,
    route: &str,
    tenant_id: &str,
    status: u16,
    duration_ms: u32,
    extra_labels: &[(&str, String)],
) -> RecordOutcome {
    record_request_with_trace(
        service,
        route,
        tenant_id,
        status,
        duration_ms,
        extra_labels,
        None,
    )
}

/// Record one request's RED metrics and attach a trace exemplar to duration.
pub fn record_request_with_trace(
    service: &str,
    route: &str,
    tenant_id: &str,
    status: u16,
    duration_ms: u32,
    extra_labels: &[(&str, String)],
    trace_id: Option<&str>,
) -> RecordOutcome {
    let started = Instant::now();
    let status_class = status_class(status);
    let error_class = error_class(status);
    let request_labels = request_labels(service, route, tenant_id, status_class, extra_labels);
    let duration_labels = duration_labels(service, route, tenant_id, extra_labels);

    let mut blocked_metric = None;
    if !cardinality_guard::check(service, REQUESTS_TOTAL, &request_labels) {
        blocked_metric = Some(REQUESTS_TOTAL.to_string());
    } else if !cardinality_guard::check(service, DURATION_MS, &duration_labels) {
        blocked_metric = Some(DURATION_MS.to_string());
    }

    if let Some(metric) = blocked_metric.clone() {
        emit_cardinality_blocked(service, &metric);
        observe_self_metrics(service, started);
        return RecordOutcome {
            recorded: false,
            blocked_metric: Some(metric),
            status_class,
            error_class,
        };
    }

    if let Some(sdk) = SDK.get() {
        let request_kv = to_key_values(&request_labels);
        let duration_kv = to_key_values(&duration_labels);
        sdk.requests.add(1, &request_kv);
        sdk.duration.record(duration_ms as f64, &duration_kv);
    }
    capture_counter(REQUESTS_TOTAL, &request_labels, 1);
    capture_histogram(DURATION_MS, &duration_labels, duration_ms as f64);
    if let Some(trace_id) = trace_id.filter(|value| tracecontext::is_valid_trace_id(value)) {
        capture_exemplar(
            DURATION_MS,
            &duration_labels,
            HistogramExemplar {
                trace_id: trace_id.to_ascii_lowercase(),
                value: duration_ms as f64,
            },
        );
        record_exemplar_emission(service);
    }

    if let Some(error_class) = error_class {
        let error_labels = error_labels(service, route, tenant_id, error_class, extra_labels);
        if !cardinality_guard::check(service, ERRORS_TOTAL, &error_labels) {
            emit_cardinality_blocked(service, ERRORS_TOTAL);
        } else {
            if let Some(sdk) = SDK.get() {
                sdk.errors.add(1, &to_key_values(&error_labels));
            }
            capture_counter(ERRORS_TOTAL, &error_labels, 1);
        }
    }

    observe_self_metrics(service, started);
    RecordOutcome {
        recorded: true,
        blocked_metric: None,
        status_class,
        error_class,
    }
}

/// Record how request TraceContext was established.
pub fn record_tracecontext_extraction(outcome: &str) {
    let labels = vec![Label::new("outcome", outcome)];
    if let Some(sdk) = SDK.get() {
        sdk.tracecontext_extracted.add(1, &to_key_values(&labels));
    }
    capture_counter(TRACECONTEXT_EXTRACTED_TOTAL, &labels, 1);
}

/// Record that a structured log event was emitted inside OBS context.
pub fn record_log_enrichment(service: &str) {
    let labels = vec![Label::new("service", service)];
    if let Some(sdk) = SDK.get() {
        sdk.log_enrichment.add(1, &to_key_values(&labels));
    }
    capture_counter(LOG_ENRICHMENT_TOTAL, &labels, 1);
}

fn record_exemplar_emission(service: &str) {
    let labels = vec![Label::new("service", service)];
    if let Some(sdk) = SDK.get() {
        sdk.exemplar_emission.add(1, &to_key_values(&labels));
    }
    capture_counter(EXEMPLAR_EMISSION_TOTAL, &labels, 1);
}

/// Derive the bounded status class label.
pub fn status_class(status: u16) -> &'static str {
    match status {
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "other",
    }
}

/// Generic status extractor used by the proc macro. Router-level middleware
/// records exact response status; the macro fallback is intentionally generic
/// so it preserves arbitrary handler signatures.
pub fn status_from_response_like<T>(_value: &T) -> u16 {
    200
}

/// Current test-capture snapshot.
pub fn snapshot() -> MetricSnapshot {
    SNAPSHOT.lock().expect("metric snapshot lock").clone()
}

/// Reset deterministic test state and enable capture.
pub fn reset_for_tests() {
    CAPTURE_FOR_TESTS.store(true, Ordering::SeqCst);
    SNAPSHOT.lock().expect("metric snapshot lock").clear();
    cardinality_guard::reset_for_tests();
}

fn build_provider(service_name: &str, version: &str) -> Result<SdkMeterProvider, InitError> {
    let mut builder = SdkMeterProvider::builder().with_resource(Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", version.to_string()),
    ]));

    if env::var("CYBEROS_OBS_SDK_DISABLE_OTLP").as_deref() != Ok("1") {
        let endpoint =
            env::var("CYBEROS_OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".into());
        let mut exporter = opentelemetry_otlp::MetricExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint)
            .with_timeout(Duration::from_secs(10));
        if let Some(token) = service_token(service_name) {
            let mut metadata = MetadataMap::new();
            let value = MetadataValue::try_from(format!("Bearer {token}"))
                .map_err(|e| InitError::Metadata(e.to_string()))?;
            metadata.insert("authorization", value);
            exporter = exporter.with_metadata(metadata);
        }
        let exporter = exporter
            .build()
            .map_err(|e| InitError::Exporter(e.to_string()))?;
        let reader = PeriodicReader::builder(exporter, runtime::Tokio)
            .with_interval(Duration::from_secs(5))
            .with_timeout(Duration::from_secs(10))
            .build();
        builder = builder.with_reader(reader);
    }

    Ok(builder.build())
}

fn service_token(service_name: &str) -> Option<String> {
    let specific = format!(
        "CYBEROS_OBS_TOKEN_{}",
        service_name
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            })
            .collect::<String>()
    );
    env::var(&specific)
        .ok()
        .or_else(|| env::var("CYBEROS_OBS_TOKEN").ok())
}

fn request_labels(
    service: &str,
    route: &str,
    tenant_id: &str,
    status_class: &str,
    extra_labels: &[(&str, String)],
) -> Vec<Label> {
    let mut labels = base_labels(service, route, tenant_id, extra_labels);
    labels.push(Label::new("status_class", status_class));
    labels
}

fn duration_labels(
    service: &str,
    route: &str,
    tenant_id: &str,
    extra_labels: &[(&str, String)],
) -> Vec<Label> {
    base_labels(service, route, tenant_id, extra_labels)
}

fn error_labels(
    service: &str,
    route: &str,
    tenant_id: &str,
    error_class: &str,
    extra_labels: &[(&str, String)],
) -> Vec<Label> {
    let mut labels = base_labels(service, route, tenant_id, extra_labels);
    labels.push(Label::new("error_class", error_class));
    labels
}

fn base_labels(
    service: &str,
    route: &str,
    tenant_id: &str,
    extra_labels: &[(&str, String)],
) -> Vec<Label> {
    let mut labels = vec![
        Label::new("service", service),
        Label::new("route", route),
        Label::new("tenant_id", tenant_id),
    ];
    labels.extend(
        extra_labels
            .iter()
            .map(|(key, value)| Label::new(*key, value.clone())),
    );
    labels
}

fn error_class(status: u16) -> Option<&'static str> {
    match status {
        400..=499 => Some("client_error"),
        500..=599 => Some("server_error"),
        _ => None,
    }
}

fn to_key_values(labels: &[Label]) -> Vec<KeyValue> {
    labels
        .iter()
        .map(|label| KeyValue::new(label.key.clone(), label.value.clone()))
        .collect()
}

fn emit_cardinality_blocked(service: &str, metric: &str) {
    let labels = vec![Label::new("service", service), Label::new("metric", metric)];
    if let Some(sdk) = SDK.get() {
        sdk.cardinality_blocked.add(1, &to_key_values(&labels));
    }
    capture_counter(SDK_CARDINALITY_BLOCKED_TOTAL, &labels, 1);
}

fn observe_self_metrics(service: &str, started: Instant) {
    let latency_ns = started.elapsed().as_nanos().min(u64::MAX as u128) as u64;
    let labels = vec![Label::new("service", service)];
    if let Some(sdk) = SDK.get() {
        let kv = to_key_values(&labels);
        sdk.record_calls.add(1, &kv);
        sdk.record_latency.record(latency_ns, &kv);
    }
    capture_counter(SDK_RECORD_CALLS_TOTAL, &labels, 1);
    capture_histogram(SDK_RECORD_LATENCY_NS, &labels, latency_ns as f64);
}

fn capture_counter(metric: &str, labels: &[Label], value: u64) {
    if !capture_enabled() {
        return;
    }
    let mut snapshot = SNAPSHOT.lock().expect("metric snapshot lock");
    *snapshot
        .counters
        .entry(SeriesKey::from_labels(metric, labels))
        .or_insert(0) += value;
}

fn capture_histogram(metric: &str, labels: &[Label], value: f64) {
    if !capture_enabled() {
        return;
    }
    let mut snapshot = SNAPSHOT.lock().expect("metric snapshot lock");
    let buckets = snapshot
        .histograms
        .entry(SeriesKey::from_labels(metric, labels))
        .or_insert_with(|| vec![0; STANDARD_BUCKETS_MS.len()]);
    for (index, boundary) in STANDARD_BUCKETS_MS.iter().enumerate() {
        if value <= *boundary {
            buckets[index] += 1;
        }
    }
}

fn capture_exemplar(metric: &str, labels: &[Label], exemplar: HistogramExemplar) {
    if !capture_enabled() {
        return;
    }
    let mut snapshot = SNAPSHOT.lock().expect("metric snapshot lock");
    snapshot
        .exemplars
        .entry(SeriesKey::from_labels(metric, labels))
        .or_default()
        .push(exemplar);
}

fn capture_enabled() -> bool {
    CAPTURE_FOR_TESTS.load(Ordering::SeqCst)
        || env::var("CYBEROS_OBS_SDK_CAPTURE").as_deref() == Ok("1")
}

impl MetricSnapshot {
    fn clear(&mut self) {
        self.counters.clear();
        self.histograms.clear();
        self.exemplars.clear();
    }
}
