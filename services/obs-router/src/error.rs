//! obs-router error type (TASK-OBS-007).

/// Errors from receiving or parsing an Alertmanager webhook.
#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    /// The webhook body was not valid Alertmanager JSON (§10 - parse fails -> sev-1; Alertmanager retries).
    #[error("alertmanager webhook parse failed: {0}")]
    ParseFailed(String),
    /// The shared-secret header was missing or wrong (§1 #13 -> 401).
    #[error("unauthenticated webhook")]
    Unauthenticated,
}
