//! FR-MCP-004 authorization-server discovery (RFC 8414, clause #20).
//!
//! `GET /.well-known/oauth-authorization-server` returns this document so MCP clients can discover the
//! gateway's endpoints, supported grant types, and PKCE method without hard-coding them. It is
//! distinct from FR-MCP-005 Protected Resource Metadata. `jwks_uri` points at the key set the access
//! tokens are verified against - the FR-AUTH-004 JWKS - so it is passed in rather than assumed.

use serde_json::{json, Value};

/// Build the RFC 8414 authorization-server metadata document.
///
/// `issuer` is the gateway's canonical base URL; the endpoint URLs are derived from it. `jwks_uri` is
/// where the signing keys' public halves are published (the auth service's `/.well-known/jwks.json`).
/// `scopes_supported` comes from the FR-MCP-001 `tools/list` registry (DEC-813).
pub fn authorization_server_metadata(
    issuer: &str,
    jwks_uri: &str,
    scopes_supported: &[String],
) -> Value {
    let base = issuer.trim_end_matches('/');
    json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/authorize"),
        "token_endpoint": format!("{base}/token"),
        "registration_endpoint": format!("{base}/register"),
        "revocation_endpoint": format!("{base}/revoke"),
        "introspection_endpoint": format!("{base}/introspect"),
        "jwks_uri": jwks_uri,
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"],
        "code_challenge_methods_supported": ["S256"],
        "token_endpoint_auth_methods_supported": ["none", "client_secret_basic", "private_key_jwt"],
        "scopes_supported": scopes_supported,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata_derives_endpoints_from_issuer_and_trims_trailing_slash() {
        let scopes = vec!["mcp:tools".to_string()];
        let doc = authorization_server_metadata(
            "https://mcp.cyberos.world/",
            "https://auth/jwks",
            &scopes,
        );
        assert_eq!(doc["issuer"], "https://mcp.cyberos.world");
        assert_eq!(doc["token_endpoint"], "https://mcp.cyberos.world/token");
        assert_eq!(
            doc["registration_endpoint"],
            "https://mcp.cyberos.world/register"
        );
        assert_eq!(doc["jwks_uri"], "https://auth/jwks");
    }

    #[test]
    fn metadata_pins_the_safe_oauth_2_1_profile() {
        let doc = authorization_server_metadata("https://mcp", "https://auth/jwks", &[]);
        assert_eq!(doc["response_types_supported"], json!(["code"]));
        assert_eq!(
            doc["grant_types_supported"],
            json!(["authorization_code", "refresh_token"])
        );
        assert_eq!(doc["code_challenge_methods_supported"], json!(["S256"]));
    }

    #[test]
    fn metadata_carries_the_registered_scopes() {
        let scopes = vec!["mcp:tools".to_string(), "mcp:introspect".to_string()];
        let doc = authorization_server_metadata("https://mcp", "https://auth/jwks", &scopes);
        assert_eq!(
            doc["scopes_supported"],
            json!(["mcp:tools", "mcp:introspect"])
        );
    }
}
