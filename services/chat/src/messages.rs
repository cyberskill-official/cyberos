//! Messages: post (members only; optional thread reply; fans out live), list (members only; top-level or a
//! thread, newest first, excludes deleted), edit (sender only), and soft-delete (sender or a channel manager).

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub channel_id: Uuid,
    pub sender_subject_id: Uuid,
    pub body: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub parent_id: Option<Uuid>,
    pub edited_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub attachment_id: Option<Uuid>,
    /// Folded emoji reactions on this message (emoji + count + whether the caller reacted). Empty except on
    /// the list path, which attaches them in one extra query; post/edit/search return `[]` (the client folds
    /// live ReactionChanged events on top of the list).
    #[serde(default)]
    pub reactions: Vec<crate::reactions::ReactionSummary>,
    /// Attachment metadata for this message (multi-file, ordered), folded in on the list and post paths so
    /// the client renders files without a per-attachment meta round-trip. `attachment_id` stays as the first
    /// attachment for clients on the older cached shell.
    #[serde(default)]
    pub attachments: Vec<crate::attachments::AttachmentMeta>,
    /// Number of (non-deleted) thread replies to this message. Folded in on the list path only, so the client
    /// can show a "N replies" chip on the parent without opening the thread; other paths return 0.
    #[serde(default)]
    pub reply_count: i64,
    /// TASK-CHAT-268 §1 #5 — the caller has blocked this message's sender, and this is a GROUP channel: the row
    /// survives (its id and its position in the conversation), the content does not. Never true in a DM,
    /// because a blocked sender's DM messages are not returned at all (§1 #6).
    ///
    /// `skip_serializing_if` keeps the field ABSENT from the wire for every normal reader, so the shape for
    /// the 99.99% case is unchanged and no existing client parser has to learn a new field. A client that
    /// does not know about blocking sees what it always saw — an empty body — and renders an empty row
    /// rather than crashing.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub blocked_sender: bool,
}

pub(crate) type MessageRow = (
    Uuid,
    Uuid,
    Uuid,
    Uuid,
    String,
    chrono::DateTime<chrono::Utc>,
    Option<Uuid>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<Uuid>,
);

pub(crate) const COLS: &str =
    "id, tenant_id, channel_id, sender_subject_id, body, created_at, parent_id, edited_at, deleted_at, attachment_id";

pub(crate) fn to_message(r: MessageRow) -> Message {
    Message {
        id: r.0,
        tenant_id: r.1,
        channel_id: r.2,
        sender_subject_id: r.3,
        body: r.4,
        created_at: r.5,
        parent_id: r.6,
        edited_at: r.7,
        deleted_at: r.8,
        attachment_id: r.9,
        reactions: Vec::new(),
        attachments: Vec::new(),
        reply_count: 0,
        blocked_sender: false,
    }
}

#[derive(Debug, Deserialize)]
pub struct PostMessage {
    pub body: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    /// Legacy single attachment (older clients). Merged into `attachment_ids` at the front.
    #[serde(default)]
    pub attachment_id: Option<Uuid>,
    /// Ordered attachments for this message (richer-messages cluster). Each must be an upload into this
    /// channel; capped at the configured max_files.
    #[serde(default)]
    pub attachment_ids: Vec<Uuid>,
    /// Subjects this message @-mentions. Validated against channel membership (and never the sender); invalid
    /// or non-member ids are dropped.
    #[serde(default)]
    pub mentions: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct EditMessage {
    pub body: String,
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub before: Option<chrono::DateTime<chrono::Utc>>,
    pub limit: Option<i64>,
    pub parent_id: Option<Uuid>,
    /// Jump-to-message: return a window of top-level messages surrounding this id (half the limit on each
    /// side), instead of the latest page. Used by global search result navigation.
    pub around: Option<Uuid>,
}

/// Require current membership; returns the caller's role in the channel.
async fn require_member(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    channel: Uuid,
    subject: Uuid,
) -> Result<String, (StatusCode, String)> {
    match db::role_in_channel(tx, channel, subject)
        .await
        .map_err(crate::internal)?
    {
        Some(role) => Ok(role),
        None => Err((StatusCode::FORBIDDEN, "not a channel member".to_string())),
    }
}

/// Max message body size (post + edit). Generous for chat; bounds DB weight, every list, and AI cost.
const MAX_MESSAGE_BODY_BYTES: usize = 16 * 1024;

/// Minimum search-term length. The trigram GIN index needs 3 chars; shorter terms would full-scan.
const MIN_SEARCH_CHARS: usize = 3;

pub async fn post(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<PostMessage>,
) -> Result<(StatusCode, Json<Message>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let sender = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    // Merge the legacy single attachment with the multi-file list (legacy first), ordered and deduped.
    let mut attachment_ids: Vec<Uuid> = Vec::new();
    for aid in body.attachment_id.iter().chain(body.attachment_ids.iter()) {
        if !attachment_ids.contains(aid) {
            attachment_ids.push(*aid);
        }
    }
    if body.body.trim().is_empty() && attachment_ids.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "body or attachments are required".to_string(),
        ));
    }
    if body.body.len() > MAX_MESSAGE_BODY_BYTES {
        return Err((StatusCode::BAD_REQUEST, "message is too long".to_string()));
    }
    if attachment_ids.len() > st.attachments.max_files {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "at most {} attachments per message",
                st.attachments.max_files
            ),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    require_member(&mut tx, channel, sender).await?;
    if !attachment_ids.is_empty() {
        // Every referenced attachment must be an upload into this channel (RLS already scopes the tenant).
        let found: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM chat_attachments WHERE id = ANY($1) AND channel_id = $2",
        )
        .bind(&attachment_ids)
        .bind(channel)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
        if found.len() != attachment_ids.len() {
            return Err((
                StatusCode::BAD_REQUEST,
                "attachment not in this channel".to_string(),
            ));
        }
    }
    if let Some(pid) = body.parent_id {
        let parent: Option<(Uuid,)> = sqlx::query_as(
            "SELECT id FROM chat_messages WHERE id = $1 AND channel_id = $2 AND deleted_at IS NULL",
        )
        .bind(pid)
        .bind(channel)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
        if parent.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                "parent message not in this channel".to_string(),
            ));
        }
    }
    // TASK-MEMORY-122 §1 #4 — read the channel kind in this same tx (a cheap PK lookup) so the capture
    // emitter can tag channel vs DM. Off-by-default (only matters when capture is on); never fails the send.
    // The same lookup carries archived_at: an archived channel is read-only.
    let ch: Option<(String, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as("SELECT kind, archived_at FROM chat_channels WHERE id = $1")
            .bind(channel)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;
    let (channel_kind, archived_at) = ch.unwrap_or(("group".to_string(), None));
    if archived_at.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            "channel is archived (read-only)".to_string(),
        ));
    }
    let sql = format!(
        "INSERT INTO chat_messages (tenant_id, channel_id, sender_subject_id, body, parent_id, attachment_id)
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING {COLS}"
    );
    let row: MessageRow = sqlx::query_as(&sql)
        .bind(tenant)
        .bind(channel)
        .bind(sender)
        .bind(&body.body)
        .bind(body.parent_id)
        // The legacy column carries the FIRST attachment so older cached clients still render something.
        .bind(attachment_ids.first())
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::internal)?;

    // Link every attachment to the message, in order.
    for (ord, aid) in attachment_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO chat_message_attachments (message_id, attachment_id, tenant_id, ord)
             VALUES ($1, $2, $3, $4) ON CONFLICT (message_id, attachment_id) DO NOTHING",
        )
        .bind(row.0)
        .bind(aid)
        .bind(tenant)
        .bind(ord as i16)
        .execute(&mut *tx)
        .await
        .map_err(crate::internal)?;
    }

    // Resolve @-mentions to real channel members (never the sender), record one row per mentioned member in
    // the same transaction, and keep the validated list to hand to the notify fan-out. Unknown or non-member
    // ids are silently dropped.
    let mention_ids: Vec<Uuid> = if body.mentions.is_empty() {
        Vec::new()
    } else {
        let valid: Vec<(Uuid,)> = sqlx::query_as(
            "SELECT subject_id FROM chat_channel_members
             WHERE channel_id = $1 AND subject_id = ANY($2) AND subject_id <> $3",
        )
        .bind(channel)
        .bind(&body.mentions)
        .bind(sender)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
        let ids: Vec<Uuid> = valid.into_iter().map(|(s,)| s).collect();
        for mid in &ids {
            sqlx::query(
                "INSERT INTO chat_mentions (message_id, channel_id, tenant_id, subject_id)
                 VALUES ($1, $2, $3, $4) ON CONFLICT (message_id, subject_id) DO NOTHING",
            )
            .bind(row.0)
            .bind(channel)
            .bind(tenant)
            .bind(mid)
            .execute(&mut *tx)
            .await
            .map_err(crate::internal)?;
        }
        ids
    };
    // Fold this message's attachment metadata into the response (same shape the list path returns), so the
    // sender renders files immediately without meta round-trips.
    let attachment_metas = crate::attachments::metas_for_messages(&mut tx, &[row.0])
        .await
        .map_err(crate::internal)?
        .remove(&row.0)
        .unwrap_or_default();
    tx.commit().await.map_err(crate::internal)?;

    let mut message = to_message(row);
    message.attachments = attachment_metas;
    st.hub.publish(
        channel,
        crate::realtime::ChatEvent::Message(message.clone()),
    );
    tokio::spawn(crate::push::notify(
        st.clone(),
        channel,
        tenant,
        sender,
        message.id,
    ));
    // Fan the message out to every member's per-user notification socket (all but the sender), so a client
    // sees unread/activity for a channel it is not currently viewing. Off the response path; best-effort.
    // `mention_ids` is empty until the mentions cluster resolves @-mentions.
    tokio::spawn(crate::notify::fanout(
        st.clone(),
        channel,
        tenant,
        message.clone(),
        channel_kind.clone(),
        mention_ids,
    ));
    audit::emit(
        &st,
        tenant,
        sender,
        "chat.message_posted",
        serde_json::json!({"channel_id": channel, "message_id": message.id, "parent_id": message.parent_id}),
    )
    .await;
    // TASK-MEMORY-122 §1 #4, #7 — emit chat.message_created off the response path (spawned, best-effort). The
    // capturer is None unless CAPTURE_ENABLED is on, so this is a no-op by default; the body never leaves
    // chat's DB (content_ref is a pointer to the chat_messages row).
    if let Some(cap) = st.capturer.clone() {
        let has_attachment = !message.attachments.is_empty();
        let (mid, kind) = (message.id, channel_kind);
        tokio::spawn(async move {
            crate::capture::emit_message_created(
                Some(&cap),
                tenant,
                sender,
                channel,
                &kind,
                mid,
                has_attachment,
            )
            .await;
        });
    }
    Ok((StatusCode::CREATED, Json(message)))
}

pub async fn list(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let limit = q.limit.unwrap_or(50).clamp(1, 200);
    // The paging cursor, or None for "the newest page".
    //
    // Deliberately NOT `q.before.unwrap_or_else(chrono::Utc::now)`. That defaulted the cursor to the APP
    // server's wall clock and compared it against `created_at`, which POSTGRES generates with ITS clock
    // (`DEFAULT now()`). The two clocks are not the same clock. If the database runs even milliseconds
    // ahead — trivially true on a Docker Desktop VM, and possible on any separately-NTP'd host — then
    // `created_at < before` is false for the most recently written rows and they silently vanish from the
    // first page. A message you just sent is missing until you refresh.
    //
    // The SQL below now says `created_at < COALESCE($2, now())`, so when there is no cursor Postgres
    // supplies its own `now()` and the comparison is against the clock that wrote the row. Skew becomes
    // impossible by construction rather than merely unlikely.
    //
    // Found by TASK-CHAT-268's `unblock_restores_in_place`, which writes a message and lists it with no
    // intervening round-trip. It is a pre-existing bug in `list`, not a blocking bug.
    let before: Option<chrono::DateTime<chrono::Utc>> = q.before;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    require_member(&mut tx, channel, subject).await?;
    let rows: Vec<MessageRow> = if let Some(target) = q.around {
        // Jump window: half the limit at-or-before the target (inclusive), half after, merged ascending
        // and returned DESC like every other page. 404 when the target is not a live message here.
        let at: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
            "SELECT created_at FROM chat_messages
             WHERE id = $1 AND channel_id = $2 AND deleted_at IS NULL",
        )
        .bind(target)
        .bind(channel)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
        let at = match at {
            Some((t,)) => t,
            None => {
                return Err((
                    StatusCode::NOT_FOUND,
                    "message not found in this channel".to_string(),
                ))
            }
        };
        let half = (limit / 2).max(1);
        let sql_before = format!(
            "SELECT {COLS} FROM chat_messages
             WHERE channel_id = $1 AND created_at <= $2 AND deleted_at IS NULL AND parent_id IS NULL
             ORDER BY created_at DESC LIMIT $3"
        );
        let mut before_rows: Vec<MessageRow> = sqlx::query_as(&sql_before)
            .bind(channel)
            .bind(at)
            .bind(half + 1)
            .fetch_all(&mut *tx)
            .await
            .map_err(crate::internal)?;
        let sql_after = format!(
            "SELECT {COLS} FROM chat_messages
             WHERE channel_id = $1 AND created_at > $2 AND deleted_at IS NULL AND parent_id IS NULL
             ORDER BY created_at ASC LIMIT $3"
        );
        let after_rows: Vec<MessageRow> = sqlx::query_as(&sql_after)
            .bind(channel)
            .bind(at)
            .bind(half)
            .fetch_all(&mut *tx)
            .await
            .map_err(crate::internal)?;
        // after_rows are ASC; the page contract is DESC (newest first): after (reversed) then before.
        // Wrapped in Ok so this branch types like the two sqlx branches below (their map_err follows).
        let mut merged: Vec<MessageRow> = after_rows.into_iter().rev().collect();
        merged.append(&mut before_rows);
        Ok(merged)
    } else if let Some(pid) = q.parent_id {
        let sql = format!(
            "SELECT {COLS} FROM chat_messages
             WHERE channel_id = $1 AND created_at < COALESCE($2::timestamptz, now())
               AND deleted_at IS NULL AND parent_id = $3
             ORDER BY created_at DESC LIMIT $4"
        );
        sqlx::query_as(&sql)
            .bind(channel)
            .bind(before)
            .bind(pid)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
    } else {
        let sql = format!(
            "SELECT {COLS} FROM chat_messages
             WHERE channel_id = $1 AND created_at < COALESCE($2::timestamptz, now())
               AND deleted_at IS NULL AND parent_id IS NULL
             ORDER BY created_at DESC LIMIT $3"
        );
        sqlx::query_as(&sql)
            .bind(channel)
            .bind(before)
            .bind(limit)
            .fetch_all(&mut *tx)
            .await
    }
    .map_err(crate::internal)?;

    // ───────────────────────── TASK-CHAT-268 enforcement point 1 of 4: the message list (§1 #4) ────────
    // The blocked-set is read ONCE per request (memoised in AppState) and threaded through — never queried
    // per message. This is the hot path; an N+1 here would be the most expensive thing in the service.
    let blocked = crate::blocks::blocked_by(&st, tenant, subject).await;
    let is_dm = {
        let k: Option<(String,)> = sqlx::query_as("SELECT kind FROM chat_channels WHERE id = $1")
            .bind(channel)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;
        k.map(|r| r.0).unwrap_or_else(|| "group".to_string()) == "direct"
    };

    // Fold reactions onto the returned messages in one extra query (least-invasive: post/edit/search are
    // untouched and still return `reactions: []`). The caller's own reactions are flagged via `subject`.
    let mut out: Vec<Message> = rows.into_iter().map(to_message).collect();

    // §1 #6 — in a DM, a blocked sender's messages are not returned AT ALL: not collapsed, not flagged, not
    // present. In a group channel a collapsed row is context (it explains a gap in a conversation you are
    // still part of); in a DM there is no conversation left to contextualise, and a column of placeholders
    // is not information — it is a drip-feed of the harassment you asked to stop.
    //
    // Dropped BEFORE `ids` is taken, so the reaction / attachment / reply-count folds below never even see
    // these rows.
    if is_dm && !blocked.is_empty() {
        out.retain(|m| !blocked.contains(&m.sender_subject_id));
    }

    let ids: Vec<Uuid> = out.iter().map(|m| m.id).collect();
    if !ids.is_empty() {
        let mut react_rows: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
            "SELECT message_id, subject_id, emoji FROM chat_reactions WHERE message_id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
        // §1 #9 / AC 10 — a blocked person's reaction is not counted for the blocker. Filtered on the RAW
        // rows, by reactor id, before the fold: `ReactionSummary` keeps only an emoji + a count, so once
        // summarize() has run the reactor is gone and there is nothing left to filter on. Getting this wrong
        // leaks the blocked person's existence through an off-by-one count.
        react_rows.retain(|(_, reactor, _)| !blocked.contains(reactor));
        let mut by_message = crate::reactions::summarize(&react_rows, subject);
        for m in &mut out {
            if let Some(r) = by_message.remove(&m.id) {
                m.reactions = r;
            }
        }
        // Attachment metadata folds in the same way (one query for the page).
        let mut atts = crate::attachments::metas_for_messages(&mut tx, &ids)
            .await
            .map_err(crate::internal)?;
        for m in &mut out {
            if let Some(a) = atts.remove(&m.id) {
                m.attachments = a;
            }
        }
        // Reply counts fold in the same way: how many non-deleted replies each returned message has, so the
        // parent can show a "N replies" chip. One grouped query for the whole page.
        let reply_rows: Vec<(Uuid, i64)> = sqlx::query_as(
            "SELECT parent_id, count(*) FROM chat_messages
             WHERE parent_id = ANY($1) AND deleted_at IS NULL
             GROUP BY parent_id",
        )
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
        let mut counts: std::collections::HashMap<Uuid, i64> = reply_rows.into_iter().collect();
        for m in &mut out {
            if let Some(c) = counts.remove(&m.id) {
                m.reply_count = c;
            }
        }
    }

    // §1 #5 — the group-channel collapse. Runs LAST, after every fold above, because the reaction and
    // attachment folds would otherwise re-populate exactly what we are clearing. The row keeps its id and
    // its position; the content goes. Removing the row outright would silently rewrite the channel's history
    // for one participant — replies to a vanished message become nonsense — and leave the blocker more
    // confused than protected.
    if !is_dm && !blocked.is_empty() {
        for m in out.iter_mut() {
            if blocked.contains(&m.sender_subject_id) {
                m.body = String::new();
                m.attachments.clear();
                m.reactions.clear();
                m.blocked_sender = true;
            }
        }
    }

    let _ = tx.commit().await;

    Ok(Json(out))
}

pub async fn edit(
    State(st): State<AppState>,
    Path((channel, msg)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    Json(body): Json<EditMessage>,
) -> Result<Json<Message>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let caller = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    if body.body.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "body is required".to_string()));
    }
    if body.body.len() > MAX_MESSAGE_BODY_BYTES {
        return Err((StatusCode::BAD_REQUEST, "message is too long".to_string()));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    require_member(&mut tx, channel, caller).await?;
    let channel_kind: String = sqlx::query_scalar("SELECT kind FROM chat_channels WHERE id = $1")
        .bind(channel)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?
        .unwrap_or_else(|| "group".to_string());
    let sql = format!(
        "UPDATE chat_messages SET body = $1, edited_at = now()
         WHERE id = $2 AND channel_id = $3 AND sender_subject_id = $4 AND deleted_at IS NULL
         RETURNING {COLS}"
    );
    let row: Option<MessageRow> = sqlx::query_as(&sql)
        .bind(&body.body)
        .bind(msg)
        .bind(channel)
        .bind(caller)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    match row {
        Some(r) => {
            let m = to_message(r);
            st.hub.publish(
                channel,
                crate::realtime::ChatEvent::MessageEdited {
                    id: m.id,
                    sender: m.sender_subject_id,
                    body: m.body.clone(),
                    edited_at: m.edited_at,
                },
            );
            audit::emit(
                &st,
                tenant,
                caller,
                "chat.message_edited",
                serde_json::json!({"channel_id": channel, "message_id": m.id}),
            )
            .await;
            // TASK-MEMORY-122 §1 #4, #7 — chat.message_edited off the response path; no-op unless capture on.
            if let Some(cap) = st.capturer.clone() {
                let mid = m.id;
                tokio::spawn(async move {
                    crate::capture::emit_message_edited(
                        Some(&cap),
                        tenant,
                        caller,
                        channel,
                        &channel_kind,
                        mid,
                    )
                    .await;
                });
            }
            Ok(Json(m))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            "message not found or not yours".to_string(),
        )),
    }
}

pub async fn delete(
    State(st): State<AppState>,
    Path((channel, msg)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let caller = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let role = require_member(&mut tx, channel, caller).await?;
    let channel_kind: String = sqlx::query_scalar("SELECT kind FROM chat_channels WHERE id = $1")
        .bind(channel)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?
        .unwrap_or_else(|| "group".to_string());
    let snd: Option<(Uuid,)> = sqlx::query_as(
        "SELECT sender_subject_id FROM chat_messages WHERE id = $1 AND channel_id = $2 AND deleted_at IS NULL",
    )
    .bind(msg)
    .bind(channel)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let sender = match snd {
        Some(s) => s.0,
        None => return Err((StatusCode::NOT_FOUND, "message not found".to_string())),
    };
    if sender != caller && !db::is_manager(&role) {
        return Err((
            StatusCode::FORBIDDEN,
            "cannot delete another member's message".to_string(),
        ));
    }
    sqlx::query("UPDATE chat_messages SET deleted_at = now(), body = '' WHERE id = $1")
        .bind(msg)
        .execute(&mut *tx)
        .await
        .map_err(crate::internal)?;
    // Deleting a message also hard-purges its attachments (metadata rows here; closes the audit gap where
    // the bytes outlived the message). Fs payloads are unlinked after the commit, so a rollback loses nothing.
    let purged_keys = crate::attachments::purge_for_message(&mut tx, msg)
        .await
        .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;
    for key in &purged_keys {
        st.attachments.store.delete(key).await;
    }

    st.hub.publish(
        channel,
        crate::realtime::ChatEvent::MessageDeleted { id: msg },
    );
    audit::emit(
        &st,
        tenant,
        caller,
        "chat.message_deleted",
        serde_json::json!({"channel_id": channel, "message_id": msg}),
    )
    .await;
    // TASK-MEMORY-122 §1 #4, #7 — chat.message_deleted (content_ref:none) off the response path; the actor is
    // the caller (sender or a manager). No-op unless capture on.
    if let Some(cap) = st.capturer.clone() {
        tokio::spawn(async move {
            crate::capture::emit_message_deleted(
                Some(&cap),
                tenant,
                caller,
                channel,
                &channel_kind,
                msg,
            )
            .await;
        });
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

/// Accent- and case-insensitive substring search over a channel's live messages (Vietnamese-friendly via
/// chat_norm = lower(unaccent(..)), backed by a GIN trigram index). TASK-CHAT-101 slice 3.
pub async fn search(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let term = q.q.trim().to_string();
    if term.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "q is required".to_string()));
    }
    if term.chars().count() < MIN_SEARCH_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            "search term must be at least 3 characters".to_string(),
        ));
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 200);

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    require_member(&mut tx, channel, subject).await?;
    let sql = format!(
        "SELECT {COLS} FROM chat_messages
         WHERE channel_id = $1 AND deleted_at IS NULL
           AND chat_norm(body) LIKE '%' || chat_norm($2) || '%'
         ORDER BY created_at DESC LIMIT $3"
    );
    let rows: Vec<MessageRow> = sqlx::query_as(&sql)
        .bind(channel)
        .bind(&term)
        .bind(limit)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
    let _ = tx.commit().await;

    Ok(Json(rows.into_iter().map(to_message).collect()))
}

/// GET /v1/chat/search?q= - tenant-wide search over every channel the CALLER belongs to (groups and DMs
/// alike), same accent- and case-insensitive matching as the per-channel search, newest first. Membership is
/// enforced in the join, so a message in a channel the caller cannot read never surfaces; RLS scopes the
/// tenant. The result's channel_id is what the client jumps to (find-and-organize cluster).
pub async fn search_all(
    State(st): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<Message>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let term = q.q.trim().to_string();
    if term.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "q is required".to_string()));
    }
    if term.chars().count() < MIN_SEARCH_CHARS {
        return Err((
            StatusCode::BAD_REQUEST,
            "search term must be at least 3 characters".to_string(),
        ));
    }
    let limit = q.limit.unwrap_or(50).clamp(1, 200);

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let rows: Vec<MessageRow> = sqlx::query_as(
        "SELECT msg.id, msg.tenant_id, msg.channel_id, msg.sender_subject_id, msg.body,
                msg.created_at, msg.parent_id, msg.edited_at, msg.deleted_at, msg.attachment_id
         FROM chat_messages msg
         JOIN chat_channel_members mem
           ON mem.channel_id = msg.channel_id AND mem.subject_id = $1
         WHERE msg.deleted_at IS NULL
           AND chat_norm(msg.body) LIKE '%' || chat_norm($2) || '%'
         ORDER BY msg.created_at DESC LIMIT $3",
    )
    .bind(subject)
    .bind(&term)
    .bind(limit)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;

    Ok(Json(rows.into_iter().map(to_message).collect()))
}
