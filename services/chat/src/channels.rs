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
    /// Short purpose line shown in the header (groups; empty for DMs).
    #[serde(default)]
    pub topic: String,
    /// 'private' (member-only) or 'public' (browsable + self-joinable). DMs are always private.
    #[serde(default)]
    pub visibility: String,
    /// Set = the channel is archived (read-only, out of the browser); None = live.
    pub archived_at: Option<chrono::DateTime<chrono::Utc>>,
}

type ChannelRow = (
    Uuid,
    Uuid,
    String,
    Uuid,
    chrono::DateTime<chrono::Utc>,
    String,
    Option<Uuid>,
    String,
    String,
    Option<chrono::DateTime<chrono::Utc>>,
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
        topic: r.7,
        visibility: r.8,
        archived_at: r.9,
    }
}

fn valid_visibility(v: &str) -> bool {
    matches!(v, "private" | "public")
}

#[derive(Debug, Deserialize)]
pub struct CreateChannel {
    pub name: String,
    /// Optional purpose line.
    #[serde(default)]
    pub topic: String,
    /// 'private' (default) or 'public'.
    #[serde(default)]
    pub visibility: Option<String>,
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
    let visibility = body.visibility.unwrap_or_else(|| "private".to_string());
    if !valid_visibility(&visibility) {
        return Err((
            StatusCode::BAD_REQUEST,
            "visibility must be private or public".to_string(),
        ));
    }
    let topic = body.topic.trim().to_string();

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let row: (Uuid, Uuid, String, Uuid, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO chat_channels (tenant_id, name, created_by, kind, topic, visibility)
         VALUES ($1, $2, $3, 'group', $4, $5)
         RETURNING id, tenant_id, name, created_by, created_at",
    )
    .bind(tenant)
    .bind(&name)
    .bind(creator)
    .bind(&topic)
    .bind(&visibility)
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
        topic,
        visibility,
        archived_at: None,
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
                topic: String::new(),
                visibility: "private".to_string(),
                archived_at: None,
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
            topic: String::new(),
            visibility: "private".to_string(),
            archived_at: None,
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
                ELSE NULL END AS other_subject_id,
                c.topic, c.visibility, c.archived_at
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

/// Fields a manager can change on a group channel. Absent fields are untouched. `archived` flips the
/// archived state (owner-only); the others need owner or admin. DMs are unmanaged and reject all of it.
#[derive(Debug, Deserialize)]
pub struct UpdateChannel {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub visibility: Option<String>,
    pub archived: Option<bool>,
}

/// PATCH /v1/chat/channels/{id} - rename, set the topic, flip visibility, or archive/unarchive.
pub async fn update(
    State(st): State<AppState>,
    axum::extract::Path(channel): axum::extract::Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<UpdateChannel>,
) -> Result<Json<Channel>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let caller = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    let name = match &body.name {
        Some(n) => {
            let n = n.trim().to_string();
            if n.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "name cannot be empty".to_string()));
            }
            Some(n)
        }
        None => None,
    };
    if let Some(v) = &body.visibility {
        if !valid_visibility(v) {
            return Err((
                StatusCode::BAD_REQUEST,
                "visibility must be private or public".to_string(),
            ));
        }
    }
    let topic = body.topic.as_ref().map(|t| t.trim().to_string());
    if name.is_none() && topic.is_none() && body.visibility.is_none() && body.archived.is_none() {
        return Err((StatusCode::BAD_REQUEST, "nothing to change".to_string()));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let role = match db::role_in_channel(&mut tx, channel, caller)
        .await
        .map_err(crate::internal)?
    {
        Some(r) => r,
        None => return Err((StatusCode::FORBIDDEN, "not a channel member".to_string())),
    };
    if !db::is_manager(&role) {
        return Err((
            StatusCode::FORBIDDEN,
            "only owner or admin can manage a channel".to_string(),
        ));
    }
    if body.archived.is_some() && role != "owner" {
        return Err((
            StatusCode::FORBIDDEN,
            "only the owner can archive or unarchive".to_string(),
        ));
    }
    let kind: Option<(String,)> = sqlx::query_as("SELECT kind FROM chat_channels WHERE id = $1")
        .bind(channel)
        .fetch_optional(&mut *tx)
        .await
        .map_err(crate::internal)?;
    match kind {
        None => return Err((StatusCode::NOT_FOUND, "channel not found".to_string())),
        Some((k,)) if k == "direct" => {
            return Err((
                StatusCode::BAD_REQUEST,
                "direct messages cannot be managed".to_string(),
            ))
        }
        Some(_) => {}
    }

    let row: Option<ChannelRow> = sqlx::query_as(
        "UPDATE chat_channels SET
             name = COALESCE($2, name),
             topic = COALESCE($3, topic),
             visibility = COALESCE($4, visibility),
             archived_at = CASE
                 WHEN $5::boolean IS NULL THEN archived_at
                 WHEN $5 THEN COALESCE(archived_at, now())
                 ELSE NULL
             END
         WHERE id = $1
         RETURNING id, tenant_id, name, created_by, created_at, kind,
                   NULL::uuid AS other_subject_id, topic, visibility, archived_at",
    )
    .bind(channel)
    .bind(&name)
    .bind(&topic)
    .bind(&body.visibility)
    .bind(body.archived)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;
    let row = row.ok_or((StatusCode::NOT_FOUND, "channel not found".to_string()))?;

    audit::emit(
        &st,
        tenant,
        caller,
        "chat.channel_updated",
        serde_json::json!({
            "channel_id": channel,
            "renamed": name.is_some(),
            "topic_changed": topic.is_some(),
            "visibility": body.visibility,
            "archived": body.archived,
        }),
    )
    .await;
    Ok(Json(to_channel(row)))
}

/// One row in the tenant's channel browser: a public, non-archived group channel.
#[derive(Debug, Serialize)]
pub struct BrowseChannel {
    pub id: Uuid,
    pub name: String,
    pub topic: String,
    pub member_count: i64,
    pub is_member: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// GET /v1/chat/channels/browse - every public, live group channel in the caller's tenant, with a member
/// count and whether the caller already belongs. RLS scopes the tenant; membership is NOT required (that is
/// the point of public channels).
pub async fn browse(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<BrowseChannel>>, (StatusCode, String)> {
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
    let rows: Vec<(
        Uuid,
        String,
        String,
        i64,
        bool,
        chrono::DateTime<chrono::Utc>,
    )> = sqlx::query_as(
        "SELECT c.id, c.name, c.topic,
                    (SELECT count(*) FROM chat_channel_members m WHERE m.channel_id = c.id),
                    EXISTS(SELECT 1 FROM chat_channel_members m
                            WHERE m.channel_id = c.id AND m.subject_id = $1),
                    c.created_at
             FROM chat_channels c
             WHERE c.kind = 'group' AND c.visibility = 'public' AND c.archived_at IS NULL
             ORDER BY lower(c.name)",
    )
    .bind(subject)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;

    Ok(Json(
        rows.into_iter()
            .map(|r| BrowseChannel {
                id: r.0,
                name: r.1,
                topic: r.2,
                member_count: r.3,
                is_member: r.4,
                created_at: r.5,
            })
            .collect(),
    ))
}

/// POST /v1/chat/channels/{id}/join - self-join a public, live group channel as a member. Idempotent: a
/// second join returns the existing membership.
pub async fn join(
    State(st): State<AppState>,
    axum::extract::Path(channel): axum::extract::Path<Uuid>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<crate::members::Member>), (StatusCode, String)> {
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
    let ch: Option<(String, String, Option<chrono::DateTime<chrono::Utc>>)> =
        sqlx::query_as("SELECT kind, visibility, archived_at FROM chat_channels WHERE id = $1")
            .bind(channel)
            .fetch_optional(&mut *tx)
            .await
            .map_err(crate::internal)?;
    let (kind, visibility, archived_at) = match ch {
        Some(c) => c,
        None => return Err((StatusCode::NOT_FOUND, "channel not found".to_string())),
    };
    if kind != "group" || visibility != "public" || archived_at.is_some() {
        return Err((
            StatusCode::FORBIDDEN,
            "channel is not open to join".to_string(),
        ));
    }
    // The no-op DO UPDATE keeps RETURNING populated when the caller is already a member (idempotent join).
    let row: (Uuid, Uuid, String, chrono::DateTime<chrono::Utc>) = sqlx::query_as(
        "INSERT INTO chat_channel_members (channel_id, tenant_id, subject_id, role)
         VALUES ($1, $2, $3, 'member')
         ON CONFLICT (channel_id, subject_id) DO UPDATE SET role = chat_channel_members.role
         RETURNING channel_id, subject_id, role, joined_at",
    )
    .bind(channel)
    .bind(tenant)
    .bind(caller)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    audit::emit(
        &st,
        tenant,
        caller,
        "chat.channel_joined",
        serde_json::json!({"channel_id": channel}),
    )
    .await;
    // FR-MEMORY-122 §1 #4, #7 — chat.channel_joined for the joining subject; no-op unless capture on.
    if let Some(cap) = st.capturer.clone() {
        tokio::spawn(async move {
            crate::capture::emit_channel_membership(Some(&cap), tenant, caller, channel, true)
                .await;
        });
    }
    Ok((
        StatusCode::CREATED,
        Json(crate::members::Member {
            channel_id: row.0,
            subject_id: row.1,
            role: row.2,
            joined_at: row.3,
        }),
    ))
}
