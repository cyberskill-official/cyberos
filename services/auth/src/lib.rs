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
#![warn(missing_docs)]

pub mod handlers;
pub mod idempotency;
pub mod jwt;
pub mod keygen;
pub mod middleware;
pub mod models;
pub mod rbac;
pub mod state;

pub use state::AppState;

/// Crate version, surfaced on /healthz for ops triage.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
