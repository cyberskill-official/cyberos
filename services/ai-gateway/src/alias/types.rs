//! FR-AI-006 §3 — Alias resolution types.

use crate::policy::{ProviderKind, Residency};

/// Result of a successful alias resolution.
#[derive(Debug, Clone)]
pub struct ResolvedModel {
    /// Provider kind (bedrock | anthropic | openai | vertex | bge).
    pub provider_kind: ProviderKind,
    /// Provider region (None for providers without regional pinning).
    pub region: Option<String>,
    /// Concrete model identifier (e.g. "anthropic.claude-3-5-sonnet-20241022-v2:0").
    pub model: String,
    /// Fallback position: 0 = primary or override, 1 = first fallback, etc.
    pub fallback_position: u8,
    /// Whether the resolved provider is ZDR-attested.
    pub is_zdr: bool,
    /// Latency class for timeout budgeting.
    pub latency_class: LatencyClass,
}

/// Error returned by [`super::resolve`].
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AliasError {
    /// The alias isn't in the supported set.
    #[error("unknown alias '{alias}'; supported: {supported:?}")]
    UnknownAlias {
        alias: String,
        supported: Vec<String>,
    },
    /// The resolved (provider, model) has no cost-table entry.
    #[error("resolved model {model} (provider {provider:?}) missing from cost table")]
    ResolvedModelMissingCostEntry {
        provider: ProviderKind,
        model: String,
    },
    /// Policy requires ZDR but the resolved provider isn't attested.
    #[error(
        "ZDR violation: {resolved_model} (provider {resolved_provider:?}) is not ZDR-attested"
    )]
    ZdrViolation {
        resolved_provider: ProviderKind,
        resolved_model: String,
        /// The attestation if present (is_zdr=false), or None if no entry exists.
        attestation: Option<crate::zdr::ZdrAttestation>,
    },
    /// Policy's residency pin doesn't match the resolved provider's region.
    #[error("residency violation: region {resolved_region:?} does not match policy residency {policy_residency:?}")]
    ResidencyViolation {
        resolved_region: Option<String>,
        policy_residency: Residency,
        attempted_alias: String,
        vn1_no_provider: bool,
    },
    /// Multiple residency override globs match the alias.
    #[error("residency override ambiguous for alias '{alias}': {patterns:?}")]
    ResidencyOverrideAmbiguous {
        alias: String,
        patterns: Vec<String>,
    },
    /// A residency override glob has invalid syntax.
    #[error("residency override invalid for alias '{alias}': {pattern}: {reason}")]
    ResidencyOverrideInvalid {
        alias: String,
        pattern: String,
        reason: String,
    },
    /// No provider in the chain has this alias.
    #[error("no provider has alias '{alias}' (tried {providers_tried} providers)")]
    NoProviderHasAlias { alias: String, providers_tried: u8 },
}

/// Latency class for timeout budgeting. FR-AI-006 §3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LatencyClass {
    /// Typically <2s p95 (haiku, gpt-4o-mini, embeds, rerank).
    Fast,
    /// Typically <5s p95 (sonnet, gpt-4o).
    Standard,
    /// Typically <30s p95 (opus, long-context chat).
    Slow,
}
