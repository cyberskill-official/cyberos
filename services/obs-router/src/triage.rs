//! The CUO `obs.triage-alert` client (FR-OBS-007 §1 #2). The router calls CUO with the alert; CUO
//! returns a confidence plus summary, suggested runbook, and suspected cause. A timeout (5s budget,
//! §1 #9) or error is absorbed by the orchestration as `Triage::failed` (confidence 0), which routes to
//! PagerDuty - never a silent drop.

use crate::alertmanager_webhook::Alert;

/// CUO's triage result for an alert.
#[derive(Debug, Clone)]
pub struct Triage {
    pub confidence: f64,
    pub summary: String,
    pub suggested_runbook: Option<String>,
    pub suspected_cause: String,
}

impl Triage {
    /// The safe fallback when CUO times out or errors: confidence 0 (routes to PagerDuty), no runbook.
    pub fn failed() -> Self {
        Self {
            confidence: 0.0,
            summary: "CUO triage unavailable - paging on-call".to_string(),
            suggested_runbook: None,
            suspected_cause: "unknown (triage failed)".to_string(),
        }
    }
}

/// Error invoking CUO triage.
#[derive(Debug, thiserror::Error)]
pub enum TriageError {
    #[error("cuo triage timed out")]
    Timeout,
    #[error("cuo triage failed: {0}")]
    Failed(String),
}

/// Invokes the CUO `obs.triage-alert` skill. The production impl is an HTTP call wrapped in a 5s
/// timeout; the orchestration treats any `Err` as confidence 0.
pub trait TriageClient {
    fn triage(
        &self,
        alert: &Alert,
    ) -> impl std::future::Future<Output = Result<Triage, TriageError>>;
}
