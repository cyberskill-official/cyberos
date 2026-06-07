//! FR-AI-008 — AWS Bedrock provider implementation.

use std::time::Instant;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;

use super::types::*;
use super::{Provider, RouterError};
use crate::policy::ProviderKind;

/// AWS Bedrock provider.
#[derive(Debug)]
pub struct BedrockProvider;

#[async_trait]
impl Provider for BedrockProvider {
    fn kind(&self) -> ProviderKind {
        ProviderKind::Bedrock
    }

    async fn call_chat(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        let region = std::env::var("CYBEROS_AI_GATEWAY_BEDROCK_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
        let base_url = std::env::var("CYBEROS_AI_GATEWAY_BEDROCK_BASE_URL")
            .unwrap_or_else(|_| format!("https://bedrock-runtime.{region}.amazonaws.com"));
        let url = format!("{}/model/{}/invoke", base_url.trim_end_matches('/'), model);
        let client = reqwest::Client::new();
        let mut builder = client
            .post(url)
            .header("content-type", "application/json")
            .json(&json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": req.messages.iter().map(|message| {
                    json!({
                        "role": message.role,
                        "content": message.content,
                    })
                }).collect::<Vec<_>>(),
                "max_tokens": req.max_tokens.unwrap_or(1024),
                "temperature": req.temperature,
            }));
        if let Ok(api_key) = std::env::var("CYBEROS_AI_GATEWAY_BEDROCK_API_KEY") {
            builder = builder.bearer_auth(api_key);
        }
        builder = super::http::apply_trace_headers(builder, req);

        let response =
            super::http::send_with_deadline(builder, deadline, ProviderKind::Bedrock).await?;
        if !response.status().is_success() {
            return Err(super::http::error_from_response(ProviderKind::Bedrock, response).await);
        }
        let raw: BedrockMessageResponse =
            response
                .json()
                .await
                .map_err(|err| RouterError::InvalidResponse {
                    reason: format!("bedrock response json parse failed: {err}"),
                })?;
        normalize_bedrock(raw, model)
    }

    async fn call_embed(
        &self,
        _req: &EmbedRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<EmbedResponse, RouterError> {
        Err(RouterError::InvalidResponse {
            reason: "Bedrock embed not yet wired".into(),
        })
    }

    async fn call_chat_streaming(
        &self,
        req: &ChatCompleteRequest,
        model: &str,
        deadline: Instant,
    ) -> Result<ProviderStreamResponse, RouterError> {
        let region = std::env::var("CYBEROS_AI_GATEWAY_BEDROCK_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
        let base_url = std::env::var("CYBEROS_AI_GATEWAY_BEDROCK_BASE_URL")
            .unwrap_or_else(|_| format!("https://bedrock-runtime.{region}.amazonaws.com"));
        let url = format!(
            "{}/model/{}/invoke-with-response-stream",
            base_url.trim_end_matches('/'),
            model
        );
        let client = reqwest::Client::new();
        let mut builder = client
            .post(url)
            .header("content-type", "application/json")
            .json(&json!({
                "anthropic_version": "bedrock-2023-05-31",
                "messages": req.messages.iter().map(|message| {
                    json!({
                        "role": message.role,
                        "content": message.content,
                    })
                }).collect::<Vec<_>>(),
                "max_tokens": req.max_tokens.unwrap_or(1024),
                "temperature": req.temperature,
            }));
        if let Ok(api_key) = std::env::var("CYBEROS_AI_GATEWAY_BEDROCK_API_KEY") {
            builder = builder.bearer_auth(api_key);
        }
        builder = super::http::apply_trace_headers(builder, req);

        let response =
            super::http::send_with_deadline(builder, deadline, ProviderKind::Bedrock).await?;
        if !response.status().is_success() {
            return Err(super::http::error_from_response(ProviderKind::Bedrock, response).await);
        }
        Ok(super::streaming::response_to_provider_stream(
            response,
            ProviderKind::Bedrock,
            super::streaming::StreamDialect::Bedrock,
        ))
    }
}

#[derive(Debug, Deserialize)]
struct BedrockMessageResponse {
    id: Option<String>,
    content: Vec<BedrockContentBlock>,
    usage: Option<BedrockUsage>,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct BedrockContentBlock {
    #[serde(default)]
    text: String,
}

#[derive(Debug, Deserialize)]
struct BedrockUsage {
    input_tokens: u32,
    output_tokens: u32,
}

fn normalize_bedrock(
    raw: BedrockMessageResponse,
    model: &str,
) -> Result<ProviderResponse, RouterError> {
    let content = raw
        .content
        .into_iter()
        .map(|block| block.text)
        .collect::<Vec<_>>()
        .join("");
    if content.is_empty() {
        return Err(RouterError::InvalidResponse {
            reason: "bedrock response missing text content".into(),
        });
    }
    let usage = raw.usage.ok_or_else(|| RouterError::InvalidResponse {
        reason: "bedrock response missing usage".into(),
    })?;

    let finish_reason = super::normalize::finish_reason_from_provider(raw.stop_reason.as_deref());
    Ok(ProviderResponse {
        id: raw.id.unwrap_or_else(|| format!("bedrock:{model}")),
        usage: ProviderUsage {
            prompt_tokens: usage.input_tokens,
            completion_tokens: usage.output_tokens,
            cached_input_tokens: 0,
        },
        choices: vec![super::normalize::text_choice(content, finish_reason)],
        finish_reason,
        latency_ms: 0,
        cache_state: CacheState::None,
        attempts: vec![],
    })
}
