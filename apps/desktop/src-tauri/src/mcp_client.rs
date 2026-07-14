//! HTTP client for the CyberOS mcp-gateway (TASK-MCP-001). JSON-RPC 2.0 over `POST /mcp`; the gateway is
//! stateless per request, so `tools/list` works without an `initialize` handshake. The backend owns these
//! calls, so the webview never hits CORS.
//!
//! `tools/call` currently returns the closed error `-32004 module_unreachable` until TASK-MCP-002 wires
//! federated dispatch; the picker surfaces that as-is so the operator can see the tool exists and is
//! permitted, even before its module is reachable.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// One tool as listed by the gateway. `annotations` and `input_schema` are passed through as raw JSON so
/// the frontend can read `readOnlyHint` / `destructiveHint` and render the argument form.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub annotations: Value,
    #[serde(rename = "inputSchema", default)]
    pub input_schema: Value,
}

pub struct McpClient {
    base: String,
    client: reqwest::Client,
}

impl McpClient {
    pub fn new(base: impl Into<String>) -> Self {
        let base = base.into();
        Self {
            base: base.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// GET /mcp/healthz - true on 2xx.
    pub async fn health(&self) -> bool {
        match self.client.get(format!("{}/mcp/healthz", self.base)).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }

    /// One JSON-RPC 2.0 call. Returns the `result` value, or an `[code] message` error string.
    async fn rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let body = json!({ "jsonrpc": "2.0", "id": 1, "method": method, "params": params });
        let resp = self
            .client
            .post(format!("{}/mcp", self.base))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("request failed: {e} (is the mcp-gateway running?)"))?;
        let v: Value = resp.json().await.map_err(|e| format!("response parse failed: {e}"))?;
        if let Some(err) = v.get("error") {
            let code = err.get("code").and_then(Value::as_i64).unwrap_or(0);
            let msg = err.get("message").and_then(Value::as_str).unwrap_or("error");
            return Err(format!("[{code}] {msg}"));
        }
        v.get("result").cloned().ok_or_else(|| "missing result in JSON-RPC response".to_string())
    }

    /// All tools, following `nextCursor` pagination.
    pub async fn list_tools(&self) -> Result<Vec<ToolInfo>, String> {
        let mut out: Vec<ToolInfo> = Vec::new();
        let mut cursor: Option<String> = None;
        loop {
            let params = match &cursor {
                Some(c) => json!({ "cursor": c }),
                None => json!({}),
            };
            let result = self.rpc("tools/list", params).await?;
            if let Some(arr) = result.get("tools").and_then(Value::as_array) {
                for t in arr {
                    let info: ToolInfo =
                        serde_json::from_value(t.clone()).map_err(|e| format!("tool parse failed: {e}"))?;
                    out.push(info);
                }
            }
            cursor = result.get("nextCursor").and_then(Value::as_str).map(str::to_string);
            if cursor.is_none() {
                break;
            }
        }
        Ok(out)
    }

    /// Invoke a tool. Returns the `tools/call` result (content blocks etc.) or an error string.
    pub async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, String> {
        self.rpc("tools/call", json!({ "name": name, "arguments": arguments })).await
    }
}
