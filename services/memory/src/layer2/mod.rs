//! Layer-2 ingest pipeline.
//!
//! - `ingest`       — orchestrates `binlog_tail -> chain_anchor -> entity_extract -> pgvector`
//! - `binlog_tail`  — polls Layer 1's append-only audit log for new rows
//! - `chain_anchor` — verifies every row's chain hash before materializing
//! - `entity_extract` — pulls people/orgs/projects/decisions out of bodies
//! - `pgvector`     — writes l2_memory + computes embeddings (FR-AI-019)
//! - `cursor`       — per-tenant ingest cursor (DEC-073)
//!
//! Wave 1 first-slice ships the public types + cursor read/write. The
//! actual ingest loop (`ingest::run`) is stubbed and returns
//! `not_yet_implemented` so the binary boots cleanly. Future FRs fill in
//! the body without changing module boundaries.

pub mod binlog_tail;
pub mod chain_anchor;
pub mod cursor;
pub mod entity_extract;
pub mod ingest;
pub mod pgvector;

pub use cursor::{Cursor, CursorStore};
