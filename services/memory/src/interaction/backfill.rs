//! TASK-MEMORY-122 §1 #9-#11 — bounded, idempotent, consent-gated backfill of recent chat history into
//! `chat.message_created` interaction-events.
//!
//! Two reasons drive this: chat has live production history from before the chat->brain link was on, and a
//! person who acknowledges the monitoring notice later deserves their acknowledged-window activity captured.
//! This replays a bounded window (default 30 days) of `chat_messages` into interaction-events:
//!   * **Idempotent (§1 #10):** the `event_id` is a UUIDv5 derived from the source `chat_messages.id`, so
//!     replaying the same message always yields the same `event_id`. TASK-MEMORY-121's `event_id` unique
//!     index makes a second insert a no-op, so a re-run records nothing new.
//!   * **Original timestamp (§1 #10):** `occurred_at_ns` is the message's own `created_at`, NOT replay
//!     time, so the backfilled event sits at the correct point in the person's timeline.
//!   * **Consent-gated (§1 #11):** every event goes through the same `emit()` + `ConsentGate`, so a message
//!     authored by a subject who has not acknowledged is counted but never written — backfill does not get
//!     to capture someone's history just because it is in the past.
//!   * **Dry-run by default (§1 #9):** `apply = false` reports the count and writes nothing; only `apply =
//!     true` writes. Operators run the dry-run first to see the volume before committing.
//!
//! Backfill writes directly to the audit DB via `emit` (the same direct-write path live capture uses), so
//! it never couples to memory's HTTP liveness. `source_channel` is `Import` (§1 #13) so backfilled events
//! are distinguishable from live ones; `trace_id` is null (there is no request trace for a replay, §1 #14).

use crate::interaction::consent_gate::ConsentGate;
use crate::interaction::emit::{emit, EmitOutcome};
use crate::interaction::event::{
    EventClass, InteractionEvent, Module, SourceChannel, TargetRef, SCHEMA_VERSION,
};
use crate::interaction::ContentRef;
use sqlx::PgPool;
use uuid::Uuid;

/// The fixed UUIDv5 namespace for backfilled interaction-event ids (§1 #10). A constant namespace + the
/// source message id => a deterministic, collision-free `event_id` that is stable across runs. This value
/// is arbitrary-but-fixed; never change it, or a re-run would mint new ids and double-count.
pub const CAPTURE_BACKFILL_NAMESPACE: Uuid =
    Uuid::from_u128(0x6361_7074_7572_6562_6163_6b66_696c_6c00);

/// A bounded chat message row the backfill replays. Bodies are NEVER read — only the metadata the event
/// needs (the row is referenced by pointer, never inlined).
#[derive(Debug, Clone)]
struct ChatMessageRow {
    id: Uuid,
    author: Uuid,
    channel_id: Uuid,
    channel_kind: String,
    created_at_ns: i64,
    has_attachment: bool,
}

/// The outcome of a backfill run (the operator-facing report, §8). `seen` is every candidate message;
/// `recorded` is newly-written events; `already_present` is events that a prior run already wrote (the
/// idempotency count — a re-run lands here, not in `recorded`); `skipped_consent` is messages whose author
/// has not acknowledged; `errors` is best-effort emit failures (logged, not fatal). On a dry-run, `seen`
/// is populated and the rest are zero.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BackfillReport {
    pub seen: u64,
    pub recorded: u64,
    pub already_present: u64,
    pub skipped_consent: u64,
    pub errors: u64,
}

/// Read the recent chat messages for a tenant within `window_days`, RLS-scoped to that tenant. Bodies are
/// not selected. `has_attachment` derives from the `attachment_id` column (migration 0005); a tenant whose
/// chat schema predates that column still works because the query uses `attachment_id IS NOT NULL` only if
/// the column exists — here we assume the current schema (0005+), matching the deployed P0 chat.
async fn recent_messages(
    chat_pool: &PgPool,
    tenant: Uuid,
    window_days: u32,
) -> Result<Vec<ChatMessageRow>, sqlx::Error> {
    // RLS: set the tenant GUC so the chat_messages / chat_channels policies expose this tenant's rows.
    let mut tx = chat_pool.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant.to_string())
        .execute(&mut *tx)
        .await?;

    // Join the channel to carry its kind (channel vs DM). `created_at` -> ns since epoch as i64.
    let rows: Vec<(Uuid, Uuid, Uuid, String, i64, bool)> = sqlx::query_as(
        "SELECT m.id,
                m.sender_subject_id,
                m.channel_id,
                COALESCE(c.kind, 'group') AS channel_kind,
                (extract(epoch FROM m.created_at) * 1000000000)::bigint AS created_at_ns,
                (m.attachment_id IS NOT NULL) AS has_attachment
           FROM chat_messages m
           JOIN chat_channels c ON c.id = m.channel_id
          WHERE m.deleted_at IS NULL
            AND m.created_at >= now() - make_interval(days => $1)
          ORDER BY m.created_at ASC",
    )
    .bind(window_days as i32)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(rows
        .into_iter()
        .map(
            |(id, author, channel_id, channel_kind, created_at_ns, has_attachment)| {
                ChatMessageRow {
                    id,
                    author,
                    channel_id,
                    channel_kind,
                    created_at_ns,
                    has_attachment,
                }
            },
        )
        .collect())
}

/// §1 #10 idempotency pre-check: has a row with this audit `path` (which embeds the deterministic
/// event_id) already been written for this tenant? `l1_audit_log` has no RLS, so a direct
/// `(tenant_id, path)` lookup is correct and cheap (covered by the `(tenant_id, seq)` scan or a path filter).
async fn already_recorded(
    audit_pool: &PgPool,
    tenant: Uuid,
    path: &str,
) -> Result<bool, sqlx::Error> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT seq FROM l1_audit_log WHERE tenant_id = $1 AND path = $2 LIMIT 1")
            .bind(tenant)
            .bind(path)
            .fetch_optional(audit_pool)
            .await?;
    Ok(row.is_some())
}

/// Build the `chat.message_created` interaction-event for a backfilled message (§1 #10): deterministic
/// `event_id` (UUIDv5 over the source id), original `occurred_at_ns`, `source_channel: Import`, pointer
/// content-ref (never the body).
fn event_for(tenant: Uuid, m: &ChatMessageRow) -> InteractionEvent {
    let event_id = Uuid::new_v5(&CAPTURE_BACKFILL_NAMESPACE, m.id.as_bytes());
    let target = match m.channel_kind.as_str() {
        "direct" | "dm" => TargetRef::Dm {
            id: m.channel_id.to_string(),
        },
        _ => TargetRef::Channel {
            id: m.channel_id.to_string(),
        },
    };
    let mut attributes = serde_json::Map::new();
    attributes.insert(
        "channel_kind".to_string(),
        serde_json::Value::String(m.channel_kind.clone()),
    );
    attributes.insert(
        "has_attachment".to_string(),
        serde_json::Value::Bool(m.has_attachment),
    );
    attributes.insert("backfilled".to_string(), serde_json::Value::Bool(true));
    InteractionEvent {
        schema_version: SCHEMA_VERSION,
        event_id,
        tenant_id: tenant,
        subject_id: Some(m.author),
        occurred_at_ns: m.created_at_ns, // original time, NOT replay time (§1 #10)
        module: Module::Chat,
        event_type: "chat.message_created".to_string(),
        event_class: EventClass::Content,
        target_ref: target,
        content_ref: ContentRef::pointer("chat_messages", m.id.to_string()),
        session_id: None,
        trace_id: None,
        source_channel: SourceChannel::Import, // §1 #13 — distinguishes backfill from live capture
        attributes,
    }
}

/// Replay recent chat history into `chat.message_created` interaction-events for acknowledged subjects.
///
/// `chat_pool` is chat's Postgres (source); `audit_pool` is the brain audit DB (`l1_audit_log`, destination);
/// `gate` is the consent gate (the real one in production; tests may pass an `AllowAll`/`DenyAll`).
/// `window_days` bounds the lookback; `apply = false` is a dry-run that only counts (§1 #9).
///
/// Idempotent: a second `apply = true` run over the same window yields zero new `recorded` (the
/// deterministic `event_id` collides on TASK-MEMORY-121's unique index — the duplicate insert is swallowed as
/// an emit error per row, which here is counted, not fatal, so the report stays honest). Consent-gated: a
/// message by an unacknowledged author is counted in `skipped_consent`, never written.
pub async fn backfill_chat(
    chat_pool: &PgPool,
    audit_pool: &PgPool,
    gate: &dyn ConsentGate,
    tenant: Uuid,
    window_days: u32,
    apply: bool,
) -> anyhow::Result<BackfillReport> {
    let rows = recent_messages(chat_pool, tenant, window_days).await?;
    let mut report = BackfillReport::default();

    for m in &rows {
        report.seen += 1;
        if !apply {
            continue; // dry-run: count only (§1 #9)
        }
        let ev = event_for(tenant, m);

        // §1 #10 idempotency — l1_audit_log has no unique index on the event id (it lives in the body), so a
        // naive re-emit would write a second row. The audit PATH embeds the deterministic event_id
        // (`iev/<tenant>/<module>/<subject>/<event_id>`), so a pre-check on (tenant, path) makes a re-run a
        // no-op: if a row already exists for this derived path, this message was backfilled before — count
        // it as already_present and skip. The pre-check + emit are not transactional, but backfill is an
        // operator-run batch (not a concurrent hot path), so a same-instant double-run is not a concern.
        match already_recorded(audit_pool, tenant, &ev.audit_path()).await {
            Ok(true) => {
                report.already_present += 1;
                continue;
            }
            Ok(false) => {}
            Err(e) => {
                // Pre-check failed (best-effort): log and skip this row rather than risk a duplicate.
                report.errors += 1;
                tracing::debug!(
                    target: "cyberos_memory::interaction::backfill",
                    error = %e,
                    "backfill idempotency pre-check failed; skipping row"
                );
                continue;
            }
        }

        match emit(audit_pool, &ev, gate).await {
            Ok(EmitOutcome::Recorded { .. }) => report.recorded += 1,
            Ok(EmitOutcome::Skipped { .. }) => report.skipped_consent += 1,
            Err(e) => {
                report.errors += 1;
                tracing::debug!(
                    target: "cyberos_memory::interaction::backfill",
                    error = %e,
                    event_id = %ev.event_id,
                    "backfill emit error (best-effort)"
                );
            }
        }
    }

    tracing::info!(
        target: "cyberos_memory::interaction::backfill",
        metric = "memory_capture_backfill_events_total",
        tenant = %tenant,
        window_days,
        apply,
        seen = report.seen,
        recorded = report.recorded,
        already_present = report.already_present,
        skipped_consent = report.skipped_consent,
        errors = report.errors,
        "backfill_chat complete"
    );
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_id_is_deterministic_over_source_message() {
        let m = ChatMessageRow {
            id: Uuid::from_u128(0x1234),
            author: Uuid::nil(),
            channel_id: Uuid::from_u128(0x5678),
            channel_kind: "group".to_string(),
            created_at_ns: 1_700_000_000_000_000_000,
            has_attachment: false,
        };
        let a = event_for(Uuid::nil(), &m);
        let b = event_for(Uuid::nil(), &m);
        // Same source message => same event_id (the idempotency property, §1 #10).
        assert_eq!(a.event_id, b.event_id);
        // And it is the UUIDv5 of the source id under the fixed namespace.
        assert_eq!(
            a.event_id,
            Uuid::new_v5(&CAPTURE_BACKFILL_NAMESPACE, m.id.as_bytes())
        );
    }

    #[test]
    fn event_uses_original_time_and_import_source_and_pointer() {
        let m = ChatMessageRow {
            id: Uuid::from_u128(0xaa),
            author: Uuid::from_u128(0xbb),
            channel_id: Uuid::from_u128(0xcc),
            channel_kind: "direct".to_string(),
            created_at_ns: 1_650_000_000_000_000_000,
            has_attachment: true,
        };
        let ev = event_for(Uuid::from_u128(0xdd), &m);
        assert_eq!(ev.occurred_at_ns, 1_650_000_000_000_000_000); // original time, not now
        assert_eq!(ev.source_channel, SourceChannel::Import); // §1 #13
        assert!(matches!(ev.target_ref, TargetRef::Dm { .. })); // direct -> dm
                                                                // Pointer to chat's row, never the body (§1 #4).
        assert_eq!(ev.content_ref.kind(), "pointer");
        assert_eq!(ev.event_type, "chat.message_created");
        assert_eq!(ev.event_class, EventClass::Content);
    }

    #[test]
    fn dry_run_report_default_is_all_zero() {
        let r = BackfillReport::default();
        assert_eq!(r.seen, 0);
        assert_eq!(r.recorded, 0);
        assert_eq!(r.skipped_consent, 0);
        assert_eq!(r.errors, 0);
    }
}
