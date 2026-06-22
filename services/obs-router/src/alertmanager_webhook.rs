//! Alertmanager v2 webhook parsing (FR-OBS-007 §1 #1). A webhook may carry many alerts; each is
//! normalised to an `Alert` carrying exactly the fields routing and the audit row need: name, severity,
//! status, fingerprint, trace_id, summary (§10 - "multiple alerts in one webhook -> iterate per alert").
//! Unknown JSON fields are ignored so a schema addition does not break parsing.

use std::collections::BTreeMap;

use serde::Deserialize;

use crate::error::RouterError;
use crate::severity::Severity;

/// Whether an alert is firing or resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertStatus {
    Firing,
    Resolved,
}

impl AlertStatus {
    fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("resolved") {
            AlertStatus::Resolved
        } else {
            AlertStatus::Firing
        }
    }
}

/// The raw Alertmanager webhook envelope (only the fields used; unknown fields are ignored).
#[derive(Debug, Deserialize)]
pub struct Webhook {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub alerts: Vec<RawAlert>,
}

/// One raw alert inside the webhook.
#[derive(Debug, Deserialize)]
pub struct RawAlert {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub labels: BTreeMap<String, String>,
    #[serde(default)]
    pub annotations: BTreeMap<String, String>,
    #[serde(default)]
    pub fingerprint: String,
}

/// A normalised alert: exactly the fields routing and the audit row need.
#[derive(Debug, Clone)]
pub struct Alert {
    pub name: String,
    pub severity: Severity,
    pub status: AlertStatus,
    pub fingerprint: String,
    pub trace_id: Option<String>,
    pub summary: Option<String>,
}

impl Webhook {
    /// Parse an Alertmanager webhook body.
    pub fn parse(body: &str) -> Result<Self, RouterError> {
        serde_json::from_str(body).map_err(|e| RouterError::ParseFailed(e.to_string()))
    }

    /// The normalised alerts in this webhook, in order.
    pub fn alerts(&self) -> Vec<Alert> {
        self.alerts.iter().map(RawAlert::normalize).collect()
    }
}

impl RawAlert {
    fn normalize(&self) -> Alert {
        Alert {
            name: self
                .labels
                .get("alertname")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string()),
            severity: Severity::parse(self.labels.get("severity").map_or("", String::as_str)),
            status: AlertStatus::parse(&self.status),
            fingerprint: self.fingerprint.clone(),
            trace_id: self.labels.get("trace_id").cloned(),
            summary: self.annotations.get("summary").cloned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"{
        "version": "4",
        "status": "firing",
        "unknownTopLevelField": 123,
        "alerts": [
            {
                "status": "firing",
                "labels": { "alertname": "HighErrorRate", "severity": "critical", "trace_id": "abc123" },
                "annotations": { "summary": "5xx spiking" },
                "fingerprint": "fp-1",
                "generatorURL": "http://x"
            },
            {
                "status": "resolved",
                "labels": { "alertname": "DiskWarn", "severity": "warning" },
                "annotations": {},
                "fingerprint": "fp-2"
            }
        ]
    }"#;

    #[test]
    fn parses_and_normalises_multiple_alerts() {
        let wh = Webhook::parse(SAMPLE).unwrap();
        let alerts = wh.alerts();
        assert_eq!(alerts.len(), 2);

        assert_eq!(alerts[0].name, "HighErrorRate");
        assert_eq!(alerts[0].severity, Severity::Sev1);
        assert_eq!(alerts[0].status, AlertStatus::Firing);
        assert_eq!(alerts[0].trace_id.as_deref(), Some("abc123"));
        assert_eq!(alerts[0].summary.as_deref(), Some("5xx spiking"));
        assert_eq!(alerts[0].fingerprint, "fp-1");

        assert_eq!(alerts[1].name, "DiskWarn");
        assert_eq!(alerts[1].severity, Severity::Sev3);
        assert_eq!(alerts[1].status, AlertStatus::Resolved);
        assert_eq!(alerts[1].trace_id, None);
    }

    #[test]
    fn missing_alertname_and_severity_get_safe_defaults() {
        let wh =
            Webhook::parse(r#"{"alerts":[{"status":"firing","labels":{},"fingerprint":"f"}]}"#)
                .unwrap();
        let a = &wh.alerts()[0];
        assert_eq!(a.name, "unknown");
        assert_eq!(a.severity, Severity::Sev2); // unknown severity -> cautious sev2
    }

    #[test]
    fn malformed_json_is_parse_failed() {
        let e = Webhook::parse("not json").unwrap_err();
        assert!(matches!(e, RouterError::ParseFailed(_)));
    }
}
