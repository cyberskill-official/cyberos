//! FR-MCP-008 — Elicitation request/response state.

use std::collections::BTreeMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Elicitation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElicitationStatus {
    /// Waiting for user response.
    Pending,
    /// User supplied an answer.
    Answered,
    /// Request expired or was cancelled.
    Closed,
}

/// Elicitation record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Elicitation {
    /// Stable id.
    pub elicitation_id: Uuid,
    /// Prompt shown to the user.
    pub prompt: String,
    /// Current status.
    pub status: ElicitationStatus,
    /// Optional answer.
    pub answer: Option<String>,
    /// Creation time.
    pub created_at: DateTime<Utc>,
}

/// In-memory elicitation store.
#[derive(Debug, Default)]
pub struct ElicitationStore {
    items: RwLock<BTreeMap<Uuid, Elicitation>>,
}

impl ElicitationStore {
    /// Create a pending elicitation.
    pub fn request(&self, prompt: &str) -> Result<Elicitation, String> {
        if prompt.trim().is_empty() {
            return Err("prompt_required".into());
        }
        let item = Elicitation {
            elicitation_id: Uuid::new_v4(),
            prompt: prompt.into(),
            status: ElicitationStatus::Pending,
            answer: None,
            created_at: Utc::now(),
        };
        self.items
            .write()
            .expect("poisoned")
            .insert(item.elicitation_id, item.clone());
        Ok(item)
    }

    /// Answer a pending elicitation.
    pub fn answer(&self, id: Uuid, answer: &str) -> Result<Elicitation, String> {
        let mut guard = self.items.write().expect("poisoned");
        let item = guard
            .get_mut(&id)
            .ok_or_else(|| "elicitation_not_found".to_string())?;
        if item.status != ElicitationStatus::Pending {
            return Err("elicitation_not_pending".into());
        }
        item.status = ElicitationStatus::Answered;
        item.answer = Some(answer.into());
        Ok(item.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elicitation_answers_once() {
        let store = ElicitationStore::default();
        let item = store.request("Approve delete?").unwrap();
        assert_eq!(
            store.answer(item.elicitation_id, "yes").unwrap().status,
            ElicitationStatus::Answered
        );
        assert!(store.answer(item.elicitation_id, "again").is_err());
    }
}
