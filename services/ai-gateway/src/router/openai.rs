//! FR-AI-008 — OpenAI API provider implementation.

use std::time::Instant;

use async_trait::async_trait;

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
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        // TODO: Wire reqwest call to api.openai.com/v1/chat/completions.
        Err(RouterError::InvalidResponse {
            reason: "OpenAI provider not yet wired".into(),
        })
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
