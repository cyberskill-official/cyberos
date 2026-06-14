//! EU AI Act view.

use crate::memory::AuditRow;
use crate::views::{count_kind, count_prefix, unique_payload_field, ViewDefinition};

/// EU AI Act row kinds.
pub const KINDS: &[&str] = &[
    "ai.invocation",
    "ai.persona_loaded",
    "ai.zdr_violation",
    "ai.residency_violation",
    "ai.cli_",
    "cli.",
];

/// View definition.
pub fn definition() -> ViewDefinition {
    ViewDefinition {
        id: "eu-ai-act",
        regulation: "EU AI Act",
        kinds: KINDS,
        summarize,
    }
}

fn summarize(rows: &[AuditRow]) -> serde_json::Value {
    serde_json::json!({
        "total_calls": count_kind(rows, "ai.invocation"),
        "unique_personas": unique_payload_field(rows, "persona_handle"),
        "zdr_violations": count_kind(rows, "ai.zdr_violation"),
        "residency_violations": count_kind(rows, "ai.residency_violation"),
        "cli_mutations": count_prefix(rows, "ai.cli_") + count_prefix(rows, "cli."),
    })
}
