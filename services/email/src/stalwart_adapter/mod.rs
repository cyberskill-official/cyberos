//! FR-EMAIL-001 §3.6 — Stalwart adapter layer.
//!
//! Slice 1 ships the inbound + outbound shape; the wire to a live Stalwart
//! instance lands in FR-EMAIL-002 alongside the JWT bridge plugin.

pub mod inbound;
pub mod outbound;

pub use inbound::*;
pub use outbound::*;
