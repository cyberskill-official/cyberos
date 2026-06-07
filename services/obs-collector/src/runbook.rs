//! FR-OBS-007 — Alertmanager to CUO runbook routing.

use serde::{Deserialize, Serialize};

/// Alert routed out of Alertmanager.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Alert {
    /// Alert name.
    pub name: String,
    /// Tenant scope.
    pub tenant_id: String,
    /// CUO triage confidence.
    pub confidence: f32,
    /// Severity label.
    pub severity: String,
}

/// Routing target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertRoute {
    /// Post to CHAT with `obs.triage-alert@1` output.
    Chat {
        /// CHAT channel name.
        channel: String,
    },
    /// Escalate to PagerDuty.
    PagerDuty {
        /// PagerDuty service key.
        service: String,
    },
}

/// Route with the FR threshold: confidence >= 0.70 goes to CHAT, otherwise PagerDuty.
pub fn route_alert(alert: &Alert) -> AlertRoute {
    if alert.confidence >= 0.70 {
        AlertRoute::Chat {
            channel: format!("tenant-{}-ops", alert.tenant_id),
        }
    } else {
        AlertRoute::PagerDuty {
            service: format!("{}-{}", alert.tenant_id, alert.severity),
        }
    }
}
