//! CUO `obs.triage-alert@1` invocation.

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::alertmanager_webhook::Alert;

/// Suggested runbook returned by the CUO skill.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunbookRef {
    /// KB article id.
    pub kb_article_id: String,
    /// Human-readable runbook title.
    pub title: String,
    /// Runbook URL.
    pub url: String,
}

/// CUO triage result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriageResult {
    /// Confidence in the triage, clamped to `[0, 1]` by the router.
    pub confidence: f64,
    /// Human summary.
    pub summary: String,
    /// Suspected cause.
    pub suspected_cause: String,
    /// Suggested runbook, if any.
    #[serde(default)]
    pub suggested_runbook: Option<RunbookRef>,
}

impl TriageResult {
    /// Fallback used when CUO is unavailable.
    pub fn fallback() -> Self {
        Self {
            confidence: 0.0,
            summary: "CUO unavailable; review alert manually.".to_string(),
            suspected_cause: "unknown".to_string(),
            suggested_runbook: None,
        }
    }

    /// Clamp confidence to the valid range.
    pub fn clamp_confidence(mut self) -> Self {
        self.confidence = self.confidence.clamp(0.0, 1.0);
        self
    }
}

/// CUO client error.
#[derive(Debug, Error)]
pub enum CuoError {
    /// HTTP transport failed.
    #[error("cuo_http: {0}")]
    Http(String),
    /// Response parse failed.
    #[error("cuo_decode: {0}")]
    Decode(String),
    /// CUO timed out.
    #[error("cuo_timeout")]
    Timeout,
}

/// Client capable of invoking `obs.triage-alert@1`.
#[async_trait]
pub trait TriageClient: Send + Sync + std::fmt::Debug {
    /// Triage one alert.
    async fn triage(&self, alert: &Alert) -> Result<TriageResult, CuoError>;
}

/// HTTP CUO client.
#[derive(Debug, Clone)]
pub struct HttpCuoClient {
    endpoint: String,
    client: reqwest::Client,
}

impl HttpCuoClient {
    /// Create a client for a CUO invoke endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl TriageClient for HttpCuoClient {
    async fn triage(&self, alert: &Alert) -> Result<TriageResult, CuoError> {
        let response = self
            .client
            .post(&self.endpoint)
            .json(&serde_json::json!({
                "skill_id": "obs.triage-alert@1",
                "input": { "alert": alert },
            }))
            .send()
            .await
            .map_err(|err| CuoError::Http(err.to_string()))?;
        let response = response
            .error_for_status()
            .map_err(|err| CuoError::Http(err.to_string()))?;
        response
            .json::<TriageResult>()
            .await
            .map_err(|err| CuoError::Decode(err.to_string()))
    }
}

/// Run CUO triage with the FR-OBS-007 timeout budget.
pub async fn triage_with_timeout(
    client: &dyn TriageClient,
    alert: &Alert,
    timeout: Duration,
) -> Result<TriageResult, CuoError> {
    tokio::time::timeout(timeout, client.triage(alert))
        .await
        .map_err(|_| CuoError::Timeout)?
        .map(TriageResult::clamp_confidence)
}
