//! FR-OBS-001 §3 — Validate the otelcol-contrib YAML config matches the contract.
//!
//! The validation runs at supervisor startup. The collector is rejected from starting
//! if the receivers / processors / exporters / pipelines do not match the required
//! shape. This catches deployment misconfiguration at boot rather than after the first
//! dropped span.

use std::path::Path;

use serde::Deserialize;
use thiserror::Error;

/// Errors detected by [`validate`].
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Failed to read the config file.
    #[error("read: {0}")]
    Read(#[from] std::io::Error),
    /// YAML did not parse.
    #[error("parse: {0}")]
    Parse(#[from] serde_yaml::Error),
    /// Config parsed but did not satisfy the FR-OBS-001 §3 contract.
    #[error("validation: {0}")]
    Validation(String),
}

/// FR-OBS-001 §3 — Validate the config matches the slice-1 pipeline contract.
pub fn validate(path: &Path) -> Result<(), ConfigError> {
    let raw = std::fs::read_to_string(path)?;
    let cfg: CollectorConfig = serde_yaml::from_str(&raw)?;

    // FR-OBS-001 §3 — required receivers/processors/exporters.
    if cfg.receivers.otlp.is_none() {
        return Err(ConfigError::Validation(
            "missing receiver: otlp (FR-OBS-001 §3)".into(),
        ));
    }
    if cfg.exporters.loki.is_none() {
        return Err(ConfigError::Validation(
            "missing exporter: loki (FR-OBS-001 §3)".into(),
        ));
    }
    if cfg.exporters.prometheusremotewrite.is_none() {
        return Err(ConfigError::Validation(
            "missing exporter: prometheusremotewrite (FR-OBS-001 §3)".into(),
        ));
    }
    if cfg.exporters.otlp_tempo.is_none() {
        return Err(ConfigError::Validation(
            "missing exporter: otlp/tempo (FR-OBS-001 §3)".into(),
        ));
    }
    let tail_sampling = cfg.processors.tail_sampling.as_ref().ok_or_else(|| {
        ConfigError::Validation("missing processor: tail_sampling (FR-OBS-006 §1)".into())
    })?;

    // FR-OBS-001 §1 #11 — pii_scrub processor MUST be present on logs+traces pipelines.
    let pipelines = &cfg.service.pipelines;
    for (signal, p) in [("logs", &pipelines.logs), ("traces", &pipelines.traces)] {
        if !p
            .processors
            .iter()
            .any(|s: &String| s.contains("pii_scrub"))
        {
            return Err(ConfigError::Validation(format!(
                "pipeline {signal}: missing pii_scrub processor (FR-OBS-001 §1 #11)"
            )));
        }
    }
    validate_tail_sampling_pipeline(pipelines, tail_sampling)?;

    // FR-OBS-001 §1 #2 — bearertokenauth extension MUST be configured.
    if cfg.extensions.bearertokenauth.is_none() {
        return Err(ConfigError::Validation(
            "missing extension: bearertokenauth (FR-OBS-001 §1 #2)".into(),
        ));
    }

    // FR-OBS-001 §1 #7 — file_storage extension MUST be configured.
    if cfg.extensions.file_storage.is_none() {
        return Err(ConfigError::Validation(
            "missing extension: file_storage (FR-OBS-001 §1 #7)".into(),
        ));
    }

    // The local stack pins otelcol-contrib 0.110.0. Keep a small guard for
    // fields that passed our shape check but are rejected by that collector.
    if has_mapping_key(cfg.exporters.loki.as_ref(), "labels") {
        return Err(ConfigError::Validation(
            "exporter loki: labels is not supported by otelcol-contrib 0.110.0".into(),
        ));
    }
    if has_mapping_key(
        cfg.exporters.prometheusremotewrite.as_ref(),
        "sending_queue",
    ) {
        return Err(ConfigError::Validation(
            "exporter prometheusremotewrite: sending_queue is not supported by otelcol-contrib 0.110.0"
                .into(),
        ));
    }
    if !mapping_bool(cfg.extensions.file_storage.as_ref(), "create_directory").unwrap_or(false) {
        return Err(ConfigError::Validation(
            "extension file_storage: create_directory must be true".into(),
        ));
    }

    Ok(())
}

fn has_mapping_key(value: Option<&serde_yaml::Value>, key: &str) -> bool {
    value
        .and_then(serde_yaml::Value::as_mapping)
        .is_some_and(|m| m.contains_key(serde_yaml::Value::String(key.to_string())))
}

fn mapping_bool(value: Option<&serde_yaml::Value>, key: &str) -> Option<bool> {
    value
        .and_then(serde_yaml::Value::as_mapping)
        .and_then(|m| m.get(serde_yaml::Value::String(key.to_string())))
        .and_then(serde_yaml::Value::as_bool)
}

fn validate_tail_sampling_pipeline(
    pipelines: &Pipelines,
    tail_sampling: &serde_yaml::Value,
) -> Result<(), ConfigError> {
    if pipelines
        .logs
        .processors
        .iter()
        .any(|processor| processor == "tail_sampling")
        || pipelines
            .metrics
            .processors
            .iter()
            .any(|processor| processor == "tail_sampling")
    {
        return Err(ConfigError::Validation(
            "tail_sampling processor must be traces-only (FR-OBS-006 §1 #10)".into(),
        ));
    }

    let traces = &pipelines.traces.processors;
    let pii = traces
        .iter()
        .position(|processor| processor.contains("pii_scrub"));
    let tail = traces
        .iter()
        .position(|processor| processor == "tail_sampling");
    let batch = traces.iter().position(|processor| processor == "batch");
    match (pii, tail, batch) {
        (Some(pii), Some(tail), Some(batch)) if pii < tail && tail < batch => {}
        _ => {
            return Err(ConfigError::Validation(
                "traces pipeline must order attributes/pii_scrub before tail_sampling before batch"
                    .into(),
            ));
        }
    }

    let decision_wait = mapping_string(Some(tail_sampling), "decision_wait")
        .ok_or_else(|| ConfigError::Validation("tail_sampling missing decision_wait".into()))?;
    let decision_wait_seconds = parse_duration_seconds(&decision_wait).ok_or_else(|| {
        ConfigError::Validation("tail_sampling decision_wait must be duration seconds".into())
    })?;
    if decision_wait_seconds < 20 {
        return Err(ConfigError::Validation(
            "tail_sampling decision_wait must be at least 20s".into(),
        ));
    }

    if mapping_i64(Some(tail_sampling), "num_traces").unwrap_or_default() < 100_000 {
        return Err(ConfigError::Validation(
            "tail_sampling num_traces must be at least 100000".into(),
        ));
    }

    let policies = mapping_seq(Some(tail_sampling), "policies")
        .ok_or_else(|| ConfigError::Validation("tail_sampling missing policies".into()))?;
    require_tail_policy(policies, "status_code")?;
    require_http_5xx_policy(policies)?;
    require_tail_policy(policies, "string_attribute")?;
    require_tail_policy(policies, "latency")?;
    require_probabilistic_10_policy(policies)?;
    Ok(())
}

fn require_tail_policy(policies: &[serde_yaml::Value], typ: &str) -> Result<(), ConfigError> {
    if policies
        .iter()
        .any(|policy| mapping_string(Some(policy), "type").as_deref() == Some(typ))
    {
        return Ok(());
    }
    Err(ConfigError::Validation(format!(
        "tail_sampling missing {typ} policy"
    )))
}

fn require_http_5xx_policy(policies: &[serde_yaml::Value]) -> Result<(), ConfigError> {
    let has_http_5xx = policies.iter().any(|policy| {
        if mapping_string(Some(policy), "type").as_deref() != Some("numeric_attribute") {
            return false;
        }
        let Some(numeric) = mapping_value(Some(policy), "numeric_attribute") else {
            return false;
        };
        let key = mapping_string(Some(numeric), "key");
        let min = mapping_i64(Some(numeric), "min_value");
        let max = mapping_i64(Some(numeric), "max_value");
        matches!(
            (key.as_deref(), min, max),
            (
                Some("http.response.status_code" | "http.status_code"),
                Some(500),
                Some(599..)
            )
        )
    });
    if has_http_5xx {
        Ok(())
    } else {
        Err(ConfigError::Validation(
            "tail_sampling missing HTTP 5xx numeric_attribute policy".into(),
        ))
    }
}

fn require_probabilistic_10_policy(policies: &[serde_yaml::Value]) -> Result<(), ConfigError> {
    let has_10 = policies.iter().any(|policy| {
        if mapping_string(Some(policy), "type").as_deref() != Some("probabilistic") {
            return false;
        }
        let Some(probabilistic) = mapping_value(Some(policy), "probabilistic") else {
            return false;
        };
        mapping_f64(Some(probabilistic), "sampling_percentage")
            .is_some_and(|pct| (pct - 10.0).abs() < f64::EPSILON)
    });
    if has_10 {
        Ok(())
    } else {
        Err(ConfigError::Validation(
            "tail_sampling missing 10 percent probabilistic policy".into(),
        ))
    }
}

fn mapping_value<'a>(
    value: Option<&'a serde_yaml::Value>,
    key: &str,
) -> Option<&'a serde_yaml::Value> {
    value
        .and_then(serde_yaml::Value::as_mapping)
        .and_then(|m| m.get(serde_yaml::Value::String(key.to_string())))
}

fn mapping_string(value: Option<&serde_yaml::Value>, key: &str) -> Option<String> {
    mapping_value(value, key)
        .and_then(serde_yaml::Value::as_str)
        .map(ToOwned::to_owned)
}

fn mapping_i64(value: Option<&serde_yaml::Value>, key: &str) -> Option<i64> {
    mapping_value(value, key).and_then(serde_yaml::Value::as_i64)
}

fn mapping_f64(value: Option<&serde_yaml::Value>, key: &str) -> Option<f64> {
    mapping_value(value, key).and_then(serde_yaml::Value::as_f64)
}

fn mapping_seq<'a>(
    value: Option<&'a serde_yaml::Value>,
    key: &str,
) -> Option<&'a Vec<serde_yaml::Value>> {
    mapping_value(value, key).and_then(serde_yaml::Value::as_sequence)
}

fn parse_duration_seconds(value: &str) -> Option<u64> {
    value
        .strip_suffix('s')
        .and_then(|raw| raw.parse::<u64>().ok())
}

#[derive(Debug, Deserialize)]
struct CollectorConfig {
    receivers: Receivers,
    processors: Processors,
    exporters: Exporters,
    extensions: Extensions,
    service: ServiceBlock,
}

#[derive(Debug, Deserialize)]
struct Receivers {
    otlp: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct Processors {
    tail_sampling: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct Exporters {
    loki: Option<serde_yaml::Value>,
    prometheusremotewrite: Option<serde_yaml::Value>,
    #[serde(rename = "otlp/tempo")]
    otlp_tempo: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct Extensions {
    bearertokenauth: Option<serde_yaml::Value>,
    file_storage: Option<serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
struct ServiceBlock {
    pipelines: Pipelines,
}

#[derive(Debug, Deserialize)]
struct Pipelines {
    logs: Pipeline,
    metrics: Pipeline,
    traces: Pipeline,
}

#[derive(Debug, Deserialize)]
struct Pipeline {
    #[serde(default)]
    processors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn validate_accepts_canonical_config() {
        let yaml = r#"
receivers:
  otlp: { protocols: { grpc: { endpoint: "0.0.0.0:4317" } } }
processors:
  batch: { timeout: 10s }
  tail_sampling:
    decision_wait: 30s
    num_traces: 100000
    expected_new_traces_per_sec: 1000
    policies:
      - name: errors_100pct
        type: status_code
        status_code: { status_codes: [ERROR] }
      - name: http_5xx_100pct
        type: numeric_attribute
        numeric_attribute: { key: http.response.status_code, min_value: 500, max_value: 599 }
      - name: flagged_tenants_100pct
        type: string_attribute
        string_attribute: { key: tenant_id, values: [tenant-a] }
      - name: slow_traces_100pct
        type: latency
        latency: { threshold_ms: 2000 }
      - name: normal_sample_10pct
        type: probabilistic
        probabilistic: { sampling_percentage: 10 }
exporters:
  loki: { endpoint: "http://loki:3100" }
  prometheusremotewrite: { endpoint: "http://prometheus:9090/api/v1/write" }
  otlp/tempo: { endpoint: "tempo:4317" }
extensions:
  bearertokenauth: { scheme: "Bearer", filename: "/etc/otelcol/collector.token" }
  file_storage: { directory: "/var/lib/otelcol/file_storage", create_directory: true }
service:
  extensions: [file_storage, bearertokenauth]
  pipelines:
    logs:    { receivers: [otlp], processors: [resource, attributes/pii_scrub, batch], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [resource, batch], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [resource, attributes/pii_scrub, tail_sampling, batch], exporters: [otlp/tempo] }
"#;
        let f = NamedTempFile::new().unwrap();
        std::fs::write(f.path(), yaml).unwrap();
        validate(f.path()).expect("validate");
    }

    #[test]
    fn validate_accepts_repo_configs() {
        let manifest_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        validate(&manifest_dir.join("config/otel-collector-config.yaml")).expect("service config");
        validate(&manifest_dir.join("../../deploy/obs/otel-collector-config.yaml"))
            .expect("deploy config");
    }

    #[test]
    fn validate_rejects_missing_pii_scrub() {
        let yaml = r#"
receivers:
  otlp: {}
exporters:
  loki: {}
  prometheusremotewrite: {}
  otlp/tempo: {}
extensions:
  bearertokenauth: {}
  file_storage: {}
service:
  pipelines:
    logs:    { receivers: [otlp], processors: [batch], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [batch], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [batch], exporters: [otlp/tempo] }
"#;
        let f = NamedTempFile::new().unwrap();
        std::fs::write(f.path(), yaml).unwrap();
        let res = validate(f.path());
        assert!(res.is_err());
    }

    #[test]
    fn validate_rejects_collector_schema_regressions() {
        let cases = [
            (
                "loki labels",
                r#"
receivers: { otlp: {} }
exporters:
  loki: { endpoint: "http://loki:3100", labels: { service: "service.name" } }
  prometheusremotewrite: { endpoint: "http://prometheus:9090/api/v1/write" }
  otlp/tempo: { endpoint: "tempo:4317" }
extensions:
  bearertokenauth: {}
  file_storage: { directory: "/var/lib/otelcol/file_storage", create_directory: true }
service:
  pipelines:
    logs:    { receivers: [otlp], processors: [attributes/pii_scrub], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [attributes/pii_scrub], exporters: [otlp/tempo] }
"#,
            ),
            (
                "prometheusremotewrite sending_queue",
                r#"
receivers: { otlp: {} }
exporters:
  loki: { endpoint: "http://loki:3100" }
  prometheusremotewrite:
    endpoint: "http://prometheus:9090/api/v1/write"
    sending_queue: { enabled: true }
  otlp/tempo: { endpoint: "tempo:4317" }
extensions:
  bearertokenauth: {}
  file_storage: { directory: "/var/lib/otelcol/file_storage", create_directory: true }
service:
  pipelines:
    logs:    { receivers: [otlp], processors: [attributes/pii_scrub], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [attributes/pii_scrub], exporters: [otlp/tempo] }
"#,
            ),
            (
                "file_storage create_directory",
                r#"
receivers: { otlp: {} }
exporters:
  loki: { endpoint: "http://loki:3100" }
  prometheusremotewrite: { endpoint: "http://prometheus:9090/api/v1/write" }
  otlp/tempo: { endpoint: "tempo:4317" }
extensions:
  bearertokenauth: {}
  file_storage: { directory: "/var/lib/otelcol/file_storage" }
service:
  pipelines:
    logs:    { receivers: [otlp], processors: [attributes/pii_scrub], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [attributes/pii_scrub], exporters: [otlp/tempo] }
"#,
            ),
        ];

        for (name, yaml) in cases {
            let f = NamedTempFile::new().unwrap();
            std::fs::write(f.path(), yaml).unwrap();
            assert!(validate(f.path()).is_err(), "{name}");
        }
    }
}
