//! FR-MCP-001 §1 #18 + #19 — In-memory federated tool catalog.
//!
//! Modules register via `POST /v1/mcp/register` (handler lands with FR-MCP-002). The
//! gateway snapshots the catalog at every `tools/list` call. Read-mostly; writes go via
//! the registration handler.

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::SystemTime;

use serde_json::Value;

use crate::annotations::ToolAnnotations;
use crate::federation::health::{classify, ServerHealthStatus};
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

/// Per-module server record backing the FR-MCP-002 heartbeat lifecycle. One per registered
/// module; its tools share its health.
#[derive(Debug, Clone)]
struct ServerRecord {
    /// The module's MCP endpoint (last registered).
    endpoint: String,
    /// When the module last registered or heartbeated.
    last_heartbeat: SystemTime,
    /// Set by an explicit deregister; terminal until the module registers again.
    deregistered: bool,
}

/// In-memory registry. Thread-safe via `RwLock` for the slice-1 scale (50 tenants × <500
/// tools). FR-MCP-002 will swap this for an `ArcSwap` or DashMap as call volume grows.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    inner: RwLock<HashMap<String, ToolEntry>>,
    /// module name -> server health record (FR-MCP-002).
    servers: RwLock<HashMap<String, ServerRecord>>,
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
        let replaced = {
            let mut guard = self.inner.write().expect("poisoned");
            guard
                .insert(
                    name.clone(),
                    ToolEntry {
                        name,
                        description,
                        input_schema,
                        annotations,
                        module: module.clone(),
                        endpoint: endpoint.clone(),
                        requires_scope,
                    },
                )
                .is_some()
        };
        // FR-MCP-002: registering (or re-registering) a tool is the owning module's first
        // heartbeat and clears any prior deregistration.
        {
            let mut servers = self.servers.write().expect("poisoned");
            servers.insert(
                module,
                ServerRecord {
                    endpoint,
                    last_heartbeat: SystemTime::now(),
                    deregistered: false,
                },
            );
        }
        replaced
    }

    /// Record a heartbeat for a module (FR-MCP-002). Returns `false` if the module is not
    /// known (it must register before heartbeating). Clears any prior deregistration.
    pub fn record_heartbeat(&self, module: &str, now: SystemTime) -> bool {
        let mut servers = self.servers.write().expect("poisoned");
        match servers.get_mut(module) {
            Some(rec) => {
                rec.last_heartbeat = now;
                rec.deregistered = false;
                true
            }
            None => false,
        }
    }

    /// Mark a module deregistered (FR-MCP-002). Returns `false` if the module is unknown.
    /// Its tools stay in the catalog but are withdrawn from listing/dispatch until it
    /// registers again.
    pub fn mark_deregistered(&self, module: &str) -> bool {
        let mut servers = self.servers.write().expect("poisoned");
        match servers.get_mut(module) {
            Some(rec) => {
                rec.deregistered = true;
                true
            }
            None => false,
        }
    }

    /// Health of one module's server as of `now`, or `None` if the module is unknown.
    pub fn server_status(&self, module: &str, now: SystemTime) -> Option<ServerHealthStatus> {
        let servers = self.servers.read().expect("poisoned");
        servers.get(module).map(|rec| {
            let age = now.duration_since(rec.last_heartbeat).unwrap_or_default();
            classify(age, rec.deregistered)
        })
    }

    /// Snapshot of every known module's health as of `now`, sorted by module name. For
    /// `/mcp/healthz`.
    pub fn server_health(&self, now: SystemTime) -> Vec<(String, ServerHealthStatus)> {
        let servers = self.servers.read().expect("poisoned");
        let mut out: Vec<(String, ServerHealthStatus)> = servers
            .iter()
            .map(|(module, rec)| {
                let age = now.duration_since(rec.last_heartbeat).unwrap_or_default();
                (module.clone(), classify(age, rec.deregistered))
            })
            .collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }

    /// Descriptors for tools whose owning module is currently available (healthy or
    /// degraded) as of `now`, sorted by name. Tools on unhealthy/deregistered modules are
    /// withdrawn (FR-MCP-002 skill_unavailable propagation). A tool whose module has no
    /// record is treated as available (defensive; registration always creates a record).
    pub fn available_descriptors_sorted(&self, now: SystemTime) -> Vec<ToolDescriptor> {
        let snap = self.inner.read().expect("poisoned");
        let servers = self.servers.read().expect("poisoned");
        let available = |module: &str| -> bool {
            servers
                .get(module)
                .map(|rec| {
                    let age = now.duration_since(rec.last_heartbeat).unwrap_or_default();
                    classify(age, rec.deregistered).is_available()
                })
                .unwrap_or(true)
        };
        let mut names: Vec<&String> = snap
            .iter()
            .filter(|(_, e)| available(&e.module))
            .map(|(n, _)| n)
            .collect();
        names.sort();
        names.into_iter().map(|n| snap[n].to_descriptor()).collect()
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
        assert_eq!(
            names,
            vec!["cyberos.test.a", "cyberos.test.b", "cyberos.test.c"]
        );
    }

    // ---- FR-MCP-002 heartbeat / health lifecycle -------------------------------------

    fn reg(r: &ToolRegistry, name: &str, module: &str) {
        r.register(
            name.into(),
            "x".into(),
            json!({}),
            ToolAnnotations::default(),
            module.into(),
            "http://x/mcp".into(),
            vec![],
        );
    }

    #[test]
    fn register_creates_a_healthy_server() {
        let r = ToolRegistry::new();
        reg(&r, "cyberos.a.t", "a");
        assert_eq!(
            r.server_status("a", std::time::SystemTime::now()).unwrap(),
            ServerHealthStatus::Healthy
        );
        assert!(r.server_status("unknown", std::time::SystemTime::now()).is_none());
    }

    #[test]
    fn heartbeat_age_drives_status() {
        use std::time::Duration;
        let r = ToolRegistry::new();
        reg(&r, "cyberos.a.t", "a");
        let base = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        assert!(r.record_heartbeat("a", base));
        assert_eq!(
            r.server_status("a", base + Duration::from_secs(5)).unwrap(),
            ServerHealthStatus::Healthy
        );
        assert_eq!(
            r.server_status("a", base + Duration::from_secs(20)).unwrap(),
            ServerHealthStatus::Degraded
        );
        assert_eq!(
            r.server_status("a", base + Duration::from_secs(40)).unwrap(),
            ServerHealthStatus::Unhealthy
        );
        assert!(!r.record_heartbeat("unknown", base), "heartbeat for unknown module is rejected");
    }

    #[test]
    fn deregister_is_terminal_until_reregister() {
        let r = ToolRegistry::new();
        reg(&r, "cyberos.a.t", "a");
        assert!(r.mark_deregistered("a"));
        assert_eq!(
            r.server_status("a", std::time::SystemTime::now()).unwrap(),
            ServerHealthStatus::Deregistered
        );
        // Re-registering clears the deregistration.
        reg(&r, "cyberos.a.t", "a");
        assert_eq!(
            r.server_status("a", std::time::SystemTime::now()).unwrap(),
            ServerHealthStatus::Healthy
        );
    }

    #[test]
    fn available_descriptors_withdraw_unhealthy_modules() {
        use std::time::Duration;
        let r = ToolRegistry::new();
        reg(&r, "cyberos.a.t", "a");
        reg(&r, "cyberos.b.t", "b");
        let base = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000);
        r.record_heartbeat("a", base); // a last beat at base
        let now = base + Duration::from_secs(40);
        r.record_heartbeat("b", now); // b fresh as of the query time
        let names: Vec<_> = r
            .available_descriptors_sorted(now)
            .into_iter()
            .map(|d| d.name)
            .collect();
        assert_eq!(names, vec!["cyberos.b.t".to_string()], "unhealthy a is withdrawn, b remains");
        // healthz still reports both modules.
        assert_eq!(r.server_health(now).len(), 2);
    }
}
