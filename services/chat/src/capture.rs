//! FR-MEMORY-122 §1 #4 — CHAT's capture surface (the per-module `capture.rs`, DEC-2714).
//!
//! Thin typed helpers that translate CHAT's domain events into FR-MEMORY-121 interaction-events and record
//! them through the shared [`cyberos_capture::Capturer`] over the chat->brain audit pool (the pool opened
//! from `CHAT_AUDIT_DATABASE_URL`; DEC-2713 turns that link ON). Message bodies are NEVER inlined: created
//! and edited events carry `content_ref: pointer{store:"chat_messages", id}`; a deleted message carries
//! `content_ref: none` (the body is gone — a pointer would dangle). Channel/DM/presence activity carries no
//! content at all.
//!
//! Everything here is best-effort and OFF by default:
//!   * Each helper takes `Option<&Capturer>`; when `None` (the default, because `CAPTURE_ENABLED` is unset
//!     and/or no audit pool is configured) it returns immediately and does nothing. The message still sent,
//!     the channel still created — capture is invisible.
//!   * When a capturer is present, the helper builds the event and calls `capture_metered`, which routes
//!     through FR-MEMORY-121 `emit()` (consent-gated: a sender who has not acknowledged the notice yields a
//!     `Skipped` outcome and zero rows) and swallows any error. A capture failure NEVER fails or delays the
//!     send. Call sites further spawn the emit so it cannot delay the HTTP response (see messages.rs).

use cyberos_capture::{Capturer, ContentRef, EventClass, InteractionEvent, Module, SourceChannel, TargetRef};
use uuid::Uuid;

/// Build the right `target_ref` for a channel-scoped event: a DM channel uses `dm{id}`, any other kind
/// uses `channel{id}`. `channel_kind` is chat's `chat_channels.kind` ("direct" => dm).
fn channel_target(channel_id: Uuid, channel_kind: &str) -> TargetRef {
    match channel_kind {
        "direct" | "dm" => TargetRef::Dm {
            id: channel_id.to_string(),
        },
        _ => TargetRef::Channel {
            id: channel_id.to_string(),
        },
    }
}

/// The source channel CHAT tags its events with. The chat clients do not yet pass an explicit channel hint
/// on every call, so this is `Web` for now (the P0 product is the web console; FR-MEMORY-122 §1 #13 lets a
/// client refine it later). Kept as one helper so the choice is in one place.
pub fn default_source() -> SourceChannel {
    SourceChannel::Web
}

/// §1 #4 — `chat.message_created` (`content`). `target_ref` is the channel/DM; `content_ref` POINTS at
/// chat's own `chat_messages` row (never the raw body); attributes carry `channel_kind` + `has_attachment`.
/// Best-effort no-op when `cap` is `None`.
pub async fn emit_message_created(
    cap: Option<&Capturer>,
    tenant: Uuid,
    author: Uuid,
    channel_id: Uuid,
    channel_kind: &str,
    message_id: Uuid,
    has_attachment: bool,
) {
    let Some(cap) = cap else { return };
    let ev = InteractionEvent::builder(Module::Chat, "chat.message_created", EventClass::Content)
        .tenant(tenant)
        .subject(author)
        .occurred_now()
        .target(channel_target(channel_id, channel_kind))
        // §1 #4 — pointer to chat's own row, NEVER the raw body.
        .content(ContentRef::pointer("chat_messages", message_id.to_string()))
        .source(default_source())
        .attribute(
            "channel_kind",
            serde_json::Value::String(channel_kind.to_string()),
        )
        .attribute("has_attachment", serde_json::Value::Bool(has_attachment))
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "chat.message_created event build failed (best-effort)"),
    }
}

/// §1 #4 — `chat.message_edited` (`content`, same pointer target as created). Best-effort no-op when `None`.
pub async fn emit_message_edited(
    cap: Option<&Capturer>,
    tenant: Uuid,
    author: Uuid,
    channel_id: Uuid,
    channel_kind: &str,
    message_id: Uuid,
) {
    let Some(cap) = cap else { return };
    let ev = InteractionEvent::builder(Module::Chat, "chat.message_edited", EventClass::Content)
        .tenant(tenant)
        .subject(author)
        .occurred_now()
        .target(channel_target(channel_id, channel_kind))
        .content(ContentRef::pointer("chat_messages", message_id.to_string()))
        .source(default_source())
        .attribute(
            "channel_kind",
            serde_json::Value::String(channel_kind.to_string()),
        )
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "chat.message_edited event build failed (best-effort)"),
    }
}

/// §1 #4 — `chat.message_deleted` (`content`, `content_ref: none` — the body is gone, a pointer would
/// dangle). Best-effort no-op when `None`.
pub async fn emit_message_deleted(
    cap: Option<&Capturer>,
    tenant: Uuid,
    actor: Uuid,
    channel_id: Uuid,
    channel_kind: &str,
    message_id: Uuid,
) {
    let Some(cap) = cap else { return };
    let ev = InteractionEvent::builder(Module::Chat, "chat.message_deleted", EventClass::Content)
        .tenant(tenant)
        .subject(actor)
        .occurred_now()
        .target(channel_target(channel_id, channel_kind))
        .content(ContentRef::None)
        .source(default_source())
        .attribute(
            "channel_kind",
            serde_json::Value::String(channel_kind.to_string()),
        )
        .attribute("message_id", serde_json::Value::String(message_id.to_string()))
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "chat.message_deleted event build failed (best-effort)"),
    }
}

/// §1 #4 — `chat.channel_created` (`admin`, `target_ref: channel{id}`). Best-effort no-op when `None`.
pub async fn emit_channel_created(cap: Option<&Capturer>, tenant: Uuid, creator: Uuid, channel_id: Uuid) {
    let Some(cap) = cap else { return };
    let ev = InteractionEvent::builder(Module::Chat, "chat.channel_created", EventClass::Admin)
        .tenant(tenant)
        .subject(creator)
        .occurred_now()
        .target(TargetRef::Channel {
            id: channel_id.to_string(),
        })
        .content(ContentRef::None)
        .source(default_source())
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "chat.channel_created event build failed (best-effort)"),
    }
}

/// §1 #4 — `chat.channel_joined` / `chat.channel_left` (`activity`, `target_ref: channel{id}`). `joined`
/// selects the verb. Best-effort no-op when `None`.
pub async fn emit_channel_membership(
    cap: Option<&Capturer>,
    tenant: Uuid,
    subject: Uuid,
    channel_id: Uuid,
    joined: bool,
) {
    let Some(cap) = cap else { return };
    let verb = if joined {
        "chat.channel_joined"
    } else {
        "chat.channel_left"
    };
    let ev = InteractionEvent::builder(Module::Chat, verb, EventClass::Activity)
        .tenant(tenant)
        .subject(subject)
        .occurred_now()
        .target(TargetRef::Channel {
            id: channel_id.to_string(),
        })
        .content(ContentRef::None)
        .source(default_source())
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, verb, "chat channel-membership event build failed (best-effort)"),
    }
}

/// §1 #4 — `chat.dm_opened` (`activity`, `target_ref: dm{id}`). Emitted when a DM channel is first created
/// (find-or-create returns an existing one without re-emitting). Best-effort no-op when `None`.
pub async fn emit_dm_opened(cap: Option<&Capturer>, tenant: Uuid, opener: Uuid, channel_id: Uuid) {
    let Some(cap) = cap else { return };
    let ev = InteractionEvent::builder(Module::Chat, "chat.dm_opened", EventClass::Activity)
        .tenant(tenant)
        .subject(opener)
        .occurred_now()
        .target(TargetRef::Dm {
            id: channel_id.to_string(),
        })
        .content(ContentRef::None)
        .source(default_source())
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "chat.dm_opened event build failed (best-effort)"),
    }
}

/// §1 #4, #8 — `chat.presence_changed` (`presence`, `target_ref: channel{id}`, `attributes:{state}` where
/// state ∈ `online | offline`). The caller emits this ONLY on the 0<->1 connection-count edge (realtime.rs).
/// `trace_id` is null for a websocket presence edge (no request trace in scope, §1 #14). Best-effort no-op
/// when `None`.
pub async fn emit_presence_changed(
    cap: Option<&Capturer>,
    tenant: Uuid,
    subject: Uuid,
    channel_id: Uuid,
    state: &str,
) {
    let Some(cap) = cap else { return };
    let ev = InteractionEvent::builder(Module::Chat, "chat.presence_changed", EventClass::Presence)
        .tenant(tenant)
        .subject(subject)
        .occurred_now()
        .target(TargetRef::Channel {
            id: channel_id.to_string(),
        })
        .content(ContentRef::None)
        .source(default_source())
        .attribute("state", serde_json::Value::String(state.to_string()))
        .build();
    match ev {
        Ok(ev) => cap.capture_metered(&ev).await,
        Err(e) => tracing::warn!(error = %e, "chat.presence_changed event build failed (best-effort)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_target_maps_kind() {
        let id = Uuid::nil();
        assert!(matches!(
            channel_target(id, "direct"),
            TargetRef::Dm { .. }
        ));
        assert!(matches!(
            channel_target(id, "group"),
            TargetRef::Channel { .. }
        ));
    }

    // The helpers are no-ops when no capturer is present (the default-OFF state). They must not panic and
    // must do nothing — proving capture is invisible to chat with CAPTURE_ENABLED unset.
    #[tokio::test]
    async fn helpers_are_noops_without_a_capturer() {
        let (t, s, c, m) = (Uuid::nil(), Uuid::nil(), Uuid::nil(), Uuid::nil());
        emit_message_created(None, t, s, c, "group", m, false).await;
        emit_message_edited(None, t, s, c, "group", m).await;
        emit_message_deleted(None, t, s, c, "group", m).await;
        emit_channel_created(None, t, s, c).await;
        emit_channel_membership(None, t, s, c, true).await;
        emit_dm_opened(None, t, s, c).await;
        emit_presence_changed(None, t, s, c, "online").await;
    }
}
