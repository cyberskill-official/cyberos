//! FR-MCP-001 §1 #6 + DEC-264 — `ToolAnnotations` per MCP 2025-11-25 spec.

use serde::{Deserialize, Serialize};

/// Spec-defined tool annotations. Exposed to MCP clients in the `tools/list` response.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolAnnotations {
    /// Short human-readable title.
    pub title: String,
    /// `true` ⇒ tool does not modify state (e.g. search, fetch).
    #[serde(rename = "readOnlyHint", default)]
    pub read_only_hint: bool,
    /// `true` ⇒ tool may modify state irreversibly (e.g. send, delete).
    #[serde(rename = "destructiveHint", default)]
    pub destructive_hint: bool,
    /// `true` ⇒ same args → same result, no side effects on repeat.
    #[serde(rename = "idempotentHint", default)]
    pub idempotent_hint: bool,
    /// `true` ⇒ tool reaches into systems outside the agent's local context.
    #[serde(rename = "openWorldHint", default)]
    pub open_world_hint: bool,
}

impl ToolAnnotations {
    /// Common pattern: read-only + idempotent (search-style tools).
    pub fn read_only_idempotent(title: &str) -> Self {
        Self {
            title: title.into(),
            read_only_hint: true,
            destructive_hint: false,
            idempotent_hint: true,
            open_world_hint: false,
        }
    }

    /// Common pattern: destructive (delete-style tools — requires Elicitation per FR-MCP-006).
    pub fn destructive(title: &str) -> Self {
        Self {
            title: title.into(),
            read_only_hint: false,
            destructive_hint: true,
            idempotent_hint: false,
            open_world_hint: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_only_idempotent_flips_correct_hints() {
        let a = ToolAnnotations::read_only_idempotent("search");
        assert!(a.read_only_hint);
        assert!(a.idempotent_hint);
        assert!(!a.destructive_hint);
    }

    #[test]
    fn destructive_flips_correct_hints() {
        let a = ToolAnnotations::destructive("delete");
        assert!(a.destructive_hint);
        assert!(!a.read_only_hint);
    }
}
