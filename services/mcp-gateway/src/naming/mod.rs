//! SEP-986 naming convention validator (TASK-MCP-003).
//!
//! Enforces the `cyberos.{module}.{verb}_{noun}` skill ID pattern. Slice 1 shipped the pure
//! validator here: the closed [`Sep986Verb`] enum, the pre-compiled regex, and the module registry.
//! Slice 2 wired it into registration ([`crate::federation::register::validate`], DEC-2362): a real
//! module that registers a non-conforming tool ID is rejected before the tool can become callable.
//! As part of that, the one pre-existing non-conforming production tool was migrated
//! (`cyberos.obs.triage` -> `cyberos.obs.execute_triage`); the dev/reference fixture
//! (`cyberos.demo.echo` / `cyberos.demo.now`) is exempt via `NAMING_EXEMPT_MODULES`. Slice 3 shipped
//! the CI grep gate (`scripts/check_sep986_naming.sh` + `.github/workflows/mcp-sep986-check.yml`,
//! DEC-2362) and the four DEC-2364 audit kinds on `oauth::audit` (`skill_name_validated` /
//! `_rejected` / `naming_ci_check_passed` / `_failed`), emitted from the register path in `router`.
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
