//! FR-MCP-006 — Tool-annotation gating.

use serde::{Deserialize, Serialize};

use crate::annotations::ToolAnnotations;

/// Decision for a tool call.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateDecision {
    /// Call may proceed.
    Allow,
    /// User confirmation is required first.
    RequireConfirmation {
        /// Stable reason code.
        reason: String,
    },
}

/// Evaluate annotations and caller confirmation state.
pub fn evaluate(annotations: &ToolAnnotations, confirmed: bool) -> GateDecision {
    if annotations.destructive_hint && !confirmed {
        GateDecision::RequireConfirmation {
            reason: "destructive_tool_requires_confirmation".into(),
        }
    } else {
        GateDecision::Allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_requires_confirmation() {
        assert!(matches!(
            evaluate(&ToolAnnotations::destructive("Delete"), false),
            GateDecision::RequireConfirmation { .. }
        ));
    }
}
