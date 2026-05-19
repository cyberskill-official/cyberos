//! FR-MCP-001 §1 #7 — `tools/call` handler (slice-1 scaffold).
//!
//! The slice-1 surface validates the request shape + looks up the tool in the registry
//! + dispatches to the owning module's MCP endpoint. The dispatch implementation lands
//! when FR-MCP-002 wires the registration handler; for now, dispatch returns
//! `-32004 module_unreachable` (the closed error code per FR-MCP-001 §1 #8).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::federation::registry::ToolRegistry;
use crate::protocol::errors::{codes, err, err_with};
use crate::protocol::jsonrpc::RpcError;

/// Client-supplied params for `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallParams {
    /// Tool name (SEP-986 form).
    pub name: String,
    /// Arguments object (validated against the tool's `inputSchema` server-side).
    #[serde(default)]
    pub arguments: Value,
}

/// One content block per the spec.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Plain text.
    #[serde(rename = "text")]
    Text {
        /// The text.
        text: String,
    },
    /// Base64-encoded image.
    #[serde(rename = "image")]
    Image {
        /// Base64-encoded bytes.
        data: String,
        /// IANA media type.
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Resource pointer.
    #[serde(rename = "resource")]
    Resource {
        /// Pointed-at resource.
        resource: ResourceRef,
    },
}

/// Resource pointer payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    /// URI of the resource.
    pub uri: String,
}

/// `tools/call` response shape per FR-MCP-001 §1 #7.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallResult {
    /// One or more content blocks.
    pub content: Vec<Content>,
    /// `true` on tool-side error (vs JSON-RPC transport error).
    #[serde(rename = "isError")]
    pub is_error: bool,
    /// Optional structured payload alongside the human-readable content.
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Value>,
}

/// Dispatch (slice-1: registry lookup + permission gate + return module_unreachable).
///
/// FR-MCP-002 will replace the `module_unreachable` branch with an actual HTTP POST to
/// the module's registered endpoint.
pub async fn dispatch(
    registry: &ToolRegistry,
    params: &ToolsCallParams,
    caller_scopes: &[String],
) -> Result<ToolsCallResult, RpcError> {
    let entry = registry.lookup(&params.name).ok_or_else(|| {
        err_with(
            codes::TOOL_NOT_FOUND,
            "tool_not_found",
            serde_json::json!({ "name": params.name }),
        )
    })?;

    // Permission gate: each tool's `requires_scope` MUST all be present on the caller.
    let missing: Vec<&str> = entry
        .requires_scope
        .iter()
        .filter(|s| !caller_scopes.iter().any(|c| c == *s))
        .map(|s| s.as_str())
        .collect();
    if !missing.is_empty() {
        return Err(err_with(
            codes::UNAUTHORIZED,
            "insufficient_scope",
            serde_json::json!({ "required_scopes": missing }),
        ));
    }

    // Slice-1: registry exists but the live dispatch handler does not. Return the
    // canonical error code so clients can distinguish "tool exists but module down"
    // from "tool name unknown".
    Err(err(
        codes::MODULE_UNREACHABLE,
        "module_unreachable (FR-MCP-002 dispatch not yet wired)",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::ToolAnnotations;

    fn registry_with(name: &str, requires_scope: Vec<String>) -> ToolRegistry {
        let r = ToolRegistry::new();
        r.register(
            name.to_string(),
            "test".into(),
            serde_json::json!({"type":"object"}),
            ToolAnnotations::read_only_idempotent("test"),
            "test-module".into(),
            "http://localhost/test".into(),
            requires_scope,
        );
        r
    }

    #[tokio::test]
    async fn dispatch_missing_tool_returns_tool_not_found() {
        let r = ToolRegistry::new();
        let res = dispatch(
            &r,
            &ToolsCallParams {
                name: "cyberos.unknown.tool".into(),
                arguments: serde_json::json!({}),
            },
            &["mcp:tools".into()],
        )
        .await;
        let e = res.unwrap_err();
        assert_eq!(e.code, -32003);
    }

    #[tokio::test]
    async fn dispatch_missing_scope_returns_unauthorized() {
        let r = registry_with("cyberos.test.tool_0", vec!["scope:admin".into()]);
        let res = dispatch(
            &r,
            &ToolsCallParams {
                name: "cyberos.test.tool_0".into(),
                arguments: serde_json::json!({}),
            },
            &["mcp:tools".into()],
        )
        .await;
        let e = res.unwrap_err();
        assert_eq!(e.code, -32001);
    }

    #[tokio::test]
    async fn dispatch_returns_module_unreachable_until_fr_mcp_002() {
        let r = registry_with("cyberos.test.tool_0", vec!["mcp:tools".into()]);
        let res = dispatch(
            &r,
            &ToolsCallParams {
                name: "cyberos.test.tool_0".into(),
                arguments: serde_json::json!({}),
            },
            &["mcp:tools".into()],
        )
        .await;
        let e = res.unwrap_err();
        assert_eq!(e.code, -32004);
    }
}
