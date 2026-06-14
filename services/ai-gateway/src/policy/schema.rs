//! FR-AI-005 §3 — `TenantPolicy` schema and validators.
//!
//! Closed schema; the `schemars` derives generate the JSONSchema mirror that lives at
//! `config/tenants/SCHEMA.json` (CI gate: regenerated schema MUST byte-match committed
//! schema). See FR-AI-005 §1 #1.

use std::collections::HashMap;

use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Top-level per-tenant policy. One YAML file per tenant; loaded keyed by `tenant_id`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TenantPolicy {
    /// Canonical tenant identifier (e.g. `org:cyberskill`).
    pub tenant_id: String,
    /// Optional jurisdiction tag used by residency defaulting rules.
    ///
    /// `VN` tenants must explicitly pin residency; non-VN tenants missing a pin default
    /// to Sg1 in the loader.
    #[serde(default)]
    pub tenant_jurisdiction: Option<String>,
    /// AI-gate-specific policy block.
    pub ai_policy: AiPolicy,
}

/// AI Gateway policy knobs. Consumed by FR-AI-001 (cost ledger), FR-AI-006/008 (router),
/// FR-AI-015 (ZDR), FR-AI-016 (residency).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AiPolicy {
    /// Hard cap on monthly USD spend across all providers. YAML accepts string
    /// (`"150.00"`) or number (`150`). Range checked at load time: [0.01, 1_000_000].
    #[serde(with = "rust_decimal::serde::str")]
    #[schemars(with = "String")]
    pub monthly_cap_usd: Decimal,

    /// Fraction of cap at which to emit a warn event (default 0.80; range [0.5, 0.95]).
    #[serde(default = "default_warn_threshold")]
    #[schemars(range(min = 0.5, max = 0.95))]
    pub warn_threshold: f64,

    /// If true, precheck() refuses at cap; if false, allows over-spend.
    #[serde(default = "default_hard_stop")]
    pub hard_stop: bool,

    /// Primary provider for routing (FR-AI-006/008 consumes this).
    pub primary_provider: Provider,

    /// Fallback chain (in order); empty = no fallback.
    #[serde(default)]
    pub fallback_chain: Vec<Provider>,

    /// Per-call timeout (precheck + provider + reconcile combined; seconds; range [1, 600]).
    #[serde(default = "default_call_timeout_seconds")]
    #[schemars(range(min = 1, max = 600))]
    pub call_timeout_seconds: u32,

    /// Residency pin — provider selection respects this (FR-AI-016).
    #[serde(default = "default_residency")]
    pub residency: Residency,

    /// Per-alias residency overrides. Glob syntax supports exact aliases plus `*`
    /// wildcards; ambiguous matches are refused at alias-resolution time.
    #[serde(default)]
    pub residency_override: Option<HashMap<String, Residency>>,

    /// Require ZDR (Zero Data Retention) — refuse non-ZDR providers (FR-AI-015).
    #[serde(default)]
    pub zdr_required: bool,

    /// Export redacted AI traces to the self-hosted LangSmith stack.
    ///
    /// Default false; tenants must explicitly opt in through the operator CLI.
    #[serde(default)]
    pub langsmith_export: bool,

    /// Emergency override (CFO-signed) to allow over-cap calls.
    #[serde(default)]
    pub emergency_override: EmergencyOverride,

    /// Persona pinning — restrict calls to this exact `agent_persona` version.
    /// `None` = any registered persona allowed.
    #[serde(default)]
    pub allowed_personas: Option<Vec<String>>,

    /// Per-alias overrides — beats primary + fallback chain.
    /// FR-AI-006 §1 #2.
    #[serde(default)]
    pub alias_overrides: Option<HashMap<String, OverrideTarget>>,

    /// If true, providers without regional pinning (e.g. Anthropic native)
    /// fail residency checks. Default false. FR-AI-006 §1 #7.
    #[serde(default)]
    pub residency_requires_regional_provider: Option<bool>,

    /// Extra PII entity types to redact beyond the EN baseline (FR-AI-011 §1 #10).
    /// Values are Presidio entity-type names (e.g. "VN_CCCD", "VN_MST").
    #[serde(default)]
    pub pii_redaction_extra: Option<Vec<String>>,

    /// Tenant-scoped PII allowlist regexes for legitimate subject-matter identifiers.
    /// FR-AI-012 uses this for KYC/vendor flows where selected VN identifiers may pass through.
    #[serde(default)]
    pub pii_allowlist: Option<Vec<String>>,
}

/// Override target for a specific alias. FR-AI-006 §3.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OverrideTarget {
    /// The provider + model_alias_map to use for this alias.
    pub provider: Provider,
}

/// Provider tag union. FR-AI-005 §3.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Provider {
    /// AWS Bedrock provider.
    Bedrock {
        /// Region (e.g. `ap-southeast-1`).
        #[schemars(regex(pattern = r"^(us|eu|ap|sa|af|ca|me)-[a-z]+-\d+$"))]
        region: String,
        /// Map of model alias → provider-specific model id.
        model_alias_map: HashMap<String, String>,
    },
    /// Anthropic API provider.
    Anthropic {
        /// Map of model alias → provider-specific model id.
        model_alias_map: HashMap<String, String>,
    },
    /// OpenAI API provider.
    Openai {
        /// Map of model alias → provider-specific model id.
        model_alias_map: HashMap<String, String>,
    },
    /// Google Vertex AI provider.
    Vertex {
        /// GCP project id.
        project: String,
        /// Region.
        #[schemars(regex(pattern = r"^[a-z]+-[a-z]+\d*$"))]
        region: String,
        /// Map of model alias → provider-specific model id.
        model_alias_map: HashMap<String, String>,
    },
    /// Self-hosted BGE sidecar provider.
    Bge {
        /// AWS-style region where this sidecar is deployed.
        #[schemars(regex(pattern = r"^(us|eu|ap|sa|af|ca|me)-[a-z]+-\d+$"))]
        region: String,
        /// Map of model alias → provider-specific model id.
        model_alias_map: HashMap<String, String>,
    },
}

impl Provider {
    /// Get the provider kind (simplified identity).
    pub fn kind(&self) -> ProviderKind {
        match self {
            Self::Bedrock { .. } => ProviderKind::Bedrock,
            Self::Anthropic { .. } => ProviderKind::Anthropic,
            Self::Openai { .. } => ProviderKind::Openai,
            Self::Vertex { .. } => ProviderKind::Vertex,
            Self::Bge { .. } => ProviderKind::Bge,
        }
    }

    /// Get the provider region (if regional).
    pub fn region(&self) -> Option<String> {
        match self {
            Self::Bedrock { region, .. } => Some(region.clone()),
            Self::Anthropic { .. } => None,
            Self::Openai { .. } => None,
            Self::Vertex { region, .. } => Some(region.clone()),
            Self::Bge { region, .. } => Some(region.clone()),
        }
    }

    /// Look up a model alias in this provider's model_alias_map.
    pub fn model_for_alias(&self, alias: &str) -> Option<&str> {
        let map = match self {
            Self::Bedrock {
                model_alias_map, ..
            } => model_alias_map,
            Self::Anthropic {
                model_alias_map, ..
            } => model_alias_map,
            Self::Openai {
                model_alias_map, ..
            } => model_alias_map,
            Self::Vertex {
                model_alias_map, ..
            } => model_alias_map,
            Self::Bge {
                model_alias_map, ..
            } => model_alias_map,
        };
        map.get(alias).map(|s| s.as_str())
    }
}

/// Provider kind — simplified enum for cost-table lookups and metric labels.
///
/// This is a subset of [`Provider`] that carries no configuration (region, aliases).
/// Used by FR-AI-007 (cost table) and FR-AI-006 (alias resolution) where only the
/// provider identity matters, not its full config.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    /// AWS Bedrock.
    Bedrock,
    /// Anthropic API.
    Anthropic,
    /// OpenAI API.
    Openai,
    /// Google Vertex AI.
    Vertex,
    /// Self-hosted BGE (FR-AI-019).
    Bge,
}

impl ProviderKind {
    /// Stable string conversion for OTel metric labels.
    ///
    /// Uses the serde rename (lowercase) to avoid coupling OBS dashboards to
    /// Rust enum variant names.
    pub fn as_metric_label(&self) -> &'static str {
        match self {
            Self::Bedrock => "bedrock",
            Self::Anthropic => "anthropic",
            Self::Openai => "openai",
            Self::Vertex => "vertex",
            Self::Bge => "bge",
        }
    }
}

/// Residency pin. Slice 1 records the value; FR-AI-016 enforces it at routing time.
///
/// Wire form is hyphenated (`sg-1`, `eu-1`, `us-1`, `vn-1`) per FR-AI-005 §3 + the
/// EXAMPLE.tenant.yaml reference. `rename_all = "kebab-case"` would emit `sg1` (no
/// hyphen — serde treats `Sg1` as one word + digit) so each variant carries an
/// explicit `rename`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub enum Residency {
    /// Singapore.
    #[serde(rename = "sg-1")]
    Sg1,
    /// Frankfurt.
    #[serde(rename = "eu-1")]
    Eu1,
    /// us-east-1.
    #[serde(rename = "us-1")]
    Us1,
    /// Vietnam.
    #[serde(rename = "vn-1")]
    Vn1,
}

/// Emergency-override block — allows over-cap calls when signed by the listed approvers.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EmergencyOverride {
    /// Whether the override is enabled at all.
    #[serde(default)]
    pub enabled: bool,
    /// Required attestations (e.g. `["cfo_signoff", "audit_row"]`).
    #[serde(default)]
    pub requires: Vec<String>,
    /// Maximum cap multiplier this override permits (e.g. 1.5 = 150% of cap).
    #[serde(default = "default_override_multiplier")]
    pub max_multiplier: f64,
}

fn default_warn_threshold() -> f64 {
    0.80
}
fn default_hard_stop() -> bool {
    true
}
fn default_call_timeout_seconds() -> u32 {
    60
}
fn default_residency() -> Residency {
    Residency::Sg1
}
fn default_override_multiplier() -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sensible() {
        assert_eq!(default_warn_threshold(), 0.80);
        assert!(default_hard_stop());
        assert_eq!(default_call_timeout_seconds(), 60);
        assert_eq!(default_override_multiplier(), 1.0);
    }
}
