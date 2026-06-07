//! FR-AI-008 §3 — Failover chain construction.

use crate::alias::ResolvedModel;
use crate::policy::{ProviderKind, TenantPolicy};

use super::Provider;

/// A concrete provider/model endpoint in the failover chain.
pub struct ProviderEndpoint {
    pub(crate) provider: Box<dyn Provider>,
    pub(crate) model: String,
    pub(crate) fallback_position: u8,
}

impl ProviderEndpoint {
    pub fn new(
        provider: Box<dyn Provider>,
        model: impl Into<String>,
        fallback_position: u8,
    ) -> Self {
        Self {
            provider,
            model: model.into(),
            fallback_position,
        }
    }

    pub fn provider_kind(&self) -> ProviderKind {
        self.provider.kind()
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub fn fallback_position(&self) -> u8 {
        self.fallback_position
    }
}

impl std::fmt::Debug for ProviderEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProviderEndpoint")
            .field("provider_kind", &self.provider.kind())
            .field("model", &self.model)
            .field("fallback_position", &self.fallback_position)
            .finish()
    }
}

/// Build the ordered `(Provider impl, model name)` chain for this call.
///
/// Position 0 = primary; positions 1.. = fallback chain in declared order.
/// Providers that don't carry the requested alias are skipped.
pub fn build_provider_chain(
    resolved: &ResolvedModel,
    policy: &TenantPolicy,
    alias: &str,
) -> Vec<ProviderEndpoint> {
    let mut chain = Vec::new();

    if let Some(provider) = make_provider(resolved.provider_kind) {
        chain.push(ProviderEndpoint::new(
            provider,
            resolved.model.clone(),
            resolved.fallback_position,
        ));
    }

    // Fallback chain. If `resolved` already came from fallback N, continue at
    // N+1 so the same provider/model is not attempted twice.
    for (idx, fb) in policy.ai_policy.fallback_chain.iter().enumerate() {
        let fallback_position = (idx + 1) as u8;
        if fallback_position <= resolved.fallback_position {
            continue;
        }
        if let Some(model) = fb.model_for_alias(alias) {
            if let Some(provider) = make_provider(fb.kind()) {
                chain.push(ProviderEndpoint::new(
                    provider,
                    model.to_string(),
                    fallback_position,
                ));
            }
        }
    }

    chain
}

fn make_provider(kind: ProviderKind) -> Option<Box<dyn Provider>> {
    match kind {
        ProviderKind::Bedrock => Some(Box::new(super::bedrock::BedrockProvider)),
        ProviderKind::Anthropic => Some(Box::new(super::anthropic::AnthropicProvider)),
        ProviderKind::Openai => Some(Box::new(super::openai::OpenAIProvider)),
        ProviderKind::Vertex | ProviderKind::Bge => None,
    }
}
