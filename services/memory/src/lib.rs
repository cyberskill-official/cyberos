//! `cyberos-memory` — Layer-2 ingest pipeline + search/graph projection.
//!
//! Implements FR-MEMORY-101 (Layer-2 ingest), FR-MEMORY-102 (rebuild CI gate),
//! FR-MEMORY-108 (search API). Per DEC-070, Layer 1 (the append-only chain
//! in the personal memory) is the source of truth — this service maintains
//! a read scale-out projection in Postgres + pgvector (relational graph edges in l2_edge).

#![forbid(unsafe_code)]
// `missing_docs` is deferred — see services/auth/src/lib.rs for the rationale.
// Tracking: FR-MEMORY-NNN-restore-missing-docs-lint (TBD).
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

// FR-MEMORY-123 — the BRAIN: the captured FR-MEMORY-121 interaction-event log becomes a fast, persistent,
// citable brain. Ingestion + embedding (via the ai-gateway) + rolling summaries + hot/warm/cold tiering +
// access-scoped, provenance-carrying recall. A DERIVED, rebuildable lens over l1_audit_log (the chain stays
// the system of record); it does NOT touch the live auth/chat services.
pub mod brain;
pub mod embeddings;
// FR-MEMORY-121 — the interaction-event capture primitive (event shape + emit + content_ref + consent
// gate). Aux rows on the existing l1_audit_log chain; no second store.
pub mod interaction;
pub mod layer2;
pub mod rebuild;
pub mod search;
pub mod state;

pub use state::AppState;

/// Crate version, surfaced on /healthz for ops triage.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
