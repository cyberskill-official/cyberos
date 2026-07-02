//! File attachments. Slice 3 (FR-CHAT-101) began with base64 JSON uploads into a bytea column; the
//! richer-messages cluster adds a raw-bytes upload route, a pluggable byte store (db | fs volume - see
//! storage.rs), a client-visible limits endpoint, and multi-attachment messages (metadata folding +
//! purge-on-delete helpers used by messages.rs). The original base64 route and single `attachment_id`
//! stay working so already-open clients on the cached PWA shell are unaffected.

use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::Json;
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Deserialize)]
pub struct UploadAttachment {
    pub filename: String,
    #[serde(default = "octet_stream")]
    pub content_type: String,
    pub data_base64: String,
}

fn octet_stream() -> String {
    "application/octet-stream".to_string()
}

#[derive(Debug, Serialize)]
pub struct Attachment {
    pub id: Uuid,
    pub channel_id: Uuid,
    pub uploader_subject_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// The shared insert path behind both upload routes: member gate, size cap, byte store write, row insert,
/// audit. `bytes` is already decoded.
async fn store_upload(
    st: &AppState,
    channel: Uuid,
    headers: &HeaderMap,
    filename: String,
    content_type: String,
    bytes: Vec<u8>,
) -> Result<Attachment, (StatusCode, String)> {
    let claims = auth::authenticate(st, headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let uploader = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let filename = filename.trim().to_string();
    if filename.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "filename is required".to_string()));
    }
    if bytes.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "empty file".to_string()));
    }
    if bytes.len() > st.attachments.max_bytes {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "file exceeds the {} MB limit",
                st.attachments.max_bytes / (1024 * 1024)
            ),
        ));
    }
    let content_type = if content_type.trim().is_empty() {
        octet_stream()
    } else {
        content_type
    };
    let size = bytes.len() as i64;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    if db::role_in_channel(&mut tx, channel, uploader)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }

    // Write the payload to the configured store first, then record the row. The id is generated here so the
    // fs key exists before the insert; on insert failure the orphaned file is harmless and tiny.
    let id = Uuid::new_v4();
    let storage_key = st
        .attachments
        .store
        .put(tenant, id, &bytes)
        .await
        .map_err(crate::internal)?;
    let db_bytes: Option<&[u8]> = match storage_key {
        Some(_) => None,
        None => Some(&bytes),
    };
    let row: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "INSERT INTO chat_attachments
            (id, tenant_id, channel_id, uploader_subject_id, filename, content_type, size_bytes, data,
             storage, storage_key)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
         RETURNING created_at",
    )
    .bind(id)
    .bind(tenant)
    .bind(channel)
    .bind(uploader)
    .bind(&filename)
    .bind(&content_type)
    .bind(size)
    .bind(db_bytes)
    .bind(st.attachments.store.kind())
    .bind(&storage_key)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let att = Attachment {
        id,
        channel_id: channel,
        uploader_subject_id: uploader,
        filename: filename.clone(),
        content_type,
        size_bytes: size,
        created_at: row.0,
    };
    audit::emit(
        st,
        tenant,
        uploader,
        "chat.attachment_uploaded",
        serde_json::json!({"channel_id": channel, "attachment_id": att.id, "filename": filename, "size_bytes": size}),
    )
    .await;
    Ok(att)
}

/// POST /v1/chat/channels/{id}/attachments - the original base64 JSON route (kept for compatibility).
pub async fn upload(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<UploadAttachment>,
) -> Result<(StatusCode, Json<Attachment>), (StatusCode, String)> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.data_base64.as_bytes())
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "data_base64 is not valid base64".to_string(),
            )
        })?;
    let att = store_upload(
        &st,
        channel,
        &headers,
        body.filename,
        body.content_type,
        bytes,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(att)))
}

#[derive(Debug, Deserialize)]
pub struct RawUploadQuery {
    pub filename: String,
}

/// POST /v1/chat/channels/{id}/uploads?filename=... - raw request-body upload (no base64 inflation), the
/// route new clients use. Content type comes from the Content-Type header.
pub async fn upload_raw(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    Query(q): Query<RawUploadQuery>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Attachment>), (StatusCode, String)> {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let att = store_upload(
        &st,
        channel,
        &headers,
        q.filename,
        content_type,
        body.to_vec(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(att)))
}

pub async fn download(
    State(st): State<AppState>,
    Path(att): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, String)> {
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
    // (channel_id, filename, content_type, data, storage, storage_key)
    type DownloadRow = (
        Uuid,
        String,
        String,
        Option<Vec<u8>>,
        String,
        Option<String>,
    );
    let row: Option<DownloadRow> = sqlx::query_as(
        "SELECT channel_id, filename, content_type, data, storage, storage_key
             FROM chat_attachments WHERE id = $1",
    )
    .bind(att)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let (channel, filename, content_type, data, storage, storage_key) = match row {
        Some(r) => r,
        None => return Err((StatusCode::NOT_FOUND, "attachment not found".to_string())),
    };
    if db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let _ = tx.commit().await;

    // db rows carry their bytes; fs rows read from the store by key. A missing payload is a 404, not a 500 -
    // the row may outlive its bytes if the volume was recreated.
    let payload: Vec<u8> = if storage == "db" {
        data.ok_or((
            StatusCode::NOT_FOUND,
            "attachment payload missing".to_string(),
        ))?
    } else {
        let key = storage_key.ok_or((
            StatusCode::NOT_FOUND,
            "attachment payload missing".to_string(),
        ))?;
        st.attachments.store.get(&key).await.map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                "attachment payload missing".to_string(),
            )
        })?
    };

    let safe = filename.replace(['"', '\r', '\n'], "");
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type)
        .header(
            "content-disposition",
            format!("attachment; filename=\"{safe}\""),
        )
        .body(Body::from(payload))
        .map_err(crate::internal)
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentMeta {
    pub id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
}

/// GET /v1/chat/attachments/{id}/meta - filename + content type (member-only), so a client can render a
/// message's linked attachment without downloading the bytes.
pub async fn meta(
    State(st): State<AppState>,
    Path(att): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<AttachmentMeta>, (StatusCode, String)> {
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
    let row: Option<(Uuid, String, String, i64)> = sqlx::query_as(
        "SELECT channel_id, filename, content_type, size_bytes FROM chat_attachments WHERE id = $1",
    )
    .bind(att)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let (channel, filename, content_type, size_bytes) = match row {
        Some(r) => r,
        None => return Err((StatusCode::NOT_FOUND, "attachment not found".to_string())),
    };
    if db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let _ = tx.commit().await;
    Ok(Json(AttachmentMeta {
        id: att,
        filename,
        content_type,
        size_bytes,
    }))
}

/// Fold attachment metadata onto a set of messages in one query (mirrors reactions::summarize). Sources both
/// the link table (multi-file) and the legacy single `attachment_id` column, deduped, link-table order kept.
pub async fn metas_for_messages(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    message_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, Vec<AttachmentMeta>>, sqlx::Error> {
    let mut by_message: std::collections::HashMap<Uuid, Vec<AttachmentMeta>> =
        std::collections::HashMap::new();
    if message_ids.is_empty() {
        return Ok(by_message);
    }
    let rows: Vec<(Uuid, Uuid, String, String, i64)> = sqlx::query_as(
        "SELECT x.message_id, a.id, a.filename, a.content_type, a.size_bytes
         FROM (
             SELECT message_id, attachment_id, ord
               FROM chat_message_attachments WHERE message_id = ANY($1)
             UNION
             SELECT m.id, m.attachment_id, 0
               FROM chat_messages m
              WHERE m.id = ANY($1) AND m.attachment_id IS NOT NULL
                AND NOT EXISTS (SELECT 1 FROM chat_message_attachments l
                                 WHERE l.message_id = m.id AND l.attachment_id = m.attachment_id)
         ) x
         JOIN chat_attachments a ON a.id = x.attachment_id
         ORDER BY x.message_id, x.ord",
    )
    .bind(message_ids)
    .fetch_all(&mut **tx)
    .await?;
    for (mid, id, filename, content_type, size_bytes) in rows {
        by_message.entry(mid).or_default().push(AttachmentMeta {
            id,
            filename,
            content_type,
            size_bytes,
        });
    }
    Ok(by_message)
}

/// Hard-purge every attachment linked to a message (called from the message soft-delete, same transaction).
/// Deletes the metadata rows and returns the fs storage keys, which the caller unlinks AFTER the commit so a
/// rolled-back delete never loses bytes.
pub async fn purge_for_message(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    message: Uuid,
) -> Result<Vec<String>, sqlx::Error> {
    let ids: Vec<(Uuid,)> = sqlx::query_as(
        "SELECT attachment_id FROM chat_message_attachments WHERE message_id = $1
         UNION
         SELECT attachment_id FROM chat_messages WHERE id = $1 AND attachment_id IS NOT NULL",
    )
    .bind(message)
    .fetch_all(&mut **tx)
    .await?;
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = ids.into_iter().map(|(a,)| a).collect();
    // The messages.attachment_id FK is ON DELETE SET NULL and the link table cascades, so deleting the
    // attachment rows is enough; collect fs keys first.
    let keys: Vec<(Option<String>,)> = sqlx::query_as(
        "SELECT storage_key FROM chat_attachments WHERE id = ANY($1) AND storage <> 'db'",
    )
    .bind(&ids)
    .fetch_all(&mut **tx)
    .await?;
    sqlx::query("DELETE FROM chat_attachments WHERE id = ANY($1)")
        .bind(&ids)
        .execute(&mut **tx)
        .await?;
    Ok(keys.into_iter().filter_map(|(k,)| k).collect())
}
