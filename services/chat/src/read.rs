//! Read receipts (FR-CHAT-101 slice 4): a per-(channel, subject) last-read marker, and an unread count.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth, db, AppState};

#[derive(Debug, Deserialize)]
pub struct MarkRead {
    pub message_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct Unread {
    pub unread: i64,
}

pub async fn mark(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<MarkRead>,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    if db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let m: Option<(chrono::DateTime<chrono::Utc>,)> =
        sqlx::query_as("SELECT created_at FROM chat_messages WHERE id = $1 AND channel_id = $2")
            .bind(body.message_id)
            .bind(channel)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;
    let at = match m {
        Some(x) => x.0,
        None => {
            return Err((
                StatusCode::NOT_FOUND,
                "message not in this channel".to_string(),
            ))
        }
    };
    sqlx::query(
        "INSERT INTO chat_read_markers (channel_id, tenant_id, subject_id, last_read_message_id, last_read_at)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (channel_id, subject_id)
         DO UPDATE SET last_read_message_id = EXCLUDED.last_read_message_id, last_read_at = EXCLUDED.last_read_at",
    )
    .bind(channel)
    .bind(tenant)
    .bind(subject)
    .bind(body.message_id)
    .bind(at)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;
    // Broadcast the read receipt so other members' clients can render "Seen" live (read-receipts UI).
    st.hub.publish(
        channel,
        crate::realtime::ChatEvent::Read {
            subject,
            last_read_message_id: body.message_id,
            last_read_at: at,
        },
    );
    Ok(StatusCode::NO_CONTENT)
}

pub async fn unread(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Unread>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    if db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let marker: Option<(chrono::DateTime<chrono::Utc>,)> = sqlx::query_as(
        "SELECT last_read_at FROM chat_read_markers WHERE channel_id = $1 AND subject_id = $2",
    )
    .bind(channel)
    .bind(subject)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let count: (i64,) = if let Some((at,)) = marker {
        sqlx::query_as(
            "SELECT count(*) FROM chat_messages
             WHERE channel_id = $1 AND deleted_at IS NULL AND sender_subject_id <> $2 AND created_at > $3",
        )
        .bind(channel)
        .bind(subject)
        .bind(at)
        .fetch_one(&mut *tx)
        .await
    } else {
        sqlx::query_as(
            "SELECT count(*) FROM chat_messages
             WHERE channel_id = $1 AND deleted_at IS NULL AND sender_subject_id <> $2",
        )
        .bind(channel)
        .bind(subject)
        .fetch_one(&mut *tx)
        .await
    }
    .map_err(crate::internal)?;
    let _ = tx.commit().await;
    Ok(Json(Unread { unread: count.0 }))
}

#[derive(Debug, Serialize)]
pub struct ChannelUnread {
    pub channel_id: Uuid,
    pub unread: i64,
    /// Unread @-mentions of the caller in this channel (a subset of `unread`), for a distinct mention badge.
    pub mentions: i64,
}

/// GET /v1/chat/unread - the caller's unread count (and unread-mention count) for every channel they belong
/// to, in one query (replacing the client's per-channel N calls). Unread = live messages from other senders
/// created after the caller's last-read marker, or all such messages when there is no marker yet; unread
/// mentions are those same messages that @-mention the caller. Tenant-scoped by the RLS GUC.
pub async fn unread_summary(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ChannelUnread>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let rows: Vec<(Uuid, i64, i64)> = sqlx::query_as(
        "SELECT m.channel_id,
                count(msg.id) FILTER (
                    WHERE msg.deleted_at IS NULL
                      AND msg.sender_subject_id <> $1
                      AND (rm.last_read_at IS NULL OR msg.created_at > rm.last_read_at)
                ) AS unread,
                count(men.message_id) FILTER (
                    WHERE msg.deleted_at IS NULL
                      AND (rm.last_read_at IS NULL OR msg.created_at > rm.last_read_at)
                ) AS mentions
           FROM chat_channel_members m
           LEFT JOIN chat_read_markers rm ON rm.channel_id = m.channel_id AND rm.subject_id = $1
           LEFT JOIN chat_messages msg ON msg.channel_id = m.channel_id
           LEFT JOIN chat_mentions men ON men.message_id = msg.id AND men.subject_id = $1
          WHERE m.subject_id = $1
          GROUP BY m.channel_id",
    )
    .bind(subject)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;
    Ok(Json(
        rows.into_iter()
            .map(|(channel_id, unread, mentions)| ChannelUnread {
                channel_id,
                unread,
                mentions,
            })
            .collect(),
    ))
}

#[derive(Debug, Serialize)]
pub struct Receipt {
    pub subject_id: Uuid,
    pub last_read_message_id: Uuid,
    pub last_read_at: chrono::DateTime<chrono::Utc>,
}

/// GET /v1/chat/channels/{id}/receipts - every member's last-read marker, for "Seen" indicators on open.
pub async fn receipts(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Vec<Receipt>>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    if db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let rows: Vec<(Uuid, Uuid, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        "SELECT subject_id, last_read_message_id, last_read_at FROM chat_read_markers WHERE channel_id = $1",
    )
    .bind(channel)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;
    Ok(Json(
        rows.into_iter()
            .map(|(subject_id, last_read_message_id, last_read_at)| Receipt {
                subject_id,
                last_read_message_id,
                last_read_at,
            })
            .collect(),
    ))
}
