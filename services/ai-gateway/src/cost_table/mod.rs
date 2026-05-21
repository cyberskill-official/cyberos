//! FR-AI-007 — Provider cost-table loader.
//!
//! YAML-backed, hot-reloadable rate table for AI provider pricing.
//! Exposes `lookup(provider, model) -> Option<CostRate>` for synchronous
//! in-memory reads on the precheck hot path.
//!
//! See FR-AI-007 for normative behaviour and acceptance criteria.

pub mod loader;
pub mod schema;

pub use loader::{entry_count, init_cost_table, loaded_at, lookup};
pub use schema::{CostRate, CostTableHandle, FileFailure, LoaderInitError};
