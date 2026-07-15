//! TASK-MCP-001 §1 #7 — `tools/call` handler, with TASK-MCP-002 federated dispatch.
//!
//! The handler validates the request shape, looks up the tool in the federated
//! registry, enforces the per-tool scope gate, then forwards the call over JSON-RPC
//! to the owning module's registered MCP endpoint and returns the module's result.
//! Transport failures (the module is down, unreachable, or returns a non-2xx / non-JSON
//! response) map to the closed error code `-32004 module_unreachable` (TASK-MCP-001 §1 #8);
//! `tool_not_found` (-32003) and `insufficient_scope` (-32001) are raised before any
//! network call. Tools reach the registry via `federation::register` (TASK-MCP-002).

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::federation::registry::{ToolEntry, ToolRegistry};
use crate::protocol::errors::{codes, err, err_with};
use crate::protocol::jsonrpc::RpcError;

/// How long the gateway waits on a module's `tools/call` before giving up.
const FORWARD_TIMEOUT_SECS: u64 = 30;
/// Cap on establishing the TCP/TLS connection, so an unreachable module fails fast.
const CONNECT_TIMEOUT_SECS: u64 = 5;

/// Client-supplied params for `tools/call`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsCallParams {
    /// Tool name (SEP-986 form).
    pub name: String,
    /// Arguments object (validated against the tool's `inputSchema` server-side).
    #[serde(default)]
    pub arguments: Value,
    /// Optional MCP `_meta` envelope. The gateway reads the TASK-MCP-006 confirmation reference from it
    /// when a caller re-invokes a destructive tool after answering its confirmation elicitation.
    #[serde(rename = "_meta", default, skip_serializing_if = "Option::is_none")]
    pub meta: Option<ToolCallMeta>,
}

/// The subset of MCP `_meta` the gateway reads on `tools/call`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolCallMeta {
    /// The id of an elicited confirmation (TASK-MCP-006), set when re-invoking after confirming.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation_id: Option<String>,
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

/// `tools/call` response shape per TASK-MCP-001 §1 #7.
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

/// Registry lookup + per-tool scope gate. Returns the resolved entry, or the closed
/// JSON-RPC error (`tool_not_found` / `insufficient_scope`). Factored out and pure so the
/// gate logic is unit-testable without a network call.
pub fn prepare(
    registry: &ToolRegistry,
    params: &ToolsCallParams,
    caller_scopes: &[String],
) -> Result<ToolEntry, RpcError> {
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
    Ok(entry)
}

/// Dispatch a `tools/call`: lookup + scope gate, then forward to the owning module's
/// registered MCP endpoint (TASK-MCP-002) and return its result.
pub async fn dispatch(
    registry: &ToolRegistry,
    params: &ToolsCallParams,
    caller_scopes: &[String],
) -> Result<ToolsCallResult, RpcError> {
    let entry = prepare(registry, params, caller_scopes)?;

    // TASK-MCP-002: refuse before any network call if the owning module's server is unhealthy
    // or deregistered (it has missed its heartbeats). A module with no health record is
    // treated as available (defensive; registration always creates one).
    if let Some(status) = registry.server_status(&entry.module, std::time::SystemTime::now()) {
        if !status.is_available() {
            return Err(err_with(
                codes::SKILL_UNAVAILABLE,
                "skill_unavailable",
                serde_json::json!({ "module": entry.module, "health": status.as_str() }),
            ));
        }
    }

    forward_to_module(&entry.endpoint, &entry.module, params).await
}

/// Forward a `tools/call` to a module's MCP endpoint over JSON-RPC and return its result.
/// Any transport-level failure maps to `module_unreachable`.
async fn forward_to_module(
    endpoint: &str,
    module: &str,
    params: &ToolsCallParams,
) -> Result<ToolsCallResult, RpcError> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": { "name": params.name, "arguments": params.arguments },
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(FORWARD_TIMEOUT_SECS))
        .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .build()
        .map_err(|e| {
            err(
                codes::INTERNAL_ERROR,
                &format!("http_client_build_failed: {e}"),
            )
        })?;

    let resp = client
        .post(endpoint)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            err_with(
                codes::MODULE_UNREACHABLE,
                "module_unreachable",
                serde_json::json!({ "module": module, "endpoint": endpoint, "error": e.to_string() }),
            )
        })?;

    let status_ok = resp.status().is_success();
    let body: Value = resp.json().await.map_err(|e| {
        err_with(
            codes::MODULE_UNREACHABLE,
            "module_unreachable",
            serde_json::json!({ "module": module, "reason": "non_json_response", "error": e.to_string() }),
        )
    })?;

    parse_forward_response(status_ok, body, module, endpoint)
}

/// Map a module's JSON-RPC response envelope into our `tools/call` result. Pure (no I/O)
/// so it is unit-testable. A non-2xx status or a JSON-RPC `error` envelope is treated as
/// `module_unreachable` (the module is reachable but the protocol call did not succeed);
/// a tool-side failure is expected to come back in-band as `result.isError = true`.
fn parse_forward_response(
    status_ok: bool,
    body: Value,
    module: &str,
    endpoint: &str,
) -> Result<ToolsCallResult, RpcError> {
    if !status_ok {
        return Err(err_with(
            codes::MODULE_UNREACHABLE,
            "module_unreachable",
            serde_json::json!({ "module": module, "endpoint": endpoint, "reason": "http_error" }),
        ));
    }
    if let Some(result) = body.get("result") {
        let parsed: ToolsCallResult = serde_json::from_value(result.clone()).map_err(|e| {
            err_with(
                codes::INTERNAL_ERROR,
                "module_returned_malformed_result",
                serde_json::json!({ "module": module, "error": e.to_string() }),
            )
        })?;
        return Ok(parsed);
    }
    if let Some(error) = body.get("error") {
        return Err(err_with(
            codes::MODULE_UNREACHABLE,
            "module_unreachable",
            serde_json::json!({ "module": module, "reason": "module_jsonrpc_error", "module_error": error }),
        ));
    }
    Err(err_with(
        codes::INTERNAL_ERROR,
        "module_response_missing_result_and_error",
        serde_json::json!({ "module": module }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::ToolAnnotations;

    fn registry_with(name: &str, endpoint: &str, requires_scope: Vec<String>) -> ToolRegistry {
        let r = ToolRegistry::new();
        r.register(
            name.to_string(),
            "test".into(),
            serde_json::json!({"type":"object"}),
            ToolAnnotations::read_only_idempotent("test"),
            "test-module".into(),
            endpoint.into(),
            requires_scope,
        );
        r
    }

    fn params(name: &str) -> ToolsCallParams {
        ToolsCallParams {
            name: name.into(),
            arguments: serde_json::json!({}),
            meta: None,
        }
    }

    // ---- gate (prepare) -- pure, no network ------------------------------------------

    #[test]
    fn prepare_missing_tool_returns_tool_not_found() {
        let r = ToolRegistry::new();
        let e = prepare(&r, &params("cyberos.unknown.tool"), &["mcp:tools".into()]).unwrap_err();
        assert_eq!(e.code, codes::TOOL_NOT_FOUND);
    }

    #[test]
    fn prepare_missing_scope_returns_unauthorized() {
        let r = registry_with(
            "cyberos.test.tool_0",
            "http://localhost/test",
            vec!["scope:admin".into()],
        );
        let e = prepare(&r, &params("cyberos.test.tool_0"), &["mcp:tools".into()]).unwrap_err();
        assert_eq!(e.code, codes::UNAUTHORIZED);
    }

    #[test]
    fn prepare_with_all_scopes_resolves_the_entry() {
        let r = registry_with(
            "cyberos.test.tool_0",
            "http://mod.internal/mcp",
            vec!["mcp:tools".into()],
        );
        let entry = prepare(&r, &params("cyberos.test.tool_0"), &["mcp:tools".into()]).unwrap();
        assert_eq!(entry.endpoint, "http://mod.internal/mcp");
    }

    // ---- response mapping (parse_forward_response) -- pure, no network ---------------

    #[test]
    fn parse_forward_response_extracts_module_result() {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "content": [{ "type": "text", "text": "pong" }], "isError": false }
        });
        let r = parse_forward_response(true, body, "memory", "http://m/mcp").unwrap();
        assert!(!r.is_error);
        assert_eq!(r.content.len(), 1);
    }

    #[test]
    fn parse_forward_response_non_2xx_is_module_unreachable() {
        let e = parse_forward_response(false, serde_json::json!({}), "memory", "http://m/mcp")
            .unwrap_err();
        assert_eq!(e.code, codes::MODULE_UNREACHABLE);
    }

    #[test]
    fn parse_forward_response_jsonrpc_error_is_module_unreachable() {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "code": -32601, "message": "method not found" }
        });
        let e = parse_forward_response(true, body, "memory", "http://m/mcp").unwrap_err();
        assert_eq!(e.code, codes::MODULE_UNREACHABLE);
    }

    #[test]
    fn parse_forward_response_missing_result_and_error_is_internal() {
        let e = parse_forward_response(
            true,
            serde_json::json!({"jsonrpc": "2.0"}),
            "memory",
            "http://m/mcp",
        )
        .unwrap_err();
        assert_eq!(e.code, codes::INTERNAL_ERROR);
    }

    // ---- end-to-end gate + transport: an unreachable module fails closed -------------

    #[tokio::test]
    async fn dispatch_to_unreachable_module_is_module_unreachable() {
        // Loopback discard port refuses immediately, so this exercises the real reqwest
        // path (bounded by connect_timeout) without depending on any live module.
        let r = registry_with(
            "cyberos.test.tool_0",
            "http://127.0.0.1:9/mcp",
            vec!["mcp:tools".into()],
        );
        let e = dispatch(&r, &params("cyberos.test.tool_0"), &["mcp:tools".into()])
            .await
            .unwrap_err();
        assert_eq!(e.code, codes::MODULE_UNREACHABLE);
    }

    #[tokio::test]
    async fn dispatch_on_unhealthy_module_is_skill_unavailable() {
        use std::time::{Duration, SystemTime};
        // A reachable endpoint would be fine, but the module has missed its heartbeats, so
        // the call must be refused before any network attempt.
        let r = registry_with(
            "cyberos.test.tool_0",
            "http://127.0.0.1:9/mcp",
            vec!["mcp:tools".into()],
        );
        r.record_heartbeat("test-module", SystemTime::now() - Duration::from_secs(60));
        let e = dispatch(&r, &params("cyberos.test.tool_0"), &["mcp:tools".into()])
            .await
            .unwrap_err();
        assert_eq!(e.code, codes::SKILL_UNAVAILABLE);
    }
}
