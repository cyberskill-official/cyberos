//! INV — invoice / cash rails. TASK-INV-004 ships the Wise webhook intake.

#![deny(missing_debug_implementations)]

pub mod wise;

pub use wise::{
    parse_event_type, verify_signature, WiseEventType, WiseReceiptState, WiseVerifyError,
    WISE_EVENT_TYPE_CARDINALITY, WISE_RECEIPT_STATE_CARDINALITY,
};
