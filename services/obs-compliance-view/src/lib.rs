//! cyberos-obs-compliance-view — FR-OBS-008 compliance views.
//!
//! The service exposes read-only, tenant-scoped views over memory audit rows
//! for EU AI Act, PDPL, SOC 2, and ISO 27001 evidence.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod auth;
pub mod chain_proof;
pub mod error;
pub mod export;
pub mod memory;
pub mod metrics;
pub mod router;
pub mod views;

pub use router::{app, AppState, ViewQuery};

/// Service banner emitted at startup.
pub const SERVICE_BANNER: &str = concat!(
    "cyberos-obs-compliance-view v",
    env!("CARGO_PKG_VERSION"),
    " — compliance views (FR-OBS-008)"
);
