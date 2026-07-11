//! Workspace moderation queue (FR-CHAT-269). An administrator reviews the reports FR-CHAT-267 files, acts,
//! and is audited.
//!
//! FR-CHAT-267 gives every member a Report button. Without this module that button files into a table nobody
//! can read — which is worse than no button at all, because it tells someone being harassed that something
//! will happen, and nothing does.
//!
//! Three properties are load-bearing, and two of them are about restraint:
//!
//! * **The DM carve-out (§1 #9).** A report about a direct message discloses THAT MESSAGE and nothing else.
//!   Not the thread around it. Fetching the surrounding messages is the easiest thing to build and the most
//!   natural thing to want ("I need context") — and it would quietly convert a safety feature into an
//!   employer surveillance tool: file a report, and your private correspondence becomes readable by your
//!   boss. The reporter consented to disclosing the one message they reported. That is all they get.
//!
//! * **Group context requires the admin to ALREADY be a channel member (§1 #8).** A private channel the
//!   admin is not in is not their business, and a report must not become a skeleton key into it. If they
//!   need the context they can join the channel — visibly, as an act with its own audit row — rather than
//!   acquiring a silent read.
//!
//! * **The role gate fails closed (§1 #2).** See `auth::require_moderator`. Channel roles grant nothing.
//!
//! Blocks (FR-CHAT-268) are deliberately NOT applied here (§1 #10): an administrator who has blocked someone
//! must still be able to adjudicate a report about them. There is no call to `blocks::blocked_by` in this
//! file, and there must not be one.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

const NOTE_MAX_CHARS: usize = 1000;
const QUEUE_DEFAULT_LIMIT: i64 = 25;
const QUEUE_MAX_LIMIT: i64 = 100;
/// §1 #17. Matches the window published at cyberskill.world/en/cyberos/delete-account.
const RETENTION_DAYS: i64 = 90;

/// §1 #5 — severity, as a SQL expression rather than a stored column.
///
/// A column would need backfilling on every rank change, and the rank IS a product decision that will
/// change. This is one place to edit, and it is legible to whoever reads it next. Inlined as a CASE (not a
/// SQL function) so the ordering lives in the codebase, not in a migration nobody re-reads.
///
/// Lower number = worse. `self_harm` outranks everything, and no volume of `spam` may push it down.
const SEVERITY_CASE: &str = "CASE r.reason
        WHEN 'self_harm'  THEN 0
        WHEN 'illegal'    THEN 1
        WHEN 'violence'   THEN 2
        WHEN 'sexual'     THEN 3
        WHEN 'hate'       THEN 4
        WHEN 'harassment' THEN 5
        WHEN 'spam'       THEN 6
        ELSE 7
    END";

/// The group's lead report — the lowest report id in the fold, used both as the id the reviewer opens and as
/// the tiebreak leg of the keyset cursor. So it must be STABLE for a given group, not merely arbitrary.
///
/// Written the long way instead of `min(r.id)` because **PostgreSQL has no `min(uuid)` aggregate**. min/max
/// are defined for numeric, string, date/time and enum types plus inet, interval, money, oid, pg_lsn, tid and
/// xid8 — uuid is not on that list, and `min(r.id)` fails at PARSE time with "function min(uuid) does not
/// exist". Parse time, note, not row time: the query 500s even against a tenant with no reports at all, which
/// is precisely how this was caught.
///
/// `array_agg(x ORDER BY x)` then `[1]` takes the minimum under uuid's own btree ordering, is portable across
/// every server version, and yields exactly the value `min(r.id)` would have if it existed. Do not "simplify"
/// it to `min(r.id::text)::uuid`: that happens to agree today only because the canonical text form is
/// fixed-width lowercase hex, and it silently re-sorts if that ever stops being true.
const LEAD_REPORT_ID: &str = "(array_agg(r.id ORDER BY r.id))[1]";

/// The same ranking in Rust, for tests and for any caller that needs it without a round-trip. Kept beside
/// SEVERITY_CASE on purpose: if one changes and the other does not, `severity_rank_matches_sql_case` fails.
pub fn severity_rank(reason: &str) -> i16 {
    match reason {
        "self_harm" => 0,
        "illegal" => 1,
        "violence" => 2,
        "sexual" => 3,
        "hate" => 4,
        "harassment" => 5,
        "spam" => 6,
        _ => 7, // "other", and anything the enum grows later
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    Dismiss,
    DeleteMessage,
    RemoveMember,
}

impl Action {
    pub fn as_label(&self) -> &'static str {
        match self {
            Action::Dismiss => "dismiss",
            Action::DeleteMessage => "delete_message",
            Action::RemoveMember => "remove_member",
        }
    }
    /// A dismissal is not an action taken against the content; it is a decision that none was needed.
    fn terminal_status(&self) -> &'static str {
        match self {
            Action::Dismiss => "dismissed",
            _ => "actioned",
        }
    }
}

// ── Queue ────────────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct QueueQuery {
    pub status: Option<String>,
    pub reason: Option<String>,
    pub cursor: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct QueueEntry {
    pub lead_report_id: Uuid,
    pub target_kind: String,
    pub target_message_id: Option<Uuid>,
    pub target_attachment_id: Option<Uuid>,
    pub target_subject_id: Option<Uuid>,
    pub report_count: i64,
    pub severity: i32,
    pub reasons: Vec<String>,
    pub last_reported_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct QueuePage {
    pub entries: Vec<QueueEntry>,
    /// Absent when this is the last page. There is no "all" mode (§1 #6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// One grouped row off the queue query, in SELECT order:
/// lead_report_id, target_kind, target_message_id, target_attachment_id, target_subject_id,
/// report_count, severity, reasons, last_reported_at.
type QueueRow = (
    Uuid,
    String,
    Option<Uuid>,
    Option<Uuid>,
    Option<Uuid>,
    i64,
    i32,
    Vec<String>,
    chrono::DateTime<chrono::Utc>,
);

/// Keyset cursor over the queue's sort key. Opaque to the client by construction (base64), and keyset rather
/// than OFFSET so a report resolved mid-scroll cannot make the next page skip an entry.
fn encode_cursor(severity: i32, last: chrono::DateTime<chrono::Utc>, lead: Uuid) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(format!("{severity}:{}:{lead}", last.timestamp_micros()))
}

fn decode_cursor(s: &str) -> Option<(i32, chrono::DateTime<chrono::Utc>, Uuid)> {
    let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(s)
        .ok()?;
    let text = String::from_utf8(raw).ok()?;
    let mut parts = text.splitn(3, ':');
    let severity: i32 = parts.next()?.parse().ok()?;
    let micros: i64 = parts.next()?.parse().ok()?;
    let lead: Uuid = parts.next()?.parse().ok()?;
    Some((
        severity,
        chrono::DateTime::from_timestamp_micros(micros)?,
        lead,
    ))
}

/// GET /v1/chat/admin/reports — the queue.
///
/// Grouped by target (§1 #4) and ordered severity-first (§1 #5). Both are structural defences against the
/// real failure mode of every moderation queue, which is not that the reviewer rejects a good report — it is
/// that they stop opening the queue. Three duplicate rows for one message, and a `self_harm` report buried
/// under forty spam reports, are exactly how that happens.
pub async fn queue(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<QueueQuery>,
) -> Result<Json<QueuePage>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    auth::require_moderator(&claims)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let limit = q
        .limit
        .unwrap_or(QUEUE_DEFAULT_LIMIT)
        .clamp(1, QUEUE_MAX_LIMIT);
    let status = q.status.unwrap_or_else(|| "open".to_string());
    let cursor = q.cursor.as_deref().and_then(decode_cursor);

    // The GROUP BY must happen BEFORE the LIMIT, or page one shows three copies of one message and the
    // reviewer never reaches page two. Grouping after pagination is the classic version of this bug and it
    // is invisible in a fixture with four rows — so it is done in SQL, in one pass.
    //
    // The HAVING clause is the keyset cursor, spelled out rather than written as a row comparison
    // `(sev, ts, id) < ($3,$4,$5)`. A row comparison is lexicographic ALL-ASCENDING, and this ORDER BY is
    // mixed — `severity ASC, last_reported_at DESC, lead_report_id DESC`. For severity we need `>` (walking
    // UP the ranks), for the other two `<` (walking DOWN in time). Written as a row comparison it would
    // re-serve entries the reviewer has already seen the moment the page crosses a severity boundary — and a
    // fixture where every report shares one severity would never notice.
    //
    // Fetch limit+1 to learn whether another page exists without a second COUNT.
    let sql = format!(
        "SELECT {LEAD_REPORT_ID}                            AS lead_report_id,
                r.target_kind,
                r.target_message_id,
                r.target_attachment_id,
                r.target_subject_id,
                count(*)                                    AS report_count,
                min({SEVERITY_CASE})::int                   AS severity,
                array_agg(DISTINCT r.reason)                AS reasons,
                max(r.created_at)                           AS last_reported_at
           FROM chat_reports r
          WHERE r.status = $1
            AND ($2::text IS NULL OR r.reason = $2)
          GROUP BY r.target_kind, r.target_message_id, r.target_attachment_id, r.target_subject_id
         HAVING ($3::int IS NULL
                 OR min({SEVERITY_CASE})::int > $3::int
                 OR (min({SEVERITY_CASE})::int = $3::int
                     AND max(r.created_at) < $4::timestamptz)
                 OR (min({SEVERITY_CASE})::int = $3::int
                     AND max(r.created_at) = $4::timestamptz
                     AND {LEAD_REPORT_ID} < $5::uuid))
          ORDER BY severity ASC, last_reported_at DESC, lead_report_id DESC
          LIMIT $6"
    );

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let rows: Vec<QueueRow> = sqlx::query_as(&sql)
        .bind(&status)
        .bind(q.reason.as_deref())
        .bind(cursor.map(|c| c.0))
        .bind(cursor.map(|c| c.1))
        .bind(cursor.map(|c| c.2))
        .bind(limit + 1)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
    let _ = tx.commit().await;

    let mut entries: Vec<QueueEntry> = rows
        .into_iter()
        .map(|r| QueueEntry {
            lead_report_id: r.0,
            target_kind: r.1,
            target_message_id: r.2,
            target_attachment_id: r.3,
            target_subject_id: r.4,
            report_count: r.5,
            severity: r.6,
            reasons: r.7,
            last_reported_at: r.8,
        })
        .collect();

    let next_cursor = if entries.len() as i64 > limit {
        entries.truncate(limit as usize);
        entries
            .last()
            .map(|e| encode_cursor(e.severity, e.last_reported_at, e.lead_report_id))
    } else {
        None
    };

    Ok(Json(QueuePage {
        entries,
        next_cursor,
    }))
}

// ── Detail ───────────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ReportDetail {
    pub id: Uuid,
    pub reason: String,
    pub detail: Option<String>,
    pub reported_at: chrono::DateTime<chrono::Utc>,
    /// Administrators see WHO reported; the reported person never does. Three reports from one person who
    /// habitually reports their colleagues is a different situation from three reports by three people, and
    /// a decision made without that is a decision made blind. The asymmetry is the point.
    pub reporter_subject_id: Uuid,
    pub report_count: i64,

    pub target_kind: String,
    pub target_message_id: Option<Uuid>,
    pub target_subject_id: Option<Uuid>,
    pub channel_id: Option<Uuid>,

    pub snapshot_body: Option<String>,
    pub snapshot_filename: Option<String>,
    pub snapshot_sender_id: Option<Uuid>,
    pub snapshot_taken_at: chrono::DateTime<chrono::Utc>,

    /// §1 #7 — has the sender since removed the original? That absence is itself evidence, and a reviewer
    /// must be able to tell "this is what they said" from "this is what they said, and they have since
    /// deleted it".
    pub original_present: bool,

    pub status: String,
    pub resolution: Option<String>,

    /// §1 #8, #9 — populated ONLY for a GROUP-channel target whose channel the admin is ALREADY in. Always
    /// empty for a DM. There is no flag to override this, and there must never be one.
    pub context: Vec<crate::messages::Message>,
}

/// GET /v1/chat/admin/reports/:id
pub async fn detail(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ReportDetail>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    auth::require_moderator(&claims)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let admin = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;

    #[allow(clippy::type_complexity)]
    let row: Option<(
        Uuid,
        String,
        Option<String>,
        chrono::DateTime<chrono::Utc>,
        Uuid,
        String,
        Option<Uuid>,
        Option<Uuid>,
        Option<Uuid>,
        Option<String>,
        Option<String>,
        Option<Uuid>,
        chrono::DateTime<chrono::Utc>,
        String,
        Option<String>,
    )> = sqlx::query_as(
        "SELECT id, reason, detail, created_at, reporter_subject_id, target_kind, target_message_id,
                target_subject_id, channel_id, snapshot_body, snapshot_filename, snapshot_sender_id,
                snapshot_taken_at, status, resolution
           FROM chat_reports WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;

    // §1 (§3) / AC 16 — 404, never 403, for a report in another tenant. RLS already returned zero rows; a
    // 403 here would confirm that the id exists in a workspace the caller cannot see.
    let r = row.ok_or((StatusCode::NOT_FOUND, "no such report".to_string()))?;

    let report_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM chat_reports
          WHERE target_kind = $1
            AND target_message_id    IS NOT DISTINCT FROM $2
            AND target_subject_id    IS NOT DISTINCT FROM $3",
    )
    .bind(&r.5)
    .bind(r.6)
    .bind(r.7)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;

    // §1 #7. A message deleted by its sender leaves `deleted_at` set and `body` emptied — so "still there"
    // means a live, non-deleted row.
    let original_present: bool = match r.6 {
        Some(mid) => sqlx::query_scalar::<_, Option<Uuid>>(
            "SELECT id FROM chat_messages WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(mid)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?
        .flatten()
        .is_some(),
        None => true, // a subject target has no "original" to lose
    };

    let context = surrounding_context(&mut tx, &r.5, r.6, r.8, admin).await?;

    let _ = tx.commit().await;

    Ok(Json(ReportDetail {
        id: r.0,
        reason: r.1,
        detail: r.2,
        reported_at: r.3,
        reporter_subject_id: r.4,
        report_count,
        target_kind: r.5,
        target_message_id: r.6,
        target_subject_id: r.7,
        channel_id: r.8,
        snapshot_body: r.9,
        snapshot_filename: r.10,
        snapshot_sender_id: r.11,
        snapshot_taken_at: r.12,
        original_present,
        status: r.13,
        resolution: r.14,
        context,
    }))
}

/// §1 #8, #9 — the whole privacy argument of this FR, in one function.
///
/// Returns the messages around the reported one, and ONLY when BOTH hold:
///   1. the channel is a GROUP channel (never a DM), and
///   2. the administrator is ALREADY a member of it.
///
/// Every other case returns empty. There is no override parameter, no `?include_context=true`, and no admin
/// bypass — because the moment one exists, someone will use it.
async fn surrounding_context(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    target_kind: &str,
    message_id: Option<Uuid>,
    channel_id: Option<Uuid>,
    admin: Uuid,
) -> Result<Vec<crate::messages::Message>, (StatusCode, String)> {
    // Only a message has a "surrounding".
    if target_kind != "message" {
        return Ok(Vec::new());
    }
    let (Some(mid), Some(channel)) = (message_id, channel_id) else {
        return Ok(Vec::new());
    };

    let kind: Option<(String,)> = sqlx::query_as("SELECT kind FROM chat_channels WHERE id = $1")
        .bind(channel)
        .fetch_optional(&mut **tx)
        .await
        .map_err(crate::internal)?;

    // §1 #9 — a DM discloses the reported message and NOTHING else. Alice reported one line from Bob. She
    // did not hand her employer her private correspondence, and a moderation feature that quietly does is a
    // surveillance feature wearing a safety badge.
    if kind.map(|k| k.0).as_deref() != Some("group") {
        return Ok(Vec::new());
    }

    // §1 #8 — the admin must ALREADY be in the channel. A report is not a skeleton key. If they want the
    // context, they can join — which is visible, and audited.
    if db::role_in_channel(tx, channel, admin)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Ok(Vec::new());
    }

    let rows: Vec<crate::messages::MessageRow> = sqlx::query_as(&format!(
        "(SELECT {cols} FROM chat_messages
           WHERE channel_id = $1 AND deleted_at IS NULL
             AND created_at <= (SELECT created_at FROM chat_messages WHERE id = $2)
           ORDER BY created_at DESC LIMIT 4)
         UNION
         (SELECT {cols} FROM chat_messages
           WHERE channel_id = $1 AND deleted_at IS NULL
             AND created_at > (SELECT created_at FROM chat_messages WHERE id = $2)
           ORDER BY created_at ASC LIMIT 3)",
        cols = crate::messages::COLS
    ))
    .bind(channel)
    .bind(mid)
    .fetch_all(&mut **tx)
    .await
    .map_err(crate::internal)?;

    let mut out: Vec<crate::messages::Message> =
        rows.into_iter().map(crate::messages::to_message).collect();
    out.sort_by_key(|m| m.created_at);
    Ok(out)
}

// ── Resolve ──────────────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ResolveRequest {
    pub action: Action,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Resolution {
    pub id: Uuid,
    pub action: String,
    pub status: String,
    pub resolved_by_subject_id: Option<Uuid>,
    /// The reports closed alongside this one (§1 #13).
    pub sibling_report_ids: Vec<Uuid>,
    /// True when this call LOST the compare-and-swap and is being handed the winner's outcome (§1 #12).
    pub already_resolved: bool,
}

/// POST /v1/chat/admin/reports/:id/resolve
pub async fn resolve(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    Json(req): Json<ResolveRequest>,
) -> Result<(StatusCode, Json<Resolution>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    auth::require_moderator(&claims)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let admin = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    if let Some(n) = req.note.as_deref() {
        if n.chars().count() > NOTE_MAX_CHARS {
            return Err((StatusCode::BAD_REQUEST, "note is too long".to_string()));
        }
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;

    // §1 #12 — compare-and-swap on `status = 'open'`. Two admins opening the queue at once is the NORMAL
    // case in a small workspace, not an edge case. Without this, `delete_message` runs twice: the second
    // deletion is a harmless no-op, but the AUDIT CHAIN then claims two people independently decided to
    // delete the same message, which is false — and the audit chain is the artefact we ask people to trust.
    #[allow(clippy::type_complexity)]
    let won: Option<(String, Option<Uuid>, Option<Uuid>, Option<Uuid>, Option<Uuid>)> =
        sqlx::query_as(
            "UPDATE chat_reports
                SET status = $2,
                    resolution = $3,
                    resolved_at = now(),
                    resolved_by_subject_id = $4,
                    purge_after = now() + ($5 || ' days')::interval
              WHERE id = $1 AND status = 'open'
          RETURNING target_kind, target_message_id, target_subject_id, channel_id, snapshot_sender_id",
        )
        .bind(id)
        .bind(req.action.terminal_status())
        .bind(req.action.as_label())
        .bind(admin)
        .bind(RETENTION_DAYS.to_string())
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;

    let Some((kind, msg_id, subj_id, channel_id, snap_sender)) = won else {
        // Lost the race, or it was already resolved. Hand back the WINNER's outcome with 200 — do not
        // re-apply the action, do not emit a second audit row. An error would be technically accurate and
        // practically useless: from this caller's point of view, the outcome they wanted has happened.
        let existing: Option<(String, Option<String>, Option<Uuid>)> = sqlx::query_as(
            "SELECT status, resolution, resolved_by_subject_id FROM chat_reports WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
        let _ = tx.commit().await;

        let (status, resolution, by) =
            existing.ok_or((StatusCode::NOT_FOUND, "no such report".to_string()))?;
        return Ok((
            StatusCode::OK,
            Json(Resolution {
                id,
                action: resolution.unwrap_or_default(),
                status,
                resolved_by_subject_id: by,
                sibling_report_ids: Vec::new(),
                already_resolved: true,
            }),
        ));
    };

    // §1 #13 — close every OTHER open report against the same target, in this same transaction. A message
    // can only be deleted once, and leaving five siblings open against a message that no longer exists is
    // how a queue fills with ghosts and a reviewer stops trusting it.
    //
    // IS NOT DISTINCT FROM, not `=`: two of the three target columns are NULL on any given row, and `=`
    // yields NULL against NULL — the sibling update would silently match nothing. Same trap as the partial
    // unique index in FR-CHAT-267, wearing a different hat.
    let siblings: Vec<Uuid> = sqlx::query_scalar(
        "UPDATE chat_reports
            SET status = $2,
                resolution = $3,
                resolved_at = now(),
                resolved_by_subject_id = $4,
                purge_after = now() + ($5 || ' days')::interval
          WHERE status = 'open'
            AND id <> $1
            AND target_kind = $6
            AND target_message_id    IS NOT DISTINCT FROM $7
            AND target_subject_id    IS NOT DISTINCT FROM $8
        RETURNING id",
    )
    .bind(id)
    .bind(req.action.terminal_status())
    .bind(req.action.as_label())
    .bind(admin)
    .bind(RETENTION_DAYS.to_string())
    .bind(&kind)
    .bind(msg_id)
    .bind(subj_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;

    // Apply the effect.
    let mut purged_keys: Vec<String> = Vec::new();
    let mut removed: Option<(Uuid, Uuid)> = None; // (channel, subject)
    match req.action {
        Action::Dismiss => {}
        Action::DeleteMessage => {
            let mid = msg_id.ok_or((
                StatusCode::BAD_REQUEST,
                "delete_message on a non-message report".to_string(),
            ))?;
            // Mirrors messages::delete exactly: soft-delete, blank the body, hard-purge the attachments.
            sqlx::query("UPDATE chat_messages SET deleted_at = now(), body = '' WHERE id = $1")
                .bind(mid)
                .execute(&mut *tx)
                .await
                .map_err(crate::internal)?;
            purged_keys = crate::attachments::purge_for_message(&mut tx, mid)
                .await
                .map_err(crate::internal)?;
            // The report's snapshot is untouched — it is the evidence, and this is precisely the moment it
            // earns its keep (FR-CHAT-267 §1 #4).
        }
        Action::RemoveMember => {
            // §1 #14 — from the CHANNEL the report came from. NOT from the workspace. Firing someone is not
            // a chat feature: one click in a chat UI must not sever a person's access to the organisation's
            // systems. Removing them from where the harm happened stops the harm; the rest is a
            // conversation between humans, and identity lives in the AUTH module.
            let channel = channel_id.ok_or((
                StatusCode::BAD_REQUEST,
                "remove_member on a report with no channel".to_string(),
            ))?;
            // Whom: the reported subject, or the sender of the reported message.
            let subject = subj_id.or(snap_sender).ok_or((
                StatusCode::BAD_REQUEST,
                "remove_member: no subject to remove".to_string(),
            ))?;
            sqlx::query(
                "DELETE FROM chat_channel_members WHERE channel_id = $1 AND subject_id = $2",
            )
            .bind(channel)
            .bind(subject)
            .execute(&mut *tx)
            .await
            .map_err(crate::internal)?;
            removed = Some((channel, subject));
        }
    }

    tx.commit().await.map_err(crate::internal)?;

    // Attachment bytes are unlinked AFTER the commit, so a rollback loses nothing.
    for key in &purged_keys {
        st.attachments.store.delete(key).await;
    }

    // §1 #15 — the DECISION row. Exactly one, and it carries no content: not the snapshot, not the note
    // (§1 #16). Same reasoning as FR-CHAT-267 §1 #8 — the audit chain is hash-chained and replicated, which
    // is the wrong place for a copy of content someone asked us to remove.
    audit::emit(
        &st,
        tenant,
        admin,
        "chat.report_resolved",
        serde_json::json!({
            "report_id":          id,
            "action":             req.action.as_label(),
            "sibling_report_ids": siblings,
        }),
    )
    .await;

    // ...and the EFFECT rows, emitted by the same event types the ordinary handlers use. A resolution that
    // deletes a message therefore produces TWO rows, and that is correct: one records the decision, one
    // records the effect. A `dismiss` produces only the first (§1 #15).
    if let Some(mid) = msg_id.filter(|_| req.action == Action::DeleteMessage) {
        if let Some(channel) = channel_id {
            st.hub.publish(
                channel,
                crate::realtime::ChatEvent::MessageDeleted { id: mid },
            );
        }
        audit::emit(
            &st,
            tenant,
            admin,
            "chat.message_deleted",
            serde_json::json!({"channel_id": channel_id, "message_id": mid, "by": "moderator"}),
        )
        .await;
    }
    if let Some((channel, subject)) = removed {
        st.hub
            .publish(channel, crate::realtime::ChatEvent::Kicked { subject });
        audit::emit(
            &st,
            tenant,
            admin,
            "chat.member_removed",
            serde_json::json!({"channel_id": channel, "subject_id": subject, "by": "moderator"}),
        )
        .await;
    }

    Ok((
        StatusCode::OK,
        Json(Resolution {
            id,
            action: req.action.as_label().to_string(),
            status: req.action.terminal_status().to_string(),
            resolved_by_subject_id: Some(admin),
            sibling_report_ids: siblings,
            already_resolved: false,
        }),
    ))
}

// ── Retention ────────────────────────────────────────────────────────────────────────────────────

/// §1 #17 — delete resolved reports, snapshot included, once their retention window expires.
///
/// A JOB, not a trigger: a trigger would delete rows out from under an administrator mid-read. It deletes
/// the ROW, not merely the snapshot columns, because a resolved report stripped of its evidence is not a
/// record of anything — it is a row asserting that something happened with nothing to show for it. The
/// durable record is the `chat.report_resolved` audit row, which by design carries no content.
///
/// Runs with the nil-tenant GUC so it sweeps every workspace in one pass (the same admin bypass every other
/// cross-tenant maintenance path uses).
pub async fn purge_resolved_reports(pool: &db::Pool) -> Result<u64, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(Uuid::nil().to_string())
        .execute(&mut *tx)
        .await?;
    let n = sqlx::query(
        "DELETE FROM chat_reports WHERE purge_after IS NOT NULL AND purge_after < now()",
    )
    .execute(&mut *tx)
    .await?
    .rows_affected();
    tx.commit().await?;
    if n > 0 {
        tracing::info!(target: "cyberos_chat::moderation", purged = n, "resolved reports purged past retention");
    }
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_rank_matches_the_sql_case() {
        // The Rust rank and the SQL CASE are two copies of one product decision. If someone edits one and
        // forgets the other, the queue's ordering silently diverges from what this function claims — so pin
        // both to the same table here, where the mismatch is legible.
        for (reason, rank) in [
            ("self_harm", 0),
            ("illegal", 1),
            ("violence", 2),
            ("sexual", 3),
            ("hate", 4),
            ("harassment", 5),
            ("spam", 6),
            ("other", 7),
        ] {
            assert_eq!(severity_rank(reason), rank, "rust rank for {reason}");
            assert!(
                SEVERITY_CASE.contains(&format!("'{reason}'")) || reason == "other",
                "SQL CASE must name {reason} (or fall through to ELSE)"
            );
            assert!(
                SEVERITY_CASE.contains(&format!("THEN {rank}")) || reason == "other",
                "SQL CASE must rank {reason} as {rank}"
            );
        }
        // §1 #5 — the ordering that actually matters: no volume of spam may outrank one self_harm.
        assert!(severity_rank("self_harm") < severity_rank("spam"));
        assert!(severity_rank("self_harm") < severity_rank("harassment"));
        // An unknown reason must sort LAST, never first — a new enum value must not leapfrog self_harm.
        assert_eq!(severity_rank("something_new"), 7);
    }

    #[test]
    fn action_labels_are_the_wire_format() {
        assert_eq!(Action::Dismiss.as_label(), "dismiss");
        assert_eq!(Action::DeleteMessage.as_label(), "delete_message");
        assert_eq!(Action::RemoveMember.as_label(), "remove_member");
        // A dismissal is a decision that no action was needed — it is NOT "actioned".
        assert_eq!(Action::Dismiss.terminal_status(), "dismissed");
        assert_eq!(Action::DeleteMessage.terminal_status(), "actioned");
        assert_eq!(Action::RemoveMember.terminal_status(), "actioned");
    }

    #[test]
    fn action_rejects_free_text() {
        assert_eq!(
            serde_json::from_str::<Action>("\"delete_message\"").unwrap(),
            Action::DeleteMessage
        );
        // §1 #11 — a closed set. "ban" is not an action.
        assert!(serde_json::from_str::<Action>("\"ban\"").is_err());
        assert!(serde_json::from_str::<Action>("\"remove_from_workspace\"").is_err());
    }

    #[test]
    fn cursor_round_trips_and_rejects_garbage() {
        let ts = chrono::DateTime::from_timestamp_micros(1_752_000_000_000_000).unwrap();
        let id = Uuid::from_bytes([7; 16]);
        let c = encode_cursor(4, ts, id);
        assert!(!c.contains(':'), "the cursor must be opaque to the client");
        assert_eq!(decode_cursor(&c), Some((4, ts, id)));

        // A client that invents a cursor gets None (and the handler then serves page one) rather than a 500.
        assert_eq!(decode_cursor("not-base64!!"), None);
        assert_eq!(decode_cursor(""), None);
    }
}
