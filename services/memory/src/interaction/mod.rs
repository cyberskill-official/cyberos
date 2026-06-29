//! FR-MEMORY-121 — the single BRAIN capture primitive: the one work-interaction event shape every CyberOS
//! module emits, plus its emit API, content-reference discipline, and consent gate.
//!
//! Layout:
//!   * [`event`] — the typed [`InteractionEvent`] + closed enums + the typed builder + the canonical
//!     audit-row body (DEC-2700, DEC-2704).
//!   * [`content_ref`] — [`ContentRef`], a closed pointer/hash/none union; never raw content (DEC-2701).
//!   * [`consent_gate`] — the [`ConsentGate`] hook + default-deny [`DenyAll`] stub (DEC-2702); the real
//!     FR-EVAL-001-backed impl is wired by FR-MEMORY-122.
//!   * [`emit`] — [`emit`], which validates, consults the gate, and writes the event as an aux row on the
//!     hash-chained `l1_audit_log` via `cyberos-audit-chain` (DEC-2703).
//!
//! The published contract other modules depend on is `services/memory/contracts/interaction-event.schema
//! .json` (frozen at `schema_version: 1`).

pub mod backfill;
pub mod consent_gate;
pub mod content_ref;
pub mod emit;
pub mod event;

// Public API surface — emitters import from `cyberos_memory::interaction`.
pub use backfill::{backfill_chat, BackfillReport};
pub use consent_gate::{AllowAll, CachingGate, ConsentGate, DenyAll};
pub use content_ref::ContentRef;
pub use emit::{emit, EmitError, EmitOutcome, SkipReason};
pub use event::{
    canonical_audit_body, EventClass, InteractionEvent, InteractionEventBuilder, Module,
    SourceChannel, TargetRef, AUDIT_ROW_KIND, MAX_ATTRIBUTES_BYTES, MAX_BODY_BYTES, SCHEMA_VERSION,
};
