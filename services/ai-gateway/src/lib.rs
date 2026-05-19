//! cyberos-ai-gateway — AI Gateway service for the CyberOS platform.
//!
//! ## Module map
//!
//! - [`policy`] — FR-AI-005: per-tenant YAML policy loader (cap · warn · override · residency).
//! - [`memory_writer`] — FR-AI-003: subprocess bridge to the canonical memory Writer.
//! - [`cost_ledger`] — FR-AI-001/002/004: pre-call check · post-call reconcile · expiry cleanup.
//!
//! ## P0 slice 1 (shipped here)
//!
//! - **FR-AI-005**: tenant policy loader (fully implemented, all 10 ACs tested).
//! - **FR-AI-003**: memory-writer subprocess bridge (core happy path + path-traversal guard;
//!   chain-verification + concurrent serialisation tested).
//!
//! Subsequent slices land FR-AI-001/002/004 (cost ledger) and FR-AI-006..022 (router · PII ·
//! residency · cache · operator CLI) per the build order locked in `docs/feature-requests/BACKLOG.md`.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod policy;
pub mod memory_writer;

/// Service banner used by binaries on startup.
pub const SERVICE_BANNER: &str = concat!(
    "cyberos-ai-gateway v",
    env!("CARGO_PKG_VERSION"),
    " — cost-of-everything gate (FR-AI-001..022)"
);
