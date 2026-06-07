//! FR-OBS-008 — Compliance view scoping over memory audit rows.

use serde::{Deserialize, Serialize};

/// Supported compliance view families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Framework {
    /// EU AI Act Article 12 evidence.
    EuAiAct,
    /// Vietnam PDPL evidence.
    Pdpl,
    /// SOC 2 control evidence.
    Soc2,
    /// ISO 27001 control evidence.
    Iso27001,
}

/// Audit-row projection consumed by the compliance exporter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditProjection {
    /// Tenant scope.
    pub tenant_id: String,
    /// Canonical row kind.
    pub row_kind: String,
    /// Chain hash.
    pub chain: String,
    /// Optional actor id.
    pub actor: Option<String>,
}

/// Return rows visible to a framework for one tenant.
pub fn scoped_rows(
    rows: &[AuditProjection],
    tenant_id: &str,
    framework: Framework,
) -> Vec<AuditProjection> {
    rows.iter()
        .filter(|row| row.tenant_id == tenant_id)
        .filter(|row| kind_matches(&row.row_kind, framework))
        .cloned()
        .collect()
}

fn kind_matches(kind: &str, framework: Framework) -> bool {
    match framework {
        Framework::EuAiAct => kind.starts_with("ai.") || kind.starts_with("cuo."),
        Framework::Pdpl => {
            kind.contains("dsar") || kind.contains("consent") || kind.contains("privacy")
        }
        Framework::Soc2 => {
            kind.contains("auth") || kind.contains("access") || kind.contains("audit")
        }
        Framework::Iso27001 => {
            kind.contains("risk") || kind.contains("incident") || kind.contains("access")
        }
    }
}
