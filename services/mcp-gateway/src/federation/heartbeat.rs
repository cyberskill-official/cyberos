//! FR-MCP-002 — Per-module registration and heartbeat lifecycle.

use std::collections::BTreeMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Module health state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthState {
    /// Heartbeats are arriving.
    Healthy,
    /// Three consecutive misses.
    Unhealthy,
}

/// Registered module server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleServer {
    /// Module id, e.g. `memory`.
    pub module: String,
    /// MCP endpoint.
    pub endpoint: String,
    /// Last heartbeat timestamp.
    pub last_heartbeat_at: DateTime<Utc>,
    /// Consecutive missed heartbeats.
    pub missed_heartbeats: u8,
    /// Current state.
    pub state: HealthState,
}

/// Thread-safe module registry.
#[derive(Debug, Default)]
pub struct ModuleRegistry {
    servers: RwLock<BTreeMap<String, ModuleServer>>,
}

impl ModuleRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register or replace a server and mark it healthy.
    pub fn register(&self, module: &str, endpoint: &str) -> Result<ModuleServer, String> {
        if module.trim().is_empty() || endpoint.trim().is_empty() {
            return Err("module_and_endpoint_required".into());
        }
        let server = ModuleServer {
            module: module.into(),
            endpoint: endpoint.into(),
            last_heartbeat_at: Utc::now(),
            missed_heartbeats: 0,
            state: HealthState::Healthy,
        };
        self.servers
            .write()
            .expect("poisoned")
            .insert(module.into(), server.clone());
        Ok(server)
    }

    /// Record a heartbeat and reset miss count.
    pub fn heartbeat(&self, module: &str) -> Result<ModuleServer, String> {
        let mut guard = self.servers.write().expect("poisoned");
        let server = guard
            .get_mut(module)
            .ok_or_else(|| "module_not_registered".to_string())?;
        server.last_heartbeat_at = Utc::now();
        server.missed_heartbeats = 0;
        server.state = HealthState::Healthy;
        Ok(server.clone())
    }

    /// Mark one missed interval; the third miss flips to unhealthy.
    pub fn mark_missed(&self, module: &str) -> Result<ModuleServer, String> {
        let mut guard = self.servers.write().expect("poisoned");
        let server = guard
            .get_mut(module)
            .ok_or_else(|| "module_not_registered".to_string())?;
        server.missed_heartbeats = server.missed_heartbeats.saturating_add(1);
        if server.missed_heartbeats >= 3 {
            server.state = HealthState::Unhealthy;
        }
        Ok(server.clone())
    }

    /// Snapshot servers sorted by module id.
    pub fn snapshot(&self) -> Vec<ModuleServer> {
        self.servers
            .read()
            .expect("poisoned")
            .values()
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn third_missed_heartbeat_marks_unhealthy() {
        let reg = ModuleRegistry::new();
        reg.register("memory", "http://memory/mcp").unwrap();
        assert_eq!(
            reg.mark_missed("memory").unwrap().state,
            HealthState::Healthy
        );
        assert_eq!(
            reg.mark_missed("memory").unwrap().state,
            HealthState::Healthy
        );
        assert_eq!(
            reg.mark_missed("memory").unwrap().state,
            HealthState::Unhealthy
        );
        assert_eq!(reg.heartbeat("memory").unwrap().state, HealthState::Healthy);
    }
}
