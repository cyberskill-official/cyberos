//! cyberos-obs-proxy - the tenant-aware Grafana proxy (FR-OBS-002).
//!
//! The proxy sits between Grafana and the OBS backends (Loki, Prometheus, Tempo) and AST-injects a
//! `tenant_id` label filter into every query so a tenant can never read another tenant's telemetry.
//!
//! Slice 1 (this file set) ships the LogQL injector and the shared error types - the hand-rolled,
//! no-external-crate security primitive. Later slices add the PromQL injector (promql-parser@0.4),
//! the TraceQL injector, JWT auth against the FR-AUTH-004 JWKS, the axum proxy router, and audit
//! emission. See `docs/feature-requests/obs/FR-OBS-002-tenant-aware-grafana.md`.

pub mod audit;
pub mod auth;
pub mod error;
pub mod forwarder;
pub mod handler;
pub mod inject;
pub mod proxy;

pub use error::{Backend, ProxyError};
