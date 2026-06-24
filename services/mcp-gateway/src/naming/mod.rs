//! SEP-986 naming convention validator (FR-MCP-003).
//!
//! Enforces the `cyberos.{module}.{verb}_{noun}` skill ID pattern. This slice ships the pure
//! validator: the closed [`Sep986Verb`] enum, the pre-compiled regex, and the module registry.
//! The registration-enforcement hook (DEC-2362) and the memory-audit emission (DEC-2364) land in a
//! follow-on slice, because enforcing the pattern at registration first requires the existing
//! non-conforming tool names (`cyberos.demo.echo`, `cyberos.obs.triage`) to be migrated.
//!
//! ## Governance
//!
//! Adding a module requires module-owner sign-off, an RFC, and a PR updating
//! [`module_registry`]. Extending the verb enum requires a SEP RFC; the
//! `sep986_verb_enum_cardinality_test` fails if a variant is added without updating its assertion,
//! acting as a governance tripwire.

use std::sync::LazyLock;

use regex::Regex;

pub mod module_registry;
pub mod validator;

pub use validator::{validate_sync, NamingError, Sep986Verb, ValidationResult};

/// Pre-compiled SEP-986 regex, compiled exactly once per process.
///
/// Pattern: `^cyberos\.([a-z][a-z0-9_]*)\.([a-z]+)_([a-z][a-z0-9_]*)$`. Capture groups are
/// 1 = module, 2 = verb (validated against [`Sep986Verb`]), 3 = noun (snake_case).
pub(crate) static SKILL_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^cyberos\.([a-z][a-z0-9_]*)\.([a-z]+)_([a-z][a-z0-9_]*)$")
        .expect("SEP-986 regex is a compile-time constant and must never fail")
});
