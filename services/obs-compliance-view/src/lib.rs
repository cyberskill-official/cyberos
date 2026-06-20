//! cyberos-obs-compliance-view - read-only compliance views over the memory audit chain (FR-OBS-008).
//!
//! Slice 1 (this file set) ships the pure core: the per-view audit-kind table (which row kinds each
//! regulation's view selects - the auditable contract), the time-window validation, and the Ed25519
//! chain-proof an auditor verifies independently of CyberOS. Later slices add the auditor-JWT auth, the
//! read-only memory query, the PII-scan defence, the summary rendering, the PDF / JSON export, and the
//! axum shell. See `docs/feature-requests/obs/FR-OBS-008-compliance-view-scoping.md`.

pub mod proof;
pub mod views;
pub mod window;

pub use proof::{sign, verify, Proof};
pub use views::View;
pub use window::{validate, WindowError, MAX_WINDOW_SECS};
