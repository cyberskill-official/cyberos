//! ISO 27001 view.

use crate::memory::AuditRow;
use crate::views::{count_prefix, ViewDefinition};

/// ISO 27001 row kinds.
pub const KINDS: &[&str] = &[
    "asset.",
    "tenant.",
    "subject.",
    "risk.",
    "access_control.",
    "auth.role_",
    "auth.subject_role",
];

/// View definition.
pub fn definition() -> ViewDefinition {
    ViewDefinition {
        id: "iso27001",
        regulation: "ISO 27001",
        kinds: KINDS,
        summarize,
    }
}

fn summarize(rows: &[AuditRow]) -> serde_json::Value {
    serde_json::json!({
        "asset_inventory_rows": count_prefix(rows, "asset.") + count_prefix(rows, "tenant.") + count_prefix(rows, "subject."),
        "risk_assessment_rows": count_prefix(rows, "risk."),
        "access_control_review_rows": count_prefix(rows, "access_control.") + count_prefix(rows, "auth.role_") + count_prefix(rows, "auth.subject_role"),
    })
}
