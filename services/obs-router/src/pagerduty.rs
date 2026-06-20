//! PagerDuty Events API v2 client (FR-OBS-007 §1 #3). Triggers an incident via the enqueue endpoint,
//! using the alert fingerprint as the `dedup_key` so PagerDuty does its own de-duplication of re-fires.
//! An unset routing key fails, which (for a PagerDuty-intended route) the §1 #11 chain last-resorts to
//! CHAT.

use crate::alertmanager_webhook::Alert;
use crate::notify::{NotifyError, PagerDutyClient};
use crate::severity::Severity;
use crate::triage::Triage;

pub struct HttpPagerDutyClient {
    client: reqwest::Client,
    routing_key: Option<String>,
    endpoint: String,
}

impl HttpPagerDutyClient {
    pub fn new(routing_key: Option<String>, endpoint: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            routing_key,
            endpoint,
        }
    }
}

/// Map CyberOS severity to the PagerDuty Events API severity vocabulary.
fn pagerduty_severity(severity: Severity) -> &'static str {
    match severity {
        Severity::Sev1 => "critical",
        Severity::Sev2 => "error",
        Severity::Sev3 => "warning",
        Severity::Sev4 => "info",
    }
}

impl PagerDutyClient for HttpPagerDutyClient {
    async fn trigger(&self, alert: &Alert, triage: &Triage, request_id: &str) -> Result<(), NotifyError> {
        let Some(routing_key) = self.routing_key.as_ref() else {
            return Err(NotifyError("OBS_PAGERDUTY_ROUTING_KEY not configured".into()));
        };
        let body = serde_json::json!({
            "routing_key": routing_key,
            "event_action": "trigger",
            "dedup_key": alert.fingerprint,
            "payload": {
                "summary": format!("[{}] {}", alert.severity.label(), alert.name),
                "severity": pagerduty_severity(alert.severity),
                "source": "cyberos-obs-router",
                "custom_details": {
                    "suspected_cause": triage.suspected_cause,
                    "cuo_confidence": triage.confidence,
                    "trace_id": alert.trace_id,
                    "request_id": request_id,
                },
            },
        });
        self.client
            .post(&self.endpoint)
            .json(&body)
            .send()
            .await
            .map_err(|e| NotifyError(e.to_string()))?
            .error_for_status()
            .map_err(|e| NotifyError(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_maps_to_pagerduty_vocab() {
        assert_eq!(pagerduty_severity(Severity::Sev1), "critical");
        assert_eq!(pagerduty_severity(Severity::Sev2), "error");
        assert_eq!(pagerduty_severity(Severity::Sev3), "warning");
        assert_eq!(pagerduty_severity(Severity::Sev4), "info");
    }
}
