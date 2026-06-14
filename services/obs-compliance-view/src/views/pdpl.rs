//! Vietnam PDPL view.

use crate::memory::AuditRow;
use crate::views::{count_kind, count_prefix, ViewDefinition};

/// PDPL row kinds.
pub const KINDS: &[&str] = &[
    "dsar.",
    "privacy.dsar",
    "ai.residency_violation",
    "obs.langsmith_export",
    "obs.langsmith_export_enabled",
    "consent.",
    "ai.pii_redaction",
];

/// View definition.
pub fn definition() -> ViewDefinition {
    ViewDefinition {
        id: "pdpl",
        regulation: "PDPL",
        kinds: KINDS,
        summarize,
    }
}

fn summarize(rows: &[AuditRow]) -> serde_json::Value {
    serde_json::json!({
        "dsar_rows": count_prefix(rows, "dsar.") + count_prefix(rows, "privacy.dsar"),
        "cross_border_rows": count_kind(rows, "ai.residency_violation") + count_prefix(rows, "obs.langsmith_export"),
        "consent_rows": count_prefix(rows, "consent.") + count_kind(rows, "obs.langsmith_export_enabled"),
        "pii_redaction_rows": count_prefix(rows, "ai.pii_redaction"),
    })
}
