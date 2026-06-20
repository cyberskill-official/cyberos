//! cyberos-obs-router - routes Alertmanager webhook fires through CUO's `obs.triage-alert` skill to CHAT
//! or PagerDuty (FR-OBS-007).
//!
//! Slice 1 (this file set) ships the pure routing core: severity parsing and the (severity, confidence)
//! routing decision, with confidence clamping and the sev-1-always-pages-both rule. This is the part
//! that decides where every alert goes, exhaustively testable without any network. Later slices add the
//! Alertmanager webhook parser, the CUO / CHAT / PagerDuty clients, deduplication, the ack handler, and
//! the axum shell. See `docs/feature-requests/obs/FR-OBS-007-alertmanager-cuo-runbook-routing.md`.

pub mod route;
pub mod severity;

pub use route::{clamp_confidence, decide, Route, CONFIDENCE_FLOOR};
pub use severity::Severity;
