//! cyberos-obs-router — FR-OBS-007 Alertmanager routing service.
//!
//! The service accepts Alertmanager webhooks, invokes the CUO
//! `obs.triage-alert@1` skill, and routes each alert to CHAT, PagerDuty, or
//! both according to severity and confidence.

#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

pub mod ack_handler;
pub mod alertmanager_webhook;
pub mod chat_post;
pub mod cuo_triage;
pub mod dedup;
pub mod memory;
pub mod metrics;
pub mod pagerduty;
pub mod router;
pub mod severity;

pub use router::{app, RouterConfig, RouterError, RouterState};

/// Service banner emitted at startup.
pub const SERVICE_BANNER: &str = concat!(
    "cyberos-obs-router v",
    env!("CARGO_PKG_VERSION"),
    " — Alertmanager CUO triage router (FR-OBS-007)"
);
