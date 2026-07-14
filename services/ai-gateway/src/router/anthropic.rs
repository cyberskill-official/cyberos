//! TASK-AI-008 — Anthropic API provider implementation.

use std::time::Instant;

use async_trait::async_trait;

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
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        // TODO: Wire reqwest call to api.anthropic.com/v1/messages.
        Err(RouterError::InvalidResponse {
            reason: "Anthropic provider not yet wired".into(),
        })
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
