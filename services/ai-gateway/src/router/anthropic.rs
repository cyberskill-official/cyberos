//! FR-AI-008 — Anthropic API provider implementation.

use std::time::Instant;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::types::*;
use super::{Provider, RouterError};
use crate::policy::ProviderKind;

/// Anthropic API provider.
#[derive(Debug)]
pub struct AnthropicProvider;

#[async_trait]
impl Provider for AnthropicProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Anthropic
    }

    async fn call_chat(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        let base_url = std::env::var("CYBEROS_AI_GATEWAY_ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com".to_string());
        let url = format!("{}/v1/messages", base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let mut builder = client
            .post(url)
            .header("anthropic-version", "2023-06-01")
            .json(&json!({
                "model": model,
                "messages": req.messages.iter().map(|message| {
                    json!({
                        "role": message.role,
                        "content": message.content,
                    })
                }).collect::<Vec<_>>(),
                "max_tokens": req.max_tokens.unwrap_or(1024),
                "temperature": req.temperature,
            }));
        if let Ok(api_key) = std::env::var("CYBEROS_AI_GATEWAY_ANTHROPIC_API_KEY") {
            builder = builder.header("x-api-key", api_key);
        }
        builder = super::http::apply_trace_headers(builder, req);

        let response =
            super::http::send_with_deadline(builder, deadline, ProviderKind::Anthropic).await?;
        if !response.status().is_success() {
            return Err(super::http::error_from_response(ProviderKind::Anthropic, response).await);
        }
        let raw: AnthropicMessageResponse =
            response
                .json()
                .await
                .map_err(|err| RouterError::InvalidResponse {
                    reason: format!("anthropic response json parse failed: {err}"),
                })?;
        normalize_anthropic(raw)
    }

    async fn call_embed(
        &self,
        _req: &EmbedRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        Err(RouterError::InvalidResponse {
            reason: "Anthropic embed not yet wired".into(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicMessageResponse {
    id: String,
    content: Vec<AnthropicContentBlock>,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

fn normalize_anthropic(raw: AnthropicMessageResponse) -> Result<ProviderResponse, RouterError> {
    let content = raw
        .content
        .into_iter()
        .map(|block| block.text)
        .collect::<Vec<_>>()
        .join("");
    if content.is_empty() {
        return Err(RouterError::InvalidResponse {
            reason: "anthropic response missing text content".into(),
        });
    }

    let finish_reason = super::normalize::finish_reason_from_provider(raw.stop_reason.as_deref());
    Ok(ProviderResponse {
        id: raw.id,
        usage: ProviderUsage {
            prompt_tokens: raw.usage.input_tokens,
            completion_tokens: raw.usage.output_tokens,
            cached_input_tokens: 0,
        },
        choices: vec![super::normalize::text_choice(content, finish_reason)],
        finish_reason,
        latency_ms: 0,
        cache_state: CacheState::None,
        attempts: vec![],
    })
}
