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

pub mod brain_bridge;
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
pub mod rbac;
pub mod saml;
pub mod saml_sig;
pub mod state;
pub mod travel;
pub mod travel_admin;
pub mod travel_policy;

pub use state::AppState;

/// Crate version, surfaced on /healthz for ops triage.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
