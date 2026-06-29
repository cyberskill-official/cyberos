//! FR-MCP-007 tasks primitive: a long-running tool call tracked by an opaque handle, polled for status,
//! and read for its result when complete.
//!
//! In the synchronous gateway this realizes the build-plan slice ("a task starts and returns a handle;
//! status moves running -> complete; the result is fetchable; an unknown task id errors cleanly") as an
//! in-memory store, the dev-real analog of the in-memory
//! [`ToolRegistry`](crate::federation::registry::ToolRegistry). A worker (or, in tests, the caller)
//! drives a task through its terminal state; the store records the transitions and serves status.
//!
//! Deferred to the DB slice (none load-bearing for the lifecycle): the `mcp_tasks` table + RLS, the
//! per-module bounded-concurrency worker pool, KMS-encrypted payloads, NATS progress push, checkpoints
//! and crash resume, the `long_running` annotation routing in `tools/call`, idempotency keys, per-tenant
//! rate limiting, TTL expiry sweeping, and 30-day pruning. The closed status enum and the
//! start -> running -> terminal lifecycle are the durable contract and live here unchanged.

use std::collections::HashMap;
use std::sync::RwLock;

use serde_json::{json, Value};
use uuid::Uuid;

/// The lifecycle of a task (DEC-1101). [`TaskStatus::ALL`] pins the cardinality at 6.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TaskStatus {
    /// Queued, not yet started (over a concurrency limit in the deferred worker pool).
    Pending,
    /// Executing.
    Running,
    /// Finished successfully; a result is available.
    Completed,
    /// Finished with an error.
    Failed,
    /// Cancelled by the caller.
    Cancelled,
    /// Past its TTL (set by the deferred sweeper).
    Expired,
}

impl TaskStatus {
    /// Every variant, for the cardinality test.
    pub const ALL: [TaskStatus; 6] = [
        TaskStatus::Pending,
        TaskStatus::Running,
        TaskStatus::Completed,
        TaskStatus::Failed,
        TaskStatus::Cancelled,
        TaskStatus::Expired,
    ];

    /// The snake_case wire label.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::Failed => "failed",
            TaskStatus::Cancelled => "cancelled",
            TaskStatus::Expired => "expired",
        }
    }

    /// Whether the task has reached a terminal state (no further transitions).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed
                | TaskStatus::Failed
                | TaskStatus::Cancelled
                | TaskStatus::Expired
        )
    }
}

/// The unit a task reports progress in (DEC-1114). [`TaskProgressUnit::ALL`] pins the cardinality at 4.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TaskProgressUnit {
    /// 0-100 percent.
    Percent,
    /// A count of items.
    Items,
    /// A count of bytes.
    Bytes,
    /// No quantified progress.
    None,
}

impl TaskProgressUnit {
    /// Every variant, for the cardinality test.
    pub const ALL: [TaskProgressUnit; 4] = [
        TaskProgressUnit::Percent,
        TaskProgressUnit::Items,
        TaskProgressUnit::Bytes,
        TaskProgressUnit::None,
    ];

    /// The snake_case wire label.
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskProgressUnit::Percent => "percent",
            TaskProgressUnit::Items => "items",
            TaskProgressUnit::Bytes => "bytes",
            TaskProgressUnit::None => "none",
        }
    }
}

/// A task error, populated when the status is `Failed`.
#[derive(Clone, Debug)]
pub struct TaskError {
    /// A short machine code (e.g. `result_too_large`).
    pub code: String,
    /// A human-readable message.
    pub message: String,
}

/// A single task and its current state.
#[derive(Clone, Debug)]
pub struct Task {
    /// Server-generated handle.
    pub id: Uuid,
    /// The tool the task runs.
    pub tool_id: String,
    /// Lifecycle status.
    pub status: TaskStatus,
    /// The result payload, once completed.
    pub result: Option<Value>,
    /// The error, once failed.
    pub error: Option<TaskError>,
}

impl Task {
    /// The spec-facing status view a poll returns: handle, status, and the result or error when present.
    pub fn status_view(&self) -> Value {
        let mut v = json!({
            "task_id": self.id,
            "tool_id": self.tool_id,
            "status": self.status.as_str(),
        });
        if let Some(r) = &self.result {
            v["result"] = r.clone();
        }
        if let Some(e) = &self.error {
            v["error"] = json!({ "code": e.code, "message": e.message });
        }
        v
    }
}

/// The outcome of a cancel request (drives the HTTP status the handler returns).
#[derive(Debug, Eq, PartialEq)]
pub enum CancelOutcome {
    /// The task was pending/running and is now cancelled.
    Cancelled,
    /// No such task.
    NotFound,
    /// The task already reached a terminal state - not cancellable.
    AlreadyTerminal,
}

/// In-memory task store. Thread-safe via `RwLock`, matching the
/// [`ToolRegistry`](crate::federation::registry::ToolRegistry) pattern; the persistent table + RLS land
/// in the DB slice.
#[derive(Debug, Default)]
pub struct TaskStore {
    inner: RwLock<HashMap<Uuid, Task>>,
}

impl TaskStore {
    /// Empty store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a task for `tool_id` and return its handle (status `Running`).
    pub fn start(&self, tool_id: &str) -> Task {
        let id = Uuid::new_v4();
        let t = Task {
            id,
            tool_id: tool_id.to_string(),
            status: TaskStatus::Running,
            result: None,
            error: None,
        };
        self.inner.write().expect("poisoned").insert(id, t.clone());
        t
    }

    /// Look up a task by id.
    pub fn get(&self, id: Uuid) -> Option<Task> {
        self.inner.read().expect("poisoned").get(&id).cloned()
    }

    /// Mark a non-terminal task completed with `result`. Returns whether it transitioned.
    pub fn complete(&self, id: Uuid, result: Value) -> bool {
        self.transition(id, move |t| {
            t.status = TaskStatus::Completed;
            t.result = Some(result);
        })
    }

    /// Mark a non-terminal task failed with an error. Returns whether it transitioned.
    pub fn fail(&self, id: Uuid, code: &str, message: &str) -> bool {
        self.transition(id, |t| {
            t.status = TaskStatus::Failed;
            t.error = Some(TaskError {
                code: code.to_string(),
                message: message.to_string(),
            });
        })
    }

    /// Mark a non-terminal task expired (TTL elapsed). Returns whether it transitioned.
    pub fn expire(&self, id: Uuid) -> bool {
        self.transition(id, |t| t.status = TaskStatus::Expired)
    }

    /// Cancel a task. Pending/running -> cancelled; an unknown id or an already-terminal task is
    /// reported so the handler can return 404 / 409.
    pub fn cancel(&self, id: Uuid) -> CancelOutcome {
        let mut g = self.inner.write().expect("poisoned");
        match g.get_mut(&id) {
            None => CancelOutcome::NotFound,
            Some(t) if t.status.is_terminal() => CancelOutcome::AlreadyTerminal,
            Some(t) => {
                t.status = TaskStatus::Cancelled;
                CancelOutcome::Cancelled
            }
        }
    }

    /// Apply `f` to a non-terminal task, returning whether it was found and non-terminal.
    fn transition(&self, id: Uuid, f: impl FnOnce(&mut Task)) -> bool {
        let mut g = self.inner.write().expect("poisoned");
        match g.get_mut(&id) {
            Some(t) if !t.status.is_terminal() => {
                f(t);
                true
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_and_unit_have_the_pinned_cardinalities() {
        assert_eq!(TaskStatus::ALL.len(), 6);
        assert_eq!(TaskProgressUnit::ALL.len(), 4);
        let mut labels: Vec<&str> = TaskStatus::ALL.iter().map(|s| s.as_str()).collect();
        labels.sort_unstable();
        assert_eq!(
            labels,
            vec![
                "cancelled",
                "completed",
                "expired",
                "failed",
                "pending",
                "running"
            ]
        );
    }

    #[test]
    fn start_runs_then_completes_with_a_fetchable_result() {
        let store = TaskStore::new();
        let t = store.start("cyberos.kb.reindex");
        assert_eq!(t.status, TaskStatus::Running);
        assert!(store.complete(t.id, json!({ "indexed": 42 })));
        let done = store.get(t.id).unwrap();
        assert_eq!(done.status, TaskStatus::Completed);
        assert_eq!(done.result.unwrap()["indexed"], 42);
    }

    #[test]
    fn fail_records_the_error() {
        let store = TaskStore::new();
        let t = store.start("cyberos.kb.reindex");
        assert!(store.fail(t.id, "result_too_large", "10 MiB cap exceeded"));
        let f = store.get(t.id).unwrap();
        assert_eq!(f.status, TaskStatus::Failed);
        assert_eq!(f.error.unwrap().code, "result_too_large");
    }

    #[test]
    fn cancel_is_terminal_and_blocks_further_transitions() {
        let store = TaskStore::new();
        let t = store.start("cyberos.kb.reindex");
        assert_eq!(store.cancel(t.id), CancelOutcome::Cancelled);
        assert_eq!(store.get(t.id).unwrap().status, TaskStatus::Cancelled);
        // A terminal task neither completes nor cancels again.
        assert!(!store.complete(t.id, json!({})));
        assert_eq!(store.cancel(t.id), CancelOutcome::AlreadyTerminal);
    }

    #[test]
    fn unknown_task_id_errors_cleanly() {
        let store = TaskStore::new();
        assert!(store.get(Uuid::new_v4()).is_none());
        assert_eq!(store.cancel(Uuid::new_v4()), CancelOutcome::NotFound);
        assert!(!store.complete(Uuid::new_v4(), json!({})));
    }
}
