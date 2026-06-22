//! FR-AI-008 §3 — Failover chain construction.

use crate::alias::ResolvedModel;
use crate::policy::{ProviderKind, TenantPolicy};

use super::Provider;

/// Build the ordered `(Provider impl, model name)` chain for this call.
///
/// Position 0 = primary; positions 1.. = fallback chain in declared order.
/// Providers that don't carry the requested alias are skipped.
pub fn build_provider_chain(
    resolved: &ResolvedModel,
    policy: &TenantPolicy,
    alias: &str,
) -> Vec<(Box<dyn Provider>, String)> {
    let mut chain: Vec<(Box<dyn Provider>, String)> = Vec::new();

    // Primary provider
    chain.push((make_provider(resolved.provider_kind), resolved.model.clone()));

    // Fallback chain
    for fb in &policy.ai_policy.fallback_chain {
        if let Some(model) = fb.model_for_alias(alias) {
            chain.push((make_provider(fb.kind()), model.to_string()));
        }
    }

    chain
}

fn make_provider(kind: ProviderKind) -> Box<dyn Provider> {
    match kind {
        ProviderKind::Bedrock => Box::new(super::bedrock::BedrockProvider),
        ProviderKind::Anthropic => Box::new(super::anthropic::AnthropicProvider),
        ProviderKind::Openai => Box::new(super::openai::OpenAIProvider),
        ProviderKind::Ollama => Box::new(super::ollama::OllamaProvider::from_env()),
        ProviderKind::LocalOpenai => Box::new(super::local_openai::LocalOpenaiProvider::from_env()),
        ProviderKind::Vertex => unimplemented!("Vertex lands in slice 4 (FR-AI-017)"),
        ProviderKind::Bge => unimplemented!("BGE is embedding-only; chat path doesn't use BGE"),
    }
}
