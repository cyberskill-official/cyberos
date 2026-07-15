//! TASK-AUTH-110 §1 #6 + DEC-2492 - OIDC + RFC 8414 provider discovery.
//!
//! `GET /.well-known/openid-configuration` returns this so first-party RPs
//! (Mattermost's native OIDC connector, the portal) discover the endpoints
//! without hard-coding them. `issuer` is the single configured canonical AUTH URL
//! (DEC-2498); `jwks_uri` is the existing TASK-AUTH-004 `/.well-known/jwks.json`
//! (DEC-2481, one key system). The document pins the safe OIDC profile: code
//! response type only, authorization_code grant, S256 PKCE, RS256 id_token.

use serde_json::{json, Value};

/// Build the OIDC + RFC 8414 metadata document.
///
/// `grant_types` is `["authorization_code"]`, plus `"refresh_token"` when any
/// active RP has `allow_refresh` (DEC-2499).
pub fn openid_configuration(issuer: &str, jwks_uri: &str, grant_types: &[&str]) -> Value {
    let base = issuer.trim_end_matches('/');
    json!({
        "issuer": base,
        "authorization_endpoint": format!("{base}/v1/auth/op/authorize"),
        "token_endpoint":         format!("{base}/v1/auth/op/token"),
        "userinfo_endpoint":      format!("{base}/v1/auth/op/userinfo"),
        "end_session_endpoint":   format!("{base}/v1/auth/op/logout"),
        "jwks_uri": jwks_uri,
        "response_types_supported": ["code"],
        "grant_types_supported": grant_types,
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "code_challenge_methods_supported": ["S256"],
        "scopes_supported": ["openid", "profile", "email", "offline_access"],
        "token_endpoint_auth_methods_supported": ["client_secret_basic", "client_secret_post"],
        "claims_supported": [
            "sub", "email", "email_verified", "name",
            "preferred_username", "tenant_id", "roles"
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn derives_endpoints_and_trims_trailing_slash() {
        let doc = openid_configuration(
            "https://auth.cyberos.world/",
            "https://auth.cyberos.world/.well-known/jwks.json",
            &["authorization_code"],
        );
        assert_eq!(doc["issuer"], "https://auth.cyberos.world");
        assert_eq!(
            doc["authorization_endpoint"],
            "https://auth.cyberos.world/v1/auth/op/authorize"
        );
        assert_eq!(
            doc["token_endpoint"],
            "https://auth.cyberos.world/v1/auth/op/token"
        );
        assert_eq!(
            doc["userinfo_endpoint"],
            "https://auth.cyberos.world/v1/auth/op/userinfo"
        );
    }

    #[test]
    fn pins_the_safe_oidc_profile() {
        let doc =
            openid_configuration("https://auth", "https://auth/jwks", &["authorization_code"]);
        assert_eq!(doc["response_types_supported"], json!(["code"]));
        assert_eq!(doc["code_challenge_methods_supported"], json!(["S256"]));
        assert_eq!(
            doc["id_token_signing_alg_values_supported"],
            json!(["RS256"])
        );
    }

    #[test]
    fn carries_refresh_grant_when_enabled() {
        let doc = openid_configuration(
            "https://auth",
            "https://auth/jwks",
            &["authorization_code", "refresh_token"],
        );
        assert_eq!(
            doc["grant_types_supported"],
            json!(["authorization_code", "refresh_token"])
        );
    }
}
