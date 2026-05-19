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
// `missing_docs` is deferred — see services/auth/src/lib.rs for the rationale.
// Tracking: FR-AI-NNN-restore-missing-docs-lint (TBD).
#![allow(missing_docs)]
// Style-class clippy lints suppressed at crate level — see services/auth/src/lib.rs
// for the rationale and tracking FR. Same hygiene-wave plan.
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]
// Preemptive style-class allows matching the auth crate's baseline — keeps
// the same lint posture across the workspace so a future CI run doesn't
// surface module-by-module lint storms.
#![allow(clippy::doc_overindented_list_items)]
#![allow(clippy::let_and_return)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::needless_borrow)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::useless_format)]
#![allow(clippy::manual_pattern_char_comparison)]
#![allow(clippy::double_ended_iterator_last)]
#![allow(dead_code)]

pub mod policy;
pub mod memory_writer;

/// Service banner used by binaries on startup.
pub const SERVICE_BANNER: &str = concat!(
    "cyberos-ai-gateway v",
    env!("CARGO_PKG_VERSION"),
    " — cost-of-everything gate (FR-AI-001..022)"
);
