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
}

type MessageRow = (
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

const COLS: &str =
    "id, tenant_id, channel_id, sender_subject_id, body, created_at, parent_id, edited_at, deleted_at, attachment_id";

fn to_message(r: MessageRow) -> Message {
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
    }
}

#[derive(Debug, Deserialize)]
pub struct PostMessage {
    pub body: String,
    #[serde(default)]
    pub parent_id: Option<Uuid>,
    #[serde(default)]
    pub attachment_id: Option<Uuid>,
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
    if body.body.trim().is_empty() && body.attachment_id.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            "body or attachment_id is required".to_string(),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    require_member(&mut tx, channel, sender).await?;
    if let Some(aid) = body.attachment_id {
        let a: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM chat_attachments WHERE id = $1 AND channel_id = $2")
                .bind(aid)
                .bind(channel)
                .fetch_optional(&mut *tx)
                .await
                .map_err(crate::internal)?;
        if a.is_none() {
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
    // FR-MEMORY-122 §1 #4 — read the channel kind in this same tx (a cheap PK lookup) so the capture
    // emitter can tag channel vs DM. Off-by-default (only matters when capture is on); never fails the send.
    let channel_kind: String = sqlx::query_scalar("SELECT kind FROM chat_channels WHERE id = $1")
        .bind(channel)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?
        .unwrap_or_else(|| "group".to_string());
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
        .bind(body.attachment_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let message = to_message(row);
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
    audit::emit(
        &st,
        tenant,
        sender,
        "chat.message_posted",
        serde_json::json!({"channel_id": channel, "message_id": message.id, "parent_id": message.parent_id}),
    )
    .await;
    // FR-MEMORY-122 §1 #4, #7 — emit chat.message_created off the response path (spawned, best-effort). The
    // capturer is None unless CAPTURE_ENABLED is on, so this is a no-op by default; the body never leaves
    // chat's DB (content_ref is a pointer to the chat_messages row).
    if let Some(cap) = st.capturer.clone() {
        let has_attachment = message.attachment_id.is_some();
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
    let before = q.before.unwrap_or_else(chrono::Utc::now);

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    require_member(&mut tx, channel, subject).await?;
    let rows: Vec<MessageRow> = if let Some(pid) = q.parent_id {
        let sql = format!(
            "SELECT {COLS} FROM chat_messages
             WHERE channel_id = $1 AND created_at < $2 AND deleted_at IS NULL AND parent_id = $3
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
             WHERE channel_id = $1 AND created_at < $2 AND deleted_at IS NULL AND parent_id IS NULL
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

    // Fold reactions onto the returned messages in one extra query (least-invasive: post/edit/search are
    // untouched and still return `reactions: []`). The caller's own reactions are flagged via `subject`.
    let mut out: Vec<Message> = rows.into_iter().map(to_message).collect();
    let ids: Vec<Uuid> = out.iter().map(|m| m.id).collect();
    if !ids.is_empty() {
        let react_rows: Vec<(Uuid, Uuid, String)> = sqlx::query_as(
            "SELECT message_id, subject_id, emoji FROM chat_reactions WHERE message_id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&mut *tx)
        .await
        .map_err(crate::internal)?;
        let mut by_message = crate::reactions::summarize(&react_rows, subject);
        for m in &mut out {
            if let Some(r) = by_message.remove(&m.id) {
                m.reactions = r;
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
            // FR-MEMORY-122 §1 #4, #7 — chat.message_edited off the response path; no-op unless capture on.
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
    tx.commit().await.map_err(crate::internal)?;

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
    // FR-MEMORY-122 §1 #4, #7 — chat.message_deleted (content_ref:none) off the response path; the actor is
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
/// chat_norm = lower(unaccent(..)), backed by a GIN trigram index). FR-CHAT-101 slice 3.
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
