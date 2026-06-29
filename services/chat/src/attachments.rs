//! File attachments (FR-CHAT-101 slice 3): upload (base64 JSON, member-only, size-capped) and download
//! (member-only, streamed). Slice 3 stores bytes in Postgres; object storage is a later slice.

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::Json;
use base64::Engine;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

/// Slice-3 cap for DB-backed storage.
const MAX_BYTES: usize = 5 * 1024 * 1024;

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

pub async fn upload(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<UploadAttachment>,
) -> Result<(StatusCode, Json<Attachment>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let uploader = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let filename = body.filename.trim().to_string();
    if filename.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "filename is required".to_string()));
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(body.data_base64.as_bytes())
        .map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "data_base64 is not valid base64".to_string(),
            )
        })?;
    if bytes.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "empty file".to_string()));
    }
    if bytes.len() > MAX_BYTES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "file exceeds the 5 MB limit".to_string(),
        ));
    }
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
    let row: (Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO chat_attachments
            (tenant_id, channel_id, uploader_subject_id, filename, content_type, size_bytes, data)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         RETURNING id, created_at",
    )
    .bind(tenant)
    .bind(channel)
    .bind(uploader)
    .bind(&filename)
    .bind(&body.content_type)
    .bind(size)
    .bind(&bytes)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let att = Attachment {
        id: row.0,
        channel_id: channel,
        uploader_subject_id: uploader,
        filename: filename.clone(),
        content_type: body.content_type.clone(),
        size_bytes: size,
        created_at: row.1,
    };
    audit::emit(
        &st,
        tenant,
        uploader,
        "chat.attachment_uploaded",
        serde_json::json!({"channel_id": channel, "attachment_id": att.id, "filename": filename, "size_bytes": size}),
    )
    .await;
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
    let row: Option<(Uuid, String, String, Vec<u8>)> = sqlx::query_as(
        "SELECT channel_id, filename, content_type, data FROM chat_attachments WHERE id = $1",
    )
    .bind(att)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let (channel, filename, content_type, data) = match row {
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

    let safe = filename.replace(['"', '\r', '\n'], "");
    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", content_type)
        .header(
            "content-disposition",
            format!("attachment; filename=\"{safe}\""),
        )
        .body(Body::from(data))
        .map_err(crate::internal)
}

#[derive(Debug, Serialize)]
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
