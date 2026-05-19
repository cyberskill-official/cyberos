//! FR-MCP-001 §1 #5 — `initialize` handshake.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::MCP_PROTOCOL_VERSION;

use super::capabilities::{Capabilities, ServerInfo};
use super::errors::protocol_mismatch;
use super::jsonrpc::RpcError;

/// Client-supplied params for `initialize`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Client's requested protocol version.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Optional client info block; opaque at this layer.
    #[serde(default, rename = "clientInfo")]
    pub client_info: Option<Value>,
    /// Optional client capabilities block; opaque at this layer.
    #[serde(default)]
    pub capabilities: Option<Value>,
}

/// Server-built response shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    /// The version the server agreed on (always `MCP_PROTOCOL_VERSION`).
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    /// Server capabilities.
    pub capabilities: Capabilities,
    /// Server identity block.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
    /// Human-readable instructions for the client.
    pub instructions: String,
}

/// Build the server's initialize response from the client's params. Returns the closed
/// `protocol_version_mismatch` error if the client's version is not exactly the version
/// pinned by [`crate::MCP_PROTOCOL_VERSION`] (per DEC-260).
pub fn build_response(params: &InitializeParams) -> Result<InitializeResult, RpcError> {
    if params.protocol_version != MCP_PROTOCOL_VERSION {
        return Err(protocol_mismatch(&params.protocol_version, &[MCP_PROTOCOL_VERSION]));
    }
    Ok(InitializeResult {
        protocol_version: MCP_PROTOCOL_VERSION.to_string(),
        capabilities: Capabilities::default(),
        server_info: ServerInfo::default_for_gateway(),
        instructions:
            "Federation of CyberOS modules. All calls audit-chained via memory. OAuth 2.1 PKCE auth via FR-MCP-004."
                .to_string(),
    })
}

/// Convenience: encode the success response as a `serde_json::Value` for inclusion in a
/// JSON-RPC `Response`.
pub fn build_response_value(params: &InitializeParams) -> Result<Value, RpcError> {
    let r = build_response(params)?;
    Ok(json!({
        "protocolVersion": r.protocol_version,
        "capabilities": r.capabilities,
        "serverInfo": r.server_info,
        "instructions": r.instructions,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_version_returns_ok() {
        let params = InitializeParams {
            protocol_version: MCP_PROTOCOL_VERSION.to_string(),
            client_info: None,
            capabilities: None,
        };
        let r = build_response(&params).unwrap();
        assert_eq!(r.protocol_version, MCP_PROTOCOL_VERSION);
        assert_eq!(r.server_info.name, "cyberos.mcp-gateway");
        assert!(!r.instructions.is_empty());
    }

    #[test]
    fn mismatched_version_returns_protocol_mismatch_error() {
        let params = InitializeParams {
            protocol_version: "2024-01-01".to_string(),
            client_info: None,
            capabilities: None,
        };
        let err = build_response(&params).unwrap_err();
        assert_eq!(err.code, -32600);
    }
}
