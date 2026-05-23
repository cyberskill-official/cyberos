//! YAML schema for SKILL.md frontmatter — minimum-viable subset.
//!
//! Full 33-field shape per FR-SKILL-103 §2.1 + v0.2.5 + v0.2.6. This phase-A
//! scaffold carries only the fields the FR-111/113 validators touch; the rest
//! land when FR-SKILL-104 (capability broker) ships.

use serde::{Deserialize, Serialize};

/// Description length bounds per FR-SKILL-111 §1 #2 (raised from FR-SKILL-103's
/// baseline 200-char cap to Anthropic's published 1024-char max).
pub const DESCRIPTION_MIN_LEN: usize = 80;
pub const DESCRIPTION_MAX_LEN: usize = 1024;

/// Minimum-viable SKILL.md frontmatter for phase-A validation.
///
/// Additional fields will land when FR-SKILL-103 ships its full broker — until
/// then we use `#[serde(flatten)]` to preserve unknown keys in `extras` so the
/// validators don't reject otherwise-valid skills.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,

    #[serde(default)]
    pub metadata: Option<SkillMetadata>,

    #[serde(default)]
    pub allowed_memory_scopes: Option<serde_yaml::Value>,

    #[serde(default)]
    pub allowed_tools: Vec<String>,

    #[serde(default)]
    pub allowed_mcp_tools: Vec<String>,

    #[serde(default)]
    pub signature: Option<String>,

    #[serde(default)]
    pub untrusted_inputs: Option<UntrustedInputs>,

    /// Catch-all for the remaining 30+ frontmatter fields we don't validate yet.
    #[serde(flatten)]
    pub extras: serde_yaml::Mapping,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkillMetadata {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(flatten)]
    pub extras: serde_yaml::Mapping,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UntrustedInputs {
    pub wrap_in_marker: MarkerName,
    #[serde(default)]
    pub injection_scan: Option<String>,
    #[serde(default)]
    pub on_marker_hit: Option<String>,
}

/// Frozen v1 marker namespace per FR-SKILL-113 §1 #2.
///
/// Future expansion (FR-SKILL-117): add `UntrustedContentStrict`,
/// `UntrustedPiiRedacted`, etc. as separate variants.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkerName {
    UntrustedContent,
}

impl MarkerName {
    /// The canonical string form (matches the body XML tag name).
    pub fn as_str(&self) -> &'static str {
        match self {
            MarkerName::UntrustedContent => "untrusted_content",
        }
    }
}
