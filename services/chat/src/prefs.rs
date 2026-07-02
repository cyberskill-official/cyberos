//! Per-channel notification preferences (remaining-gaps wave): all | mentions | none, per member. The
//! DEFAULT is 'all' and is represented by the ABSENCE of a row - GET returns only overrides, and setting
//! 'all' deletes the row. notify.rs consults these when fanning a message out, so a muted channel stays
//! silent at the source (no socket event, no desktop notification), not just hidden in the client.

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Serialize)]
pub struct ChannelPref {
    pub channel_id: Uuid,
    pub notify: String,
}

#[derive(Debug, Deserialize)]
pub struct SetPref {
    pub notify: String,
}

fn valid_mode(m: &str) -> bool {
    matches!(m, "all" | "mentions" | "none")
}

/// GET /v1/chat/prefs - the caller's non-default channel notification modes, one call for the whole sidebar.
pub async fn list(
    State(st): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ChannelPref>>, (StatusCode, String)> {
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
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT channel_id, notify FROM chat_channel_prefs WHERE subject_id = $1")
            .bind(subject)
            .fetch_all(&mut *tx)
            .await
            .map_err(crate::internal)?;
    let _ = tx.commit().await;
    Ok(Json(
        rows.into_iter()
            .map(|(channel_id, notify)| ChannelPref { channel_id, notify })
            .collect(),
    ))
}

/// PUT /v1/chat/channels/{id}/prefs - set the caller's own mode for one channel (member-gated). 'all'
/// removes the override row.
pub async fn set(
    State(st): State<AppState>,
    Path(channel): Path<Uuid>,
    headers: HeaderMap,
    Json(body): Json<SetPref>,
) -> Result<Json<ChannelPref>, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    if !valid_mode(&body.notify) {
        return Err((
            StatusCode::BAD_REQUEST,
            "notify must be all, mentions, or none".to_string(),
        ));
    }

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
    if body.notify == "all" {
        sqlx::query("DELETE FROM chat_channel_prefs WHERE channel_id = $1 AND subject_id = $2")
            .bind(channel)
            .bind(subject)
            .execute(&mut *tx)
            .await
            .map_err(crate::internal)?;
    } else {
        sqlx::query(
            "INSERT INTO chat_channel_prefs (channel_id, subject_id, tenant_id, notify)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (channel_id, subject_id)
             DO UPDATE SET notify = EXCLUDED.notify, updated_at = now()",
        )
        .bind(channel)
        .bind(subject)
        .bind(tenant)
        .bind(&body.notify)
        .execute(&mut *tx)
        .await
        .map_err(crate::internal)?;
    }
    tx.commit().await.map_err(crate::internal)?;

    audit::emit(
        &st,
        tenant,
        subject,
        "chat.notify_pref_set",
        serde_json::json!({"channel_id": channel, "notify": body.notify}),
    )
    .await;
    Ok(Json(ChannelPref {
        channel_id: channel,
        notify: body.notify,
    }))
}

/// The channel's non-default modes keyed by member, for the notify fan-out (one query per message send).
pub async fn modes_for_channel(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    channel: Uuid,
) -> Result<HashMap<Uuid, String>, sqlx::Error> {
    let rows: Vec<(Uuid, String)> =
        sqlx::query_as("SELECT subject_id, notify FROM chat_channel_prefs WHERE channel_id = $1")
            .bind(channel)
            .fetch_all(&mut **tx)
            .await?;
    Ok(rows.into_iter().collect())
}

/// Pure delivery rule shared by the fan-out: should this member's notify socket get this event?
pub fn should_deliver(mode: Option<&str>, is_mention: bool) -> bool {
    match mode.unwrap_or("all") {
        "none" => false,
        "mentions" => is_mention,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delivery_rule() {
        assert!(should_deliver(None, false), "default is all");
        assert!(should_deliver(Some("all"), false));
        assert!(
            !should_deliver(Some("none"), true),
            "muted swallows even mentions"
        );
        assert!(should_deliver(Some("mentions"), true));
        assert!(!should_deliver(Some("mentions"), false));
        assert!(
            should_deliver(Some("garbage"), false),
            "unknown modes fail open"
        );
    }
}
