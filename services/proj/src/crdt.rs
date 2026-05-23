//! FR-PROJ-003 — deterministic CRDT/LWW collaboration helpers.
//!
//! This is the server-side model around Yjs updates: binary Yjs payloads are
//! stored and merged by `(client_id, clock)`, while scalar metadata uses an
//! LWW register with actor-id tie breaking for deterministic replay.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollaborativeField {
    Description,
    CommentBody,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct YjsUpdate {
    pub document_id: Uuid,
    pub field: CollaborativeField,
    pub client_id: String,
    pub clock: u64,
    pub payload: Vec<u8>,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrdtDocument {
    pub document_id: Uuid,
    pub field: CollaborativeField,
    pub updates: Vec<YjsUpdate>,
    pub state_vector: BTreeMap<String, u64>,
}

impl CrdtDocument {
    pub fn new(document_id: Uuid, field: CollaborativeField) -> Self {
        Self {
            document_id,
            field,
            updates: Vec::new(),
            state_vector: BTreeMap::new(),
        }
    }

    pub fn apply(&mut self, update: YjsUpdate) -> Result<bool, CrdtError> {
        if update.document_id != self.document_id || update.field != self.field {
            return Err(CrdtError::WrongDocument);
        }
        let current = self
            .state_vector
            .get(&update.client_id)
            .copied()
            .unwrap_or(0);
        if update.clock <= current {
            return Ok(false);
        }
        self.state_vector
            .insert(update.client_id.clone(), update.clock);
        self.updates.push(update);
        self.updates.sort_by(|a, b| {
            (a.client_id.as_str(), a.clock, a.received_at).cmp(&(
                b.client_id.as_str(),
                b.clock,
                b.received_at,
            ))
        });
        Ok(true)
    }

    pub fn missing_for(&self, peer_vector: &BTreeMap<String, u64>) -> Vec<YjsUpdate> {
        self.updates
            .iter()
            .filter(|u| u.clock > peer_vector.get(&u.client_id).copied().unwrap_or(0))
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CrdtError {
    #[error("update targets a different document or field")]
    WrongDocument,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LwwValue<T> {
    pub value: T,
    pub updated_at: DateTime<Utc>,
    pub actor_id: Uuid,
}

pub fn lww_merge<T: Clone>(left: LwwValue<T>, right: LwwValue<T>) -> LwwValue<T> {
    if (right.updated_at, right.actor_id) >= (left.updated_at, left.actor_id) {
        right
    } else {
        left
    }
}

pub fn reconnect_state(
    local: &BTreeMap<String, u64>,
    server: &BTreeMap<String, u64>,
) -> (BTreeSet<String>, BTreeSet<String>) {
    let clients = local
        .keys()
        .chain(server.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut upload = BTreeSet::new();
    let mut download = BTreeSet::new();
    for client in clients {
        let l = local.get(&client).copied().unwrap_or(0);
        let s = server.get(&client).copied().unwrap_or(0);
        if l > s {
            upload.insert(client.clone());
        }
        if s > l {
            download.insert(client);
        }
    }
    (upload, download)
}
