//! FR-MCP-001 §1 #5 + DEC-266 — Server-advertised capabilities for `initialize`.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// MCP-spec capabilities advertisement (closed shape per DEC-266).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    /// `tools` capability — `listChanged: true` means we'll push `notifications/tools/list_changed`.
    pub tools: ListChanged,
    /// `prompts` capability.
    pub prompts: ListChanged,
    /// `resources` capability — `subscribe: true` for resource updates.
    pub resources: ResourcesCap,
    /// `logging` capability (empty object per spec).
    pub logging: Value,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            tools: ListChanged { list_changed: true },
            prompts: ListChanged { list_changed: true },
            resources: ResourcesCap {
                list_changed: true,
                subscribe: true,
            },
            logging: json!({}),
        }
    }
}

/// `listChanged` capability wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListChanged {
    /// Whether the server emits `*/list_changed` notifications.
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
}

/// `resources` capability wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcesCap {
    /// Whether the server emits `notifications/resources/list_changed`.
    #[serde(rename = "listChanged")]
    pub list_changed: bool,
    /// Whether `resources/subscribe` is supported.
    pub subscribe: bool,
}

/// `serverInfo` block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Machine-readable server name.
    pub name: String,
    /// Server semver.
    pub version: String,
    /// Human-readable title.
    pub title: String,
}

impl ServerInfo {
    /// Default `cyberos.mcp-gateway` server info.
    pub fn default_for_gateway() -> Self {
        Self {
            name: "cyberos.mcp-gateway".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            title: "CyberOS MCP Gateway".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_capabilities_advertise_listchanged() {
        let c = Capabilities::default();
        let s = serde_json::to_string(&c).unwrap();
        assert!(s.contains(r#""listChanged":true"#));
        assert!(s.contains(r#""subscribe":true"#));
    }

    #[test]
    fn server_info_carries_crate_version() {
        let si = ServerInfo::default_for_gateway();
        assert_eq!(si.name, "cyberos.mcp-gateway");
        assert!(!si.version.is_empty());
    }
}
