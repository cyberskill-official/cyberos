//! FR-AI-008 — OpenAI API provider implementation.

use std::time::Instant;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::types::*;
use super::{Provider, RouterError};
use crate::policy::ProviderKind;

/// OpenAI API provider.
#[derive(Debug)]
pub struct OpenAIProvider;

#[async_trait]
impl Provider for OpenAIProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Openai
    }

    async fn call_chat(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        let base_url = std::env::var("CYBEROS_AI_GATEWAY_OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com".to_string());
        let url = format!("{}/v1/chat/completions", base_url.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let mut builder = client.post(url).json(&json!({
            "model": model,
            "messages": req.messages.iter().map(|message| {
                json!({
                    "role": message.role,
                    "content": message.content,
                })
            }).collect::<Vec<_>>(),
            "max_tokens": req.max_tokens,
            "temperature": req.temperature,
        }));
        if let Ok(api_key) = std::env::var("CYBEROS_AI_GATEWAY_OPENAI_API_KEY") {
            builder = builder.bearer_auth(api_key);
        }
        builder = super::http::apply_trace_headers(builder, req);

        let response =
            super::http::send_with_deadline(builder, deadline, ProviderKind::Openai).await?;
        if !response.status().is_success() {
            return Err(super::http::error_from_response(ProviderKind::Openai, response).await);
        }
        let raw: OpenAiChatResponse =
            response
                .json()
                .await
                .map_err(|err| RouterError::InvalidResponse {
                    reason: format!("openai response json parse failed: {err}"),
                })?;
        normalize_openai(raw)
    }

    async fn call_embed(
        &self,
        _req: &EmbedRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        Err(RouterError::InvalidResponse {
            reason: "OpenAI embed not yet wired".into(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiChatResponse {
    id: String,
    usage: OpenAiUsage,
    choices: Vec<OpenAiChoice>,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    index: Option<u8>,
    message: Option<OpenAiMessage>,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiMessage {
    content: Option<String>,
}

fn normalize_openai(raw: OpenAiChatResponse) -> Result<ProviderResponse, RouterError> {
    if raw.choices.is_empty() {
        return Err(RouterError::InvalidResponse {
            reason: "openai response missing choices".into(),
        });
    }

    let choices = raw
        .choices
        .into_iter()
        .map(|choice| {
            let finish_reason =
                super::normalize::finish_reason_from_provider(choice.finish_reason.as_deref());
            Choice {
                index: choice.index.unwrap_or(0),
                content: choice
                    .message
                    .and_then(|message| message.content)
                    .unwrap_or_default(),
                tool_calls: vec![],
                finish_reason,
            }
        })
        .collect::<Vec<_>>();
    let finish_reason = choices
        .first()
        .map(|choice| choice.finish_reason)
        .unwrap_or(FinishReason::Other);

    Ok(ProviderResponse {
        id: raw.id,
        usage: ProviderUsage {
            prompt_tokens: raw.usage.prompt_tokens,
            completion_tokens: raw.usage.completion_tokens,
            cached_input_tokens: 0,
        },
        choices,
        finish_reason,
        latency_ms: 0,
        cache_state: CacheState::None,
        attempts: vec![],
    })
}
