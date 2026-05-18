//! Polls Layer-1's append-only audit log for new rows. Stubbed in Wave 1.

/// A single Layer-1 row as observed during a tail poll.
#[derive(Debug, Clone)]
pub struct L1Row {
    pub tenant_id: uuid::Uuid,
    pub seq: i64,
    pub path: String,
    pub body: String,
    pub prev_hash_hex: Option<String>,
    pub chain_anchor_hex: String,
}
