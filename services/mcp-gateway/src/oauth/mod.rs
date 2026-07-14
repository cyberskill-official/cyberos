//! TASK-MCP-004: OAuth 2.1 + PKCE authorization-code flow for the MCP Gateway.
//!
//! Slice 1 (foundation): the PKCE S256 verifier ([`pkce`]) and the closed RFC 6749 error type
//! ([`error`]).
//!
//! Slice 2 (this slice) builds the authorization server. The pure, database-free pieces land first
//! and are unit-tested here:
//! - [`enums`] - the closed grant-type / client-type / code-state / refresh-state sets, each with a
//!   cardinality tripwire (DEC-807/808, clauses #5/#4/#16/#17).
//! - [`audience`] - audience binding and the exact-match check that prevents cross-server token
//!   replay (DEC-802, RFC 8707, clauses #7/#23).
//! - [`scope`] - RFC 6749 §3.3 scope syntax and closed-set membership against the `tools/list`
//!   registry (DEC-813, clause #30).
//!
//! The database-bound endpoints (authorize, token, refresh, dynamic client registration, revoke,
//! introspect, discovery), the JWT minting against the TASK-AUTH-004 `auth_signing_keys` / JWKS, and
//! the audience check wired into the `tools/call` hot path are the remaining slice-2 work, planned in
//! `docs/tasks/mcp/MCP-004-SLICE2-PLAN.md`. They use runtime-checked `sqlx::query(...)`,
//! so they compile without a live database; only the integration tests need Postgres.
//!
//! The OAuth migrations are `0013_oauth_clients.sql` / `0014_oauth_codes.sql` /
//! `0015_oauth_refresh_families.sql` - renumbered off 0010-0012, which the backlog reserves for the
//! TASK-MCP-007 tasks and TASK-MCP-008 elicitation tables.

pub mod audience;
pub mod audit;
pub mod authorize;
pub mod authsession;
pub mod dcr;
pub mod discovery;
pub mod enums;
pub mod error;
pub mod introspect;
pub mod jwt;
pub mod pkce;
pub mod prm;
pub mod response;
pub mod revoke;
pub mod scope;
pub mod secret;
pub mod store;
pub mod token;
