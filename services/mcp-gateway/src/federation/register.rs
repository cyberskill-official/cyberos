//! FR-MCP-002 — module self-registration (control-plane).
//!
//! A module's MCP server POSTs its tool catalogue here at startup so `tools/list` and
//! `tools/call` can see it. This is the registration half of FR-MCP-002; the
//! heartbeat/health lifecycle (DEC-2350 register-at-startup + 10s heartbeat, 3 misses /
//! 30s -> unhealthy; DEC-2351 the closed `server_health_status` enum) is a separate slice
//! that builds on this one.
//!
//! Trust boundary: registration mutates what the gateway will forward `tools/call` to, so
//! it is a privileged control-plane operation, not part of the public MCP protocol surface.
//! This slice gates the HTTP route behind the `MCP_DEV_REGISTRATION` env flag (off by
//! default; see `router::handle_register`). Production must additionally authenticate the
//! caller (FR-MCP-004) and allowlist registrable endpoints before exposing this route -
//! an attacker who can register could otherwise point a tool at an endpoint of their
//! choosing and have the gateway forward calls to it.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::annotations::ToolAnnotations;
use crate::federation::registry::ToolRegistry;

/// One tool in a module's registration payload. Field names accept both the MCP wire form
/// (`inputSchema`, `requiresScope`) and snake_case for convenience.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterTool {
    /// SEP-986 tool name, e.g. `cyberos.memory.search_memory`.
    pub name: String,
    /// Human-readable description (surfaced in `tools/list`).
    #[serde(default)]
    pub description: String,
    /// JSON Schema for the tool's arguments.
    #[serde(rename = "inputSchema", alias = "input_schema", default)]
    pub input_schema: Value,
    /// Spec tool annotations (read-only / destructive / idempotent hints).
    #[serde(default)]
    pub annotations: ToolAnnotations,
    /// Scopes a caller must hold for `tools/call` to dispatch this tool.
    #[serde(rename = "requiresScope", alias = "requires_scope", default)]
    pub requires_scope: Vec<String>,
}

/// A module registering its tool catalogue with the gateway.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterRequest {
    /// Owning module name, e.g. `memory`.
    pub module: String,
    /// The module's MCP endpoint the gateway forwards `tools/call` to (http/https URL).
    pub endpoint: String,
    /// One or more tools the module exposes.
    pub tools: Vec<RegisterTool>,
}

/// Why a registration request was rejected (validated before mutating the registry).
#[derive(Debug, PartialEq, Eq)]
pub enum RegisterError {
    /// `module` was empty or whitespace.
    EmptyModule,
    /// `endpoint` was empty or whitespace.
    EmptyEndpoint,
    /// `endpoint` did not start with `http://` or `https://`.
    BadEndpointScheme,
    /// `tools` was empty.
    NoTools,
    /// The tool at this index had an empty `name`.
    EmptyToolName(usize),
}

impl RegisterError {
    /// A stable, human-readable message for the HTTP error body.
    pub fn message(&self) -> String {
        match self {
            RegisterError::EmptyModule => "module must not be empty".to_string(),
            RegisterError::EmptyEndpoint => "endpoint must not be empty".to_string(),
            RegisterError::BadEndpointScheme => {
                "endpoint must be an http:// or https:// URL".to_string()
            }
            RegisterError::NoTools => "tools must contain at least one tool".to_string(),
            RegisterError::EmptyToolName(i) => format!("tools[{i}].name must not be empty"),
        }
    }
}

/// Validate a registration request before mutating the registry. Pure (no I/O).
pub fn validate(req: &RegisterRequest) -> Result<(), RegisterError> {
    if req.module.trim().is_empty() {
        return Err(RegisterError::EmptyModule);
    }
    if req.endpoint.trim().is_empty() {
        return Err(RegisterError::EmptyEndpoint);
    }
    if !(req.endpoint.starts_with("http://") || req.endpoint.starts_with("https://")) {
        return Err(RegisterError::BadEndpointScheme);
    }
    if req.tools.is_empty() {
        return Err(RegisterError::NoTools);
    }
    for (i, t) in req.tools.iter().enumerate() {
        if t.name.trim().is_empty() {
            return Err(RegisterError::EmptyToolName(i));
        }
    }
    Ok(())
}

/// Apply a (validated) registration to the registry, returning the number of tools
/// registered. `ToolRegistry::register` upserts by name, so re-registration from the same
/// module refreshes the endpoint/description/schema rather than duplicating entries.
pub fn apply(registry: &ToolRegistry, req: &RegisterRequest) -> usize {
    for t in &req.tools {
        registry.register(
            t.name.clone(),
            t.description.clone(),
            t.input_schema.clone(),
            t.annotations.clone(),
            req.module.clone(),
            req.endpoint.clone(),
            t.requires_scope.clone(),
        );
    }
    req.tools.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample(module: &str, endpoint: &str, tool_names: &[&str]) -> RegisterRequest {
        RegisterRequest {
            module: module.to_string(),
            endpoint: endpoint.to_string(),
            tools: tool_names
                .iter()
                .map(|n| RegisterTool {
                    name: (*n).to_string(),
                    description: "desc".into(),
                    input_schema: json!({"type": "object"}),
                    annotations: ToolAnnotations::read_only_idempotent("t"),
                    requires_scope: vec!["mcp:tools".into()],
                })
                .collect(),
        }
    }

    #[test]
    fn validate_accepts_a_well_formed_request() {
        let req = sample(
            "memory",
            "http://memory.internal/mcp",
            &["cyberos.memory.search"],
        );
        assert_eq!(validate(&req), Ok(()));
    }

    #[test]
    fn validate_rejects_empty_module_endpoint_and_tools() {
        let mut req = sample("memory", "http://x/mcp", &["a"]);
        req.module = "  ".into();
        assert_eq!(validate(&req), Err(RegisterError::EmptyModule));

        let req = sample("memory", "", &["a"]);
        assert_eq!(validate(&req), Err(RegisterError::EmptyEndpoint));

        let mut req = sample("memory", "ftp://x/mcp", &["a"]);
        assert_eq!(validate(&req), Err(RegisterError::BadEndpointScheme));

        req = sample("memory", "http://x/mcp", &[]);
        assert_eq!(validate(&req), Err(RegisterError::NoTools));

        let mut req = sample("memory", "http://x/mcp", &["a", ""]);
        req.tools[1].name = "".into();
        assert_eq!(validate(&req), Err(RegisterError::EmptyToolName(1)));
    }

    #[test]
    fn apply_registers_every_tool_and_is_visible_to_lookup() {
        let registry = ToolRegistry::new();
        let req = sample(
            "memory",
            "http://memory.internal/mcp",
            &["cyberos.memory.search", "cyberos.memory.write"],
        );
        let n = apply(&registry, &req);
        assert_eq!(n, 2);
        assert_eq!(registry.len(), 2);

        let entry = registry.lookup("cyberos.memory.search").unwrap();
        assert_eq!(entry.module, "memory");
        assert_eq!(entry.endpoint, "http://memory.internal/mcp");
        assert_eq!(entry.requires_scope, vec!["mcp:tools".to_string()]);
    }

    #[test]
    fn apply_upserts_on_re_registration_without_duplicating() {
        let registry = ToolRegistry::new();
        let first = sample(
            "memory",
            "http://old.internal/mcp",
            &["cyberos.memory.search"],
        );
        apply(&registry, &first);
        assert_eq!(registry.len(), 1);

        // Same module + tool, new endpoint -> refresh, not duplicate.
        let second = sample(
            "memory",
            "http://new.internal/mcp",
            &["cyberos.memory.search"],
        );
        apply(&registry, &second);
        assert_eq!(registry.len(), 1);
        assert_eq!(
            registry.lookup("cyberos.memory.search").unwrap().endpoint,
            "http://new.internal/mcp"
        );
    }

    #[test]
    fn register_request_deserializes_from_wire_form() {
        // Modules send the MCP wire form (camelCase). Confirm it parses.
        let raw = json!({
            "module": "memory",
            "endpoint": "http://memory.internal/mcp",
            "tools": [{
                "name": "cyberos.memory.search",
                "description": "search memory",
                "inputSchema": {"type": "object"},
                "annotations": {"title": "Search", "readOnlyHint": true, "idempotentHint": true},
                "requiresScope": ["mcp:tools"]
            }]
        });
        let req: RegisterRequest = serde_json::from_value(raw).unwrap();
        assert_eq!(req.tools.len(), 1);
        assert_eq!(req.tools[0].name, "cyberos.memory.search");
        assert!(req.tools[0].annotations.read_only_hint);
        assert_eq!(req.tools[0].requires_scope, vec!["mcp:tools".to_string()]);
    }
}
