//! FR-PROJ-008 — PROJ history events chained to memory audit rows.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldDiff {
    pub field: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub issue_id: Uuid,
    pub actor_subject_id: Uuid,
    pub request_id: String,
    pub diffs: Vec<FieldDiff>,
    pub memory_audit_chain: String,
    pub event_hash: String,
}

pub fn build_history_event(
    tenant_id: Uuid,
    issue_id: Uuid,
    actor_subject_id: Uuid,
    request_id: impl Into<String>,
    diffs: Vec<FieldDiff>,
    memory_audit_chain: impl Into<String>,
) -> HistoryEvent {
    let request_id = request_id.into();
    let memory_audit_chain = memory_audit_chain.into();
    let mut hasher = Sha256::new();
    hasher.update(tenant_id.as_bytes());
    hasher.update(issue_id.as_bytes());
    hasher.update(actor_subject_id.as_bytes());
    hasher.update(request_id.as_bytes());
    for diff in &diffs {
        hasher.update(diff.field.as_bytes());
        hasher.update(diff.before.as_deref().unwrap_or("").as_bytes());
        hasher.update(diff.after.as_deref().unwrap_or("").as_bytes());
    }
    hasher.update(memory_audit_chain.as_bytes());
    HistoryEvent {
        id: Uuid::new_v4(),
        tenant_id,
        issue_id,
        actor_subject_id,
        request_id,
        diffs,
        memory_audit_chain,
        event_hash: hex(&hasher.finalize()),
    }
}

fn hex(bytes: &[u8]) -> String {
    const TABLE: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(TABLE[(b >> 4) as usize] as char);
        out.push(TABLE[(b & 0x0f) as usize] as char);
    }
    out
}
