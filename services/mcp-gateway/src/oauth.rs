//! FR-MCP-004 + FR-MCP-005 — OAuth PKCE helpers and protected-resource metadata.

use base64::Engine;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// OAuth Protected Resource Metadata response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtectedResourceMetadata {
    /// Resource identifier.
    pub resource: String,
    /// Authorization server issuer.
    pub authorization_server: String,
    /// Supported scopes.
    pub scopes_supported: Vec<String>,
    /// Bearer token methods.
    pub bearer_methods_supported: Vec<String>,
}

/// Build RFC 7636 S256 code challenge.
pub fn pkce_s256_challenge(verifier: &str) -> Result<String, String> {
    if verifier.len() < 43 || verifier.len() > 128 {
        return Err("pkce_verifier_length".into());
    }
    if !verifier
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "-._~".contains(c))
    {
        return Err("pkce_verifier_charset".into());
    }
    let digest = Sha256::digest(verifier.as_bytes());
    Ok(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(digest))
}

/// Build the well-known PRM payload.
pub fn protected_resource_metadata(resource: &str, issuer: &str) -> ProtectedResourceMetadata {
    ProtectedResourceMetadata {
        resource: resource.into(),
        authorization_server: issuer.into(),
        scopes_supported: vec![
            "mcp:tools".into(),
            "mcp:resources".into(),
            "mcp:tasks".into(),
        ],
        bearer_methods_supported: vec!["header".into()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_s256_matches_rfc_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        assert_eq!(
            pkce_s256_challenge(verifier).unwrap(),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }
}
