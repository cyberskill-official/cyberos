//! JSON-RPC 2.0 wire types per the MCP 2025-11-25 spec.
//!
//! Supports single requests, notifications (no `id`), and batch arrays. Parse-side is
//! permissive (accepts any valid JSON-RPC 2.0 message); serialise-side is strict (always
//! emits `"jsonrpc":"2.0"` and either `result` xor `error`).

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// JSON-RPC 2.0 protocol marker. Always the literal string `"2.0"` on the wire.
pub const JSONRPC_VERSION: &str = "2.0";

/// Inbound request. `id` absent ⇒ notification (no response expected). `params` is opaque
/// to the parser; method handlers parse it into their own typed shapes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Always `"2.0"`.
    pub jsonrpc: String,
    /// Request id; absent on notifications. JSON-RPC allows string, number, or null.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    /// Method name (e.g. `"tools/call"`).
    pub method: String,
    /// Method parameters; method-specific shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl Request {
    /// True when no `id` field is set (notification).
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// Outbound response (success or error).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Always `"2.0"`.
    pub jsonrpc: String,
    /// Mirrors the request's `id`.
    pub id: Value,
    /// Set on success; mutually exclusive with `error`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Set on failure; mutually exclusive with `result`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl Response {
    /// Build a success response.
    pub fn success(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Build an error response.
    pub fn error(id: Value, error: RpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error object. The MCP spec defines specific `code` values in the
/// `-32099..-32001` range in addition to the standard JSON-RPC errors; see
/// `super::errors` for the closed map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    /// Closed integer code (TASK-MCP-001 §1 #8 / DEC-272).
    pub code: i32,
    /// Short human-readable message.
    pub message: String,
    /// Optional structured payload (e.g. `{retry_after_ms}` for rate-limit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Inbound payload — either a single message or a batch.
#[derive(Debug)]
pub enum Inbound {
    /// Single request or notification.
    Single(Request),
    /// Batch of requests/notifications. Empty batch is forbidden per spec.
    Batch(Vec<Request>),
}

impl Inbound {
    /// Permissive parser. Empty batches are rejected.
    pub fn parse(raw: &[u8]) -> Result<Self, String> {
        let v: Value = serde_json::from_slice(raw).map_err(|e| format!("parse: {e}"))?;
        match v {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Err("empty batch".into());
                }
                let mut reqs = Vec::with_capacity(arr.len());
                for item in arr {
                    let r: Request =
                        serde_json::from_value(item).map_err(|e| format!("batch item: {e}"))?;
                    reqs.push(r);
                }
                Ok(Inbound::Batch(reqs))
            }
            Value::Object(_) => {
                let r: Request = serde_json::from_value(v).map_err(|e| format!("request: {e}"))?;
                Ok(Inbound::Single(r))
            }
            _ => Err("top-level must be array or object".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_single_request() {
        let raw = br#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}}"#;
        let inbound = Inbound::parse(raw).unwrap();
        match inbound {
            Inbound::Single(r) => {
                assert_eq!(r.method, "initialize");
                assert_eq!(r.id, Some(json!(1)));
            }
            _ => panic!("expected single"),
        }
    }

    #[test]
    fn parse_notification_has_no_id() {
        let raw = br#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let inbound = Inbound::parse(raw).unwrap();
        match inbound {
            Inbound::Single(r) => {
                assert!(r.is_notification());
                assert_eq!(r.method, "notifications/initialized");
            }
            _ => panic!("expected single"),
        }
    }

    #[test]
    fn parse_batch_request() {
        let raw =
            br#"[{"jsonrpc":"2.0","id":1,"method":"a"},{"jsonrpc":"2.0","id":2,"method":"b"}]"#;
        let inbound = Inbound::parse(raw).unwrap();
        match inbound {
            Inbound::Batch(reqs) => assert_eq!(reqs.len(), 2),
            _ => panic!("expected batch"),
        }
    }

    #[test]
    fn parse_empty_batch_rejected() {
        let raw = b"[]";
        assert!(Inbound::parse(raw).is_err());
    }

    #[test]
    fn parse_garbage_rejected() {
        let raw = b"not-json-at-all";
        assert!(Inbound::parse(raw).is_err());
    }

    #[test]
    fn response_success_shape() {
        let r = Response::success(json!(1), json!({"ok": true}));
        let s = serde_json::to_string(&r).unwrap();
        assert!(s.contains(r#""jsonrpc":"2.0""#));
        assert!(s.contains(r#""result":{"ok":true}"#));
        assert!(!s.contains(r#""error":"#));
    }

    #[test]
    fn response_error_shape() {
        let r = Response::error(
            json!(1),
            RpcError {
                code: -32602,
                message: "invalid_params".into(),
                data: None,
            },
        );
        let s = serde_json::to_string(&r).unwrap();
        assert!(s.contains(r#""code":-32602"#));
        assert!(!s.contains(r#""result":"#));
    }
}
