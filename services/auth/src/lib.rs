//! `cyberos-auth` — tenant + subject + RLS + JWT + MFA + SSO.
//!
//! Wave 2 first-slice (2026-05-18) ships:
//!   * tenant create  (FR-AUTH-001)
//!   * subject create (FR-AUTH-002)
//!   * RLS migrations (FR-AUTH-003)
//!
//! Remaining FR-AUTH-004..109 surfaces (JWT/JWKS, admin REST, bootstrap CLI,
//! RBAC, MFA TOTP/WebAuthn/Passkey, SAML, OIDC, impossible-travel, HIBP,
//! Lumi-tenant identity, migration tooling) land in follow-up FRs.

#![forbid(unsafe_code)]
// `missing_docs` is deferred — re-enable per-module as docs land. With CI's
// `RUSTFLAGS: -D warnings`, keeping `warn(missing_docs)` would block every PR
// on undoc'd pub items (192+ in this crate alone after the FR-AUTH-106 slice-3
// drop). Tracking: FR-AUTH-NNN-restore-missing-docs-lint (TBD).
#![allow(missing_docs)]
// Style-class clippy lints that the existing FR-AUTH-106 slice-3 drop trips
// without affecting correctness. Suppressed at crate level to unblock CI;
// re-enable + refactor as a separate hygiene wave (FR-AUTH-NNN-clippy-style-cleanup).
//   * `doc_lazy_continuation` — doc comments need blank-line / indent fix in
//     auth/src/travel.rs (cosmetic).
//   * `type_complexity` — sqlx::query_as return tuples that clippy wants
//     type-aliased; refactor postponed until the SQL surface stabilises.
//   * `too_many_arguments` — `emit_travel_audit(8 args)` exceeds default 7;
//     ergonomic struct refactor postponed for the same reason.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod memory_bridge;
pub mod cursor;
pub mod deny_list;
pub mod geoip;
pub mod handlers;
pub mod hibp;
pub mod idempotency;
pub mod jwt;
pub mod keygen;
pub mod lumi;
pub mod mfa;
pub mod middleware;
pub mod migration_state;
pub mod models;
pub mod oidc;
pub mod passkey;
pub mod password;
pub mod rate_limit;
pub mod rls;
pub mod rbac;
pub mod scope_map;
pub mod saml;
pub mod saml_sig;
pub mod sessions;
pub mod state;
pub mod travel;
pub mod travel_admin;
pub mod travel_policy;

pub use state::AppState;

/// Crate version, surfaced on /healthz for ops triage.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
