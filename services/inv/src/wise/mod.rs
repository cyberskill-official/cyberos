//! Wise webhook intake (TASK-INV-004 library + TASK-INV-012 HTTP host).

mod handler;
mod parser;
mod processor;
mod public_key;
mod signature;
mod types;
mod wal;

pub use handler::{router, WiseHostState, MAX_WEBHOOK_BODY};
pub use parser::{is_stale, parse_event_type, profile_id_mismatch, WiseEvent};
pub use processor::{CashAppStub, InMemoryReceipt, WiseProcessor};
pub use public_key::{
    CachedPublicKeys, PublicKeyError, PublicKeySource, RotatingPemSource, StaticPemSource,
    PUBLIC_KEY_TTL,
};
pub use signature::{verify_signature, WiseVerifyError};
pub use types::{
    WiseEventType, WiseReceiptState, WISE_EVENT_TYPE_CARDINALITY, WISE_RECEIPT_STATE_CARDINALITY,
    WISE_STALE_DAYS,
};
pub use wal::{WiseWalError, WiseWalItem, WiseWalQueue, WISE_WAL_CAPACITY};
