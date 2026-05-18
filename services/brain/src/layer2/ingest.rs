//! Layer-2 ingest orchestrator. Wave-1 first-slice: stub.
//!
//! Future fill-in (separate FRs):
//!   * pull a batch from `binlog_tail::poll`
//!   * verify every row with `chain_anchor::verify`
//!   * extract entities via `entity_extract::run`
//!   * upsert into pgvector + age
//!   * advance the per-tenant cursor
//!
//! The point of stubbing now is so callers can wire types without the
//! full pipeline being live yet.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("not yet implemented — FR-BRAIN-101 Wave 1 scaffold only")]
    NotYetImplemented,
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

/// Run a single ingest batch. Returns the number of rows materialized.
pub async fn run_batch() -> Result<usize, IngestError> {
    Err(IngestError::NotYetImplemented)
}
