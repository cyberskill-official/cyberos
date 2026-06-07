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

    Ok(())
}

#[derive(Debug, Deserialize)]
struct CollectorConfig {
    receivers: Receivers,
    exporters: Exporters,
    extensions: Extensions,
    service: ServiceBlock,
}

#[derive(Debug, Deserialize)]
struct Receivers {
    otlp: Option<serde_yaml::Value>,
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
    #[allow(dead_code)]
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
exporters:
  loki: { endpoint: "http://loki:3100" }
  prometheusremotewrite: { endpoint: "http://prometheus:9090/api/v1/write" }
  otlp/tempo: { endpoint: "tempo:4317" }
extensions:
  bearertokenauth: { scheme: "Bearer", filename: "/etc/otelcol/collector.token" }
  file_storage: { directory: "/var/lib/otelcol/file_storage" }
service:
  extensions: [file_storage, bearertokenauth]
  pipelines:
    logs:    { receivers: [otlp], processors: [resource, attributes/pii_scrub, batch], exporters: [loki] }
    metrics: { receivers: [otlp], processors: [resource, batch], exporters: [prometheusremotewrite] }
    traces:  { receivers: [otlp], processors: [resource, attributes/pii_scrub, batch], exporters: [otlp/tempo] }
"#;
        let f = NamedTempFile::new().unwrap();
        std::fs::write(f.path(), yaml).unwrap();
        validate(f.path()).expect("validate");
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
}
