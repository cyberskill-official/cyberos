//! Emoji reactions (FR-CHAT-101): add (member-only, idempotent) and remove (the caller's own) a reaction on
//! a message. Each mutation fans out a `ReactionChanged` event on the channel hub so other members' clients
//! patch live, and emits an additive `chat.message_reacted` audit row. Reactions are returned folded into the
//! message list (see `messages::list`), so the row render can show an emoji-and-count strip without a second
//! round-trip. Fully self-contained: no external dependency, a fixed client-side emoji set.

use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{audit, auth, db, AppState};

#[derive(Debug, Deserialize)]
pub struct AddReaction {
    pub emoji: String,
}

/// One folded reaction summary on a message: the emoji, how many subjects used it, and whether the caller is
/// one of them. The list query attaches a `Vec<ReactionSummary>` per message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: i64,
    pub mine: bool,
}

/// A reaction's longest sensible length: an emoji is a few code points. Reject anything larger so the column
/// (and the picker contract) can't be abused as a free-text store.
const MAX_EMOJI_BYTES: usize = 32;

fn clean_emoji(raw: &str) -> Result<String, (StatusCode, String)> {
    let e = raw.trim().to_string();
    if e.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "emoji is required".to_string()));
    }
    if e.len() > MAX_EMOJI_BYTES {
        return Err((StatusCode::BAD_REQUEST, "emoji is too long".to_string()));
    }
    Ok(e)
}

/// POST /v1/chat/channels/{id}/messages/{msg}/reactions - add the caller's reaction (member-only). Idempotent:
/// a repeat of the same (message, subject, emoji) is a no-op insert. Publishes ReactionChanged{added:true}.
pub async fn add(
    State(st): State<AppState>,
    Path((channel, msg)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    axum::Json(body): axum::Json<AddReaction>,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let emoji = clean_emoji(&body.emoji)?;

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
    // The message must live in this channel; guards a reaction targeting another channel's id.
    let exists: Option<(Uuid,)> = sqlx::query_as(
        "SELECT id FROM chat_messages WHERE id = $1 AND channel_id = $2 AND deleted_at IS NULL",
    )
    .bind(msg)
    .bind(channel)
    .fetch_optional(&mut *tx)
    .await
    .map_err(crate::internal)?;
    if exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            "message not in this channel".to_string(),
        ));
    }
    sqlx::query(
        "INSERT INTO chat_reactions (message_id, channel_id, tenant_id, subject_id, emoji)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (message_id, subject_id, emoji) DO NOTHING",
    )
    .bind(msg)
    .bind(channel)
    .bind(tenant)
    .bind(subject)
    .bind(&emoji)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;

    st.hub.publish(
        channel,
        crate::realtime::ChatEvent::ReactionChanged {
            message_id: msg,
            emoji: emoji.clone(),
            subject,
            added: true,
        },
    );
    audit::emit(
        &st,
        tenant,
        subject,
        "chat.message_reacted",
        serde_json::json!({"channel_id": channel, "message_id": msg, "emoji": emoji, "added": true}),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /v1/chat/channels/{id}/messages/{msg}/reactions/{emoji} - remove the caller's own reaction
/// (member-only). Publishes ReactionChanged{added:false}. A 404 if the caller had not reacted with it.
pub async fn remove(
    State(st): State<AppState>,
    Path((channel, msg, emoji)): Path<(Uuid, Uuid, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, String)> {
    let claims = auth::authenticate(&st, &headers)?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let emoji = clean_emoji(&emoji)?;

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
    let res = sqlx::query(
        "DELETE FROM chat_reactions
         WHERE message_id = $1 AND channel_id = $2 AND subject_id = $3 AND emoji = $4",
    )
    .bind(msg)
    .bind(channel)
    .bind(subject)
    .bind(&emoji)
    .execute(&mut *tx)
    .await
    .map_err(crate::internal)?;
    tx.commit().await.map_err(crate::internal)?;
    if res.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "reaction not found".to_string()));
    }

    st.hub.publish(
        channel,
        crate::realtime::ChatEvent::ReactionChanged {
            message_id: msg,
            emoji: emoji.clone(),
            subject,
            added: false,
        },
    );
    audit::emit(
        &st,
        tenant,
        subject,
        "chat.message_reacted",
        serde_json::json!({"channel_id": channel, "message_id": msg, "emoji": emoji, "added": false}),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

/// Fold raw (message_id, subject_id, emoji) reaction rows into a per-message map of summaries from the
/// caller's point of view. Pure (no DB) so the shape is unit-testable. Emojis keep first-seen order within a
/// message; `mine` is true when `me` contributed that emoji on that message.
pub fn summarize(
    rows: &[(Uuid, Uuid, String)],
    me: Uuid,
) -> std::collections::HashMap<Uuid, Vec<ReactionSummary>> {
    use std::collections::HashMap;
    // Per message: preserve emoji order while accumulating count + whether `me` reacted.
    let mut order: HashMap<Uuid, Vec<String>> = HashMap::new();
    let mut agg: HashMap<(Uuid, String), (i64, bool)> = HashMap::new();
    for (message_id, subject_id, emoji) in rows {
        let key = (*message_id, emoji.clone());
        let entry = agg.entry(key).or_insert((0, false));
        entry.0 += 1;
        if *subject_id == me {
            entry.1 = true;
        }
        let seen = order.entry(*message_id).or_default();
        if !seen.iter().any(|e| e == emoji) {
            seen.push(emoji.clone());
        }
    }
    let mut out: HashMap<Uuid, Vec<ReactionSummary>> = HashMap::new();
    for (message_id, emojis) in order {
        let mut list = Vec::with_capacity(emojis.len());
        for emoji in emojis {
            if let Some((count, mine)) = agg.get(&(message_id, emoji.clone())) {
                list.push(ReactionSummary {
                    emoji,
                    count: *count,
                    mine: *mine,
                });
            }
        }
        out.insert(message_id, list);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid(n: u8) -> Uuid {
        Uuid::from_bytes([n; 16])
    }

    #[test]
    fn summarize_counts_dedupes_and_marks_mine() {
        let me = uid(1);
        let other = uid(2);
        let m1 = uid(10);
        let m2 = uid(20);
        // m1: me+other both 👍 (count 2, mine), other ❤️ (count 1, not mine). m2: me 😂 (count 1, mine).
        let rows = vec![
            (m1, me, "👍".to_string()),
            (m1, other, "👍".to_string()),
            (m1, other, "❤️".to_string()),
            (m2, me, "😂".to_string()),
        ];
        let map = summarize(&rows, me);

        let r1 = map.get(&m1).expect("m1 has reactions");
        assert_eq!(r1.len(), 2, "two distinct emojis on m1");
        let thumbs = r1.iter().find(|r| r.emoji == "👍").unwrap();
        assert_eq!(thumbs.count, 2);
        assert!(thumbs.mine, "me reacted with 👍");
        let heart = r1.iter().find(|r| r.emoji == "❤️").unwrap();
        assert_eq!(heart.count, 1);
        assert!(!heart.mine, "me did not react with ❤️");

        let r2 = map.get(&m2).expect("m2 has reactions");
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0].count, 1);
        assert!(r2[0].mine);
    }

    #[test]
    fn summarize_empty_is_empty() {
        assert!(summarize(&[], uid(1)).is_empty());
    }

    #[test]
    fn clean_emoji_trims_and_rejects_blank_and_oversize() {
        assert_eq!(clean_emoji("  👍 ").unwrap(), "👍");
        assert!(clean_emoji("   ").is_err());
        assert!(clean_emoji(&"x".repeat(MAX_EMOJI_BYTES + 1)).is_err());
    }
}
