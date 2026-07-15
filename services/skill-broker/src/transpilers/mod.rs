//! Per-host transpilers — emit `dist/<host>/<skill>/SKILL.md` from the CCSM.
//!
//! Per TASK-SKILL-103 + the host-portability contract in
//! `modules/skill/README.md` Part 9. Each transpiler is a pure function
//! `CCSM → host-artefact-tree`. The CCSM (Canonical CyberSkill Skill
//! Manifest = `modules/skill/<name>/SKILL.md`) is the source of truth;
//! transpilers MUST NOT be hand-edited downstream of their generated output.

pub mod anthropic;

pub use anthropic::{transpile as transpile_anthropic, AnthropicSkill};
