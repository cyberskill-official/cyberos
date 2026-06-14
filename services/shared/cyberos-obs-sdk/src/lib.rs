//! CyberOS shared observability SDK.
//!
//! FR-OBS-003 centralises RED request metrics so services cannot drift into
//! incompatible label vocabularies or bucket boundaries.

pub mod axum_layer;
pub mod cardinality_guard;
pub mod exemplar;
pub mod logging;
pub mod red;
pub mod tracecontext;

pub use axum_layer::RedLayer;
pub use cyberos_obs_sdk_macros::red_instrument;
pub use red::{init, InitError};
