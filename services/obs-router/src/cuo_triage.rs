//! HTTP client for the CUO `obs.triage-alert` skill (FR-OBS-007 §1 #2). It POSTs `{skill, alert}` to the
//! configured CUO invocation URL and parses a `TriageResult`, enforcing the 5s budget (§1 #9) inside the
//! client - a timeout or any error surfaces as `TriageError`, which the orchestration treats as
//! confidence 0 (PagerDuty). An unset URL fails immediately, so the router degrades to paging.

use std::time::Duration;

use serde::Deserialize;

use crate::alertmanager_webhook::Alert;
use crate::triage::{Triage, TriageClient, TriageError};

/// The CUO triage budget (§1 #9).
const TRIAGE_TIMEOUT: Duration = Duration::from_secs(5);

pub struct HttpTriageClient {
    client: reqwest::Client,
    url: Option<String>,
}

impl HttpTriageClient {
    pub fn new(url: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }
}

#[derive(Deserialize)]
struct TriageResultDto {
    confidence: f64,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    suspected_cause: String,
    #[serde(default)]
    suggested_runbook: Option<RunbookDto>,
}

#[derive(Deserialize)]
struct RunbookDto {
    #[serde(default)]
    url: String,
}

impl TriageClient for HttpTriageClient {
    async fn triage(&self, alert: &Alert) -> Result<Triage, TriageError> {
        let Some(url) = self.url.as_ref() else {
            return Err(TriageError::Failed(
                "OBS_CUO_TRIAGE_URL not configured".into(),
            ));
        };
        let body = serde_json::json!({
            "skill": "obs.triage-alert@1",
            "alert": {
                "name": alert.name,
                "severity": alert.severity.label(),
                "fingerprint": alert.fingerprint,
                "trace_id": alert.trace_id,
                "summary": alert.summary,
            },
        });

        let call = async {
            let resp = self
                .client
                .post(url)
                .json(&body)
                .send()
                .await
                .map_err(|e| TriageError::Failed(e.to_string()))?
                .error_for_status()
                .map_err(|e| TriageError::Failed(e.to_string()))?;
            resp.json::<TriageResultDto>()
                .await
                .map_err(|e| TriageError::Failed(e.to_string()))
        };

        let dto = tokio::time::timeout(TRIAGE_TIMEOUT, call)
            .await
            .map_err(|_| TriageError::Timeout)??;

        Ok(Triage {
            confidence: dto.confidence,
            summary: dto.summary,
            suggested_runbook: dto
                .suggested_runbook
                .map(|r| r.url)
                .filter(|u| !u.is_empty()),
            suspected_cause: dto.suspected_cause,
        })
    }
}
