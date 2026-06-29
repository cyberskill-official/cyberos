//! FR-MCP-008 elicitation persistence: the DB-slice store-of-record behind [`crate::elicitation`].
//!
//! When the gateway has a database (and an authenticated caller, so a `tenant_id`/`subject` exist), the
//! router drives these functions instead of the in-memory [`ElicitationStore`](crate::elicitation::ElicitationStore);
//! the in-memory store stays the no-database dev/demo path. Because every read hits `mcp_elicitations`,
//! a pending confirmation survives a gateway restart (DEC store-of-record / resume) - there is no warmup
//! step. Response payloads are sealed through the [`Kms`] before they touch disk (DEC-1157); the only
//! plaintext-derived form persisted is the SHA-256 used for the idempotent-resubmit check.
//!
//! These are runtime-checked `sqlx::query` calls, so the module compiles without a live database. They are
//! intended to be exercised by DB-gated integration tests against Postgres (not yet in-tree); the
//! in-memory store's unit tests cover the equivalent semantics. The respond transaction mirrors
//! [`ElicitationStore::respond`](crate::elicitation::ElicitationStore::respond) exactly: validate in
//! Rust, count retries, go terminal at the cap, and treat an identical resubmit as idempotent.

use chrono::{Duration, Utc};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

use crate::elicitation::{
    response_schema, validate_response, ElicitationType, RespondOutcome, MAX_RETRIES,
};
use crate::kms::Kms;

/// Default pending lifetime for a server-raised confirmation (seconds). Well within the 1..=1800 the
/// table's CHECK allows; the timeout sweeper that acts on `expires_at` is a deferred job.
pub const DEFAULT_TIMEOUT_SECS: i64 = 300;

/// Insert a pending `confirmation` elicitation for a destructive `tools/call` (the FR-MCP-006 hold) and
/// return its server-generated id. The prompt is free-form; `choices` is empty for confirmations.
pub async fn create_confirmation(
    pool: &PgPool,
    tenant_id: Uuid,
    caller_subject_id: Uuid,
    tool_id: &str,
    prompt: Value,
) -> Result<Uuid, sqlx::Error> {
    let schema = response_schema(ElicitationType::Confirmation, &[]);
    let expires_at = Utc::now() + Duration::seconds(DEFAULT_TIMEOUT_SECS);
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO mcp_elicitations
            (tenant_id, caller_subject_id, tool_id, elicitation_type, status,
             prompt, response_schema, choices, retry_count, timeout_seconds, expires_at)
         VALUES ($1, $2, $3, 'confirmation'::elicitation_type, 'pending'::elicitation_status,
                 $4::jsonb, $5::jsonb, '[]'::jsonb, 0, $6, $7)
         RETURNING elicitation_id",
    )
    .bind(tenant_id)
    .bind(caller_subject_id)
    .bind(tool_id)
    .bind(serde_json::to_string(&prompt).unwrap_or_else(|_| "{}".to_string()))
    .bind(serde_json::to_string(&schema).unwrap_or_else(|_| "{}".to_string()))
    .bind(DEFAULT_TIMEOUT_SECS)
    .bind(expires_at)
    .fetch_one(pool)
    .await
}

/// The FR-MCP-006 gate verdict for a confirmation id: `Some(true)` approved, `Some(false)` declined,
/// `None` when there is no responded confirmation for that id owned by this caller (unknown, pending,
/// non-confirmation, or another caller's row). Scoped to `caller_subject_id` (DEC-1159) so one caller
/// cannot consult another's verdict. Reads the denormalized `confirmed` column, so the hot path never
/// opens the sealed blob.
pub async fn confirmation_state(
    pool: &PgPool,
    id: Uuid,
    caller_subject_id: Uuid,
) -> Result<Option<bool>, sqlx::Error> {
    let row = sqlx::query_scalar::<_, Option<bool>>(
        "SELECT confirmed FROM mcp_elicitations
          WHERE elicitation_id = $1
            AND caller_subject_id = $2
            AND elicitation_type = 'confirmation'::elicitation_type
            AND status = 'responded'::elicitation_status",
    )
    .bind(id)
    .bind(caller_subject_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.flatten())
}

/// The spec-facing request views of a caller's own pending elicitations (the FR-MCP-008 poll), oldest
/// first. Caller-scoped, so one caller never sees another's prompts (DEC-1159).
pub async fn pending(pool: &PgPool, caller_subject_id: Uuid) -> Result<Vec<Value>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (Uuid, String, String, String, String)>(
        "SELECT elicitation_id, tool_id, elicitation_type::text, prompt::text, response_schema::text
           FROM mcp_elicitations
          WHERE caller_subject_id = $1 AND status = 'pending'::elicitation_status
          ORDER BY created_at",
    )
    .bind(caller_subject_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, tool_id, etype, prompt, schema)| {
            json!({
                "elicitation_id": id,
                "tool_id": tool_id,
                "elicitation_type": etype,
                "prompt": serde_json::from_str::<Value>(&prompt).unwrap_or_else(|_| json!({})),
                "response_schema": serde_json::from_str::<Value>(&schema).unwrap_or_else(|_| json!({})),
                "status": "pending",
            })
        })
        .collect())
}

/// Cancel a caller's own pending elicitation. Returns whether a pending row was cancelled (false =
/// unknown, not owned, or no longer pending).
pub async fn cancel(pool: &PgPool, id: Uuid, caller_subject_id: Uuid) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(
        "UPDATE mcp_elicitations SET status = 'cancelled'::elicitation_status
          WHERE elicitation_id = $1 AND caller_subject_id = $2
            AND status = 'pending'::elicitation_status",
    )
    .bind(id)
    .bind(caller_subject_id)
    .execute(pool)
    .await?;
    Ok(res.rows_affected() == 1)
}

/// Record a caller's response. Mirrors the in-memory [`respond`](crate::elicitation::ElicitationStore::respond)
/// under a `SELECT ... FOR UPDATE` transaction so two concurrent responses cannot race: validates against
/// the persisted type+choices, counts retries (terminal at [`MAX_RETRIES`]), seals the valid payload, and
/// treats an identical resubmit (same SHA-256) as idempotent. Scoped to the calling subject (DEC-1159):
/// a row the caller does not own reads as `NotFound`.
pub async fn respond(
    pool: &PgPool,
    kms: &dyn Kms,
    id: Uuid,
    caller_subject_id: Uuid,
    payload: Value,
) -> Result<RespondOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let row = sqlx::query_as::<_, (String, String, String, Option<String>, i32)>(
        "SELECT elicitation_type::text, status::text, choices::text, response_payload_sha256, retry_count
           FROM mcp_elicitations
          WHERE elicitation_id = $1 AND caller_subject_id = $2
          FOR UPDATE",
    )
    .bind(id)
    .bind(caller_subject_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some((etype_s, status_s, choices_s, stored_sha, retry_count)) = row else {
        tx.rollback().await?;
        return Ok(RespondOutcome::NotFound);
    };
    let etype = ElicitationType::from_wire(&etype_s)
        .ok_or_else(|| sqlx::Error::Protocol(format!("unknown elicitation_type {etype_s}")))?;
    let choices: Vec<String> = serde_json::from_str(&choices_s).unwrap_or_default();
    let payload_str = serde_json::to_string(&payload).unwrap_or_default();
    let payload_sha = crate::oauth::secret::sha256_hex(&payload_str);
    let confirmed = etype == ElicitationType::Confirmation
        && payload
            .get("confirmed")
            .and_then(|c| c.as_bool())
            .unwrap_or(false);

    // Already responded: idempotent only on an identical payload (same SHA-256); otherwise not re-respondable.
    if status_s == "responded" {
        let outcome = if stored_sha.as_deref() == Some(payload_sha.as_str()) {
            RespondOutcome::AlreadyRecorded { confirmed }
        } else {
            RespondOutcome::NotPending
        };
        tx.commit().await?;
        return Ok(outcome);
    }
    if status_s != "pending" {
        tx.commit().await?;
        return Ok(RespondOutcome::NotPending);
    }

    let errs = validate_response(etype, &choices, &payload);
    if !errs.is_empty() {
        let new_retry = retry_count + 1;
        // The column's CHECK caps at MAX_RETRIES; the terminal failure is captured by status, so clamp
        // the stored counter rather than overflow the constraint.
        let stored_retry = new_retry.min(MAX_RETRIES as i32);
        if new_retry as u32 > MAX_RETRIES {
            sqlx::query(
                "UPDATE mcp_elicitations
                    SET status = 'validation_failed'::elicitation_status,
                        retry_count = $2, validation_errors = $3::jsonb
                  WHERE elicitation_id = $1",
            )
            .bind(id)
            .bind(stored_retry)
            .bind(serde_json::to_string(&errs).unwrap_or_else(|_| "[]".to_string()))
            .execute(&mut *tx)
            .await?;
            tx.commit().await?;
            return Ok(RespondOutcome::ValidationFailed(errs));
        }
        sqlx::query("UPDATE mcp_elicitations SET retry_count = $2 WHERE elicitation_id = $1")
            .bind(id)
            .bind(stored_retry)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(RespondOutcome::Invalid(errs));
    }

    let blob = kms
        .seal(payload_str.as_bytes())
        .map_err(|e| sqlx::Error::Protocol(format!("kms seal failed: {e}")))?;
    let confirmed_col: Option<bool> = (etype == ElicitationType::Confirmation).then_some(confirmed);
    sqlx::query(
        "UPDATE mcp_elicitations
            SET status = 'responded'::elicitation_status,
                response_payload_kms_blob = $2, response_payload_sha256 = $3,
                confirmed = $4, responded_at = now()
          WHERE elicitation_id = $1",
    )
    .bind(id)
    .bind(blob)
    .bind(&payload_sha)
    .bind(confirmed_col)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(RespondOutcome::Recorded { confirmed })
}
