//! FR-MCP-004: OAuth 2.1 + PKCE authorization-code flow for the MCP Gateway.
//!
//! Slice 1 (foundation): the PKCE S256 verifier ([`pkce`]) and the closed RFC 6749 error type
//! ([`error`]). The OAuth endpoints (authorize, token, refresh, dynamic client registration,
//! revoke, introspect, discovery), the JWT minting against the FR-AUTH-004 JWKS, the audience
//! verification at the tools/call hot path, and the Postgres-backed flows land in slice 2. The
//! migrations (0010-0012) are authored alongside this slice but are not yet wired or exercised.

pub mod error;
pub mod pkce;
