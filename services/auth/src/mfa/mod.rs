//! FR-AUTH-102 — Multi-Factor Authentication module.
//!
//! Sub-modules:
//!   * `totp`      — RFC 6238 TOTP generation + verification
//!   * `webauthn`  — WebAuthn factor counter validation
//!   * `recovery`  — Single-use bcrypt-hashed recovery codes
//!   * `challenge` — Challenge FSM (pending → consumed | expired | failed)
//!   * `lockout`   — 5/15/30 lockout state machine
//!   * `repo`      — Database CRUD (RLS-aware)
//!   * `handlers`  — Axum HTTP handlers

pub mod challenge;
pub mod handlers;
pub mod lockout;
pub mod recovery;
pub mod repo;
pub mod totp;
pub mod webauthn;

// Re-export handler functions so `crate::mfa::totp_enrol_start` etc. still works.
pub use handlers::{
    generate_recovery_codes, list_factors, revoke_factor, totp_enrol_finish, totp_enrol_start,
    totp_verify, verify_recovery_code, EnrolStartBody, EnrolStartResponse, RecoveryCodesResponse,
    RecoveryVerifyBody, VerifyBody,
};
