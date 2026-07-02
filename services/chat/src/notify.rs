//! Per-user notification socket (get-notified cluster). Where the per-channel `Hub` (realtime.rs) only reaches
//! a client while it holds that one channel's socket, the `Notifier` reaches a user across every channel they
//! belong to - so unread badges, the tab-title count, and desktop notifications work for a channel the user is
//! not currently viewing. On a new message, `fanout` publishes one small `NotifyEvent` to each member's
//! per-user sender (except the author). In-process, single chat container - the same scaling note as `Hub`
//! (a multi-instance deploy would move this fan-out onto Redis pub/sub).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::messages::Message;
use crate::{auth, db, AppState};

/// A cross-channel notification: a new message landed in a channel the recipient belongs to. Deliberately
/// small - the client uses it to bump an unread badge, float the channel, and (opt-in) raise a desktop
/// notification; the full message arrives over the per-channel socket when the channel is opened.
#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotifyEvent {
    Message {
        channel_id: Uuid,
        message_id: Uuid,
        sender: Uuid,
        /// 'group' or 'direct', so the client can label the notification by channel or by person.
        channel_kind: String,
        /// A short, whitespace-collapsed body preview (see `preview`).
        preview: String,
        /// True when this message @-mentions the recipient (filled by the mentions cluster; false for now).
        mention: bool,
        created_at: chrono::DateTime<chrono::Utc>,
    },
}

/// subject_id -> broadcast sender. One sender per user for the life of the process; a user's open tabs each
/// subscribe. Mirrors `realtime::Hub`, keyed by subject instead of channel.
#[derive(Clone, Default)]
pub struct Notifier {
    inner: Arc<Mutex<HashMap<Uuid, broadcast::Sender<NotifyEvent>>>>,
}

impl Notifier {
    pub fn sender(&self, subject: Uuid) -> broadcast::Sender<NotifyEvent> {
        let mut g = self.inner.lock().expect("notifier mutex poisoned");
        g.entry(subject)
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub fn publish(&self, subject: Uuid, event: NotifyEvent) {
        let _ = self.sender(subject).send(event);
    }
}

/// A one-line preview of a message body for a notification: collapse runs of whitespace to single spaces, cap
/// the length, and fall back to a neutral label for an attachment-only message. Pure, so it is unit-testable.
pub fn preview(body: &str, has_attachment: bool) -> String {
    let collapsed = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return if has_attachment {
            "sent an attachment".to_string()
        } else {
            String::new()
        };
    }
    const MAX: usize = 140;
    if collapsed.chars().count() > MAX {
        let mut s: String = collapsed.chars().take(MAX).collect();
        s.push_str("...");
        s
    } else {
        collapsed
    }
}

/// Fan a new message out to every channel member except the sender, over their per-user socket. Runs off the
/// response path (spawned from `messages::post`) and is best-effort: one tenant-scoped members query, and a
/// failure never affects the send. `mention_ids` are the members the message @-mentions (empty until the
/// mentions cluster fills it), which set `mention:true` for those recipients.
pub async fn fanout(
    st: AppState,
    channel: Uuid,
    tenant: Uuid,
    message: Message,
    channel_kind: String,
    mention_ids: Vec<Uuid>,
) {
    let mut tx = match db::tenant_tx(&st.pool, &tenant).await {
        Ok(tx) => tx,
        Err(_) => return,
    };
    let members: Vec<(Uuid,)> =
        match sqlx::query_as("SELECT subject_id FROM chat_channel_members WHERE channel_id = $1")
            .bind(channel)
            .fetch_all(&mut *tx)
            .await
        {
            Ok(m) => m,
            Err(_) => return,
        };
    let _ = tx.commit().await;

    let text = preview(
        &message.body,
        message.attachment_id.is_some() || !message.attachments.is_empty(),
    );
    for (member,) in members {
        if member == message.sender_subject_id {
            continue;
        }
        st.notifier.publish(
            member,
            NotifyEvent::Message {
                channel_id: channel,
                message_id: message.id,
                sender: message.sender_subject_id,
                channel_kind: channel_kind.clone(),
                preview: text.clone(),
                mention: mention_ids.contains(&member),
                created_at: message.created_at,
            },
        );
    }
}

#[derive(Debug, Deserialize)]
pub struct NotifyQuery {
    pub access_token: Option<String>,
}

/// GET /v1/chat/notify - one persistent per-user socket streaming `NotifyEvent`s across all the caller's
/// channels. Auth by bearer header or `?access_token` (a browser cannot set a header on a websocket). There
/// are no client-to-server frames on this socket.
pub async fn notify_ws(
    State(st): State<AppState>,
    Query(q): Query<NotifyQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, (StatusCode, String)> {
    let token = auth::bearer(&headers)
        .map(|s| s.to_string())
        .or(q.access_token)
        .ok_or((StatusCode::UNAUTHORIZED, "missing token".to_string()))?;
    let claims = st
        .authenticator
        .verify(&token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let rx = st.notifier.sender(subject).subscribe();
    Ok(ws.on_upgrade(move |socket| notify_loop(socket, rx)))
}

async fn notify_loop(mut socket: WebSocket, mut rx: broadcast::Receiver<NotifyEvent>) {
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        let text = serde_json::to_string(&ev).unwrap_or_default();
                        if socket.send(WsMessage::Text(text)).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            inbound = socket.recv() => {
                // No client-to-server frames on the notify socket; ignore any (including pings) and only use
                // recv() to detect the close.
                match inbound {
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_collapses_whitespace_caps_and_labels_attachment() {
        assert_eq!(
            preview("  hello   world \n taken", false),
            "hello world taken"
        );
        assert_eq!(preview("   ", false), "");
        assert_eq!(preview("", true), "sent an attachment");
        // An empty body with an attachment prefers the neutral label over the empty string.
        assert_eq!(preview("   \t ", true), "sent an attachment");
        let long = "a".repeat(200);
        let p = preview(&long, false);
        assert!(p.ends_with("..."));
        assert!(p.chars().count() <= 143, "140 chars + the three-dot marker");
    }

    #[test]
    fn notifier_delivers_only_to_the_subscribed_subject() {
        let n = Notifier::default();
        let me = Uuid::from_bytes([1; 16]);
        let other = Uuid::from_bytes([2; 16]);
        let mut rx = n.sender(me).subscribe();
        let ev = |mention: bool, preview: &str| NotifyEvent::Message {
            channel_id: Uuid::nil(),
            message_id: Uuid::nil(),
            sender: other,
            channel_kind: "group".to_string(),
            preview: preview.to_string(),
            mention,
            created_at: chrono::Utc::now(),
        };
        // A publish addressed to `other` must not reach my receiver; one addressed to `me` must.
        n.publish(other, ev(false, "not for me"));
        n.publish(me, ev(true, "for me"));
        match rx.try_recv().expect("one event delivered to me") {
            NotifyEvent::Message {
                preview, mention, ..
            } => {
                assert_eq!(preview, "for me");
                assert!(mention);
            }
        }
        assert!(
            rx.try_recv().is_err(),
            "no second event - other's publish is not delivered to me"
        );
    }
}
