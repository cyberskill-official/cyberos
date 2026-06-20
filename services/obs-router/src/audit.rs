//! Audit rows for routed and acked alerts (FR-OBS-007 §1 #6, #10). Pure builders plus a sink trait: the
//! production sink writes to the memory chain (best-effort - §10, "audit emit fails -> route still
//! completes"); the recording sink is for tests.

use std::sync::Mutex;

use serde_json::{json, Value};

use crate::alertmanager_webhook::Alert;
use crate::route::Route;

/// An audit row: a `row_kind` plus a JSON payload.
#[derive(Debug, Clone)]
pub struct AuditRow {
    pub kind: &'static str,
    pub payload: Value,
}

/// `obs.alert_triaged` - one per routed alert (§1 #6). Carries the trace_id so an investigator can jump
/// to Tempo from the audit row.
pub fn alert_triaged(
    alert: &Alert,
    cuo_confidence: f64,
    route: Route,
    suggested_runbook: Option<&str>,
    request_id: &str,
) -> AuditRow {
    AuditRow {
        kind: "obs.alert_triaged",
        payload: json!({
            "alert_name": alert.name,
            "severity": alert.severity.label(),
            "cuo_confidence": cuo_confidence,
            "route": route.label(),
            "suggested_runbook": suggested_runbook,
            "trace_id": alert.trace_id,
            "request_id": request_id,
        }),
    }
}

/// `obs.alert_acked` - when an operator acks an alert from CHAT (§1 #10).
pub fn alert_acked(alert_fingerprint: &str, acked_by: &str, request_id: &str) -> AuditRow {
    AuditRow {
        kind: "obs.alert_acked",
        payload: json!({
            "alert_fingerprint": alert_fingerprint,
            "acked_by": acked_by,
            "request_id": request_id,
        }),
    }
}

/// A sink for audit rows. The production impl writes to the memory chain.
pub trait AuditSink {
    fn emit(&self, row: &AuditRow);
}

/// Records rows in memory for tests.
#[derive(Default)]
pub struct RecordingSink {
    rows: Mutex<Vec<AuditRow>>,
}

impl RecordingSink {
    /// The most recent row of `kind`, if any.
    pub fn latest(&self, kind: &str) -> Option<AuditRow> {
        self.rows
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|r| r.kind == kind)
            .cloned()
    }
}

impl AuditSink for RecordingSink {
    fn emit(&self, row: &AuditRow) {
        self.rows.lock().unwrap().push(row.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alertmanager_webhook::{Alert, AlertStatus};
    use crate::severity::Severity;

    fn alert() -> Alert {
        Alert {
            name: "HighErrorRate".into(),
            severity: Severity::Sev2,
            status: AlertStatus::Firing,
            fingerprint: "fp-1".into(),
            trace_id: Some("abc123".into()),
            summary: Some("5xx spiking".into()),
        }
    }

    #[test]
    fn triaged_row_carries_the_spec_fields() {
        let row = alert_triaged(&alert(), 0.82, Route::Chat, Some("kb/runbook-1"), "req-1");
        assert_eq!(row.kind, "obs.alert_triaged");
        assert_eq!(row.payload["alert_name"], "HighErrorRate");
        assert_eq!(row.payload["severity"], "sev2");
        assert_eq!(row.payload["route"], "chat");
        assert_eq!(row.payload["cuo_confidence"], 0.82);
        assert_eq!(row.payload["trace_id"], "abc123");
        assert_eq!(row.payload["suggested_runbook"], "kb/runbook-1");
        assert_eq!(row.payload["request_id"], "req-1");
    }

    #[test]
    fn sink_records_and_returns_latest() {
        let sink = RecordingSink::default();
        sink.emit(&alert_acked("fp-1", "stephen", "req-2"));
        let got = sink.latest("obs.alert_acked").unwrap();
        assert_eq!(got.payload["alert_fingerprint"], "fp-1");
        assert_eq!(got.payload["acked_by"], "stephen");
        assert!(sink.latest("obs.alert_triaged").is_none());
    }
}
