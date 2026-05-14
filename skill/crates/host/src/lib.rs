//! cyberos-skill-host — the Rust runtime for the CyberOS skill module.
//!
//! Implements the Anthropic Agent Skills three-level progressive
//! disclosure model:
//!   Level 1 (boot)       — header-only indexing
//!   Level 2 (activation) — lazy body load
//!   Level 3 (execution)  — scripts / WASM components on demand
//!
//! Architecture per the May 2026 audit (see ../docs/SPEC.md).

pub mod registry;
pub mod loader;
pub mod activator;
pub mod capabilities;
pub mod error;
pub mod grants;
pub mod invoke;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use registry::{SkillHeader, SkillRegistry, ActivatedSkill};
pub use loader::Loader;
pub use activator::Activator;
pub use capabilities::{Capability, CapabilityBroker};
pub use error::HostError;
pub use grants::{default_grants_path, GrantEntry, GrantsFile};
pub use invoke::{ensure_granted, InvokeContext};

#[cfg(feature = "wasm")]
pub use wasm::{make_engine, run_component, wasm_cache_dir};
