//! TASK-MCP-001 §1 #8 + DEC-272 — Closed JSON-RPC error code map.

use serde_json::{json, Value};

use super::jsonrpc::RpcError;

/// MCP-spec error codes. Standard JSON-RPC ones (`-32700..-32603`) plus the MCP-defined
/// extension range (`-32001..-32099`).
pub mod codes {
    /// Malformed JSON.
    pub const PARSE_ERROR: i32 = -32700;
    /// Not a valid Request object (or protocol-version mismatch).
    pub const INVALID_REQUEST: i32 = -32600;
    /// Method not implemented (e.g. `sampling/createMessage` at slice 4).
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Bad arguments shape.
    pub const INVALID_PARAMS: i32 = -32602;
    /// Gateway-internal error.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Missing/invalid JWT, or insufficient scope.
    pub const UNAUTHORIZED: i32 = -32001;
    /// Per-(tenant, tool) rate limit exceeded.
    pub const RATE_LIMITED: i32 = -32002;
    /// Tool name not in the federated registry.
    pub const TOOL_NOT_FOUND: i32 = -32003;
    /// Owning module returned 5xx or timed out.
    pub const MODULE_UNREACHABLE: i32 = -32004;
    /// Destructive tool requires Elicitation flow (TASK-MCP-006).
    pub const ELICITATION_REQUIRED: i32 = -32005;
    /// Owning module's server is unhealthy/deregistered (TASK-MCP-002 DEC-2351): the tool is
    /// known but its module missed its heartbeats, so the call is refused before dispatch.
    pub const SKILL_UNAVAILABLE: i32 = -32006;
}

/// Build an `RpcError` from the closed code map.
pub fn err(code: i32, message: &str) -> RpcError {
    RpcError {
        code,
        message: message.to_string(),
        data: None,
    }
}

/// Build an `RpcError` with a structured `data` field.
pub fn err_with(code: i32, message: &str, data: Value) -> RpcError {
    RpcError {
        code,
        message: message.to_string(),
        data: Some(data),
    }
}

/// Build the protocol-version-mismatch error per TASK-MCP-001 §1 #5 last paragraph.
pub fn protocol_mismatch(client_version: &str, supported: &[&str]) -> RpcError {
    err_with(
        codes::INVALID_REQUEST,
        "protocol_version_mismatch",
        json!({
            "received": client_version,
            "supported": supported,
        }),
    )
}

/// Build the unauthorized error per TASK-MCP-001 §1 #11.
pub fn unauthorized(reason: &str, required_scopes: &[&str]) -> RpcError {
    err_with(
        codes::UNAUTHORIZED,
        "unauthorized",
        json!({
            "reason": reason,
            "required_scopes": required_scopes,
        }),
    )
}

/// Build the rate-limited error per TASK-MCP-001 §1 #12.
pub fn rate_limited(retry_after_ms: u32) -> RpcError {
    err_with(
        codes::RATE_LIMITED,
        "rate_limited",
        json!({ "retry_after_ms": retry_after_ms }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_table_is_stable() {
        assert_eq!(codes::PARSE_ERROR, -32700);
        assert_eq!(codes::INVALID_REQUEST, -32600);
        assert_eq!(codes::METHOD_NOT_FOUND, -32601);
        assert_eq!(codes::INVALID_PARAMS, -32602);
        assert_eq!(codes::INTERNAL_ERROR, -32603);
        assert_eq!(codes::UNAUTHORIZED, -32001);
        assert_eq!(codes::RATE_LIMITED, -32002);
        assert_eq!(codes::TOOL_NOT_FOUND, -32003);
        assert_eq!(codes::MODULE_UNREACHABLE, -32004);
        assert_eq!(codes::ELICITATION_REQUIRED, -32005);
    }

    #[test]
    fn protocol_mismatch_carries_versions() {
        let e = protocol_mismatch("2024-01-01", &["2025-11-25"]);
        assert_eq!(e.code, -32600);
        let data = e.data.unwrap();
        assert_eq!(data["received"], "2024-01-01");
        assert_eq!(data["supported"], json!(["2025-11-25"]));
    }

    #[test]
    fn unauthorized_includes_reason() {
        let e = unauthorized("expired", &["mcp:tools"]);
        assert_eq!(e.code, -32001);
        let data = e.data.unwrap();
        assert_eq!(data["reason"], "expired");
    }
}
