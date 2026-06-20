//! CHAT and PagerDuty delivery (FR-OBS-007 §1 #3, #4). The orchestration (`handle.rs`) drives these
//! with the §1 #11 fallback chain so an alert is never silently dropped.

use crate::alertmanager_webhook::Alert;
use crate::triage::Triage;

/// A delivery failure (CHAT post or PagerDuty trigger).
#[derive(Debug, thiserror::Error)]
#[error("notify failed: {0}")]
pub struct NotifyError(pub String);

/// Posts a triage summary to the CHAT `#oncall` channel - alert badge, CUO summary, suspected cause,
/// suggested runbook, trace_id link, and the ack / escalate buttons (§1 #4).
pub trait ChatClient {
    fn post(
        &self,
        alert: &Alert,
        triage: &Triage,
        request_id: &str,
    ) -> impl std::future::Future<Output = Result<(), NotifyError>>;
}

/// Pages on-call via PagerDuty.
pub trait PagerDutyClient {
    fn trigger(
        &self,
        alert: &Alert,
        triage: &Triage,
        request_id: &str,
    ) -> impl std::future::Future<Output = Result<(), NotifyError>>;
}
