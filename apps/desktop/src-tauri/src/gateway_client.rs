//! HTTP client for the CyberOS gateway. The Rust backend owns these calls (not the webview), so the app
//! never hits browser CORS - the webview only talks to Tauri commands, which call this.

use serde::{Deserialize, Serialize};

/// One chat turn, matching the gateway's `/v1/chat` message shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
}

/// A thin client bound to one gateway base URL.
pub struct GatewayClient {
    base: String,
    client: reqwest::Client,
}

impl GatewayClient {
    pub fn new(base: impl Into<String>) -> Self {
        let base = base.into();
        Self {
            base: base.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }

    /// GET /healthz - true on 2xx.
    pub async fn health(&self) -> bool {
        match self.client.get(format!("{}/healthz", self.base)).send().await {
            Ok(r) => r.status().is_success(),
            Err(_) => false,
        }
    }

    /// POST /v1/chat. The optional bearer token (from the OS keychain) is attached when present.
    pub async fn chat(
        &self,
        tenant: &str,
        alias: &str,
        messages: &[ChatTurn],
        token: Option<&str>,
    ) -> Result<serde_json::Value, String> {
        let mut req = self
            .client
            .post(format!("{}/v1/chat", self.base))
            .header("x-tenant-id", tenant)
            .json(&serde_json::json!({ "alias": alias, "messages": messages }));
        if let Some(t) = token {
            req = req.bearer_auth(t);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| format!("request failed: {e} (is the gateway running?)"))?;
        let status = resp.status();
        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("response parse failed: {e}"))?;
        if !status.is_success() {
            let err = body.get("error").and_then(|e| e.as_str()).unwrap_or("unknown error");
            return Err(format!("[{}] {}", status.as_u16(), err));
        }
        Ok(body)
    }
}
