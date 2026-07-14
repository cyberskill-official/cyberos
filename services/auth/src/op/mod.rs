//! TASK-AUTH-110: AUTH as a first-party OIDC provider (authorization server).
//!
//! The inverse of [`crate::oidc`] (TASK-AUTH-104, where AUTH is Google's client):
//! here AUTH is the provider that first-party apps (CHAT/Mattermost, PORTAL)
//! federate to, so the whole platform shares one CyberOS identity, one role set,
//! and one central revoke.
//!
//! Slice 1a (this slice) - the pure, database-free pieces, unit-tested here:
//! - [`errors`]   - the closed RFC 6749 / OIDC error set.
//! - [`discovery`] - the OIDC + RFC 8414 metadata document (`/.well-known/openid-configuration`).
//! - [`pkce`]     - PKCE S256 verification (RFC 7636).
//! - [`redirect`] - exact redirect_uri matching (DEC-2491; no wildcard / substring).
//! - [`id_token`] - the OIDC id_token claim builder + RS256 signer, reusing the
//!   TASK-AUTH-004 `auth_signing_keys` path (one JWKS for the platform, DEC-2481).
//!
//! Slice 1b-data (this slice) - the database-access + audit layer, backed by
//! migrations 0027-0030 (runtime-checked `sqlx::query(...)`, so it compiles
//! without a live database; the integration tests need Postgres):
//! - [`audit`]       - the 7 memory-row payload builders (DEC-2494), mirroring `memory_bridge`.
//! - [`code_store`]  - single-use, 60s-TTL, PKCE-bound auth codes + the consumptions guard.
//! - [`sso_session`] - create / lookup-active / touch / revoke-for-subject (the §1 #26 cascade).
//!
//! Slice 1b-endpoints (next) - authorize (SSO-cookie silent SSO or upstream-Google
//! broker via TASK-AUTH-104, revoke-gated), token (code -> id_token + access_token),
//! userinfo, the first-party RP-client registry CRUD, and the route wiring in
//! `handlers`. They compose the slice-1a + slice-1b-data pieces above.
//!
//! ADRs (audit OPEN-001, decided 2026-06-29):
//! 1. Code single-use is enforced by the sibling `auth_oidc_code_consumptions`
//!    (code_hash PK) first-insert-wins guard, leaving the codes table reapable
//!    rather than forced-append-only. The append-only forensic record is
//!    `auth_op_login_history` (0030).
//! 2. The OAuth substrate is re-implemented thin here in `auth::op` rather than
//!    extracting `services/mcp-gateway/src/oauth/` into a shared crate now; revisit
//!    if a third first-party provider appears.

pub mod audit;
pub mod code_store;
pub mod discovery;
pub mod errors;
pub mod handlers;
pub mod id_token;
pub mod pkce;
pub mod redirect;
pub mod rp_client;
pub mod sso_session;
