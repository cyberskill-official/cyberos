//! TASK-AUTH-110 §1 #12 + DEC-2491 - exact redirect_uri matching.
//!
//! A presented `redirect_uri` must be byte-exact against one of the RP's
//! registered URIs. No wildcard, no path-prefix, no query-param laxness - loose
//! matching is how open-redirect and token-leak attacks happen. On a miss the
//! caller renders an error page and does NOT redirect (redirecting to an
//! unverified URI is itself the vulnerability).

/// True iff `presented` exactly equals one of the `registered` redirect URIs.
pub fn redirect_uri_registered(registered: &[String], presented: &str) -> bool {
    registered.iter().any(|r| r == presented)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn registered() -> Vec<String> {
        vec!["https://chat.cyberos.world/signin/oidc/complete".to_string()]
    }

    #[test]
    fn exact_match_accepted() {
        assert!(redirect_uri_registered(
            &registered(),
            "https://chat.cyberos.world/signin/oidc/complete"
        ));
    }

    #[test]
    fn near_misses_rejected() {
        let reg = registered();
        for bad in [
            "https://chat.cyberos.world/signin/oidc/complete/", // trailing slash
            "https://chat.cyberos.world/signin/oidc/complete?x=1", // extra query
            "https://chat.cyberos.world/signin/oidc",           // shorter path
            "https://chat.cyberos.world/signin/oidc/complete/extra", // longer path
            "http://chat.cyberos.world/signin/oidc/complete",   // wrong scheme
            "https://evil.example/signin/oidc/complete",        // wrong host
        ] {
            assert!(!redirect_uri_registered(&reg, bad), "should reject {bad}");
        }
    }

    #[test]
    fn empty_registry_rejects_everything() {
        assert!(!redirect_uri_registered(
            &[],
            "https://chat.cyberos.world/x"
        ));
    }
}
