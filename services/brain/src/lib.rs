//! `cyberos-brain` — Layer-2 ingest pipeline + search/graph projection.
//!
//! Implements FR-BRAIN-101 (Layer-2 ingest), FR-BRAIN-102 (rebuild CI gate),
//! FR-BRAIN-108 (search API). Per DEC-070, Layer 1 (the append-only chain
//! in the personal BRAIN) is the source of truth — this service maintains
//! a read scale-out projection in Postgres + pgvector + Apache AGE.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod layer2;
pub mod search;
pub mod state;

pub use state::AppState;

/// Crate version, surfaced on /healthz for ops triage.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
