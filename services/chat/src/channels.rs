//! Channels: create (the creator becomes the owner-member) and list (the caller's channels), tenant-scoped.

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Serialize)]
pub struct Channel {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub created_by: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

type ChannelRow = (Uuid, Uuid, String, Uuid, chrono::DateTime<chrono::Utc>);

fn to_channel(r: ChannelRow) -> Channel {
    Channel {
        id: r.0,
        tenant_id: r.1,
        name: r.2,
        created_by: r.3,
        created_at: r.4,
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateChannel {
    pub name: String,
}

pub async fn create(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateChannel>,
) -> Result<(StatusCode, Json<Channel>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let creator = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let name = body.name.trim().to_string();
    if name.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".to_string()));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let row: ChannelRow = sqlx::query_as(
        "INSERT INTO chat_channels (tenant_id, name, created_by)
         VALUES ($1, $2, $3)
         RETURNING id, tenant_id, name, created_by, created_at",
    )
    .bind(tenant)
    .bind(&name)
    .bind(creator)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    sqlx::query(
        "INSERT INTO chat_channel_members (channel_id, tenant_id, subject_id, role)
         VALUES ($1, $2, $3, 'owner')",
    )
    .bind(row.0)
    .bind(tenant)
    .bind(creator)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    let channel = to_channel(row);
    audit::emit(
        &st,
        tenant,
        creator,
        "chat.channel_created",
        serde_json::json!({"channel_id": channel.id, "name": channel.name}),
    )
    .await;
    Ok((StatusCode::CREATED, Json(channel)))
}

pub async fn list(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<Channel>>, (StatusCode, String)> {
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
    let rows: Vec<ChannelRow> = sqlx::query_as(
        "SELECT c.id, c.tenant_id, c.name, c.created_by, c.created_at
         FROM chat_channels c
         JOIN chat_channel_members m ON m.channel_id = c.id
         WHERE m.subject_id = $1
         ORDER BY c.created_at",
    )
    .bind(subject)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;

    Ok(Json(rows.into_iter().map(to_channel).collect()))
}
