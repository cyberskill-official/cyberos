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

pub mod totp;
pub mod webauthn;
pub mod recovery;
pub mod challenge;
pub mod lockout;
pub mod repo;
pub mod handlers;

// Re-export handler functions so `crate::mfa::totp_enrol_start` etc. still works.
pub use handlers::{
    totp_enrol_start,
    totp_enrol_finish,
    totp_verify,
    list_factors,
    revoke_factor,
    generate_recovery_codes,
    verify_recovery_code,
    EnrolStartBody,
    EnrolStartResponse,
    VerifyBody,
    RecoveryCodesResponse,
    RecoveryVerifyBody,
};
