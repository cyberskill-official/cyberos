//! Core SEP-986 validation logic (TASK-MCP-003).
//!
//! Single public entry point: [`validate_sync`], plus the closed [`Sep986Verb`] enum and the typed
//! [`NamingError`]. The async audit-emitting wrapper and the registration hook land in a follow-on
//! slice.

use std::fmt;

use super::{module_registry, SKILL_ID_REGEX};

/// Closed set of approved SEP-986 verbs (DEC-2361).
///
/// Cardinality is fixed at 15. Extensions require a new SEP RFC; the
/// `sep986_verb_enum_cardinality_test` asserts exactly 15 variants and fails if one is added
/// without updating it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sep986Verb {
    /// `get`
    Get,
    /// `list`
    List,
    /// `create`
    Create,
    /// `update`
    Update,
    /// `delete`
    Delete,
    /// `send`
    Send,
    /// `fetch`
    Fetch,
    /// `sync`
    Sync,
    /// `validate`
    Validate,
    /// `generate`
    Generate,
    /// `execute`
    Execute,
    /// `search`
    Search,
    /// `replay`
    Replay,
    /// `accept`
    Accept,
    /// `reject`
    Reject,
}

impl Sep986Verb {
    /// All variants, in declaration order. Used by the cardinality test.
    pub fn all_variants() -> &'static [Sep986Verb] {
        &[
            Sep986Verb::Get,
            Sep986Verb::List,
            Sep986Verb::Create,
            Sep986Verb::Update,
            Sep986Verb::Delete,
            Sep986Verb::Send,
            Sep986Verb::Fetch,
            Sep986Verb::Sync,
            Sep986Verb::Validate,
            Sep986Verb::Generate,
            Sep986Verb::Execute,
            Sep986Verb::Search,
            Sep986Verb::Replay,
            Sep986Verb::Accept,
            Sep986Verb::Reject,
        ]
    }

    /// Parse a lowercase verb string into a [`Sep986Verb`], or `None` if it is not approved.
    ///
    /// Named `from_verb_str` rather than `from_str` so it does not shadow the `FromStr` trait
    /// convention (the lookup is infallible-by-`Option`, not `Result`).
    pub fn from_verb_str(s: &str) -> Option<Sep986Verb> {
        match s {
            "get" => Some(Sep986Verb::Get),
            "list" => Some(Sep986Verb::List),
            "create" => Some(Sep986Verb::Create),
            "update" => Some(Sep986Verb::Update),
            "delete" => Some(Sep986Verb::Delete),
            "send" => Some(Sep986Verb::Send),
            "fetch" => Some(Sep986Verb::Fetch),
            "sync" => Some(Sep986Verb::Sync),
            "validate" => Some(Sep986Verb::Validate),
            "generate" => Some(Sep986Verb::Generate),
            "execute" => Some(Sep986Verb::Execute),
            "search" => Some(Sep986Verb::Search),
            "replay" => Some(Sep986Verb::Replay),
            "accept" => Some(Sep986Verb::Accept),
            "reject" => Some(Sep986Verb::Reject),
            _ => None,
        }
    }

    /// The canonical lowercase string for this verb.
    pub fn as_str(&self) -> &'static str {
        match self {
            Sep986Verb::Get => "get",
            Sep986Verb::List => "list",
            Sep986Verb::Create => "create",
            Sep986Verb::Update => "update",
            Sep986Verb::Delete => "delete",
            Sep986Verb::Send => "send",
            Sep986Verb::Fetch => "fetch",
            Sep986Verb::Sync => "sync",
            Sep986Verb::Validate => "validate",
            Sep986Verb::Generate => "generate",
            Sep986Verb::Execute => "execute",
            Sep986Verb::Search => "search",
            Sep986Verb::Replay => "replay",
            Sep986Verb::Accept => "accept",
            Sep986Verb::Reject => "reject",
        }
    }
}

/// Typed errors produced by SEP-986 validation. Each is a registration refusal the caller MUST honor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NamingError {
    /// The skill ID does not match the required pattern.
    MalformedSkillId {
        /// The offending skill ID.
        skill_id: String,
        /// A human-readable explanation of the violation.
        reason: String,
    },
    /// The module segment is not in the approved module list.
    UnknownModule {
        /// The offending skill ID.
        skill_id: String,
        /// The module segment that was not recognised.
        module: String,
    },
    /// The verb segment is not in the [`Sep986Verb`] enum.
    InvalidVerb {
        /// The offending skill ID.
        skill_id: String,
        /// The verb segment that was not recognised.
        verb: String,
    },
}

impl fmt::Display for NamingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NamingError::MalformedSkillId { skill_id, reason } => {
                write!(f, "SEP-986 validation failed for '{skill_id}': {reason}")
            }
            NamingError::UnknownModule { skill_id, module } => write!(
                f,
                "SEP-986 validation failed for '{skill_id}': module '{module}' is not in the \
                 approved module list. To add a module, follow the RFC process in \
                 naming/module_registry.rs."
            ),
            NamingError::InvalidVerb { skill_id, verb } => write!(
                f,
                "SEP-986 validation failed for '{skill_id}': verb '{verb}' is not in the approved \
                 Sep986Verb enum (DEC-2361). Approved verbs: get, list, create, update, delete, \
                 send, fetch, sync, validate, generate, execute, search, replay, accept, reject. \
                 Verb additions require a SEP RFC."
            ),
        }
    }
}

impl std::error::Error for NamingError {}

/// A successful SEP-986 validation result, with the three segments extracted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    /// The validated skill ID, unchanged.
    pub skill_id: String,
    /// The module segment.
    pub module: String,
    /// The verb segment, parsed into the closed enum.
    pub verb: Sep986Verb,
    /// The noun segment (snake_case).
    pub noun: String,
}

/// Validate a skill ID against the SEP-986 naming convention.
///
/// Returns [`ValidationResult`] on success, or a specific [`NamingError`] the caller MUST treat as a
/// registration refusal. Completes in well under 1 ms: the regex is compiled once and the module
/// lookup is a binary search over the approved list.
pub fn validate_sync(skill_id: &str) -> Result<ValidationResult, NamingError> {
    let caps = SKILL_ID_REGEX.captures(skill_id).ok_or_else(|| {
        let reason = if !skill_id.starts_with("cyberos.") {
            "skill ID must start with 'cyberos.' prefix".to_string()
        } else if skill_id.chars().any(|c| c.is_uppercase()) {
            "skill ID must be fully lowercase (no uppercase letters permitted)".to_string()
        } else {
            "skill ID does not match required pattern 'cyberos.{module}.{verb}_{noun}' where \
             module and noun are snake_case identifiers"
                .to_string()
        };
        NamingError::MalformedSkillId {
            skill_id: skill_id.to_string(),
            reason,
        }
    })?;

    // The regex guarantees three capture groups when it matches, so these indices are safe.
    let module = caps.get(1).map_or("", |m| m.as_str());
    let verb_str = caps.get(2).map_or("", |m| m.as_str());
    let noun = caps.get(3).map_or("", |m| m.as_str());

    if !module_registry::is_valid_module(module) {
        return Err(NamingError::UnknownModule {
            skill_id: skill_id.to_string(),
            module: module.to_string(),
        });
    }

    let verb = Sep986Verb::from_verb_str(verb_str).ok_or_else(|| NamingError::InvalidVerb {
        skill_id: skill_id.to_string(),
        verb: verb_str.to_string(),
    })?;

    Ok(ValidationResult {
        skill_id: skill_id.to_string(),
        module: module.to_string(),
        verb,
        noun: noun.to_string(),
    })
}
