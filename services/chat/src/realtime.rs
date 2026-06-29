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
    Signal {
        from: Uuid,
        to: Uuid,
        data: serde_json::Value,
    },
    MessageEdited {
        id: Uuid,
        body: String,
        edited_at: Option<chrono::DateTime<chrono::Utc>>,
    },
    MessageDeleted {
        id: Uuid,
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
    Ok(ws.on_upgrade(move |socket| ws_loop(socket, rx, st, channel, subject)))
}

async fn ws_loop(
    mut socket: WebSocket,
    mut rx: broadcast::Receiver<ChatEvent>,
    st: AppState,
    channel: Uuid,
    me: Uuid,
) {
    if st.presence.join(channel, me) {
        st.hub.publish(
            channel,
            ChatEvent::Presence {
                subject: me,
                status: "online",
            },
        );
    }
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    Ok(ev) => {
                        // A signal is private to its addressed subject; presence and typing are not
                        // echoed back to their own originator.
                        match &ev {
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
    }
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
