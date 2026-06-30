//! FR-AI-105 - LM Studio / OpenAI-compatible local provider (no API key).
//!
//! LM Studio, llama.cpp's server, vLLM, and text-generation-webui all expose the OpenAI chat-completions
//! shape on a local endpoint (LM Studio defaults to `http://localhost:1234`). This adapter speaks that
//! shape: `POST {endpoint}/v1/chat/completions` with `{model, messages, stream:false}`. Like the Ollama
//! adapter it needs no API key, reads its endpoint from `LMSTUDIO_ENDPOINT` (one local server per
//! deployment), and fails closed - an unreachable server returns a `RouterError`, never a fabricated
//! completion. The request build and response parse are pure and unit-tested here; the live call is
//! covered by an owner-run integration check against a running LM Studio.

use std::time::Instant;

use async_trait::async_trait;
use serde_json::{json, Value};

use super::types::*;
use super::{Provider, RouterError};
use crate::policy::ProviderKind;

const DEFAULT_ENDPOINT: &str = "http://localhost:1234";

/// An OpenAI-compatible local provider (LM Studio and friends) bound to one endpoint.
#[derive(Debug, Clone)]
pub struct LocalOpenaiProvider {
    endpoint: String,
    client: reqwest::Client,
}

impl LocalOpenaiProvider {
    /// Bind to an explicit endpoint.
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Build from `LMSTUDIO_ENDPOINT` (default `http://localhost:1234`). `OPENAI_COMPAT_ENDPOINT` is
    /// accepted as an alias for non-LM-Studio OpenAI-compatible runtimes.
    pub fn from_env() -> Self {
        let endpoint = std::env::var("LMSTUDIO_ENDPOINT")
            .or_else(|_| std::env::var("OPENAI_COMPAT_ENDPOINT"))
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_ENDPOINT.to_string());
        Self::new(endpoint)
    }

    /// Build the OpenAI `/v1/chat/completions` request body. Pure - unit-tested.
    fn build_request(req: &ChatCompleteRequest, model: &str) -> Value {
        let messages: Vec<Value> = req
            .messages
            .iter()
            .map(|m| json!({ "role": m.role, "content": m.content }))
            .collect();
        let mut body = json!({ "model": model, "messages": messages, "stream": false });
        if let Some(t) = req.temperature {
            body["temperature"] = json!(t);
        }
        if let Some(n) = req.max_tokens {
            body["max_tokens"] = json!(n);
        }
        body
    }

    /// Map an OpenAI `/v1/chat/completions` response to a `ProviderResponse`. Pure - unit-tested.
    fn parse_response(body: &Value, latency_ms: u32) -> Result<ProviderResponse, RouterError> {
        let choice0 = body
            .get("choices")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .ok_or_else(|| RouterError::InvalidResponse {
                reason: "openai-compatible response missing choices[0]".into(),
            })?;
        let content = choice0
            .get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| RouterError::InvalidResponse {
                reason: "openai-compatible response missing choices[0].message.content".into(),
            })?
            .to_string();
        let finish_reason = match choice0.get("finish_reason").and_then(Value::as_str) {
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("content_filter") => FinishReason::ContentFilter,
            Some("stop") | None => FinishReason::Stop,
            Some(_) => FinishReason::Other,
        };
        let usage = body.get("usage");
        let prompt_tokens = usage
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let completion_tokens = usage
            .and_then(|u| u.get("completion_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        let id = body
            .get("id")
            .and_then(Value::as_str)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("local-{}", uuid::Uuid::new_v4()));
        Ok(ProviderResponse {
            id,
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

    /// Build the OpenAI `/v1/embeddings` request body. Pure - unit-tested.
    fn build_embed_request(req: &EmbedRequest, model: &str) -> Value {
        json!({ "model": model, "input": req.input })
    }

    /// Map an OpenAI `/v1/embeddings` response to an `EmbedResponse`. Pure - unit-tested. The shape is the
    /// OpenAI embeddings response (`{ "data": [{ "embedding": [..] }], "usage": { "prompt_tokens": n } }`),
    /// which LM Studio, llama.cpp's server, and vLLM all serve for a loaded embedding model.
    fn parse_embed_response(body: &Value) -> Result<EmbedResponse, RouterError> {
        let data = body.get("data").and_then(|d| d.as_array()).ok_or_else(|| {
            RouterError::InvalidResponse {
                reason: "openai-compatible embeddings response missing data[]".into(),
            }
        })?;
        let mut embeddings = Vec::with_capacity(data.len());
        for item in data {
            let arr = item
                .get("embedding")
                .and_then(|e| e.as_array())
                .ok_or_else(|| RouterError::InvalidResponse {
                    reason: "embeddings response item missing embedding[]".into(),
                })?;
            let vector = arr
                .iter()
                .map(|v| v.as_f64().map(|f| f as f32))
                .collect::<Option<Vec<f32>>>()
                .ok_or_else(|| RouterError::InvalidResponse {
                    reason: "embedding contained a non-numeric value".into(),
                })?;
            embeddings.push(vector);
        }
        if embeddings.is_empty() {
            return Err(RouterError::InvalidResponse {
                reason: "embeddings response contained no vectors".into(),
            });
        }
        let prompt_tokens = body
            .get("usage")
            .and_then(|u| u.get("prompt_tokens"))
            .and_then(Value::as_u64)
            .unwrap_or(0) as u32;
        Ok(EmbedResponse {
            embeddings,
            usage: ProviderUsage {
                prompt_tokens,
                completion_tokens: 0,
                cached_input_tokens: 0,
            },
        })
    }
}

#[async_trait]
impl Provider for LocalOpenaiProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::LocalOpenai
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
        let url = format!(
            "{}/v1/chat/completions",
            self.endpoint.trim_end_matches('/')
        );
        let resp = self
            .client
            .post(&url)
            .timeout(timeout)
            .json(&Self::build_request(req, model))
            .send()
            .await
            .map_err(|e| RouterError::InvalidResponse {
                reason: format!("local-openai request failed: {e}"),
            })?;
        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(RouterError::TerminalProviderError {
                provider: ProviderKind::LocalOpenai,
                status: code,
                message,
                retry_after_secs: None,
            });
        }
        let body: Value = resp
            .json()
            .await
            .map_err(|e| RouterError::InvalidResponse {
                reason: format!("local-openai response parse: {e}"),
            })?;
        let latency_ms = started.elapsed().as_millis() as u32;
        Self::parse_response(&body, latency_ms)
    }

    async fn call_embed(
        &self,
        req: &EmbedRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        let timeout = deadline.saturating_duration_since(Instant::now());
        if timeout.is_zero() {
            return Err(RouterError::DeadlineExceeded);
        }
        let url = format!("{}/v1/embeddings", self.endpoint.trim_end_matches('/'));
        let resp = self
            .client
            .post(&url)
            .timeout(timeout)
            .json(&Self::build_embed_request(req, model))
            .send()
            .await
            .map_err(|e| RouterError::InvalidResponse {
                reason: format!("local-openai embed request failed: {e}"),
            })?;
        let status = resp.status();
        if !status.is_success() {
            let code = status.as_u16();
            let message = resp.text().await.unwrap_or_default();
            return Err(RouterError::TerminalProviderError {
                provider: ProviderKind::LocalOpenai,
                status: code,
                message,
                retry_after_secs: None,
            });
        }
        let body: Value = resp
            .json()
            .await
            .map_err(|e| RouterError::InvalidResponse {
                reason: format!("local-openai embed response parse: {e}"),
            })?;
        Self::parse_embed_response(&body)
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
            max_tokens: Some(128),
            // 0.5 is exact in both f32 and f64, so the JSON number compares cleanly (0.3_f32
            // widened to f64 is 0.30000001192..., which would not equal the literal 0.3).
            temperature: Some(0.5),
            traceparent: None,
            tracestate: None,
        }
    }

    #[test]
    fn build_request_uses_openai_shape() {
        let b = LocalOpenaiProvider::build_request(&req(), "qwen2.5-7b-instruct");
        assert_eq!(b["model"], "qwen2.5-7b-instruct");
        assert_eq!(b["stream"], false);
        assert_eq!(b["messages"].as_array().unwrap().len(), 2);
        assert_eq!(b["temperature"], 0.5);
        assert_eq!(b["max_tokens"], 128);
    }

    #[test]
    fn parse_response_reads_choices_and_usage() {
        let body = json!({
            "id": "chatcmpl-local-1",
            "choices": [
                { "index": 0, "message": { "role": "assistant", "content": "hi there" }, "finish_reason": "stop" }
            ],
            "usage": { "prompt_tokens": 9, "completion_tokens": 4, "total_tokens": 13 }
        });
        let r = LocalOpenaiProvider::parse_response(&body, 21).unwrap();
        assert_eq!(r.choices[0].content, "hi there");
        assert_eq!(r.usage.prompt_tokens, 9);
        assert_eq!(r.usage.completion_tokens, 4);
        assert_eq!(r.finish_reason, FinishReason::Stop);
        assert_eq!(r.id, "chatcmpl-local-1");
        assert_eq!(r.latency_ms, 21);
    }

    #[test]
    fn parse_response_maps_length_finish() {
        let body = json!({
            "choices": [ { "message": { "content": "x" }, "finish_reason": "length" } ]
        });
        assert_eq!(
            LocalOpenaiProvider::parse_response(&body, 1)
                .unwrap()
                .finish_reason,
            FinishReason::Length
        );
    }

    #[test]
    fn parse_response_synthesizes_id_when_absent() {
        let body = json!({ "choices": [ { "message": { "content": "x" } } ] });
        let r = LocalOpenaiProvider::parse_response(&body, 1).unwrap();
        assert!(r.id.starts_with("local-"));
    }

    #[test]
    fn parse_response_errors_on_missing_choices() {
        let body = json!({ "usage": { "prompt_tokens": 1 } });
        assert!(LocalOpenaiProvider::parse_response(&body, 1).is_err());
    }

    #[test]
    fn kind_is_local_openai() {
        assert_eq!(
            LocalOpenaiProvider::new("http://x").kind(),
            ProviderKind::LocalOpenai
        );
    }

    #[tokio::test]
    async fn call_chat_fails_closed_on_unreachable_server() {
        // Nothing listens on 127.0.0.1:1, so the call must surface an Err - never a fabricated
        // completion (FR-AI-105 clause 4, fail-closed). Deterministic: no server required.
        let p = LocalOpenaiProvider::new("http://127.0.0.1:1");
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        assert!(p
            .call_chat(&req(), "qwen2.5-7b-instruct", deadline)
            .await
            .is_err());
    }

    #[test]
    fn build_embed_request_uses_openai_shape() {
        let r = EmbedRequest {
            input: vec!["hello".into(), "world".into()],
            model: "bge-m3".into(),
        };
        let b = LocalOpenaiProvider::build_embed_request(&r, "bge-m3");
        assert_eq!(b["model"], "bge-m3");
        assert_eq!(b["input"].as_array().unwrap().len(), 2);
        assert_eq!(b["input"][0], "hello");
    }

    #[test]
    fn parse_embed_response_reads_vectors_and_usage() {
        let body = json!({
            "object": "list",
            "data": [ { "object": "embedding", "index": 0, "embedding": [0.1, 0.2, 0.3] } ],
            "model": "bge-m3",
            "usage": { "prompt_tokens": 4, "total_tokens": 4 }
        });
        let r = LocalOpenaiProvider::parse_embed_response(&body).unwrap();
        assert_eq!(r.embeddings.len(), 1);
        assert_eq!(r.embeddings[0].len(), 3);
        assert_eq!(r.usage.prompt_tokens, 4);
        assert_eq!(r.usage.completion_tokens, 0);
    }

    #[test]
    fn parse_embed_response_errors_on_missing_data() {
        assert!(LocalOpenaiProvider::parse_embed_response(
            &json!({ "usage": { "prompt_tokens": 1 } })
        )
        .is_err());
    }

    #[test]
    fn parse_embed_response_errors_on_empty_data() {
        assert!(LocalOpenaiProvider::parse_embed_response(&json!({ "data": [] })).is_err());
    }

    #[tokio::test]
    async fn call_embed_fails_closed_on_unreachable_server() {
        // Nothing listens on 127.0.0.1:1, so call_embed must surface an Err, never a fabricated vector.
        let p = LocalOpenaiProvider::new("http://127.0.0.1:1");
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        let r = EmbedRequest {
            input: vec!["x".into()],
            model: "bge-m3".into(),
        };
        assert!(p.call_embed(&r, "bge-m3", deadline).await.is_err());
    }
}
