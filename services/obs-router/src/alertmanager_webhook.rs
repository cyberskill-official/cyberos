//! Alertmanager v2 webhook payload types.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::severity::{parse_severity, Severity};

/// Alertmanager webhook payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertmanagerWebhook {
    /// Alertmanager webhook version.
    #[serde(default)]
    pub version: String,
    /// Alert group key.
    #[serde(default, rename = "groupKey", alias = "group_key")]
    pub group_key: String,
    /// Group status, usually `firing` or `resolved`.
    #[serde(default)]
    pub status: String,
    /// Receiver name.
    #[serde(default)]
    pub receiver: String,
    /// Group labels.
    #[serde(default, rename = "groupLabels", alias = "group_labels")]
    pub group_labels: BTreeMap<String, String>,
    /// Common labels.
    #[serde(default, rename = "commonLabels", alias = "common_labels")]
    pub common_labels: BTreeMap<String, String>,
    /// Alert entries. FR-OBS-007 routes each one independently.
    #[serde(default)]
    pub alerts: Vec<Alert>,
}

/// Single Alertmanager alert.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    /// Alert status.
    #[serde(default)]
    pub status: String,
    /// Labels, including severity, alertname, tenant_id, trace_id, and fingerprint.
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    /// Annotations, including summary and runbook_url.
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
    /// Alert start timestamp.
    #[serde(default, rename = "startsAt", alias = "starts_at")]
    pub starts_at: Option<DateTime<Utc>>,
    /// Alert end timestamp.
    #[serde(default, rename = "endsAt", alias = "ends_at")]
    pub ends_at: Option<DateTime<Utc>>,
    /// Alertmanager fingerprint.
    #[serde(default)]
    pub fingerprint: String,
    /// Alert generator URL.
    #[serde(default, rename = "generatorURL", alias = "generator_url")]
    pub generator_url: String,
}

impl Alert {
    /// Alert fingerprint or a deterministic fallback key.
    pub fn alert_id(&self) -> String {
        if !self.fingerprint.trim().is_empty() {
            return self.fingerprint.clone();
        }
        format!(
            "{}:{}:{}",
            self.label("tenant_id").unwrap_or("unknown"),
            self.alert_name(),
            self.label("severity").unwrap_or("unknown")
        )
    }

    /// Alert name from `alertname`/`alert_name`.
    pub fn alert_name(&self) -> String {
        self.label("alertname")
            .or_else(|| self.label("alert_name"))
            .unwrap_or("unknown_alert")
            .to_string()
    }

    /// Parsed severity.
    pub fn severity(&self) -> Severity {
        parse_severity(self.label("severity").unwrap_or_default())
    }

    /// Tenant id label, if present.
    pub fn tenant_id(&self) -> Option<&str> {
        self.label("tenant_id")
    }

    /// Trace id label, if present.
    pub fn trace_id(&self) -> Option<&str> {
        self.label("trace_id")
    }

    /// Label lookup.
    pub fn label(&self, key: &str) -> Option<&str> {
        self.labels.get(key).map(String::as_str)
    }

    /// Annotation lookup.
    pub fn annotation(&self, key: &str) -> Option<&str> {
        self.annotations.get(key).map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_alertmanager_camel_case_payload() {
        let raw = r#"{
          "version": "4",
          "groupKey": "g",
          "status": "firing",
          "receiver": "cyberos",
          "alerts": [{
            "status": "firing",
            "labels": {"alertname": "LatencyHigh", "severity": "P2", "trace_id": "abc"},
            "annotations": {"summary": "p99 high"},
            "startsAt": "2026-06-15T00:00:00Z",
            "fingerprint": "fp1"
          }]
        }"#;
        let payload: AlertmanagerWebhook = serde_json::from_str(raw).unwrap();
        assert_eq!(payload.alerts[0].alert_name(), "LatencyHigh");
        assert_eq!(payload.alerts[0].severity(), Severity::Sev2);
        assert_eq!(payload.alerts[0].trace_id(), Some("abc"));
    }
}
