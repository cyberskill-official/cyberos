//! FR-MEMORY-123 §1 #1 — the per-tenant ingest cursor over the FR-MEMORY-121 event stream, plus the read
//! that lifts new interaction-event rows out of `l1_audit_log`.
//!
//! The cursor key is `l1_audit_log.seq`. `get` reads the last consumed seq (0 if never ingested); the
//! worker advances it IN THE SAME TRANSACTION as the embedding INSERT (`advance_in_tx`) so a crash between
//! the two is impossible. A restart resumes from `last_source_seq + 1`.
//!
//! The event read selects ONLY interaction-event rows (`event_type = 'memory.interaction_event'`, the
//! FR-MEMORY-121 row kind) and reaches into the JSON payload via the generated columns migration 0005 added
//! (`iev_event_type` = the interaction's own verb/kind). WIDE day-1 capture (DEC-2720): no event kind is
//! special-cased out. `subject_id` (the FR-EVAL-001 access subject) and the channel ref are pulled from the
//! Layer-1 row + payload; `chain_anchor_hex` is carried verbatim for read-time tamper detection (§1 #10).

use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::BrainEvent;

/// Read the cursor for `tenant` — the highest `l1_audit_log.seq` already embedded. Returns 0 when the tenant
/// has never been ingested (resume from seq 1). RLS-scoped via the tenant tx.
pub async fn get(pool: &PgPool, tenant_id: Uuid) -> Result<i64, sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT last_source_seq FROM brain_ingest_cursor WHERE tenant_id = $1")
            .bind(tenant_id)
            .fetch_optional(&mut *tx)
            .await?;
    tx.commit().await?;
    Ok(row.map(|(s,)| s).unwrap_or(0))
}

/// Advance the cursor for `tenant` to `last_source_seq` inside an existing transaction (§1 #1, #12). Called
/// by the ingest worker in the SAME tx as the embedding INSERT, so the cursor and the row commit atomically.
/// UPSERT so the first advance creates the row.
pub async fn advance_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
    last_source_seq: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO brain_ingest_cursor (tenant_id, last_source_seq, updated_at)
         VALUES ($1, $2, NOW())
         ON CONFLICT (tenant_id)
         DO UPDATE SET last_source_seq = GREATEST(brain_ingest_cursor.last_source_seq, EXCLUDED.last_source_seq),
                       updated_at = NOW()",
    )
    .bind(tenant_id)
    .bind(last_source_seq)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Reset the cursor to 0 for a full rebuild from the chain (§1 #14). Admin path; runs under the nil-tenant
/// RLS bypass via the caller's tx.
pub async fn reset_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    tenant_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO brain_ingest_cursor (tenant_id, last_source_seq, updated_at)
         VALUES ($1, 0, NOW())
         ON CONFLICT (tenant_id) DO UPDATE SET last_source_seq = 0, updated_at = NOW()",
    )
    .bind(tenant_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Read up to `batch_size` interaction-event rows for `tenant` strictly after `after_seq`, in seq order
/// (§1 #1). Only `event_type = 'memory.interaction_event'` rows are returned (WIDE capture, but a non-
/// interaction audit row — e.g. an `auth.token_issued` chain row — is not brain material). The `body` is the
/// text to embed; `chain_anchor_hex` is carried for read-time verify.
///
/// `subject_id` is the Layer-1 row's column. `kind` is the interaction's own verb (`iev_event_type`). The
/// channel is parsed from the payload `target_ref` when it is a channel/dm ref; otherwise `None`.
pub async fn read_after(
    pool: &PgPool,
    tenant_id: Uuid,
    after_seq: i64,
    batch_size: i64,
) -> Result<Vec<BrainEvent>, sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let rows = sqlx::query(
        r#"
        SELECT seq,
               subject_id,
               COALESCE(iev_event_type, 'memory.interaction_event') AS kind,
               ts_ns,
               COALESCE(body, '') AS body,
               chain_anchor_hex,
               (body::jsonb -> 'payload' -> 'target_ref' ->> 'kind') AS target_kind,
               (body::jsonb -> 'payload' -> 'target_ref' ->> 'id')   AS target_id
          FROM l1_audit_log
         WHERE tenant_id = $1
           AND seq > $2
           AND event_type = 'memory.interaction_event'
         ORDER BY seq ASC
         LIMIT $3
        "#,
    )
    .bind(tenant_id)
    .bind(after_seq)
    .bind(batch_size)
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let source_seq: i64 = r.try_get("seq").unwrap_or_default();
            // The Layer-1 row's subject is the access subject. A system-actor row carries the nil subject
            // (the emitter writes nil for `subject_id = None`); deny-by-default in access_scope handles it.
            let subject_id: Uuid = r
                .try_get::<Option<Uuid>, _>("subject_id")
                .ok()
                .flatten()
                .unwrap_or_else(Uuid::nil);
            let channel_id = parse_channel(
                r.try_get::<Option<String>, _>("target_kind").ok().flatten(),
                r.try_get::<Option<String>, _>("target_id").ok().flatten(),
            );
            BrainEvent {
                source_seq,
                audit_row_id: BrainEvent::make_audit_row_id(tenant_id, source_seq),
                subject_id,
                channel_id,
                kind: r.try_get::<String, _>("kind").unwrap_or_default(),
                ts_ns: r.try_get("ts_ns").unwrap_or_default(),
                body: r.try_get::<String, _>("body").unwrap_or_default(),
                chain_anchor_hex: r.try_get::<String, _>("chain_anchor_hex").unwrap_or_default(),
            }
        })
        .collect())
}

/// Map a FR-MEMORY-121 `target_ref` `{kind,id}` to a channel UUID when the ref points at a channel/dm and
/// the id parses as a UUID. Message/issue/document refs (or non-UUID ids) yield `None` — the brain only
/// tracks the channel surface for the recall `channel_scope` filter, not every target kind.
fn parse_channel(kind: Option<String>, id: Option<String>) -> Option<Uuid> {
    match (kind.as_deref(), id) {
        (Some("channel"), Some(id)) | (Some("dm"), Some(id)) => Uuid::parse_str(&id).ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_channel_extracts_channel_and_dm_uuids() {
        let u = "9c3a0000-0000-0000-0000-000000000001";
        assert_eq!(
            parse_channel(Some("channel".into()), Some(u.into())),
            Some(Uuid::parse_str(u).unwrap())
        );
        assert_eq!(
            parse_channel(Some("dm".into()), Some(u.into())),
            Some(Uuid::parse_str(u).unwrap())
        );
    }

    #[test]
    fn parse_channel_ignores_non_channel_targets() {
        let u = "9c3a0000-0000-0000-0000-000000000001";
        assert_eq!(parse_channel(Some("message".into()), Some(u.into())), None);
        assert_eq!(parse_channel(Some("document".into()), Some(u.into())), None);
        assert_eq!(parse_channel(None, None), None);
    }

    #[test]
    fn parse_channel_rejects_non_uuid_id() {
        assert_eq!(
            parse_channel(Some("channel".into()), Some("not-a-uuid".into())),
            None
        );
    }
}
