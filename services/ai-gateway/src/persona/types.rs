//! FR-AI-014 — Persona type definitions.

use std::collections::HashMap;
use std::sync::Arc;

use arc_swap::ArcSwap;
use once_cell::sync::OnceCell;
use semver::Version;

/// Persona ID (kebab-case, e.g. "cuo-cpo").
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PersonaId(pub String);

impl PersonaId {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Full persona handle: `<id>@<version>` (e.g. "cuo-cpo@0.4.1").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonaHandle {
    pub id: PersonaId,
    pub version: Version,
}

impl std::fmt::Display for PersonaHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.id.0, self.version)
    }
}

impl PersonaHandle {
    /// Parse "cuo-cpo@0.4.1" → PersonaHandle.
    ///
    /// Rejects pre-release tags and missing patch versions (§1 #14).
    pub fn parse(s: &str) -> Result<Self, PersonaParseError> {
        let (id_str, version_str) = s
            .split_once('@')
            .ok_or_else(|| PersonaParseError::MissingAt(s.to_string()))?;

        if id_str.is_empty() || !id_str.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            return Err(PersonaParseError::InvalidId(id_str.to_string()));
        }

        let version = Version::parse(version_str).map_err(|e| {
            PersonaParseError::InvalidSemver(format!("{}: {}", version_str, e))
        })?;

        // Reject pre-release (§1 #14)
        if !version.pre.is_empty() {
            return Err(PersonaParseError::PreReleaseUnsupported(version.to_string()));
        }

        Ok(Self {
            id: PersonaId(id_str.to_string()),
            version,
        })
    }

    /// Render as "cuo-cpo@0.4.1" for storage / headers / audit.
    pub fn display(&self) -> String {
        format!("{}@{}", self.id.0, self.version)
    }
}

/// Parsed persona from a Markdown file.
#[derive(Debug, Clone)]
pub struct Persona {
    pub handle: PersonaHandle,
    /// Canonicalised system prompt body.
    pub body: String,
    /// Allowed MCP tool names.
    pub allowed_tools: Vec<String>,
    /// Persona traits (e.g. "concise", "VN-aware").
    pub traits: Vec<String>,
    /// LLM hints (temperature, max_tokens, stop_sequences).
    pub llm_hints: LlmHints,
    /// Memory-relative path (e.g. "memories/personas/cuo-cpo@0.4.1.md").
    pub source_path: String,
    /// SHA-256 of canonicalised body.
    pub source_hash: [u8; 32],
}

/// LLM hints from persona frontmatter.
#[derive(Debug, Clone, Default)]
pub struct LlmHints {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Vec<String>,
}

// ─── Error types ──────────────────────────────────────────────────────────────

/// Error from `persona::load()`.
#[derive(Debug, thiserror::Error)]
pub enum PersonaError {
    #[error("unknown persona handle {handle}; available: {available:?}")]
    UnknownPersona {
        handle: String,
        available: Vec<String>,
    },

    #[error("persona body hash mismatch — possible tampering: handle={handle}")]
    Tampered {
        handle: PersonaHandle,
        expected_hash: [u8; 32],
        actual_hash: [u8; 32],
    },

    #[error("registry not initialised")]
    RegistryNotInitialised,
}

/// Error from `init_persona_registry()`.
#[derive(Debug, thiserror::Error)]
pub enum PersonaInitError {
    #[error("malformed YAML frontmatter in {path}: {reason}")]
    Schema { path: String, reason: String },

    #[error("filename {path} does not match frontmatter handle {handle}")]
    FilenameMismatch { path: String, handle: String },

    #[error("forbidden field 'system_prompt' in frontmatter at {path}; body is the canonical source")]
    ForbiddenFrontmatterField { path: String },

    #[error("registry already initialised; init_persona_registry called twice")]
    AlreadyInitialised,

    #[error("IO error reading persona at {path}: {reason}")]
    IoError { path: String, reason: String },
}

/// Error from `PersonaHandle::parse()`.
#[derive(Debug, thiserror::Error)]
pub enum PersonaParseError {
    #[error("missing '@' separator in handle '{0}'")]
    MissingAt(String),

    #[error("invalid semver in handle: {0}")]
    InvalidSemver(String),

    #[error("pre-release versions not supported in slice 3: {0}")]
    PreReleaseUnsupported(String),

    #[error("invalid persona id (must be kebab-case): '{0}'")]
    InvalidId(String),
}

// ─── Registry (ArcSwap-backed, lock-free reads) ──────────────────────────────

/// Global persona registry. `OnceCell` ensures single init; `ArcSwap` enables
/// lock-free concurrent reads with atomic swap on hot-reload.
pub(crate) static REGISTRY: OnceCell<ArcSwap<HashMap<PersonaHandle, Arc<Persona>>>> =
    OnceCell::new();
