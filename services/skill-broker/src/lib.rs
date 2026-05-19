//! `cyberos-skill-broker` — SKILL.md frontmatter validator + broker.
//!
//! Implements the Rust runtime side of:
//! - FR-SKILL-103 — frontmatter schema + parser + validators
//! - FR-SKILL-111 — description trigger enrichment (SKB-020..023)
//! - FR-SKILL-113 — XML-free frontmatter (SKB-040..042)
//!
//! See `cyberos/modules/skill/SKILL_BUNDLE_RUBRIC.md` for the rule corpus.
//!
//! Phase-A scaffolding: this crate compiles + the validators run. Full
//! integration with the runtime broker (load skill, dispatch, audit-emit)
//! lands when FR-SKILL-104 (capability broker) ships.

pub mod frontmatter;
pub mod transpilers;

pub use frontmatter::{
    FrontmatterError, SkillFrontmatter, MarkerName,
    load_and_validate, validate_description, validate_marker,
};
pub use transpilers::{transpile_anthropic, AnthropicSkill};
