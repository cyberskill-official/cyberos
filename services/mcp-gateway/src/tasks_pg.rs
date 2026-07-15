//! TASK-MCP-007 tasks persistence: the DB-slice store-of-record behind [`crate::tasks`].
//!
//! Same shape as [`crate::elicitation_pg`]: when the gateway has a database and an authenticated caller,
//! the router reads/cancels tasks here instead of the in-memory [`TaskStore`](crate::tasks::TaskStore),
//! which stays the no-database dev path. Input and result payloads are sealed through the [`Kms`] before
//! they touch disk (DEC-1125); the only plaintext-derived form persisted is the input SHA-256.
//!
//! There is no request-path task creator yet - the `long_running` annotation + async `tools/call` routing
//! that would call [`start`] is deferred - so this path is ready for that wiring rather than live. The
//! SQL is intended to be exercised by DB-gated integration tests against Postgres (not yet in-tree).
//! Runtime-checked `sqlx`, so it compiles without a live database.

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::kms::Kms;
use crate::tasks::CancelOutcome;

/// Default task TTL (seconds): 24h. The sweeper that expires past-TTL tasks is deferred.
pub const DEFAULT_TTL_SECS: i64 = 86_400;

/// Start a task: seal the input, insert as `running`, return the handle. (The deferred per-module worker
/// pool will insert as `pending` and promote to `running`; with no pool yet, start goes straight to
/// `running` to match the in-memory store.)
pub async fn start(
    pool: &PgPool,
    kms: &dyn Kms,
    tenant_id: Uuid,
    caller_subject_id: Uuid,
    tool_id: &str,
    input: Value,
) -> Result<Uuid, sqlx::Error> {
    let input_str = serde_json::to_string(&input).unwrap_or_default();
    let sha = crate::oauth::secret::sha256_hex(&input_str);
    let blob = kms
        .seal(input_str.as_bytes())
        .map_err(|e| sqlx::Error::Protocol(format!("kms seal failed: {e}")))?;
    let expires_at = Utc::now() + Duration::seconds(DEFAULT_TTL_SECS);
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO mcp_tasks
            (tenant_id, caller_subject_id, tool_id, status,
             input_payload_kms_blob, input_payload_sha256, started_at, expires_at)
         VALUES ($1, $2, $3, 'running'::task_status, $4, $5, now(), $6)
         RETURNING task_id",
    )
    .bind(tenant_id)
    .bind(caller_subject_id)
    .bind(tool_id)
    .bind(blob)
    .bind(&sha)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// The spec-facing status view of a caller's own task (mirrors [`Task::status_view`](crate::tasks::Task::status_view)):
/// handle, tool, status, and the result (opened through the KMS) or error when present. `Ok(None)` when
/// the task is unknown or not owned by the caller (DEC-1159).
pub async fn status_view(
    pool: &PgPool,
    kms: &dyn Kms,
    id: Uuid,
    caller_subject_id: Uuid,
) -> Result<Option<Value>, sqlx::Error> {
    let row = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<Vec<u8>>,
            Option<String>,
            Option<String>,
            Option<String>,
        ),
    >(
        "SELECT tool_id, status::text, result_payload_kms_blob, result_url, error_code, error_message
           FROM mcp_tasks WHERE task_id = $1 AND caller_subject_id = $2",
    )
    .bind(id)
    .bind(caller_subject_id)
    .fetch_optional(pool)
    .await?;
    let Some((tool_id, status, result_blob, result_url, error_code, error_message)) = row else {
        return Ok(None);
    };
    let mut v = json!({ "task_id": id, "tool_id": tool_id, "status": status });
    if let Some(blob) = result_blob {
        if let Ok(plain) = kms.open(&blob) {
            if let Ok(val) = serde_json::from_slice::<Value>(&plain) {
                v["result"] = val;
            }
        }
    } else if let Some(url) = result_url {
        v["result_url"] = json!(url);
    }
    if error_code.is_some() || error_message.is_some() {
        v["error"] = json!({
            "code": error_code.unwrap_or_default(),
            "message": error_message.unwrap_or_default(),
        });
    }
    Ok(Some(v))
}

/// Cancel a caller's own task under a `SELECT ... FOR UPDATE` transaction. Pending/running -> cancelled;
/// unknown/not-owned -> `NotFound`; already terminal -> `AlreadyTerminal`.
pub async fn cancel(
    pool: &PgPool,
    id: Uuid,
    caller_subject_id: Uuid,
) -> Result<CancelOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_scalar::<_, String>(
        "SELECT status::text FROM mcp_tasks
          WHERE task_id = $1 AND caller_subject_id = $2 FOR UPDATE",
    )
    .bind(id)
    .bind(caller_subject_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(status) = row else {
        tx.rollback().await?;
        return Ok(CancelOutcome::NotFound);
    };
    if matches!(
        status.as_str(),
        "completed" | "failed" | "cancelled" | "expired"
    ) {
        tx.commit().await?;
        return Ok(CancelOutcome::AlreadyTerminal);
    }
    sqlx::query(
        "UPDATE mcp_tasks SET status = 'cancelled'::task_status, completed_at = now()
          WHERE task_id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(CancelOutcome::Cancelled)
}

/// Mark a non-terminal task completed with a sealed `result`. Returns whether it transitioned.
pub async fn complete(
    pool: &PgPool,
    kms: &dyn Kms,
    id: Uuid,
    result: Value,
) -> Result<bool, sqlx::Error> {
    let result_str = serde_json::to_string(&result).unwrap_or_default();
    let blob = kms
        .seal(result_str.as_bytes())
        .map_err(|e| sqlx::Error::Protocol(format!("kms seal failed: {e}")))?;
    let res = sqlx::query(
        "UPDATE mcp_tasks
            SET status = 'completed'::task_status, result_payload_kms_blob = $2, completed_at = now()
          WHERE task_id = $1 AND status IN ('pending', 'running')",
    )
    .bind(id)
    .bind(blob)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// Mark a non-terminal task failed with an error. Returns whether it transitioned.
pub async fn fail(pool: &PgPool, id: Uuid, code: &str, message: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE mcp_tasks
            SET status = 'failed'::task_status, error_code = $2, error_message = $3, completed_at = now()
          WHERE task_id = $1 AND status IN ('pending', 'running')",
    )
    .bind(id)
    .bind(code)
    .bind(message)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}
