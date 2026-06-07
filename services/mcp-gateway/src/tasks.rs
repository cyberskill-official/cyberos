//! FR-MCP-007 — Tasks primitive for long-running work.

use std::collections::BTreeMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Long-running task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task was accepted.
    Pending,
    /// Task is executing.
    Running,
    /// Task completed.
    Completed,
    /// Task failed.
    Failed,
}

/// Task record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskRecord {
    /// Stable task id.
    pub task_id: Uuid,
    /// Tool name.
    pub tool_name: String,
    /// Status.
    pub status: TaskStatus,
    /// Last checkpoint payload.
    pub checkpoint: Option<Value>,
    /// Creation time.
    pub created_at: DateTime<Utc>,
    /// Update time.
    pub updated_at: DateTime<Utc>,
}

/// In-memory task store.
#[derive(Debug, Default)]
pub struct TaskStore {
    tasks: RwLock<BTreeMap<Uuid, TaskRecord>>,
}

impl TaskStore {
    /// Create an empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a task.
    pub fn start(&self, tool_name: &str) -> TaskRecord {
        let now = Utc::now();
        let task = TaskRecord {
            task_id: Uuid::new_v4(),
            tool_name: tool_name.into(),
            status: TaskStatus::Pending,
            checkpoint: None,
            created_at: now,
            updated_at: now,
        };
        self.tasks
            .write()
            .expect("poisoned")
            .insert(task.task_id, task.clone());
        task
    }

    /// Update status and checkpoint.
    pub fn update(
        &self,
        task_id: Uuid,
        status: TaskStatus,
        checkpoint: Option<Value>,
    ) -> Result<TaskRecord, String> {
        let mut guard = self.tasks.write().expect("poisoned");
        let task = guard
            .get_mut(&task_id)
            .ok_or_else(|| "task_not_found".to_string())?;
        task.status = status;
        task.checkpoint = checkpoint;
        task.updated_at = Utc::now();
        Ok(task.clone())
    }

    /// Fetch task status.
    pub fn get(&self, task_id: Uuid) -> Option<TaskRecord> {
        self.tasks.read().expect("poisoned").get(&task_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_can_checkpoint_and_resume() {
        let store = TaskStore::new();
        let task = store.start("cyberos.proj.export_issue");
        store
            .update(
                task.task_id,
                TaskStatus::Running,
                Some(serde_json::json!({"step": 2})),
            )
            .unwrap();
        let resumed = store.get(task.task_id).unwrap();
        assert_eq!(resumed.checkpoint.unwrap()["step"], 2);
    }
}
