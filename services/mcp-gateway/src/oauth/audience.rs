//! FR-MCP-004 audience binding (DEC-802, RFC 8707, clauses #7 and #23).
//!
//! Every access token carries `aud` = the canonical URL of the MCP resource server it was issued
//! for. Each resource server asserts, at `tools/call`, that its own canonical URL is the token's
//! audience. This is the defense against cross-server token replay (§2.4): a token stolen from
//! `mcp-a.cyberos.world` cannot be presented at `mcp-b.cyberos.world`, because the audiences differ.
//!
//! The comparison is intentionally strict - exact, case-sensitive, no trailing-slash normalization,
//! no substring. §2.9 explains why loose matching (substring/regex) opens redirect-style attacks; the
//! same discipline applies to audience comparison.

/// Build the `aud` claim for a token minted for `resource` (the resource server's canonical URL).
///
/// RFC 8707 models the audience as the resource identifier; we emit a single-element array so the
/// claim shape stays a `Vec<String>` (matching the FR-AUTH-004 `Claims.aud` type) while carrying
/// exactly one resource.
pub fn bind_audience(resource: &str) -> Vec<String> {
    vec![resource.to_string()]
}

/// Whether a token's `aud` authorizes use at the resource server identified by `expected`.
///
/// Returns `true` iff `expected` is non-empty and appears as an exact element of `token_aud`.
/// Empty `expected` (a misconfigured resource server) and empty `token_aud` (an unbound token) both
/// return `false` - fail closed. Matching is byte-exact: `https://a/` and `https://a` are different
/// audiences, and no substring of an element ever matches.
pub fn audience_matches(token_aud: &[String], expected: &str) -> bool {
    if expected.is_empty() {
        return false;
    }
    token_aud.iter().any(|a| a == expected)
}

/// The `WWW-Authenticate` header value for an audience-mismatch rejection (clause #23). Returned with
/// `401` so the client learns the token was structurally valid but not minted for this server.
pub fn audience_mismatch_challenge() -> &'static str {
    "Bearer error=\"invalid_token\", error_description=\"audience_mismatch\""
}

#[cfg(test)]
mod tests {
    use super::*;

    fn aud(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn bind_audience_emits_single_resource() {
        assert_eq!(
            bind_audience("https://mcp.cyberos.world"),
            vec!["https://mcp.cyberos.world".to_string()]
        );
    }

    #[test]
    fn exact_single_audience_matches() {
        let a = bind_audience("https://mcp.cyberos.world");
        assert!(audience_matches(&a, "https://mcp.cyberos.world"));
    }

    #[test]
    fn cross_server_token_is_rejected() {
        // A token minted for server A, presented at server B - the replay §2.4 prevents.
        let a = bind_audience("https://mcp-a.cyberos.world");
        assert!(!audience_matches(&a, "https://mcp-b.cyberos.world"));
    }

    #[test]
    fn substring_does_not_match() {
        // The classic open-redirect-style trap: a longer attacker host that contains the expected.
        let a = aud(&["https://mcp.cyberos.world.evil.com"]);
        assert!(!audience_matches(&a, "https://mcp.cyberos.world"));
        // And the reverse: expected longer than the token's audience.
        let b = aud(&["https://mcp.cyberos.world"]);
        assert!(!audience_matches(&b, "https://mcp.cyberos.world/extra"));
    }

    #[test]
    fn trailing_slash_is_significant() {
        let a = aud(&["https://mcp.cyberos.world"]);
        assert!(!audience_matches(&a, "https://mcp.cyberos.world/"));
    }

    #[test]
    fn comparison_is_case_sensitive() {
        let a = aud(&["https://MCP.cyberos.world"]);
        assert!(!audience_matches(&a, "https://mcp.cyberos.world"));
    }

    #[test]
    fn empty_audience_fails_closed() {
        assert!(!audience_matches(&[], "https://mcp.cyberos.world"));
    }

    #[test]
    fn empty_expected_fails_closed() {
        let a = aud(&["https://mcp.cyberos.world"]);
        assert!(!audience_matches(&a, ""));
    }

    #[test]
    fn multi_element_audience_matches_on_exact_member() {
        // RFC 8707 permits an array; an exact member authorizes.
        let a = aud(&["https://other.cyberos.world", "https://mcp.cyberos.world"]);
        assert!(audience_matches(&a, "https://mcp.cyberos.world"));
        assert!(!audience_matches(&a, "https://absent.cyberos.world"));
    }

    #[test]
    fn challenge_names_the_audience_mismatch() {
        let c = audience_mismatch_challenge();
        assert!(c.contains("invalid_token"));
        assert!(c.contains("audience_mismatch"));
    }
}
