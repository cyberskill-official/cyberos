//! SOC 2 view.

use crate::memory::AuditRow;
use crate::views::{count_kind, count_prefix, ViewDefinition};

/// SOC 2 row kinds.
pub const KINDS: &[&str] = &[
    "auth.token_issued",
    "auth.token_failed",
    "ai.cli_policy_updated",
    "ai.cli_breaker_reset",
    "backup.",
    "obs.backup",
    "obs.alert_triaged",
    "obs.alert_acked",
];

/// View definition.
pub fn definition() -> ViewDefinition {
    ViewDefinition {
        id: "soc2",
        regulation: "SOC 2",
        kinds: KINDS,
        summarize,
    }
}

fn summarize(rows: &[AuditRow]) -> serde_json::Value {
    serde_json::json!({
        "access_events": count_kind(rows, "auth.token_issued") + count_kind(rows, "auth.token_failed"),
        "configuration_changes": count_kind(rows, "ai.cli_policy_updated") + count_kind(rows, "ai.cli_breaker_reset"),
        "backup_attestations": count_prefix(rows, "backup.") + count_prefix(rows, "obs.backup"),
        "incident_response_rows": count_kind(rows, "obs.alert_triaged") + count_kind(rows, "obs.alert_acked"),
    })
}
