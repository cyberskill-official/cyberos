//! TASK-AI-008 — AWS Bedrock provider implementation.

use std::time::Instant;

use async_trait::async_trait;

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
        _req: &ChatCompleteRequest,
        _model: &str,
        _deadline: Instant,
    ) -> Result<ProviderResponse, RouterError> {
        // TODO: Wire aws-sdk-bedrockruntime InvokeModel call.
        // For now, returns a stub error so the router compiles.
        Err(RouterError::InvalidResponse {
            reason: "Bedrock provider not yet wired".into(),
        })
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
}
