//! `cyberos-proj` — PROJ service slices.
//!
//! Issues + Cycles + Engagements + cross-module links + status FSM.
//!
//! ### Modules
//!   * [`types`] — domain enums + structs ([`IssueStatus`], [`IssuePriority`], [`LinkType`], [`Issue`], …)
//!   * [`status_fsm`] — legal-transition table per §1 #3
//!   * [`audit`] — memory audit row builders (`proj.issue_created` + 3 more)
//!   * [`decisions`] — `proj.decision` payloads for status-change rationale
//!   * [`rate_card`] — append-only rate-card versioning helpers
//!   * [`memory_link`] — typed Issue-to-memory links
//!   * [`errors`] — `IssueError` carrying clause-level error codes
//!   * [`repo`] — sqlx CRUD layer
//!   * [`handlers`] — axum handler functions
//!   * [`links`] — bidirectional symmetric-link writer

pub mod audit;
pub mod decisions;
pub mod errors;
pub mod handlers;
pub mod links;
pub mod memory_link;
pub mod rate_card;
pub mod repo;
pub mod status_fsm;
pub mod types;

pub use errors::IssueError;
pub use types::*;
