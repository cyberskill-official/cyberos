//! Channels: create a named group (the creator becomes the owner-member), find-or-create a two-person
//! DM, and list the caller's channels - all tenant-scoped. A channel is 'group' (named, multi-member) or
//! 'direct' (a DM rendered by the partner's name). For direct channels the listing also returns the other
//! member's subject_id so the client can label the DM by person.

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
    /// 'group' or 'direct'.
    pub kind: String,
    /// For a direct channel, the other member's subject_id (the DM partner); None for a group.
    pub other_subject_id: Option<Uuid>,
}

type ChannelRow = (
    Uuid,
    Uuid,
    String,
    Uuid,
    chrono::DateTime<chrono::Utc>,
    String,
    Option<Uuid>,
);

fn to_channel(r: ChannelRow) -> Channel {
    Channel {
        id: r.0,
        tenant_id: r.1,
        name: r.2,
        created_by: r.3,
        created_at: r.4,
        kind: r.5,
        other_subject_id: r.6,
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
    let row: (Uuid, Uuid, String, Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO chat_channels (tenant_id, name, created_by, kind)
         VALUES ($1, $2, $3, 'group')
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

    let channel = Channel {
        id: row.0,
        tenant_id: row.1,
        name: row.2,
        created_by: row.3,
        created_at: row.4,
        kind: "group".to_string(),
        other_subject_id: None,
    };
    audit::emit(
        &st,
        tenant,
        creator,
        "chat.channel_created",
        serde_json::json!({"channel_id": channel.id, "name": channel.name}),
    )
    .await;
    // FR-MEMORY-122 §1 #4, #7 — chat.channel_created off the response path; no-op unless capture on. The
    // creator is seated as owner, so this one event covers the create (no separate channel_joined).
    if let Some(cap) = st.capturer.clone() {
        let cid = channel.id;
        tokio::spawn(async move {
            crate::capture::emit_channel_created(Some(&cap), tenant, creator, cid).await;
        });
    }
    Ok((StatusCode::CREATED, Json(channel)))
}

#[derive(Debug, Deserialize)]
pub struct CreateDm {
    pub subject_id: Uuid,
}

/// `POST /v1/chat/dms {subject_id}` - find-or-create the two-person direct channel between the caller and
/// the given subject. Idempotent: a second call from either side returns the same channel instead of a
/// duplicate, so "message Bob" always lands in one DM thread.
pub async fn create_dm(
    State(st): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateDm>,
) -> Result<(StatusCode, Json<Channel>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let caller = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let other = body.subject_id;
    if other == caller {
        return Err((
            StatusCode::BAD_REQUEST,
            "cannot start a DM with yourself".to_string(),
        ));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;

    // Existing DM between exactly these two? (A direct channel whose member set is {caller, other}.)
    let existing: Option<(Uuid, Uuid, String, Uuid, chrono::DateTime<chrono::Utc>)> =
        sqlx::query_as(
            "SELECT c.id, c.tenant_id, c.name, c.created_by, c.created_at
               FROM chat_channels c
              WHERE c.kind = 'direct'
                AND EXISTS (SELECT 1 FROM chat_channel_members m WHERE m.channel_id = c.id AND m.subject_id = $1)
                AND EXISTS (SELECT 1 FROM chat_channel_members m WHERE m.channel_id = c.id AND m.subject_id = $2)
                AND (SELECT count(*) FROM chat_channel_members m WHERE m.channel_id = c.id) = 2
              LIMIT 1",
        )
        .bind(caller)
        .bind(other)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
    if let Some((id, tenant_id, name, created_by, created_at)) = existing {
        tx.commit().await.map_err(crate::internal)?;
        return Ok((
            StatusCode::OK,
            Json(Channel {
                id,
                tenant_id,
                name,
                created_by,
                created_at,
                kind: "direct".to_string(),
                other_subject_id: Some(other),
            }),
        ));
    }

    // Create the direct channel and seat both people. The name is cosmetic - the client labels a DM by
    // the partner's display name from the directory, not by this string.
    let row: (Uuid, Uuid, String, Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO chat_channels (tenant_id, name, created_by, kind)
         VALUES ($1, 'dm', $2, 'direct')
         RETURNING id, tenant_id, name, created_by, created_at",
    )
    .bind(tenant)
    .bind(caller)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    for (subject, role) in [(caller, "owner"), (other, "member")] {
        sqlx::query(
            "INSERT INTO chat_channel_members (channel_id, tenant_id, subject_id, role)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (channel_id, subject_id) DO NOTHING",
        )
        .bind(row.0)
        .bind(tenant)
        .bind(subject)
        .bind(role)
        .execute(&mut *tx)
        .await
        .map_err(crate::internal)?;
    }
    tx.commit().await.map_err(crate::internal)?;

    audit::emit(
        &st,
        tenant,
        caller,
        "chat.dm_created",
        serde_json::json!({"channel_id": row.0, "other_subject_id": other}),
    )
    .await;
    // FR-MEMORY-122 §1 #4, #7 — chat.dm_opened off the response path, emitted ONLY when a new DM is created
    // (the find-or-create return above does not re-emit). No-op unless capture on.
    if let Some(cap) = st.capturer.clone() {
        let cid = row.0;
        tokio::spawn(async move {
            crate::capture::emit_dm_opened(Some(&cap), tenant, caller, cid).await;
        });
    }
    Ok((
        StatusCode::CREATED,
        Json(Channel {
            id: row.0,
            tenant_id: row.1,
            name: row.2,
            created_by: row.3,
            created_at: row.4,
            kind: "direct".to_string(),
            other_subject_id: Some(other),
        }),
    ))
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
        "SELECT c.id, c.tenant_id, c.name, c.created_by, c.created_at, c.kind,
                CASE WHEN c.kind = 'direct' THEN (
                     SELECT m2.subject_id FROM chat_channel_members m2
                      WHERE m2.channel_id = c.id AND m2.subject_id <> $1
                      LIMIT 1)
                ELSE NULL END AS other_subject_id
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
