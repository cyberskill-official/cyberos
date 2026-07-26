//! Metering — four-axis append-only usage events (TASK-TEN-004).

#![deny(missing_debug_implementations)]

pub mod axes;
pub mod policy;
pub mod recorder;
pub mod wal_queue;

pub use axes::{MeteringAxis, METERING_AXIS_CARDINALITY};
pub use policy::{OverageDecision, OveragePolicy};
pub use recorder::{InMemoryRecorder, MeteringEvent, QuantityError, RecordError, Recorder};
