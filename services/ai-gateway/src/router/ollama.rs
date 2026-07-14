//! TASK-AI-008 - Ollama provider (local / self-hosted models, no API key).
//!
//! Ollama serves models over an HTTP API on a local endpoint (default `http://localhost:11434`). Unlike
//! the cloud providers it needs no API key, which makes it the natural local-dev and self-hosted backend
//! for the gateway. The endpoint comes from `OLLAMA_ENDPOINT` (one Ollama per deployment is the norm);
//! the per-tenant policy's `model_alias_map` maps a CyberOS alias (`chat.smart`) to an Ollama model
//! (`llama3.1:8b`). It implements the same `Provider` trait the router drives, so it inherits the router's
//! retry + failover for free. The request build and response parse are pure functions, unit-tested here;
//! the live HTTP call needs a running Ollama, so it is exercised by an owner-run integration check.

use std::time::Instant;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::types::*;
use super::{Provider, RouterError};
use crate::policy::ProviderKind;

const DEFAULT_ENDPOINT: &str = "http://localhost:11434";

/// An Ollama provider bound to one endpoint.
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    endpoint: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    /// Bind to an explicit endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Build from `OLLAMA_ENDPOINT` (default `http://localhost:11434`).
    pub fn from_env() -> Self {
        let endpoint = std::env::var("OLLAMA_ENDPOINT")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
        Self::new(endpoint)
    }

    /// Build the Ollama `/api/chat` request body. Pure - unit-tested.
    fn build_request(req: &ChatCompleteRequest, model: &str) -> Value {
        let messages: Vec<Value> = req
            .messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();
        let mut options = serde_json::Map::new();
        if let Some(t) = req.temperature {
            options.insert("temperature".to_string(), json!(t));
        }
        if let Some(n) = req.max_tokens {
            options.insert("num_predict".to_string(), json!(n));
        }
        let mut body = json!({ "model": model, "messages": messages, "stream": false });
        if !options.is_empty() {
            body["options"] = Value::Object(options);
        }
        body
    }

    /// Map an Ollama `/api/chat` response to a `ProviderResponse`. Pure - unit-tested.
    fn parse_response(body: &Value, latency_ms: u32) -> Result<ProviderResponse, RouterError> {
        let content = body
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| RouterError::InvalidResponse {
                reason: "ollama response missing message.content".into(),
            })?
            .to_string();
        let prompt_tokens = body
            .get("prompt_eval_count")
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let completion_tokens = body.get("eval_count").and_then(Value::as_u64).unwrap_or(0) as u32;
        let finish_reason = match body.get("done_reason").and_then(Value::as_str) {
            Some("length") => FinishReason::Length,
            Some("stop") | None => FinishReason::Stop,
            Some(_) => FinishReason::Other,
        };
        Ok(ProviderResponse {
            id: format!("ollama-{}", uuid::Uuid::new_v4()),
            usage: ProviderUsage {
                prompt_tokens,
                completion_tokens,
                cached_input_tokens: 0,
            },
            choices: vec![Choice {
                index: 0,
                content,
                tool_calls: vec![],
                finish_reason,
            }],
            finish_reason,
            latency_ms,
            cache_state: CacheState::None,
            attempts: vec![],
        })
    }
}

#[async_trait]
impl Provider for OllamaProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Ollama
    }

    async fn call_chat(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        let started = Instant::now();
        let timeout = deadline.saturating_duration_since(Instant::now());
        if timeout.is_zero() {
            return Err(RouterError::DeadlineExceeded);
        }
        let url = format!("{}/api/chat", self.endpoint.trim_end_matches('/'));
        let resp = self
            .client
            .post(&url)
            .timeout(timeout)
            .json(&Self::build_request(req, model))
            .send()
            .await
            .map_err(|e| RouterError::InvalidResponse {
                reason: format!("ollama request failed: {e}"),
            })?;
        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(RouterError::TerminalProviderError {
                provider: ProviderKind::Ollama,
                status: code,
                message,
                retry_after_secs: None,
            });
        }
        let body: Value = resp
            .json()
            .await
            .map_err(|e| RouterError::InvalidResponse {
                reason: format!("ollama response parse: {e}"),
            })?;
        let latency_ms = started.elapsed().as_millis() as u32;
        Self::parse_response(&body, latency_ms)
    }

    async fn call_embed(
        &self,
        _req: &EmbedRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        Err(RouterError::InvalidResponse {
            reason: "ollama embed not yet wired (use /api/embeddings)".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> ChatCompleteRequest {
        ChatCompleteRequest {
            alias: "chat.smart".into(),
            messages: vec![
                Message {
                    role: "system".into(),
                    content: "be terse".into(),
                },
                Message {
                    role: "user".into(),
                    content: "hello".into(),
                },
            ],
            max_tokens: Some(256),
            // 0.5 is exact in both f32 and f64, so the JSON number compares cleanly (0.2_f32
            // widened to f64 is 0.20000000298..., which would not equal the literal 0.2).
            temperature: Some(0.5),
            traceparent: None,
            tracestate: None,
        }
    }

    #[test]
    fn build_request_maps_messages_model_and_options() {
        let b = OllamaProvider::build_request(&req(), "llama3.1:8b");
        assert_eq!(b["model"], "llama3.1:8b");
        assert_eq!(b["stream"], false);
        assert_eq!(b["messages"].as_array().unwrap().len(), 2);
        assert_eq!(b["messages"][1]["role"], "user");
        assert_eq!(b["options"]["temperature"], 0.5);
        assert_eq!(b["options"]["num_predict"], 256);
    }

    #[test]
    fn build_request_omits_options_when_absent() {
        let mut r = req();
        r.temperature = None;
        r.max_tokens = None;
        let b = OllamaProvider::build_request(&r, "m");
        assert!(b.get("options").is_none());
    }

    #[test]
    fn parse_response_extracts_content_tokens_and_finish() {
        let body = json!({
            "model": "llama3.1:8b",
            "message": { "role": "assistant", "content": "hi there" },
            "done": true,
            "done_reason": "stop",
            "prompt_eval_count": 12,
            "eval_count": 7
        });
        let r = OllamaProvider::parse_response(&body, 42).unwrap();
        assert_eq!(r.choices[0].content, "hi there");
        assert_eq!(r.usage.prompt_tokens, 12);
        assert_eq!(r.usage.completion_tokens, 7);
        assert_eq!(r.finish_reason, FinishReason::Stop);
        assert_eq!(r.latency_ms, 42);
        assert!(r.id.starts_with("ollama-"));
    }

    #[test]
    fn parse_response_maps_length_finish_reason() {
        let body = json!({ "message": { "content": "x" }, "done_reason": "length" });
        assert_eq!(
            OllamaProvider::parse_response(&body, 1)
                .unwrap()
                .finish_reason,
            FinishReason::Length
        );
    }

    #[test]
    fn parse_response_errors_on_missing_content() {
        let body = json!({ "done": true });
        assert!(OllamaProvider::parse_response(&body, 1).is_err());
    }

    #[test]
    fn kind_is_ollama() {
        assert_eq!(OllamaProvider::new("http://x").kind(), ProviderKind::Ollama);
    }

    #[tokio::test]
    async fn call_chat_fails_closed_on_unreachable_server() {
        // Nothing listens on 127.0.0.1:1, so the call must surface an Err - never a fabricated
        // completion (TASK-AI-105 clause 4, fail-closed). Deterministic: no server required.
        let p = OllamaProvider::new("http://127.0.0.1:1");
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        assert!(p.call_chat(&req(), "llama3.1:8b", deadline).await.is_err());
    }
}
