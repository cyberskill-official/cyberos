//! CHAT ack and escalation callback handlers.

use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::memory::AuditRow;
use crate::pagerduty::PagerDutyIncident;
use crate::router::{RouterError, RouterState};
use crate::severity::Route;

/// Ack callback body.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AckRequest {
    /// Operator user id or handle.
    #[serde(default = "default_user")]
    pub user: String,
}

/// Escalation callback body.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EscalateRequest {
    /// Operator user id or handle.
    #[serde(default = "default_user")]
    pub user: String,
}

fn default_user() -> String {
    "unknown".to_string()
}

/// Handle CHAT ack callback.
pub async fn handle_ack(
    state: &RouterState,
    alert_id: &str,
    user: &str,
) -> Result<(), RouterError> {
    let record = state
        .dedup
        .get(alert_id)
        .ok_or_else(|| RouterError::NotFound(alert_id.to_string()))?;
    let timestamp = Utc::now().to_rfc3339();
    if let Some(message_id) = &record.chat_message_id {
        state
            .chat
            .update_ack(message_id, user, &timestamp)
            .await
            .map_err(|err| RouterError::Chat(err.to_string()))?;
    }
    if record.route == Route::Both {
        if let Some(dedup_key) = &record.pagerduty_dedup_key {
            state
                .pagerduty
                .resolve(dedup_key)
                .await
                .map_err(|err| RouterError::PagerDuty(err.to_string()))?;
        }
    }
    state.metrics.inc_ack("chat");
    state
        .audit
        .emit(AuditRow {
            kind: "obs.alert_acked".to_string(),
            payload: serde_json::json!({
                "alert_id": alert_id,
                "alert_name": record.alert_name,
                "severity": record.severity.as_label(),
                "acked_by": user,
                "acked_at": timestamp,
                "request_id": format!("obs_router_{}", Uuid::new_v4()),
            }),
        })
        .await
        .map_err(|err| RouterError::Audit(err.to_string()))?;
    Ok(())
}

/// Handle CHAT escalate-to-PagerDuty callback.
pub async fn handle_escalate(
    state: &RouterState,
    alert_id: &str,
    user: &str,
) -> Result<(), RouterError> {
    let record = state
        .dedup
        .get(alert_id)
        .ok_or_else(|| RouterError::NotFound(alert_id.to_string()))?;
    let incident = PagerDutyIncident {
        dedup_key: alert_id.to_string(),
        alert_name: record.alert_name.clone(),
        severity: record.severity.as_label().to_string(),
        summary: format!("CHAT escalation for {}", record.alert_name),
        trace_id: record.trace_id.clone(),
    };
    state
        .pagerduty
        .trigger(incident)
        .await
        .map_err(|err| RouterError::PagerDuty(err.to_string()))?;
    state
        .audit
        .emit(AuditRow {
            kind: "obs.alert_escalated".to_string(),
            payload: serde_json::json!({
                "alert_id": alert_id,
                "alert_name": record.alert_name,
                "severity": record.severity.as_label(),
                "escalated_by": user,
                "escalated_from_chat": true,
                "trace_id": record.trace_id,
                "request_id": format!("obs_router_{}", Uuid::new_v4()),
            }),
        })
        .await
        .map_err(|err| RouterError::Audit(err.to_string()))?;
    Ok(())
}
