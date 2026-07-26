//! Metering — four-axis append-only usage events (TASK-TEN-004 / TEN-206 / TEN-207).

#![deny(missing_debug_implementations)]

pub mod admission;
pub mod axes;
pub mod drain;
pub mod pg;
pub mod policy;
pub mod recorder;
pub mod usage;
pub mod wal_queue;

pub use admission::{admit, AdmissionOutcome, AdmissionRequest};
pub use axes::{MeteringAxis, METERING_AXIS_CARDINALITY};
pub use drain::drain_to_recorder;
pub use policy::{OverageDecision, OveragePolicy};
pub use recorder::{InMemoryRecorder, MeteringEvent, QuantityError, RecordError, Recorder};
pub use usage::{sum_in_memory, utc_month_start};
