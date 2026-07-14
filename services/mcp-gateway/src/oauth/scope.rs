//! TASK-MCP-004 scope handling (DEC-813, clause #30, RFC 6749 §3.3).
//!
//! Scopes are a space-separated, case-sensitive list. Each token must be RFC 6749 §3.3 syntax
//! (visible ASCII, no whitespace, no `"` or `\`). The set of grantable scopes is closed - it comes
//! from the MCP server's `tools/list` registry (TASK-MCP-001) - so a request for an unregistered scope
//! is `400 invalid_scope`, never silently granted.

/// One RFC 6749 §3.3 scope-token: `1*( %x21 / %x23-5B / %x5D-7E )`. That is, one or more visible
/// ASCII characters excluding space (`%x20`), double-quote (`%x22`), and backslash (`%x5C`).
pub fn is_valid_scope_token(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| matches!(b, 0x21 | 0x23..=0x5B | 0x5D..=0x7E))
}

/// Split a raw `scope` parameter into its tokens on single spaces, dropping empty runs (RFC 6749
/// allows only a single space separator, but collapsing repeats is harmless and more forgiving).
/// The returned tokens are not yet validated for syntax or membership - call [`validate_scopes`].
pub fn parse_scope(raw: &str) -> Vec<String> {
    raw.split(' ')
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
}

/// Why a requested scope set was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeError {
    /// A token violates RFC 6749 §3.3 syntax (contains space, quote, backslash, control, or non-ASCII).
    Malformed(String),
    /// A well-formed token is not in the server's registered scope set (DEC-813).
    Unknown(String),
}

/// Validate a requested scope set against the closed `registered` set (the `tools/list` registry).
///
/// Returns the de-duplicated requested tokens on success. Fails on the first malformed token, then on
/// the first unknown token - the error names the offending scope so the `invalid_scope` response is
/// specific. Comparison is case-sensitive (RFC 6749 §3.3).
pub fn validate_scopes(
    requested: &[String],
    registered: &[String],
) -> Result<Vec<String>, ScopeError> {
    for s in requested {
        if !is_valid_scope_token(s) {
            return Err(ScopeError::Malformed(s.clone()));
        }
    }
    for s in requested {
        if !registered.iter().any(|r| r == s) {
            return Err(ScopeError::Unknown(s.clone()));
        }
    }
    let mut seen: Vec<String> = Vec::with_capacity(requested.len());
    for s in requested {
        if !seen.iter().any(|d| d == s) {
            seen.push(s.clone());
        }
    }
    Ok(seen)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn well_formed_tokens_pass_syntax() {
        for s in ["mcp:tools", "read", "obs.execute_triage", "a", "x-y_z.1"] {
            assert!(is_valid_scope_token(s), "{s} should be valid");
        }
    }

    #[test]
    fn syntax_rejects_space_quote_backslash_and_control() {
        for s in ["has space", "has\"quote", "has\\slash", "tab\tchar", ""] {
            assert!(!is_valid_scope_token(s), "{s:?} should be invalid");
        }
    }

    #[test]
    fn syntax_rejects_non_ascii() {
        assert!(!is_valid_scope_token("caf\u{00e9}"));
    }

    #[test]
    fn parse_splits_on_single_space_and_drops_empties() {
        assert_eq!(parse_scope("mcp:tools read"), v(&["mcp:tools", "read"]));
        assert_eq!(
            parse_scope("  mcp:tools   read  "),
            v(&["mcp:tools", "read"])
        );
        assert_eq!(parse_scope(""), Vec::<String>::new());
    }

    #[test]
    fn validate_accepts_a_registered_subset() {
        let registered = v(&["mcp:tools", "read", "write"]);
        let requested = v(&["read", "mcp:tools"]);
        assert_eq!(
            validate_scopes(&requested, &registered),
            Ok(v(&["read", "mcp:tools"]))
        );
    }

    #[test]
    fn validate_rejects_an_unknown_scope() {
        let registered = v(&["mcp:tools", "read"]);
        let requested = v(&["read", "admin"]);
        assert_eq!(
            validate_scopes(&requested, &registered),
            Err(ScopeError::Unknown("admin".to_string()))
        );
    }

    #[test]
    fn validate_rejects_a_malformed_scope_before_membership() {
        let registered = v(&["mcp:tools"]);
        let requested = v(&["bad scope"]);
        assert_eq!(
            validate_scopes(&requested, &registered),
            Err(ScopeError::Malformed("bad scope".to_string()))
        );
    }

    #[test]
    fn validate_is_case_sensitive() {
        let registered = v(&["mcp:tools"]);
        let requested = v(&["MCP:tools"]);
        assert_eq!(
            validate_scopes(&requested, &registered),
            Err(ScopeError::Unknown("MCP:tools".to_string()))
        );
    }

    #[test]
    fn validate_dedupes_the_returned_set() {
        let registered = v(&["read"]);
        let requested = v(&["read", "read"]);
        assert_eq!(validate_scopes(&requested, &registered), Ok(v(&["read"])));
    }
}
