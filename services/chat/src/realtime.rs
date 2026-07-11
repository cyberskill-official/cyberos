//! Real-time layer (FR-CHAT-101 slices 1, 4, 5): one broadcast channel per chat channel carrying a typed
//! event stream (message, presence, typing, signal), plus the websocket handler. Presence is tracked from
//! the live connections. Signal events (WebRTC offer/answer/ICE for voice/video, slice 5) are relayed only
//! to the addressed subject - the media itself and mobile clients are out of scope for the server.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message as WsMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::messages::Message;
use crate::{auth, db, AppState};

/// What flows over a channel's websocket. `signal` is only forwarded to its addressed subject.
#[derive(Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ChatEvent {
    Message(Message),
    Presence {
        subject: Uuid,
        status: &'static str,
    },
    Typing {
        subject: Uuid,
    },
    Read {
        subject: Uuid,
        last_read_message_id: Uuid,
        last_read_at: chrono::DateTime<chrono::Utc>,
    },
    Signal {
        from: Uuid,
        to: Uuid,
        data: serde_json::Value,
    },
    MessageEdited {
        id: Uuid,
        /// FR-CHAT-268 — who wrote it. Added because without it this frame was a hole in the block: a
        /// blocked person edits a message and their new text lands on the blocker's socket, since the
        /// subscriber had no way to tell whose edit it was. Additive on the wire; older clients ignore it.
        sender: Uuid,
        body: String,
        edited_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    MessageDeleted {
        id: Uuid,
    },
    ReactionChanged {
        message_id: Uuid,
        emoji: String,
        subject: Uuid,
        added: bool,
        /// Absolute count of this emoji on this message AFTER the change, so clients replace (not
        /// delta-mutate) and stay correct across event replays / reconnects.
        count: i64,
    },
    /// A subject was removed from the channel: their own live socket(s) close on receipt. Not forwarded to
    /// anyone else (authorization is otherwise only checked at upgrade, so this severs a removed member fast).
    Kicked {
        subject: Uuid,
    },
}

/// channel_id -> broadcast sender. Senders live for the process; subscribers come and go.
#[derive(Clone, Default)]
pub struct Hub {
    inner: Arc<Mutex<HashMap<Uuid, broadcast::Sender<ChatEvent>>>>,
}

impl Hub {
    pub fn sender(&self, channel: Uuid) -> broadcast::Sender<ChatEvent> {
        let mut guard = self.inner.lock().expect("hub mutex poisoned");
        guard
            .entry(channel)
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    }

    pub fn publish(&self, channel: Uuid, event: ChatEvent) {
        let _ = self.sender(channel).send(event);
    }

    /// Drop a channel's broadcast sender once it has no receivers, so the map does not grow without bound over
    /// the process lifetime. Safe against a concurrent subscribe: `sender` re-creates the entry under the same
    /// lock, and a fresh sender loses nothing because there were no receivers to miss events.
    pub fn reap(&self, channel: Uuid) {
        let mut guard = self.inner.lock().expect("hub mutex poisoned");
        if let Some(s) = guard.get(&channel) {
            if s.receiver_count() == 0 {
                guard.remove(&channel);
            }
        }
    }
}

/// Live presence: channel_id -> subject_id -> open-connection count.
#[derive(Clone, Default)]
pub struct Presence {
    inner: Arc<Mutex<HashMap<Uuid, HashMap<Uuid, u32>>>>,
}

impl Presence {
    /// Register a connection; returns true if the subject just came online in this channel (0 -> 1).
    pub fn join(&self, channel: Uuid, subject: Uuid) -> bool {
        let mut g = self.inner.lock().expect("presence mutex poisoned");
        let m = g.entry(channel).or_default();
        let c = m.entry(subject).or_insert(0);
        *c += 1;
        *c == 1
    }

    /// Drop a connection; returns true if the subject just went offline (1 -> 0).
    pub fn leave(&self, channel: Uuid, subject: Uuid) -> bool {
        let mut g = self.inner.lock().expect("presence mutex poisoned");
        if let Some(m) = g.get_mut(&channel) {
            if let Some(c) = m.get_mut(&subject) {
                *c = c.saturating_sub(1);
                if *c == 0 {
                    m.remove(&subject);
                    return true;
                }
            }
        }
        false
    }

    pub fn online(&self, channel: Uuid) -> Vec<Uuid> {
        let g = self.inner.lock().expect("presence mutex poisoned");
        g.get(&channel)
            .map(|m| m.keys().copied().collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    pub channel: Uuid,
    pub access_token: Option<String>,
}

pub async fn ws_handler(
    State(st): State<AppState>,
    Query(q): Query<WsQuery>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Result<Response, (StatusCode, String)> {
    let token = auth::bearer(&headers)
        .map(|s| s.to_string())
        .or_else(|| q.access_token.clone())
        .ok_or((StatusCode::UNAUTHORIZED, "missing token".to_string()))?;
    let claims = st
        .authenticator
        .verify(&token)
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let tenant = claims
        .tenant_uuid()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let subject = claims
        .subject_id()
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;
    let channel = q.channel;

    let mut tx = db::tenant_tx(&st.pool, &tenant)
        .await
        .map_err(crate::internal)?;
    let member = db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?;
    let _ = tx.commit().await;
    if member.is_none() {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }

    let rx = st.hub.sender(channel).subscribe();
    Ok(ws.on_upgrade(move |socket| ws_loop(socket, rx, st, tenant, channel, subject)))
}

async fn ws_loop(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<ChatEvent>,
    st: AppState,
    tenant: Uuid,
    channel: Uuid,
    me: Uuid,
) {
    // FR-CHAT-268 — the channel's kind decides DROP (direct) vs COLLAPSE (group) for a blocked sender's
    // frame. Resolved once at connect: it cannot change for the life of a channel.
    let is_dm = {
        let k: Result<Option<(String,)>, _> =
            sqlx::query_as("SELECT kind FROM chat_channels WHERE id = $1")
                .bind(channel)
                .fetch_optional(&st.pool)
                .await;
        matches!(k, Ok(Some((ref kind,))) if kind == "direct")
    };

    if st.presence.join(channel, me) {
        st.hub.publish(
            channel,
            ChatEvent::Presence {
                subject: me,
                status: "online",
            },
        );
        // FR-MEMORY-122 §1 #8 — presence_changed{online} ONLY on the 0->1 edge (presence.join returned
        // true). A second tab does NOT re-emit. Spawned + best-effort; no-op unless capture on. trace_id is
        // null for a websocket edge (§1 #14 — never fabricated).
        if let Some(cap) = st.capturer.clone() {
            tokio::spawn(async move {
                crate::capture::emit_presence_changed(Some(&cap), tenant, me, channel, "online")
                    .await;
            });
        }
    }
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(mut ev) => {
                        // ───── FR-CHAT-268 enforcement point 2 of 4: the realtime socket (§1 #4) ─────
                        // The check CANNOT live at the publish site: the per-channel broadcast is
                        // one-to-many and the sender has no idea who has blocked them. It lives HERE, at
                        // each subscriber, where `me` is known. Served off the in-process cache, so a frame
                        // costs no query — and the cache is invalidated synchronously by block/unblock, so
                        // an already-open tab stops receiving the moment the block lands (§10 row 5).
                        let blocked = crate::blocks::blocked_by(&st, tenant, me).await;
                        if !blocked.is_empty() {
                            match &mut ev {
                                ChatEvent::Message(m) if blocked.contains(&m.sender_subject_id) => {
                                    if is_dm {
                                        continue; // §1 #6 — no frame at all
                                    }
                                    // §1 #5 — collapse in place; the row keeps its id and position.
                                    m.body = String::new();
                                    m.attachments.clear();
                                    m.reactions.clear();
                                    m.blocked_sender = true;
                                }
                                // An edit by a blocked sender would otherwise push their new text straight
                                // onto the blocker's socket.
                                ChatEvent::MessageEdited { sender, .. }
                                    if blocked.contains(sender) =>
                                {
                                    continue
                                }
                                // §1 #9 — a blocked person's reaction is not shown to the blocker.
                                ChatEvent::ReactionChanged { subject, .. }
                                    if blocked.contains(subject) =>
                                {
                                    continue
                                }
                                // Presence and typing from a blocked person are noise, not content, but they
                                // still announce the person. Suppress them too.
                                ChatEvent::Presence { subject, .. } | ChatEvent::Typing { subject }
                                    if blocked.contains(subject) =>
                                {
                                    continue
                                }
                                _ => {}
                            }
                        }

                        // A signal is private to its addressed subject; presence and typing are not
                        // echoed back to their own originator.
                        match &ev {
                            // A removed member's own socket closes; everyone else never sees the frame.
                            ChatEvent::Kicked { subject } => {
                                if *subject == me {
                                    break;
                                }
                                continue;
                            }
                            ChatEvent::Signal { to, .. } if *to != me => continue,
                            ChatEvent::Presence { subject, .. } | ChatEvent::Typing { subject }
                                if *subject == me =>
                            {
                                continue
                            }
                            _ => {}
                        }
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
                match inbound {
                    Some(Ok(WsMessage::Text(t))) => handle_inbound(&st, channel, me, &t),
                    Some(Ok(_)) => {}
                    _ => break,
                }
            }
        }
    }
    if st.presence.leave(channel, me) {
        st.hub.publish(
            channel,
            ChatEvent::Presence {
                subject: me,
                status: "offline",
            },
        );
        // FR-MEMORY-122 §1 #8 — presence_changed{offline} ONLY on the 1->0 edge (the last connection
        // closed). Closing one of several tabs does NOT emit. Spawned + best-effort; no-op unless capture on.
        if let Some(cap) = st.capturer.clone() {
            tokio::spawn(async move {
                crate::capture::emit_presence_changed(Some(&cap), tenant, me, channel, "offline")
                    .await;
            });
        }
    }
    // This receiver is gone; drop it before reaping so the count reflects only the remaining connections.
    drop(rx);
    st.hub.reap(channel);
}

#[derive(Debug, Deserialize)]
struct Inbound {
    #[serde(rename = "type")]
    kind: String,
    to: Option<Uuid>,
    #[serde(default)]
    data: serde_json::Value,
}

/// Client-to-server frames: a typing ping, or a WebRTC signal addressed to another member.
fn handle_inbound(st: &AppState, channel: Uuid, me: Uuid, text: &str) {
    let Ok(msg) = serde_json::from_str::<Inbound>(text) else {
        return;
    };
    match msg.kind.as_str() {
        "typing" => st.hub.publish(channel, ChatEvent::Typing { subject: me }),
        "signal" => {
            if let Some(to) = msg.to {
                st.hub.publish(
                    channel,
                    ChatEvent::Signal {
                        from: me,
                        to,
                        data: msg.data,
                    },
                );
            }
        }
        _ => {}
    }
}

/// GET /v1/chat/channels/{id}/presence - the subjects currently connected to the channel (members only).
pub async fn presence_list(
    State(st): State<AppState>,
    axum::extract::Path(channel): axum::extract::Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<Vec<Uuid>>, (StatusCode, String)> {
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
    let member = db::role_in_channel(&mut tx, channel, subject)
        .await
        .map_err(crate::internal)?;
    let _ = tx.commit().await;
    if member.is_none() {
        return Err((StatusCode::FORBIDDEN, "not a channel member".to_string()));
    }
    Ok(Json(st.presence.online(channel)))
}
