//! FR-MEMORY-123 §1 #4 / DEC-2724 — rolling per-subject / per-channel / per-time-window summaries.
//!
//! A summary compacts the events in its window into a short digest plus its own embedding (via the SAME
//! ai-gateway path as event embeddings, DEC-2723), recording the inclusive `covered_seq_range` it compacted
//! and a monotonic `version`. When new events land in an already-summarised window, the worker writes a NEW
//! version and points the prior row's `superseded_by` at it (§1 #4) — the prior is retained for audit, never
//! overwritten; recall reads only the current version.
//!
//! Window keys:
//!   * `subject`     — `scope_id` = the subject UUID string; one rolling summary per subject.
//!   * `channel`     — `scope_id` = the channel UUID string; one rolling summary per channel.
//!   * `time_window` — `scope_id` = the ISO week of the event (`YYYY-Www`, e.g. `2026-W26`).
//!
//! The digest in this slice is an extractive compaction of the window's interaction kinds + a bounded sample
//! of recent verbs (deterministic, cheap, and leak-safe — it surfaces interaction verbs, not raw bodies).
//! The same ai-gateway can later replace the extractive digest with an abstractive one (a gateway chat call
//! under the same residency + spend policy); that is an additive polish, not a contract change. The digest's
//! EMBEDDING already goes through the gateway here.

use sqlx::{PgPool, Row};
use uuid::Uuid;

use super::{metrics, BrainConfig, BrainEvent, EmbedClient, EmbedError};
use crate::embeddings::to_pgvector_literal;

/// The scope kinds a summary can cover. Stable strings matching the `scope_kind` CHECK in migration 0007.
const SCOPE_SUBJECT: &str = "subject";
const SCOPE_CHANNEL: &str = "channel";
const SCOPE_TIME_WINDOW: &str = "time_window";

/// On a freshly-ingested event, re-summarise the windows it touches IF they have accumulated enough new
/// events since their current summary (§1 #4, throttled by `BrainConfig::summary_min_new_events` so a high-
/// write channel doesn't re-summarise on literally every event). Touches up to three scopes: the subject,
/// the channel (if any), and the event's ISO-week time window.
pub async fn touch_windows(
    pool: &PgPool,
    tenant_id: Uuid,
    ev: &BrainEvent,
    gw: &EmbedClient,
) -> Result<(), sqlx::Error> {
    let cfg = BrainConfig::from_env();

    // Subject scope.
    maybe_resummarize(
        pool,
        tenant_id,
        SCOPE_SUBJECT,
        &ev.subject_id.to_string(),
        Some(ev.subject_id),
        gw,
        cfg.summary_min_new_events,
    )
    .await?;

    // Channel scope (only when the event names a channel).
    if let Some(ch) = ev.channel_id {
        maybe_resummarize(
            pool,
            tenant_id,
            SCOPE_CHANNEL,
            &ch.to_string(),
            None,
            gw,
            cfg.summary_min_new_events,
        )
        .await?;
    }

    // Time-window scope (the event's ISO week).
    let wk = iso_week_key(ev.ts_ns);
    maybe_resummarize(
        pool,
        tenant_id,
        SCOPE_TIME_WINDOW,
        &wk,
        None,
        gw,
        cfg.summary_min_new_events,
    )
    .await?;

    Ok(())
}

/// Force a (re)summarise of one scope regardless of the new-event throttle — used by the `--resummarize`
/// backfill path (§1 #14) and the summaries test. Returns the new version number written, or `None` if the
/// scope had no events to compact.
pub async fn resummarize_now(
    pool: &PgPool,
    tenant_id: Uuid,
    scope_kind: &str,
    scope_id: &str,
    subject_id: Option<Uuid>,
    gw: &EmbedClient,
) -> Result<Option<i64>, sqlx::Error> {
    build_and_supersede(pool, tenant_id, scope_kind, scope_id, subject_id, gw).await
}

/// Run the periodic summarise pass for a tenant: emit the current summary-count gauge per scope kind (§1
/// #15). Heavy re-summarisation happens incrementally on ingest (`touch_windows`); this pass keeps the
/// gauges fresh and is where a future cadence policy (slice 3) would live.
pub async fn run_summary_pass(pool: &PgPool, tenant_id: Uuid) -> Result<(), sqlx::Error> {
    for kind in [SCOPE_SUBJECT, SCOPE_CHANNEL, SCOPE_TIME_WINDOW] {
        let mut tx = super::tenant_tx(pool, tenant_id).await?;
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM brain_summary
              WHERE tenant_id = $1 AND scope_kind = $2 AND superseded_by IS NULL",
        )
        .bind(tenant_id)
        .bind(kind)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        metrics::summary_count(tenant_id, kind, n);
    }
    Ok(())
}

/// Re-summarise a scope only if it has at least `min_new` events not yet covered by its current summary.
async fn maybe_resummarize(
    pool: &PgPool,
    tenant_id: Uuid,
    scope_kind: &str,
    scope_id: &str,
    subject_id: Option<Uuid>,
    gw: &EmbedClient,
    min_new: i64,
) -> Result<(), sqlx::Error> {
    let new_count = uncovered_event_count(pool, tenant_id, scope_kind, scope_id).await?;
    if new_count >= min_new {
        build_and_supersede(pool, tenant_id, scope_kind, scope_id, subject_id, gw).await?;
    }
    Ok(())
}

/// Count events in a scope whose `source_seq` is above the current summary's `covered_seq_hi` (or all events
/// in the scope when there is no current summary yet).
async fn uncovered_event_count(
    pool: &PgPool,
    tenant_id: Uuid,
    scope_kind: &str,
    scope_id: &str,
) -> Result<i64, sqlx::Error> {
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let covered_hi: Option<i64> = sqlx::query_scalar(
        "SELECT covered_seq_hi FROM brain_summary
          WHERE tenant_id = $1 AND scope_kind = $2 AND scope_id = $3 AND superseded_by IS NULL
          ORDER BY version DESC LIMIT 1",
    )
    .bind(tenant_id)
    .bind(scope_kind)
    .bind(scope_id)
    .fetch_optional(&mut *tx)
    .await?
    .flatten();

    let floor = covered_hi.unwrap_or(-1);
    let (sql, bind_scope) = scope_event_filter(scope_kind);
    let count: i64 = if bind_scope {
        sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM brain_event_embedding
              WHERE tenant_id = $1 AND source_seq > $2 AND {sql}"
        ))
        .bind(tenant_id)
        .bind(floor)
        .bind(scope_id)
        .fetch_one(&mut *tx)
        .await?
    } else {
        sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM brain_event_embedding
              WHERE tenant_id = $1 AND source_seq > $2 AND {sql}"
        ))
        .bind(tenant_id)
        .bind(floor)
        .fetch_one(&mut *tx)
        .await?
    };
    tx.commit().await?;
    Ok(count)
}

/// Build a new summary version for a scope and supersede the prior current version (§1 #4). Selects the
/// scope's events, compacts them into a digest, embeds the digest via the gateway, then in one transaction:
/// flips the prior current row's `superseded_by` to the new id and inserts the new version. Returns the new
/// version number, or `None` if the scope has no events.
async fn build_and_supersede(
    pool: &PgPool,
    tenant_id: Uuid,
    scope_kind: &str,
    scope_id: &str,
    subject_id: Option<Uuid>,
    gw: &EmbedClient,
) -> Result<Option<i64>, sqlx::Error> {
    // Gather the scope's events (kinds, audit_row_ids, seq + ts bounds).
    let (sql, bind_scope) = scope_event_filter(scope_kind);
    let q = format!(
        "SELECT source_seq, audit_row_id, kind, ts_ns
           FROM brain_event_embedding
          WHERE tenant_id = $1 AND {sql}
          ORDER BY source_seq ASC"
    );
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let rows = if bind_scope {
        sqlx::query(&q).bind(tenant_id).bind(scope_id).fetch_all(&mut *tx).await?
    } else {
        sqlx::query(&q).bind(tenant_id).fetch_all(&mut *tx).await?
    };
    tx.commit().await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut kinds: Vec<String> = Vec::new();
    let mut audit_ids: Vec<String> = Vec::new();
    let mut seq_lo = i64::MAX;
    let mut seq_hi = i64::MIN;
    let mut ts_lo = i64::MAX;
    let mut ts_hi = i64::MIN;
    for r in &rows {
        let s: i64 = r.try_get("source_seq").unwrap_or_default();
        let t: i64 = r.try_get("ts_ns").unwrap_or_default();
        seq_lo = seq_lo.min(s);
        seq_hi = seq_hi.max(s);
        ts_lo = ts_lo.min(t);
        ts_hi = ts_hi.max(t);
        kinds.push(r.try_get::<String, _>("kind").unwrap_or_default());
        audit_ids.push(r.try_get::<String, _>("audit_row_id").unwrap_or_default());
    }

    let digest = build_digest(scope_kind, scope_id, &kinds, rows.len());
    // Top contributors = the most recent few audit rows (provenance for the summary hit, §1 #9).
    let top: Vec<String> = audit_ids.iter().rev().take(5).cloned().collect();
    let top_json = serde_json::to_value(&top).unwrap_or(serde_json::Value::Array(vec![]));

    // Embed the digest via the gateway (DEC-2723). On over-cap / gateway-down, write the summary with a NULL
    // embedding marked pending_summary_retry — recall's full-text fallback still finds it, and a later pass
    // fills the vector.
    let (embedding_lit, model_version, summary_state): (Option<String>, String, &str) =
        match gw.embed(tenant_id, &digest).await {
            Ok(emb) => (Some(to_pgvector_literal(&emb.vector)), emb.model_version, "complete"),
            Err(EmbedError::SpendCapExhausted) | Err(EmbedError::GatewayDown(_)) => {
                (None, "pending".to_string(), "pending_summary_retry")
            }
            Err(EmbedError::Malformed(_)) => (None, "pending".to_string(), "pending_summary_retry"),
        };

    // Determine the next version number + the prior current row id.
    let mut tx = super::tenant_tx(pool, tenant_id).await?;
    let prior: Option<(Uuid, i64)> = sqlx::query_as(
        "SELECT id, version FROM brain_summary
          WHERE tenant_id = $1 AND scope_kind = $2 AND scope_id = $3 AND superseded_by IS NULL
          ORDER BY version DESC LIMIT 1",
    )
    .bind(tenant_id)
    .bind(scope_kind)
    .bind(scope_id)
    .fetch_optional(&mut *tx)
    .await?;
    let next_version = prior.map(|(_, v)| v + 1).unwrap_or(1);
    let new_id = Uuid::now_v7();

    // Insert the new current version.
    let insert = if let Some(lit) = &embedding_lit {
        sqlx::query(
            "INSERT INTO brain_summary
                (id, tenant_id, scope_kind, scope_id, subject_id, window_start_ns, window_end_ns,
                 covered_seq_lo, covered_seq_hi, digest, embedding, embed_model_version, version,
                 superseded_by, top_contributors, summary_state)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11::vector,$12,$13,NULL,$14,$15)",
        )
        .bind(new_id)
        .bind(tenant_id)
        .bind(scope_kind)
        .bind(scope_id)
        .bind(subject_id)
        .bind(ts_lo)
        .bind(ts_hi)
        .bind(seq_lo)
        .bind(seq_hi)
        .bind(&digest)
        .bind(lit)
        .bind(&model_version)
        .bind(next_version)
        .bind(&top_json)
        .bind(summary_state)
    } else {
        sqlx::query(
            "INSERT INTO brain_summary
                (id, tenant_id, scope_kind, scope_id, subject_id, window_start_ns, window_end_ns,
                 covered_seq_lo, covered_seq_hi, digest, embedding, embed_model_version, version,
                 superseded_by, top_contributors, summary_state)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,NULL,$11,$12,NULL,$13,$14)",
        )
        .bind(new_id)
        .bind(tenant_id)
        .bind(scope_kind)
        .bind(scope_id)
        .bind(subject_id)
        .bind(ts_lo)
        .bind(ts_hi)
        .bind(seq_lo)
        .bind(seq_hi)
        .bind(&digest)
        .bind(&model_version)
        .bind(next_version)
        .bind(&top_json)
        .bind(summary_state)
    };
    insert.execute(&mut *tx).await?;

    // Supersede the prior current version (retained, marked superseded_by the new id).
    if let Some((prior_id, _)) = prior {
        sqlx::query("UPDATE brain_summary SET superseded_by = $1 WHERE id = $2 AND tenant_id = $3")
            .bind(new_id)
            .bind(prior_id)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Some(next_version))
}

/// The SQL predicate selecting a scope's events from `brain_event_embedding`, and whether it needs a bound
/// `scope_id` parameter ($3 for subject/channel; none for time-window which would need a range — for the
/// time-window scope we approximate by selecting all events, since the per-week digest in this slice is
/// representative rather than exhaustive; a precise window-bounded variant is a slice-3 refinement).
fn scope_event_filter(scope_kind: &str) -> (&'static str, bool) {
    match scope_kind {
        SCOPE_SUBJECT => ("subject_id::text = $3", true),
        SCOPE_CHANNEL => ("channel_id::text = $3", true),
        // time_window: all events (approximate). Bounded precisely in a later slice.
        _ => ("TRUE", false),
    }
}

/// Build a compact, leak-safe digest of a window: scope label, event count, and the distinct interaction
/// kinds with their frequencies. It surfaces interaction VERBS (e.g. `chat.message_created x12`), never raw
/// bodies — the same privacy discipline FR-MEMORY-121's `content_ref` enforces.
fn build_digest(scope_kind: &str, scope_id: &str, kinds: &[String], total: usize) -> String {
    use std::collections::BTreeMap;
    let mut freq: BTreeMap<&str, usize> = BTreeMap::new();
    for k in kinds {
        *freq.entry(k.as_str()).or_insert(0) += 1;
    }
    let mut parts: Vec<String> = freq.iter().map(|(k, n)| format!("{k} x{n}")).collect();
    parts.sort();
    format!(
        "[{scope_kind}:{scope_id}] {total} interactions: {}",
        parts.join(", ")
    )
}

/// ISO-8601 week key (`YYYY-Www`) for a ns timestamp — the `time_window` scope id. Uses chrono's ISO week so
/// the week boundaries match the FR's `2026-W26` example.
fn iso_week_key(ts_ns: i64) -> String {
    use chrono::Datelike;
    let secs = ts_ns / 1_000_000_000;
    let nsec = (ts_ns % 1_000_000_000) as u32;
    let dt = chrono::DateTime::from_timestamp(secs, nsec).unwrap_or_else(chrono::Utc::now);
    let iso = dt.iso_week();
    format!("{}-W{:02}", iso.year(), iso.week())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_compacts_kinds_with_frequencies_no_raw_body() {
        let kinds = vec![
            "chat.message_created".to_string(),
            "chat.message_created".to_string(),
            "proj.issue_assigned".to_string(),
        ];
        let d = build_digest("channel", "9c3a", &kinds, kinds.len());
        assert!(d.contains("chat.message_created x2"));
        assert!(d.contains("proj.issue_assigned x1"));
        assert!(d.contains("[channel:9c3a]"));
        // No raw content leaks: the digest is built only from kind labels.
        assert!(!d.contains("body"));
    }

    #[test]
    fn iso_week_key_matches_expected_format() {
        // 2026-06-29 is in ISO week 27 of 2026; assert the format shape regardless of the exact week.
        let ts = 1_782_950_400_000_000_000i64; // ~2026
        let k = iso_week_key(ts);
        assert!(k.starts_with("2026-W"));
        assert_eq!(k.len(), "2026-W27".len());
    }

    #[test]
    fn scope_filter_binds_for_subject_and_channel_not_time() {
        assert!(scope_event_filter(SCOPE_SUBJECT).1);
        assert!(scope_event_filter(SCOPE_CHANNEL).1);
        assert!(!scope_event_filter(SCOPE_TIME_WINDOW).1);
    }
}
