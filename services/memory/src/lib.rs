//! `cyberos-memory` — Layer-2 ingest pipeline + search/graph projection.
//!
//! Implements FR-MEMORY-101 (Layer-2 ingest), FR-MEMORY-102 (rebuild CI gate),
//! FR-MEMORY-108 (search API). Per DEC-070, Layer 1 (the append-only chain
//! in the personal memory) is the source of truth — this service maintains
//! a read scale-out projection in Postgres + pgvector + Apache AGE.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod embeddings;
pub mod layer2;
pub mod rebuild;
pub mod search;
pub mod state;

pub use state::AppState;

/// Crate version, surfaced on /healthz for ops triage.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
