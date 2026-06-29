//! SEP-986 module registry (DEC-2363).
//!
//! Authoritative hardcoded list of approved cyberos module names for SEP-986 skill ID validation.
//!
//! ## Adding a new module
//!
//! 1. Obtain written sign-off from the module owner.
//! 2. Merge an RFC describing the module's purpose and namespace.
//! 3. Open a PR that appends the module name to [`MODULES`] (keeping it sorted) and updates
//!    `sep986_module_validation_test.rs` to assert the new module passes.
//! 4. The PR must be reviewed by the MCP module owner and at least one platform engineer.
//!
//! The list is reviewed quarterly. (Known follow-up: the cyberos workspace now also has `plugin`
//! and `website` modules; they are added here together with the registration-enforcement hook in
//! the follow-on slice, so the list and the enforcement land consistent.)

/// Sorted list of approved cyberos module names.
///
/// MUST remain sorted in ASCII order so [`is_valid_module`] can binary-search it.
const MODULES: &[&str] = &[
    "ai", "auth", "chat", "crm", "cuo", "doc", "email", "esop", "hr", "inv", "kb", "learn", "mcp",
    "memory", "obs", "okr", "portal", "proj", "res", "rew", "skill", "ten", "time",
];

/// Returns `true` if `name` is in the approved module list. O(log n) binary search.
pub fn is_valid_module(name: &str) -> bool {
    MODULES.binary_search(&name).is_ok()
}

/// Returns the full list of approved module names. Exposed for documentation and tests.
pub fn all_modules() -> &'static [&'static str] {
    MODULES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modules_are_sorted() {
        let mut sorted = MODULES.to_vec();
        sorted.sort_unstable();
        assert_eq!(
            MODULES,
            sorted.as_slice(),
            "MODULES must stay sorted for binary search"
        );
    }

    #[test]
    fn module_count_matches_spec() {
        assert_eq!(
            MODULES.len(),
            23,
            "module count changed; update this test and the RFC"
        );
    }
}
