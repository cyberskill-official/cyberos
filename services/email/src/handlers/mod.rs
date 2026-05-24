//! FR-EMAIL-001 §1 #19 — internal REST handlers.

pub mod camel;
pub mod delivery_auth;
pub mod dsar;
pub mod outbound;
pub mod status;

pub use status::*;
