//! `cyberos-proj` — FR-PROJ-001 slice 1.
//!
//! Issues + Cycles + Engagements + cross-module links + status FSM.
//!
//! ### Modules
//!   * [`types`] — domain enums + structs ([`IssueStatus`], [`IssuePriority`], [`LinkType`], [`Issue`], …)
//!   * [`status_fsm`] — legal-transition table per §1 #3
//!   * [`audit`] — memory audit row builders (`proj.issue_created` + 3 more)
//!   * [`errors`] — `IssueError` carrying clause-level error codes
//!   * [`repo`] — sqlx CRUD layer
//!   * [`handlers`] — axum handler functions
//!   * [`links`] — bidirectional symmetric-link writer

pub mod types;
pub mod status_fsm;
pub mod audit;
pub mod errors;
pub mod repo;
pub mod handlers;
pub mod links;

pub use errors::IssueError;
pub use types::*;
