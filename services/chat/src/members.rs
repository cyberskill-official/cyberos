//! Channel membership: add (owner or admin), list (any member), remove (owner only). Roles are
//! owner > admin > member; the channel creator is the first owner (FR-CHAT-101 slice 2).

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Serialize)]
pub struct Member {
    pub channel_id: Uuid,
    pub subject_id: Uuid,
    pub role: String,
    pub joined_at: chrono::DateTime<chrono::Utc>,
}

type MemberRow = (Uuid, Uuid, String, chrono::DateTime<chrono::Utc>);

fn to_member(r: MemberRow) -> Member {
    Member {
        channel_id: r.0,
        subject_id: r.1,
        role: r.2,
        joined_at: r.3,
    }
}

#[derive(Debug, Deserialize)]
pub struct AddMember {
    pub subject_id: Uuid,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "member".to_string()
}

pub async fn add(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<AddMember>,
) -> Result<(StatusCode, Json<Member>), (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let caller = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    if !matches!(body.role.as_str(), "owner" | "admin" | "member") {
        return Err((StatusCode::BAD_REQUEST, "role must be owner, admin, or member".to_string()));
    }

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    match db::role_in_channel(&mut tx, channel, caller)
        .await
        .map_err(crate::internal)?
    {
        Some(role) if db::is_manager(&role) => {}
        Some(_) => return Err((StatusCode::FORBIDDEN, "only owner or admin can add members".to_string())),
        None => return Err((StatusCode::FORBIDDEN, "not a channel member".to_string())),
    }

    let row: MemberRow = sqlx::query_as(
        "INSERT INTO chat_channel_members (channel_id, tenant_id, subject_id, role)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (channel_id, subject_id) DO UPDATE SET role = EXCLUDED.role
         RETURNING channel_id, subject_id, role, joined_at",
    )
    .bind(channel)
    .bind(tenant)
    .bind(body.subject_id)
    .bind(&body.role)
    .fetch_one(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    audit::emit(
        &st,
        tenant,
        caller,
        "chat.member_added",
        serde_json::json!({"channel_id": channel, "subject_id": body.subject_id, "role": body.role}),
    )
    .await;
    // FR-MEMORY-122 §1 #4, #7 — chat.channel_joined for the ADDED subject (the person whose membership
    // changed; the consent gate applies to them). Off the response path; no-op unless capture on.
    if let Some(cap) = st.capturer.clone() {
        let joined_subject = body.subject_id;
        tokio::spawn(async move {
            crate::capture::emit_channel_membership(Some(&cap), tenant, joined_subject, channel, true)
                .await;
        });
    }
    Ok((StatusCode::CREATED, Json(to_member(row))))
}

pub async fn list(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Vec<Member>>, (StatusCode, String)> {
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
    if db::role_in_channel(&mut tx, channel, caller)
        .await
        .map_err(crate::internal)?
        .is_none()
    {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    let rows: Vec<MemberRow> = sqlx::query_as(
        "SELECT channel_id, subject_id, role, joined_at
         FROM chat_channel_members WHERE channel_id = $1 ORDER BY joined_at",
    )
    .bind(channel)
    .fetch_all(&mut *tx)
    .await
    .map_err(crate::internal)?;
    let _ = tx.commit().await;

    Ok(Json(rows.into_iter().map(to_member).collect()))
}

pub async fn remove(
    State(st): State<AppState>,
    Path((channel, subject)): Path<(Uuid, Uuid)>,
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
    match db::role_in_channel(&mut tx, channel, caller)
        .await
        .map_err(crate::internal)?
    {
        Some(role) if role == "owner" => {}
        Some(_) => return Err((StatusCode::FORBIDDEN, "only the owner can remove members".to_string())),
        None => return Err((StatusCode::FORBIDDEN, "not a channel member".to_string())),
    }
    let res = sqlx::query("DELETE FROM chat_channel_members WHERE channel_id = $1 AND subject_id = $2")
        .bind(channel)
        .bind(subject)
        .execute(&mut *tx)
        .await
        .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;
    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "member not found".to_string()));
    }

    audit::emit(
        &st,
        tenant,
        caller,
        "chat.member_removed",
        serde_json::json!({"channel_id": channel, "subject_id": subject}),
    )
    .await;
    // FR-MEMORY-122 §1 #4, #7 — chat.channel_left for the REMOVED subject. Off the response path; no-op
    // unless capture on.
    if let Some(cap) = st.capturer.clone() {
        tokio::spawn(async move {
            crate::capture::emit_channel_membership(Some(&cap), tenant, subject, channel, false).await;
        });
    }
    Ok(StatusCode::NO_CONTENT)
}
