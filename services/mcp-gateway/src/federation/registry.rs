//! FR-MCP-001 §1 #18 + #19 — In-memory federated tool catalog.
//!
//! Modules register via `POST /v1/mcp/register` (handler lands with FR-MCP-002). The
//! gateway snapshots the catalog at every `tools/list` call. Read-mostly; writes go via
//! the registration handler.

use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::Value;

use crate::annotations::ToolAnnotations;
use crate::protocol::tools_list::ToolDescriptor;

/// Per-tool record. Public-facing fields go to MCP clients via `tools/list`; the
/// `module`/`endpoint`/`requires_scope` fields are internal-only.
#[derive(Debug, Clone)]
pub struct ToolEntry {
    /// SEP-986 name.
    pub name: String,
    /// Plain-English description.
    pub description: String,
    /// JSONSchema for the arguments.
    pub input_schema: Value,
    /// Spec-defined annotations.
    pub annotations: ToolAnnotations,
    /// Module that owns the tool (e.g. `"memory"`).
    pub module: String,
    /// Internal HTTP endpoint to dispatch to.
    pub endpoint: String,
    /// Scopes the caller MUST have (per tool).
    pub requires_scope: Vec<String>,
}

impl ToolEntry {
    /// Strip internal fields, return the spec-facing descriptor.
    pub fn to_descriptor(&self) -> ToolDescriptor {
        ToolDescriptor {
            name: self.name.clone(),
            description: self.description.clone(),
            input_schema: self.input_schema.clone(),
            annotations: self.annotations.clone(),
        }
    }
}

/// In-memory registry. Thread-safe via `RwLock` for the slice-1 scale (50 tenants × <500
/// tools). FR-MCP-002 will swap this for an `ArcSwap` or DashMap as call volume grows.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    inner: RwLock<HashMap<String, ToolEntry>>,
}

impl ToolRegistry {
    /// Empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or replace a tool. Returns whether a replacement happened.
    #[allow(clippy::too_many_arguments)]
    pub fn register(
        &self,
        name: String,
        description: String,
        input_schema: Value,
        annotations: ToolAnnotations,
        module: String,
        endpoint: String,
        requires_scope: Vec<String>,
    ) -> bool {
        let mut guard = self.inner.write().expect("poisoned");
        guard
            .insert(
                name.clone(),
                ToolEntry {
                    name,
                    description,
                    input_schema,
                    annotations,
                    module,
                    endpoint,
                    requires_scope,
                },
            )
            .is_some()
    }

    /// Look up a tool by name.
    pub fn lookup(&self, name: &str) -> Option<ToolEntry> {
        self.inner.read().expect("poisoned").get(name).cloned()
    }

    /// Snapshot all tools, sorted by name for deterministic pagination.
    pub fn snapshot_sorted(&self) -> Vec<ToolDescriptor> {
        let snap = self.inner.read().expect("poisoned");
        let mut names: Vec<&String> = snap.keys().collect();
        names.sort();
        names.into_iter().map(|n| snap[n].to_descriptor()).collect()
    }

    /// Count of registered tools.
    pub fn len(&self) -> usize {
        self.inner.read().expect("poisoned").len()
    }

    /// Whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.inner.read().expect("poisoned").is_empty()
    }

    /// Distinct module names currently registering tools, sorted.
    pub fn modules(&self) -> Vec<String> {
        let snap = self.inner.read().expect("poisoned");
        let mods: std::collections::BTreeSet<String> =
            snap.values().map(|e| e.module.clone()).collect();
        mods.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::ToolAnnotations;
    use serde_json::json;

    #[test]
    fn register_and_lookup_roundtrip() {
        let r = ToolRegistry::new();
        r.register(
            "cyberos.memory.search_memory".into(),
            "search memory memories".into(),
            json!({"type":"object"}),
            ToolAnnotations::read_only_idempotent("Search"),
            "memory".into(),
            "http://memory.internal/mcp".into(),
            vec!["mcp:tools".into()],
        );
        assert_eq!(r.len(), 1);
        let entry = r.lookup("cyberos.memory.search_memory").unwrap();
        assert_eq!(entry.module, "memory");
        assert_eq!(entry.endpoint, "http://memory.internal/mcp");
        assert!(r.lookup("cyberos.nope.nope").is_none());
    }

    #[test]
    fn snapshot_is_sorted() {
        let r = ToolRegistry::new();
        for n in ["c", "a", "b"] {
            r.register(
                format!("cyberos.test.{n}"),
                "x".into(),
                json!({}),
                ToolAnnotations::default(),
                "test".into(),
                "x".into(),
                vec![],
            );
        }
        let snap = r.snapshot_sorted();
        let names: Vec<_> = snap.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["cyberos.test.a", "cyberos.test.b", "cyberos.test.c"]);
    }
}
