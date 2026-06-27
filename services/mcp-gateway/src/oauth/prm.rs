//! FR-MCP-005 Protected Resource Metadata (RFC 9728) document builders.
//!
//! `GET /.well-known/oauth-protected-resource` advertises which authorization server issues the tokens
//! this MCP resource server accepts, plus the bearer method and signing algorithms, so a federated MCP
//! client that receives a 401 can discover where to authenticate: the `resource_metadata` parameter of
//! the `WWW-Authenticate` challenge (emitted by the tools/call gate) points here, the client fetches
//! this document, then knows which issuer to talk to. The per-module variant adds the module's
//! `scopes_supported`. Both are pure and database-free; the router supplies the issuer/resource/scope
//! values from env plus the in-memory registry, mirroring [`super::discovery`].
//!
//! This advertises the gateway's real capabilities, not the spec's aspirational set. The access-token
//! verifier ([`super::jwt`]) is RS256-only, so `resource_signing_alg_values_supported` lists `RS256`
//! only - advertising EdDSA would invite tokens the gateway cannot verify, the same reasoning that kept
//! the RFC 8414 discovery document RS256-only. The FR's per-residency four-issuer list, rate limiting,
//! drift detection, and tail-sampled telemetry are deferred until that infrastructure exists; the
//! single-issuer default reflects that the gateway is its own authorization server today.

use serde_json::{json, Value};

/// Signing algorithms the resource server accepts, tracking the `oauth::jwt` signer (RS256 only).
const SIGNING_ALGS: [&str; 1] = ["RS256"];

/// Build the gateway-aggregate PRM (RFC 9728 §2). `resource` is the canonical audience URI tokens are
/// bound to; `authorization_servers` are the issuer URLs whose tokens this resource accepts. Per
/// DEC-905 the aggregate omits `scopes_supported`.
pub fn protected_resource_metadata(resource: &str, authorization_servers: &[String]) -> Value {
    base(resource, authorization_servers, None)
}

/// Build a per-module PRM: the aggregate document plus the module's `scopes_supported` (the union of
/// its tools' required scopes from the FR-MCP-002 registry). An empty `scopes` slice is valid - it
/// means the module exposes tools that require no scope - and is distinct from a 404, which means the
/// module is not registered (§11.9).
pub fn protected_resource_metadata_for_module(
    resource: &str,
    authorization_servers: &[String],
    scopes: &[String],
) -> Value {
    base(resource, authorization_servers, Some(scopes))
}

/// Shared builder for the aggregate and per-module documents.
fn base(resource: &str, authorization_servers: &[String], scopes: Option<&[String]>) -> Value {
    let resource = resource.trim_end_matches('/');
    let mut doc = json!({
        "resource": resource,
        "authorization_servers": authorization_servers,
        "bearer_methods_supported": ["header"],
        "resource_signing_alg_values_supported": SIGNING_ALGS,
        "resource_documentation": format!("{resource}/docs"),
    });
    if let Some(scopes) = scopes {
        doc["scopes_supported"] = json!(scopes);
    }
    doc
}

/// A strong ETag for a PRM body: the first 16 hex chars of the body's SHA-256, quoted per RFC 7232
/// §2.3. Sixteen hex chars (64 bits) is enough for the one-hour cache scope (§11.2). Deterministic for
/// a given document, so `If-None-Match` revalidation yields a 304.
pub fn etag(doc: &Value) -> String {
    let body = serde_json::to_string(doc).unwrap_or_default();
    format!("\"{}\"", &super::secret::sha256_hex(&body)[..16])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregate_has_rfc9728_shape_and_omits_scopes() {
        let servers = vec!["https://auth.example".to_string()];
        let doc = protected_resource_metadata("https://mcp.example/", &servers);
        assert_eq!(doc["resource"], "https://mcp.example"); // trailing slash trimmed
        assert_eq!(doc["bearer_methods_supported"], json!(["header"]));
        assert_eq!(
            doc["resource_signing_alg_values_supported"],
            json!(["RS256"])
        );
        assert_eq!(doc["authorization_servers"], json!(["https://auth.example"]));
        assert_eq!(doc["resource_documentation"], "https://mcp.example/docs");
        assert!(
            doc.get("scopes_supported").is_none(),
            "aggregate omits scopes_supported per DEC-905"
        );
    }

    #[test]
    fn signing_algs_never_advertise_eddsa_or_hs256() {
        // The verifier is RS256-only; advertising anything else would invite unverifiable tokens.
        let doc = protected_resource_metadata("https://mcp", &[]);
        let algs = doc["resource_signing_alg_values_supported"].as_array().unwrap();
        assert!(algs.iter().all(|a| a == "RS256"));
        assert!(!algs.iter().any(|a| a == "EdDSA" || a == "HS256"));
    }

    #[test]
    fn per_module_carries_the_modules_scopes() {
        let servers = vec!["https://auth.example".to_string()];
        let scopes = vec!["projects.read".to_string(), "projects.write".to_string()];
        let doc = protected_resource_metadata_for_module(
            "https://mcp.example/projects",
            &servers,
            &scopes,
        );
        assert_eq!(doc["resource"], "https://mcp.example/projects");
        assert_eq!(
            doc["scopes_supported"],
            json!(["projects.read", "projects.write"])
        );
    }

    #[test]
    fn per_module_empty_scopes_is_present_but_empty() {
        // Distinct from a 404: the module exists, it just exposes no scoped tools.
        let doc = protected_resource_metadata_for_module("https://mcp/x", &[], &[]);
        assert_eq!(doc["scopes_supported"], json!([]));
    }

    #[test]
    fn etag_is_deterministic_and_sensitive() {
        let a = protected_resource_metadata("https://mcp", &["https://a".to_string()]);
        let b = protected_resource_metadata("https://mcp", &["https://a".to_string()]);
        let c = protected_resource_metadata("https://mcp", &["https://b".to_string()]);
        assert_eq!(etag(&a), etag(&b), "same document yields the same etag");
        assert_ne!(etag(&a), etag(&c), "different document yields a different etag");
        let tag = etag(&a);
        assert_eq!(tag.len(), 18, "two quotes plus sixteen hex chars");
        assert!(tag.starts_with('"') && tag.ends_with('"'));
    }
}
