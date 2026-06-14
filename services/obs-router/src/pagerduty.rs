//! PagerDuty Events API v2 client.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::alertmanager_webhook::Alert;
use crate::cuo_triage::TriageResult;
use crate::severity::Severity;

/// PagerDuty incident trigger payload used by the router.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagerDutyIncident {
    /// Alert fingerprint/dedup key.
    pub dedup_key: String,
    /// Alert name.
    pub alert_name: String,
    /// Severity label.
    pub severity: String,
    /// PagerDuty summary.
    pub summary: String,
    /// Trace id, if present.
    pub trace_id: Option<String>,
}

/// PagerDuty receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagerDutyReceipt {
    /// PagerDuty dedup key.
    pub dedup_key: String,
}

/// PagerDuty error.
#[derive(Debug, Error)]
pub enum PagerDutyError {
    /// HTTP transport or status failure.
    #[error("pagerduty_http: {0}")]
    Http(String),
}

/// Client capable of triggering and resolving incidents.
#[async_trait]
pub trait PagerDutyClient: Send + Sync + std::fmt::Debug {
    /// Trigger an incident.
    async fn trigger(
        &self,
        incident: PagerDutyIncident,
    ) -> Result<PagerDutyReceipt, PagerDutyError>;
    /// Resolve an incident by dedup key.
    async fn resolve(&self, dedup_key: &str) -> Result<(), PagerDutyError>;
}

/// HTTP PagerDuty Events API v2 client.
#[derive(Debug, Clone)]
pub struct HttpPagerDutyClient {
    endpoint: String,
    routing_key: String,
    client: reqwest::Client,
}

impl HttpPagerDutyClient {
    /// Create a client.
    pub fn new(endpoint: impl Into<String>, routing_key: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            routing_key: routing_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl PagerDutyClient for HttpPagerDutyClient {
    async fn trigger(
        &self,
        incident: PagerDutyIncident,
    ) -> Result<PagerDutyReceipt, PagerDutyError> {
        let dedup_key = incident.dedup_key.clone();
        self.send_event("trigger", &dedup_key, Some(incident)).await
    }

    async fn resolve(&self, dedup_key: &str) -> Result<(), PagerDutyError> {
        self.send_event("resolve", dedup_key, None)
            .await
            .map(|_| ())
    }
}

impl HttpPagerDutyClient {
    async fn send_event(
        &self,
        event_action: &str,
        dedup_key: &str,
        incident: Option<PagerDutyIncident>,
    ) -> Result<PagerDutyReceipt, PagerDutyError> {
        let payload = incident
            .as_ref()
            .map(|i| {
                serde_json::json!({
                    "summary": i.summary,
                    "source": "cyberos-obs-router",
                    "severity": i.severity,
                    "custom_details": {
                        "alert_name": i.alert_name,
                        "trace_id": i.trace_id,
                    }
                })
            })
            .unwrap_or_else(|| serde_json::json!({ "summary": "resolved" }));
        let response = self
            .client
            .post(&self.endpoint)
            .json(&serde_json::json!({
                "routing_key": self.routing_key,
                "event_action": event_action,
                "dedup_key": dedup_key,
                "payload": payload,
            }))
            .send()
            .await
            .map_err(|err| PagerDutyError::Http(err.to_string()))?;
        response
            .error_for_status()
            .map_err(|err| PagerDutyError::Http(err.to_string()))?;
        Ok(PagerDutyReceipt {
            dedup_key: dedup_key.to_string(),
        })
    }
}

/// Build the PagerDuty incident from alert + triage.
pub fn build_incident(
    alert: &Alert,
    triage: &TriageResult,
    severity: Severity,
) -> PagerDutyIncident {
    PagerDutyIncident {
        dedup_key: alert.alert_id(),
        alert_name: alert.alert_name(),
        severity: severity.as_label().to_string(),
        summary: format!("{}: {}", alert.alert_name(), triage.summary),
        trace_id: alert.trace_id().map(ToOwned::to_owned),
    }
}
