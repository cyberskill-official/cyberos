//! FR-AI-005 — Per-tenant policy loader.
//!
//! Loads `config/tenants/<tenant_id>.yaml` files at startup, validates them against the
//! closed `TenantPolicy` schema, caches them lock-free via `ArcSwap`, and hot-reloads on
//! file change via the `notify` crate. Schema-invalid files cause init to fail with all
//! errors aggregated; missing files cause `load_for_tenant()` to return
//! `PolicyError::PolicyMissing` (no silent defaults).
//!
//! Public entry points:
//!
//! - [`init_loader`] — call once at AI Gateway startup; eagerly validates all YAMLs.
//! - [`load_for_tenant`] — call from hot path; sub-microsecond on cache hit.
//! - [`shutdown_loader`] — call on graceful shutdown; idempotent.
//! - [`validate_yaml`] — pure function used by `cyberos-ai policy validate` (FR-AI-021).
//!
//! See FR-AI-005 §1 for normative behaviour, §4 for acceptance criteria.

pub mod cache;
pub mod loader;
pub mod schema;

pub use loader::{
    init_loader, load_for_tenant, shutdown_loader, validate_yaml, FileFailure, Loader,
    LoaderInitError, PolicyError,
};
pub use schema::{AiPolicy, EmergencyOverride, Provider, Residency, TenantPolicy};
