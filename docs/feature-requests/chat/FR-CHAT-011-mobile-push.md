---
id: FR-CHAT-011
title: "Mobile push delivery — APNS + FCM with privacy-preserving payload (title + sender only; no body)"
module: CHAT
priority: MUST
status: ready_to_test
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-CHAT-003, FR-CHAT-005]
depends_on: [FR-CHAT-003]
blocks: []

source_pages:
  - website/docs/modules/chat.html#mobile-push
source_decisions:
  - DEC-520 (push payload = title + sender display_name + channel_name; NEVER message body)
  - DEC-521 (devices register via token endpoint; per-device token lifecycle managed)
  - DEC-522 (delivery uses APNS HTTP/2 + FCM v1; legacy FCM-key path forbidden)

language: rust 1.81
service: cyberos/services/chat-push/
new_files:
  - services/chat-push/Cargo.toml
  - services/chat-push/src/main.rs
  - services/chat-push/src/apns.rs
  - services/chat-push/src/fcm.rs
  - services/chat-push/src/registry.rs
  - services/chat-push/tests/push_test.rs
  - services/chat/sql/init-push-devices.sql
modified_files:
  - services/chat/plugins/cyberos-push-trigger/main.go    # MM plugin emits push on new msg
allowed_tools:
  - file_read: services/chat-push/**, services/chat/**
  - file_write: services/chat-push/{src,tests}/**, services/chat/sql/**
  - bash: cd services/chat-push && cargo test
disallowed_tools:
  - include message body in push payload (per DEC-520)
  - use legacy FCM-key auth (per DEC-522 — v1 only)

effort_hours: 6
sub_tasks:
  - "0.5h: init-push-devices.sql migration"
  - "0.5h: registry.rs — register/unregister device tokens; bind to subject_id"
  - "1.0h: apns.rs — APNS HTTP/2 with JWT auth (.p8 key)"
  - "1.0h: fcm.rs — FCM v1 HTTP with service-account JWT"
  - "1.0h: payload builder (privacy: title=channel_name, body=`@<sender>`)"
  - "0.5h: MM plugin emits push trigger via webhook"
  - "0.5h: chat-push service consumes webhook → fan-out APNS + FCM"
  - "0.5h: memory audit 'chat.push_delivered' or 'chat.push_failed'"
  - "0.5h: push_test.rs — payload assertion + delivery mock"
  - "0.5h: mute settings: per-user, per-channel; respect at push trigger"
risk_if_skipped: "Mobile users miss messages; revert to slack. Without privacy-preserving payload, lock-screen exposes business content (PDPL violation). Without proper auth, APNS deprecates legacy certs in 2025. Without mute settings, every message wakes phones at 3am."
---

## §1 — Description (BCP-14 normative)

The mobile push service **MUST** deliver chat-message notifications to APNS + FCM with privacy-preserving payload. The contract:

1. **MUST** define `push_devices` table: `(id UUID PK, subject_id UUID, platform TEXT (`apns | fcm`), device_token TEXT, app_version TEXT, registered_at, last_seen_at, tenant_id)`. Composite UNIQUE on `(subject_id, device_token)`.
2. **MUST** expose `POST /api/push/register` (auth required per FR-AUTH-004) with body `{platform, device_token, app_version}` → upserts row.
3. **MUST** expose `DELETE /api/push/register/:token` → soft-delete.
4. **MUST** the Mattermost plugin `cyberos-push-trigger` on post.create:
   - Fetch channel subscribers (excluding sender) who have devices registered.
   - Filter out users with muted channel/user preferences.
   - POST webhook to chat-push service per recipient.
5. **MUST** the chat-push service receives webhook with `{post_id, channel_id, channel_name, sender_display_name, recipient_subject_id, tenant_id}`.
6. **MUST** construct privacy-preserving payload:
   - APNS: `{"aps": {"alert": {"title": "<channel_name>", "body": "@<sender_display_name>"}, "sound": "default", "thread-id": "<channel_id>"}, "cyberos": {"post_id": "<id>", "tenant_id": "<id>"}}`.
   - FCM: equivalent `{notification: {title, body}, data: {post_id, tenant_id, channel_id}, android: {priority: "high"}}`.
   - NEVER include message body content (DEC-520).
7. **MUST** use APNS HTTP/2 with JWT auth (Apple .p8 key); FCM v1 with service-account JWT (no legacy server key).
8. **MUST** handle delivery errors:
   - APNS 410 (unregistered) → soft-delete device.
   - APNS 429 (rate limit) → exp backoff + retry 3×.
   - FCM `UNREGISTERED` → soft-delete.
   - FCM `INVALID_ARGUMENT` → log; do NOT retry.
9. **MUST** emit memory audit `chat.push_delivered` or `chat.push_failed` per attempt with `{post_id, recipient_subject_id, platform, latency_ms, outcome, trace_id}`.
10. **MUST** respect user mute settings (Mattermost preferences API):
    - `notify_props.push = "none"` → no push.
    - `notify_props.push = "mention"` → push only if mentioned.
    - Channel-level mute → no push for that channel.
11. **MUST** complete fan-out within 1s p95 for ≤ 100 recipients per message.
12. **MUST** emit OTel metrics:
    - `chat_push_delivered_total{platform, outcome}`.
    - `chat_push_latency_seconds{platform}`.
    - `chat_push_registered_devices` (gauge).
13. **MUST** dedup duplicate pushes per recipient: a per-(subject, channel) suppression window of 1s collapses rapid-fire messages into one delivery. Operator-visible via `chat_push_suppressed_total` counter.
14. **MUST** include sound + badge in the payload as configurable per-platform: `aps.sound = "default"` AND `aps.badge` = unread count (queried from MM API). FCM payload includes `android.notification.sound` + `android.notification.notification_count`.
15. **MUST** support DnD (Do Not Disturb) windows per user via `notify_props.push_status_quiet_hours = {start, end, timezone}`. Pushes during DnD are queued OR suppressed based on `notify_props.push_status_strategy = queue|drop`. Default = `drop`.
16. **MUST** include sender's display name and `subject_id` in the `data` payload (NOT the title) so client apps can show profile pic without re-fetching.
17. **MUST** support "silent" pushes for app state sync (e.g. badge update only) via `content-available: 1` on APNS / `data`-only on FCM. These do NOT show a banner but trigger background fetch.
18. **MUST** sign per-tenant: APNS topic includes tenant suffix (`com.cyberskill.chat.<tenant-shortid>`) so push permissions are auditable per-tenant. FCM project per tenant (slice-4+ optimisation; current is shared project with tenant_id in data).
19. **MUST** support per-user "push for mentions only" with mention detection: if `notify_props.push = "mention"` AND post body contains `@<recipient_username>` OR `@channel` OR `@here` → push fires; else suppressed.
20. **MUST** include trace_id propagation: MM plugin trace → push webhook trace → APNS/FCM call (apns-id header on APNS) → memory audit trace_id all match.
21. **MUST** rate-limit per-recipient: max 60 push notifications per minute per (subject, tenant). Excess suppressed + `chat_push_rate_limited_total` counter; 61st triggers a "muted because of rate limit" silent push (slice-4+ enhancement; current MVP just suppresses).
22. **MUST** support APNS production vs sandbox via `apns_environment` config (`production | sandbox`); APNS sandbox endpoint = `api.sandbox.push.apple.com`.
23. **MUST** track last-successful-delivery per device: `push_devices.last_delivered_at`; devices not delivered to in 30 days are flagged as "stale" but not deleted (user may have reinstalled). Soft-deletion still happens on 410 / UNREGISTERED only.
24. **MUST** include a `priority` field in the payload: chat messages = `high` (immediate delivery); silent state-syncs = `normal` (battery-friendly). APNS `apns-priority` header + FCM `android.priority`.
25. **MUST** support per-tenant push template overrides for the title format (default: `<channel_name>`; some tenants want `<tenant_name> · <channel_name>` for multi-tenant device users). Stored in `cyberos_chat_tenant_settings.push_title_template` with placeholder `{channel}` + `{tenant}`.
26. **MUST** support per-device language preference: `push_devices.locale` (BCP 47); title format applies locale-aware localisation (e.g. "New message" → "Tin nhắn mới" in vi-VN).

---

## §2 — Why this design

**Why privacy-preserving payload (DEC-520)?** Lock-screen exposure of business content = PDPL violation. Title + sender is the canonical minimum-information notification.

**Why APNS HTTP/2 + FCM v1 (DEC-522)?** Legacy APNS certs deprecated; legacy FCM server-key deprecated 2024. Modern auth is the only supported path.

**Why soft-delete on UNREGISTERED (§1 #8)?** Hard-delete loses audit trail; soft-delete keeps "this device was registered until X" for forensics.

**Why mute settings respected (§1 #10)?** Users explicitly opt-out per channel; overriding = breach of trust.

**Why per-recipient webhook (§1 #4)?** Fan-out at the push service (not in MM plugin) keeps plugin lightweight; service can scale independently.

**Why per-(subject, channel) dedup (§1 #13)?** Rapid-fire messages (typing, copy-paste) generate a push per message — user gets phone vibration storm. 1s window collapses to one push per channel; user gets one notification, opens app to see all messages.

**Why badge count from MM (§1 #14)?** iOS users see "5 new messages" on the icon; Android similar. Without badge, lock-screen has no aggregate state. Fetching the actual count from MM API costs ~50ms but is informational-critical.

**Why DnD windows (§1 #15)?** Vietnamese workdays are long; users want phone-quiet 22:00–07:00. Without DnD, push wakes them. Queue vs drop is a user preference: queue = "I'll catch up in the morning"; drop = "yesterday's news is no news."

**Why subject_id in payload data (§1 #16)?** Client apps display sender's profile pic in the notification (especially on Android expanded view). Without subject_id, app has to fetch by display_name (slow, ambiguous if names collide).

**Why silent pushes (§1 #17)?** Badge updates + unread counts don't need a banner; silent push keeps the iOS app's badge correct without interrupting the user.

**Why per-tenant APNS topic (§1 #18)?** Multi-tenant device users (operator's phone has 3 tenants registered) need tenant-distinguishable APNS topics for permission management — they can disable notifications for one tenant without disabling all.

**Why mention-only respects @channel/@here (§1 #19)?** Mattermost's "mention" notification semantic includes channel-wide mentions, not just direct mentions. Without honouring them, important channel announcements would be missed.

**Why apns-id header for trace (§1 #20)?** APNS's `apns-id` is echoed in the delivery receipt; matching to our trace_id enables end-to-end debugging.

**Why per-recipient rate limit (§1 #21)?** Push is a stateful permission; misuse triggers iOS rate limiting (Apple suspends the topic). Self-rate-limiting at 60/min protects the topic permission for all users.

**Why APNS production vs sandbox (§1 #22)?** Dev/staging builds use sandbox; production builds use prod. Wrong endpoint = silent delivery failure (no error, just no notification).

**Why last_delivered_at (§1 #23)?** Stale devices indicate a user who reinstalled the app or stopped using mobile; flagging is useful for tenant-level analytics (e.g. "this tenant's mobile adoption is declining") without auto-deleting (which would break the few users still on the device).

**Why high vs normal priority (§1 #24)?** Apple/Google rate-limit `high` priority pushes; sending silent state-syncs as `high` wastes the budget. Per-payload priority lets us segment.

**Why per-tenant title template (§1 #25)?** Operators with multiple tenants on the same device want "ACME · #engineering" not just "engineering" to distinguish.

**Why per-device locale (§1 #26)?** A VN user receiving English-titled notifications loses UX context; localisation is a small effort with significant UX gain.

---

## §3 — API contract

```sql
-- services/chat/sql/init-push-devices.sql
CREATE TABLE push_devices (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id          UUID NOT NULL,
    platform            TEXT NOT NULL CHECK (platform IN ('apns','fcm')),
    device_token        TEXT NOT NULL,
    app_version         TEXT,
    locale              TEXT NOT NULL DEFAULT 'en-US',
    registered_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_seen_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_delivered_at   TIMESTAMPTZ,
    deleted_at          TIMESTAMPTZ,
    tenant_id           UUID NOT NULL,
    UNIQUE (subject_id, device_token)
);

CREATE INDEX idx_push_active ON push_devices (subject_id, tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_push_stale  ON push_devices (last_delivered_at) WHERE deleted_at IS NULL;

ALTER TABLE push_devices ENABLE ROW LEVEL SECURITY;
CREATE POLICY push_devices_tenant_iso ON push_devices
    USING       (tenant_id = current_setting('app.tenant_id')::uuid)
    WITH CHECK  (tenant_id = current_setting('app.tenant_id')::uuid);

-- Per-recipient rate limit state.
CREATE TABLE push_rate_limit_state (
    subject_id   UUID NOT NULL,
    tenant_id    UUID NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    count        INT NOT NULL DEFAULT 0,
    PRIMARY KEY (subject_id, tenant_id)
);

-- Tenant-level push config (extends cyberos_chat_tenant_settings).
ALTER TABLE cyberos_chat_tenant_settings ADD COLUMN IF NOT EXISTS
    push_title_template TEXT NOT NULL DEFAULT '{channel}';
ALTER TABLE cyberos_chat_tenant_settings ADD COLUMN IF NOT EXISTS
    apns_environment TEXT NOT NULL DEFAULT 'production'
        CHECK (apns_environment IN ('production','sandbox'));
```

```rust
// services/chat-push/src/apns.rs
use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};

pub async fn send(token: &str, payload: &serde_json::Value) -> Result<(), PushError> {
    let jwt = make_apns_jwt(&config().apns_team_id, &config().apns_key_id, &config().apns_p8_key)?;
    let url = format!("https://api.push.apple.com/3/device/{token}");
    let resp = http_client::post(&url)
        .header("authorization", format!("bearer {jwt}"))
        .header("apns-topic", &config().apns_topic)
        .header("apns-push-type", "alert")
        .body(serde_json::to_vec(payload)?)
        .send().await?;
    match resp.status().as_u16() {
        200 => Ok(()),
        410 => Err(PushError::DeviceUnregistered),
        429 => Err(PushError::RateLimited),
        s => Err(PushError::Other(s)),
    }
}

fn make_apns_jwt(team_id: &str, key_id: &str, p8: &EncodingKey) -> anyhow::Result<String> {
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(key_id.into());
    let claims = serde_json::json!({
        "iss": team_id,
        "iat": chrono::Utc::now().timestamp(),
    });
    Ok(encode(&header, &claims, p8)?)
}
```

```rust
// services/chat-push/src/main.rs (excerpt)
pub async fn handle_push_webhook(req: WebhookReq) -> Result<(), PushError> {
    let start = Instant::now();
    let devices = registry::active_devices_for_subject(req.recipient_subject_id, req.tenant_id).await?;
    if devices.is_empty() { return Ok(()); }   // user has no devices

    for device in devices {
        let payload = build_payload(&device.platform, &req);
        let result = match device.platform.as_str() {
            "apns" => apns::send(&device.device_token, &payload).await,
            "fcm"  => fcm::send(&device.device_token, &payload).await,
            _      => continue,
        };
        match result {
            Ok(_) => {
                emit_memory_row("chat.push_delivered", json!({
                    "post_id": req.post_id, "recipient_subject_id": req.recipient_subject_id,
                    "platform": device.platform, "latency_ms": start.elapsed().as_millis() as i64,
                    "outcome": "ok", "trace_id": req.trace_id,
                })).await;
                metrics::counter!("chat_push_delivered_total",
                    "platform" => device.platform, "outcome" => "ok").increment(1);
            }
            Err(PushError::DeviceUnregistered) => {
                registry::soft_delete(&device.id).await?;
                emit_memory_row("chat.push_failed", json!({
                    "post_id": req.post_id, "platform": device.platform,
                    "outcome": "device_unregistered", "trace_id": req.trace_id,
                })).await;
            }
            Err(PushError::RateLimited) => {
                tokio::time::sleep(Duration::from_millis(500)).await;
                // retry once; if still fails, log and continue
            }
            Err(e) => {
                emit_memory_row("chat.push_failed", json!({
                    "post_id": req.post_id, "platform": device.platform,
                    "outcome": format!("{e:?}"), "trace_id": req.trace_id,
                })).await;
            }
        }
    }
    Ok(())
}

fn build_payload(
    platform: &str,
    device: &PushDevice,
    req: &WebhookReq,
    template: &str,
    badge_count: i32,
    is_silent: bool,
) -> serde_json::Value {
    let title = render_title(template, &req.tenant_name, &req.channel_name, &device.locale);
    let body  = format!("@{}", req.sender_display_name);

    match platform {
        "apns" => {
            let mut payload = json!({
                "aps": {
                    "sound":      "default",
                    "thread-id":  req.channel_id,
                    "badge":      badge_count,
                },
                "cyberos": {
                    "post_id":            req.post_id,
                    "tenant_id":          req.tenant_id,
                    "channel_id":         req.channel_id,
                    "sender_subject_id":  req.sender_subject_id,
                    "sender_display_name": req.sender_display_name,
                    "trace_id":           req.trace_id,
                }
            });
            if is_silent {
                payload["aps"]["content-available"] = json!(1);
            } else {
                payload["aps"]["alert"] = json!({"title": title, "body": body});
            }
            payload
        }
        "fcm" => {
            let mut payload = json!({
                "message": {
                    "data": {
                        "post_id":             req.post_id,
                        "tenant_id":           req.tenant_id,
                        "channel_id":          req.channel_id,
                        "sender_subject_id":   req.sender_subject_id,
                        "sender_display_name": req.sender_display_name,
                        "trace_id":            req.trace_id,
                    },
                    "android": {
                        "priority": if is_silent { "normal" } else { "high" },
                        "notification": {
                            "sound":              "default",
                            "notification_count": badge_count
                        }
                    }
                }
            });
            if !is_silent {
                payload["message"]["notification"] = json!({"title": title, "body": body});
            }
            payload
        }
        _ => json!({})
    }
}

fn render_title(template: &str, tenant: &str, channel: &str, locale: &str) -> String {
    let mut out = template.replace("{channel}", channel).replace("{tenant}", tenant);
    // Locale-aware fallback for empty channel name.
    if out.is_empty() {
        out = match locale {
            "vi-VN" => "Tin nhắn mới".to_string(),
            _       => "New message".to_string(),
        };
    }
    out
}
```

### dnd.rs — Do-Not-Disturb window check

```rust
// services/chat-push/src/dnd.rs
pub fn is_in_dnd(
    user_props: &MmNotifyProps,
    now_utc: chrono::DateTime<chrono::Utc>,
) -> bool {
    let Some(qh) = &user_props.push_status_quiet_hours else { return false; };
    let tz: chrono_tz::Tz = qh.timezone.parse().unwrap_or(chrono_tz::UTC);
    let now_local = now_utc.with_timezone(&tz).time();
    let start = chrono::NaiveTime::parse_from_str(&qh.start, "%H:%M").unwrap_or_default();
    let end   = chrono::NaiveTime::parse_from_str(&qh.end,   "%H:%M").unwrap_or_default();
    if start <= end {
        now_local >= start && now_local < end
    } else {
        // Window crosses midnight (e.g. 22:00 → 07:00).
        now_local >= start || now_local < end
    }
}

pub enum DndAction { Send, Queue, Drop }

pub fn dnd_action(user_props: &MmNotifyProps, now_utc: chrono::DateTime<chrono::Utc>) -> DndAction {
    if !is_in_dnd(user_props, now_utc) { return DndAction::Send; }
    match user_props.push_status_strategy.as_deref().unwrap_or("drop") {
        "queue" => DndAction::Queue,
        _       => DndAction::Drop,
    }
}
```

### Rate limiter — per-recipient sliding window

```rust
pub async fn within_push_rate_limit(
    pool: &sqlx::PgPool,
    tenant_id: uuid::Uuid,
    subject_id: uuid::Uuid,
) -> sqlx::Result<bool> {
    let now = chrono::Utc::now();
    let window_start = now - chrono::Duration::minutes(1);
    let count: i64 = sqlx::query_scalar(
        r#"WITH upsert AS (
              INSERT INTO push_rate_limit_state (subject_id, tenant_id, window_start, count)
              VALUES ($1, $2, $3, 1)
              ON CONFLICT (subject_id, tenant_id) DO UPDATE SET
                count = CASE
                  WHEN push_rate_limit_state.window_start < $3
                  THEN 1
                  ELSE push_rate_limit_state.count + 1
                END,
                window_start = CASE
                  WHEN push_rate_limit_state.window_start < $3
                  THEN $4
                  ELSE push_rate_limit_state.window_start
                END
              RETURNING count
           )
           SELECT count FROM upsert"#
    ).bind(subject_id).bind(tenant_id).bind(window_start).bind(now)
     .fetch_one(pool).await?;
    Ok(count <= 60)
}
```

### Suppression cache — per-(subject, channel)

```rust
pub mod suppress {
    use std::collections::HashMap;
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    use once_cell::sync::Lazy;

    static CACHE: Lazy<Mutex<HashMap<(uuid::Uuid, uuid::Uuid), Instant>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    pub fn check_and_record(subject: uuid::Uuid, channel: uuid::Uuid) -> bool {
        let mut c = CACHE.lock().unwrap();
        let now = Instant::now();
        match c.get(&(subject, channel)) {
            Some(t) if now.duration_since(*t) < Duration::from_secs(1) => false,
            _ => { c.insert((subject, channel), now); true }
        }
    }
}
```

---

## §4 — Acceptance criteria

1. **Register device** — POST /api/push/register → 201; row in push_devices.
2. **Duplicate device token (same subject) idempotent**.
3. **Different subject same token rejected** — UNIQUE on (subject, token).
4. **Push payload has NO body** — assertion: `payload.aps.alert.body` does NOT contain the message text.
5. **APNS HTTP/2 JWT auth used**.
6. **FCM v1 service-account JWT used** (no legacy key).
7. **APNS 410 → soft-delete device**.
8. **FCM UNREGISTERED → soft-delete device**.
9. **APNS 429 → retry with backoff**.
10. **Mute = none → no push sent**.
11. **Mute = mention + no mention → no push**.
12. **Mute = mention + mentioned → push**.
13. **Channel-muted by user → no push**.
14. **Fan-out p95 < 1s for 100 recipients**.
15. **memory audit chat.push_delivered on success**.
16. **memory audit chat.push_failed on error**.
17. **OTel counters increment per platform + outcome**.
18. **RLS isolates per tenant**.
19. **Dedup suppresses rapid pushes** — fixture: 5 messages to same (subject, channel) within 1s → 1 delivery + 4 suppressions; `chat_push_suppressed_total` increments by 4 (AC for §1 #13).
20. **Badge count included** — fixture: user has 7 unread; APNS payload `aps.badge=7`, FCM `notification_count=7` (AC for §1 #14).
21. **DnD window drops by default** — fixture: user with `quiet_hours={22:00,07:00,Asia/Ho_Chi_Minh}`; push at 02:00 ICT → suppressed; counter `chat_push_dnd_dropped_total` increments (AC for §1 #15).
22. **DnD queue strategy** — same fixture with `strategy=queue`; push enqueued; redelivered at end of window (AC for §1 #15).
23. **Sender subject_id in payload data** — observe `data.sender_subject_id` matches sender (AC for §1 #16).
24. **Silent push has no banner** — fixture: send with `is_silent=true`; APNS payload `aps.content-available=1` AND no `alert`; FCM `data` only, no `notification` (AC for §1 #17).
25. **APNS topic per tenant** — observe `apns-topic` header = `com.cyberskill.chat.<tenant-shortid>` (AC for §1 #18).
26. **@channel mention triggers push for mention-only user** — fixture: user with `push=mention`; message contains `@channel` → push fires (AC for §1 #19).
27. **trace_id propagates to apns-id** — observe APNS request's `apns-id` header == trace_id (AC for §1 #20).
28. **Rate limit at 60/min/recipient** — fixture: 61 pushes to one user in 60s → 1st-60th deliver; 61st suppressed (AC for §1 #21).
29. **Sandbox APNS endpoint** — set `apns_environment=sandbox`; observe URL = `api.sandbox.push.apple.com` (AC for §1 #22).
30. **last_delivered_at updated** — fixture: successful delivery → `push_devices.last_delivered_at = NOW()` (AC for §1 #23).
31. **Priority field set per payload type** — chat msg = high; silent state-sync = normal (AC for §1 #24).
32. **Tenant title template applied** — set `push_title_template='{tenant} · {channel}'`; observe APNS title = "ACME · engineering" (AC for §1 #25).
33. **Locale-aware fallback** — empty channel name + locale=vi-VN → title = "Tin nhắn mới" (AC for §1 #26).

---

## §5 — Verification

### AC #4 — payload privacy

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac4_payload_has_no_message_body() {
    let req = test_webhook_with_message_body("super secret quarterly earnings preview").await;
    let payload = build_payload("apns", &test_device(), &req, "{channel}", 1, false);
    let json_str = serde_json::to_string(&payload).unwrap();
    assert!(!json_str.contains("super secret"));
    assert!(!json_str.contains("quarterly earnings"));
    let body = payload["aps"]["alert"]["body"].as_str().unwrap();
    assert!(body.starts_with("@"));
}
```

### AC #5/#6 — APNS HTTP/2 + FCM v1 auth

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac5_apns_uses_http2_jwt() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    handle_push_webhook(test_webhook()).await.unwrap();
    let last = env.apns_mock.last_request().await;
    assert_eq!(last.http_version, "HTTP/2.0");
    assert!(last.headers.get("authorization").unwrap().starts_with("bearer "));
    let jwt = last.headers["authorization"].strip_prefix("bearer ").unwrap();
    let header = jsonwebtoken::decode_header(jwt).unwrap();
    assert_eq!(header.alg, jsonwebtoken::Algorithm::ES256);
}

#[tokio::test(flavor = "multi_thread")]
async fn ac6_fcm_uses_service_account_jwt() {
    let env = TestEnv::new().await;
    env.register_device("fcm").await;
    handle_push_webhook(test_webhook()).await.unwrap();
    let last = env.fcm_mock.last_request().await;
    let url = last.url;
    assert!(url.contains("fcm.googleapis.com/v1/projects/"));
    assert!(!url.contains("send?")); // legacy server-key path
    assert!(last.headers.get("authorization").unwrap().starts_with("Bearer "));
}
```

### AC #7 — APNS 410 soft-deletes

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac7_apns_410_soft_deletes() {
    let env = TestEnv::new().await;
    let device = env.register_device("apns").await;
    env.apns_mock.return_410().await;
    handle_push_webhook(test_webhook_for(device.subject_id)).await.unwrap();
    let row = env.db.fetch_device(device.id).await;
    assert!(row.deleted_at.is_some());
    let audit = env.memory.last_of_kind("chat.push_failed").await.unwrap();
    assert_eq!(audit["payload"]["outcome"], "device_unregistered");
}
```

### AC #10/#11/#12 — mute settings

```rust
#[rstest]
#[case("none", "no @ mention",          false)]
#[case("none", "with @alice mention",   false)]
#[case("mention", "no @ mention",       false)]
#[case("mention", "with @alice mention", true)]
#[case("all", "no @ mention",            true)]
#[tokio::test(flavor = "multi_thread")]
async fn ac10_11_12_mute_settings(
    #[case] push_mode: &str,
    #[case] message: &str,
    #[case] expect_pushed: bool,
) {
    let env = TestEnv::new().await;
    env.set_notify_push(env.subject_id(), push_mode).await;
    env.register_device("apns").await;
    env.apns_mock.clear().await;
    handle_push_webhook(test_webhook_with_body(message)).await.unwrap();
    assert_eq!(env.apns_mock.call_count().await > 0, expect_pushed);
}
```

### AC #13 — channel mute

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac13_channel_muted_skips_push() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    env.set_channel_mute(env.subject_id(), env.channel_id(), true).await;
    handle_push_webhook(test_webhook()).await.unwrap();
    assert_eq!(env.apns_mock.call_count().await, 0);
}
```

### AC #14 — fan-out latency

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac14_fanout_under_1s_for_100() {
    let env = TestEnv::new().await;
    for i in 0..100 {
        env.register_device_for(&format!("u-{}", i), "apns").await;
    }
    let start = std::time::Instant::now();
    let recipients: Vec<_> = (0..100).map(|i| format!("u-{}", i)).collect();
    for r in &recipients {
        handle_push_webhook(test_webhook_for_recipient(r)).await.unwrap();
    }
    let dur = start.elapsed();
    assert!(dur < Duration::from_secs(1), "took {:?}", dur);
}
```

### AC #19 — dedup suppression

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac19_dedup_suppresses_rapid() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    let req = test_webhook();
    for _ in 0..5 { handle_push_webhook(req.clone()).await.unwrap(); }
    assert_eq!(env.apns_mock.call_count().await, 1);
    let m = metric_value("chat_push_suppressed_total", &[]);
    assert_eq!(m, 4);
}
```

### AC #20 — badge count

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac20_badge_count_in_payload() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    env.mm_set_unread(env.subject_id(), 7).await;
    handle_push_webhook(test_webhook()).await.unwrap();
    let last = env.apns_mock.last_payload().await;
    assert_eq!(last["aps"]["badge"], 7);
}
```

### AC #21/#22 — DnD windows

```rust
#[rstest]
#[case("drop",  Some(false))]
#[case("queue", Some(true))]
#[tokio::test(flavor = "multi_thread")]
async fn ac21_22_dnd_strategies(
    #[case] strategy: &str,
    #[case] expect_queued: Option<bool>,
) {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    env.set_dnd(env.subject_id(), "22:00", "07:00", "Asia/Ho_Chi_Minh", strategy).await;
    // Force ICT 02:00 = inside DnD
    env.set_simulated_now(chrono::Utc.with_ymd_and_hms(2026, 5, 16, 19, 0, 0).unwrap()).await;
    handle_push_webhook(test_webhook()).await.unwrap();
    assert_eq!(env.apns_mock.call_count().await, 0);
    if expect_queued == Some(true) {
        assert_eq!(env.dnd_queue_size().await, 1);
    }
}
```

### AC #24 — silent push

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac24_silent_push_no_banner() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    let req = WebhookReq { is_silent: true, ..test_webhook() };
    handle_push_webhook(req).await.unwrap();
    let last = env.apns_mock.last_payload().await;
    assert_eq!(last["aps"]["content-available"], 1);
    assert!(last["aps"].get("alert").is_none());
}
```

### AC #25 — APNS topic per tenant

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac25_apns_topic_per_tenant() {
    let env = TestEnv::new_with_tenant("acme").await;
    env.register_device("apns").await;
    handle_push_webhook(test_webhook()).await.unwrap();
    let last = env.apns_mock.last_request().await;
    let topic = last.headers.get("apns-topic").unwrap();
    assert!(topic.starts_with("com.cyberskill.chat."));
    assert!(topic.ends_with(&"acme"));
}
```

### AC #26 — @channel mention

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac26_at_channel_triggers_mention_only_user() {
    let env = TestEnv::new().await;
    env.set_notify_push(env.subject_id(), "mention").await;
    env.register_device("apns").await;
    handle_push_webhook(test_webhook_with_body("Hi @channel meeting in 5")).await.unwrap();
    assert_eq!(env.apns_mock.call_count().await, 1);
}
```

### AC #27 — trace_id ≡ apns-id

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac27_trace_id_in_apns_id_header() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    let trace = "4bf92f3577b34da6a3ce929d0e0e4736";
    handle_push_webhook(WebhookReq { trace_id: trace.into(), ..test_webhook() }).await.unwrap();
    let last = env.apns_mock.last_request().await;
    assert_eq!(last.headers.get("apns-id").unwrap(), &trace);
}
```

### AC #28 — rate limit

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac28_rate_limit_60_per_min() {
    let env = TestEnv::new().await;
    env.register_device("apns").await;
    for i in 0..61 {
        let req = test_webhook_with_id(&format!("p-{}", i));
        handle_push_webhook(req).await.unwrap();
    }
    assert_eq!(env.apns_mock.call_count().await, 60);
    let m = metric_value("chat_push_rate_limited_total", &[]);
    assert_eq!(m, 1);
}
```

### AC #29 — sandbox endpoint

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac29_apns_sandbox_endpoint() {
    let env = TestEnv::new().await;
    env.set_apns_environment(env.tenant_id(), "sandbox").await;
    env.register_device("apns").await;
    handle_push_webhook(test_webhook()).await.unwrap();
    let last = env.apns_mock.last_request().await;
    assert!(last.url.starts_with("https://api.sandbox.push.apple.com/"));
}
```

### AC #30 — last_delivered_at

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac30_last_delivered_at_updated() {
    let env = TestEnv::new().await;
    let device = env.register_device("apns").await;
    let before = env.db.fetch_device(device.id).await.last_delivered_at;
    handle_push_webhook(test_webhook()).await.unwrap();
    let after = env.db.fetch_device(device.id).await.last_delivered_at;
    assert!(after.is_some());
    assert!(before.is_none() || after.unwrap() > before.unwrap());
}
```

### AC #32/#33 — title template + locale

```rust
#[tokio::test(flavor = "multi_thread")]
async fn ac32_title_template() {
    let env = TestEnv::new_with_tenant("acme").await;
    env.set_push_title_template(env.tenant_id(), "{tenant} · {channel}").await;
    env.register_device("apns").await;
    handle_push_webhook(test_webhook_with_channel("engineering")).await.unwrap();
    let last = env.apns_mock.last_payload().await;
    assert_eq!(last["aps"]["alert"]["title"], "ACME · engineering");
}

#[test]
fn ac33_locale_fallback() {
    let t = render_title("{channel}", "Acme", "", "vi-VN");
    assert_eq!(t, "Tin nhắn mới");
    let t = render_title("{channel}", "Acme", "", "en-US");
    assert_eq!(t, "New message");
}
```

### DnD pure-function table

```rust
#[rstest]
#[case("22:00", "07:00", "Asia/Ho_Chi_Minh", "2026-05-16T19:00:00Z", true)]  // 02:00 ICT
#[case("22:00", "07:00", "Asia/Ho_Chi_Minh", "2026-05-16T05:00:00Z", false)] // 12:00 ICT
#[case("12:00", "13:00", "UTC",              "2026-05-16T12:30:00Z", true)]
#[case("12:00", "13:00", "UTC",              "2026-05-16T14:00:00Z", false)]
fn dnd_window_table(
    #[case] start: &str, #[case] end: &str, #[case] tz: &str,
    #[case] now: &str, #[case] expected: bool,
) {
    let props = MmNotifyProps {
        push_status_quiet_hours: Some(QuietHours {
            start: start.into(), end: end.into(), timezone: tz.into(),
        }),
        ..Default::default()
    };
    let now: chrono::DateTime<chrono::Utc> = now.parse().unwrap();
    assert_eq!(is_in_dnd(&props, now), expected);
}
```

---

## §6 — Implementation skeleton

The Rust modules above are the skeleton. Operational wiring:

### §6.1 — Service deployment

The chat-push service runs as a Fargate task in each tenant's CHAT cluster (FR-CHAT-003). Single instance per tenant; auto-restart on crash. Stateless — push_devices is the only persistent state; rate-limit + suppression are in-memory + DB.

### §6.2 — Apple .p8 key + Google service account

APNS `.p8` key + Apple Team ID + Key ID stored in AWS Secrets Manager per tenant (per §6.4 in FR-CHAT-003). Loaded at service start; refreshed every 24h via Secrets Manager rotation hook.

FCM service account JSON stored similarly. Token refreshed via `google-cloud-auth` crate every 50 minutes (5min before 1h expiry).

### §6.3 — Plugin → service transport

MM plugin `cyberos-push-trigger` fires on `MessageHasBeenPosted` hook. For each recipient, plugin POSTs JSON to `chat-push.<tenant>.svc.cluster.local:8080/push/trigger`. Service returns 200 immediately; actual delivery is async.

### §6.4 — DnD queue retention

Queued pushes (DnD strategy=queue) persist in `push_dnd_queue` table for up to 24h. At end-of-window, batch redelivery + dedup (if user has 50 queued pushes, deliver one "you have 50 new messages" rather than 50 individual).

### §6.5 — APNS topic registration

The APNS topic must be registered with Apple Developer Console as a Push Notification capability. Per-tenant suffix requires either: (a) one shared App ID with topic mask (preferred); (b) per-tenant App IDs (slice-4+ for cross-tenant device support).

### §6.6 — Badge count source

Badge count = MM API `/api/v4/users/me/unread` for the recipient. Cached 30s per user to avoid hammering MM on push burst.

### §6.7 — Failure routing matrix

| Failure | Audit | Operator action |
|---|---|---|
| APNS 410 | chat.push_failed (device_unregistered) | None (auto soft-delete) |
| APNS 429 | chat.push_failed (rate_limited) + retry | None |
| FCM UNREGISTERED | chat.push_failed (device_unregistered) | None (auto soft-delete) |
| FCM INVALID_ARGUMENT | chat.push_failed (invalid_argument) | Operator investigates payload |
| .p8 key invalid | SEV-1 + service refuses to start | Operator rotates |
| Service account expired | SEV-1 + service refuses | Operator rotates |
| MM unread API fails | badge=0 fallback | Investigate MM |
| Rate limit hit | chat.push_rate_limited | None (expected) |
| DnD active | chat.push_dnd_dropped OR queued | None (user setting) |

### §6.8 — Cleanup of stale devices

Nightly: `UPDATE push_devices SET ... WHERE last_delivered_at < NOW() - INTERVAL '60 days'` flags but does not delete. Operators can hard-delete via `cyberos-chat push prune --tenant <id>` (slice-4+ CLI).

### §6.9 — Operator CLI surface

```text
$ cyberos-chat push list --tenant <id>
DEVICE_ID         PLATFORM  SUBJECT       LAST_SEEN  STATUS
abc-123-...       apns      alice (u-...) 2m ago     active
def-456-...       fcm       bob (u-...)   3d ago     active

$ cyberos-chat push test --tenant <id> --device <device-id>
✓ Sent test push to device abc-123-...

$ cyberos-chat push prune --tenant <id> --older-than 60d
Found 5 stale devices; --confirm to delete.
```

---

## §7 — Dependencies

- **FR-CHAT-003** — push service runs in same Fargate cluster.
- **FR-CHAT-005** — bridge sees push delivery audit rows.
- **FR-AUTH-004** — auth for /api/push/register endpoint.

---

## §8 — Example payloads

### `chat.push_delivered`

```json
{
  "kind": "chat.push_delivered",
  "ts_ns": 1747407137000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "payload": {
    "post_id":              "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "recipient_subject_id": "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "platform":             "apns",
    "device_id":            "01HVQX8ZG2K3R4TVA7P3WV5X8Q",
    "latency_ms":           142,
    "outcome":              "ok",
    "badge_count":          7,
    "is_silent":            false
  }
}
```

### `chat.push_failed` — device unregistered

```json
{
  "kind": "chat.push_failed",
  "ts_ns": 1747407138000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id":  "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "platform": "apns",
    "device_id": "01HVQX8ZG2K3R4TVA7P3WV5X8R",
    "outcome":  "device_unregistered",
    "soft_deleted": true
  }
}
```

### APNS payload (banner)

```json
{
  "aps": {
    "alert":     { "title": "ACME · engineering", "body": "@alice" },
    "sound":     "default",
    "thread-id": "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "badge":     7
  },
  "cyberos": {
    "post_id":             "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "tenant_id":           "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "channel_id":          "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "sender_subject_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8J",
    "sender_display_name": "alice",
    "trace_id":            "4bf92f3577b34da6a3ce929d0e0e4736"
  }
}
```

### APNS payload (silent)

```json
{
  "aps": {
    "content-available": 1,
    "sound":             "default",
    "thread-id":         "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "badge":             7
  },
  "cyberos": {
    "post_id":             "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "tenant_id":           "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
    "channel_id":          "01HVQX8ZG2K3R4TVA7P3WV5X8M",
    "trace_id":            "4bf92f3577b34da6a3ce929d0e0e4736"
  }
}
```

### FCM v1 payload (banner)

```json
{
  "message": {
    "notification": {
      "title": "ACME · engineering",
      "body":  "@alice"
    },
    "data": {
      "post_id":             "01HVQX8ZG2K3R4TVA7P3WV5X8N",
      "tenant_id":           "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
      "channel_id":          "01HVQX8ZG2K3R4TVA7P3WV5X8M",
      "sender_subject_id":   "01HVQX8ZG2K3R4TVA7P3WV5X8J",
      "sender_display_name": "alice",
      "trace_id":            "4bf92f3577b34da6a3ce929d0e0e4736"
    },
    "android": {
      "priority": "high",
      "notification": {
        "sound":              "default",
        "notification_count": 7
      }
    }
  }
}
```

### Register/deregister request

```http
POST /api/push/register HTTP/1.1
Authorization: Bearer <jwt>
Content-Type: application/json

{
  "platform":     "apns",
  "device_token": "abc123def456...",
  "app_version":  "2.1.4",
  "locale":       "vi-VN"
}

→ 201 Created
{"device_id": "01HVQX8ZG2K3R4TVA7P3WV5X8Q"}
```

### `chat.push_dnd_dropped`

```json
{
  "kind": "chat.push_dnd_dropped",
  "ts_ns": 1747449937000000000,
  "tenant_id": "1f8c4d6e-1e0c-4d7a-9d2a-ad8c4b6bbf21",
  "payload": {
    "post_id":              "01HVQX8ZG2K3R4TVA7P3WV5X8N",
    "recipient_subject_id": "01HVQX8ZG2K3R4TVA7P3WV5X8K",
    "user_local_time":      "02:30:00",
    "dnd_window":           "22:00-07:00 Asia/Ho_Chi_Minh",
    "strategy":             "drop"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Web push (browser notifications) — slice 4+.
- Rich notifications (image previews) — slice 4+ (carries privacy risk).
- Notification action buttons (reply inline) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| APNS .p8 key invalid | JWT sign Err at startup | SEV-1; service refuses to start | Operator rotates key |
| APNS .p8 key revoked mid-flight | APNS 403 | SEV-1; chat.push_failed; halt deliveries until rotation | Operator |
| APNS .p8 key Team ID mismatch | APNS 400 | SEV-1 | Operator fixes config |
| FCM service account expired | JWT sign returns Err | SEV-1; halts deliveries | Operator rotates |
| FCM service account JSON malformed | startup parse Err | SEV-1; refuses to start | Operator |
| FCM project ID mismatch | 404 from FCM | SEV-2; chat.push_failed | Operator |
| Device token expired (410) | soft-delete + audit | next push to this device skipped | None (automatic) |
| Device token UNREGISTERED (FCM) | soft-delete | same | None |
| Device token INVALID_REGISTRATION (FCM) | log; do NOT delete (may be transient) | retry once | None |
| Rate limit (APNS 429) | retry with backoff | None | None |
| Rate limit (FCM RESOURCE_EXHAUSTED) | retry with backoff | None | None |
| Network partition to APNS | HTTPS Err | SEV-2; queue + retry; circuit-break after 60s | Auto recovers |
| Network partition to FCM | HTTPS Err | SEV-2; queue + retry | Auto recovers |
| Plugin emits push for muted user | mute filter at plugin | None | None |
| 1000 recipients in channel | parallel fan-out | < 1s typical; SEV-3 if > 3s | None |
| Same device registered twice (re-install) | UPSERT updates last_seen | None | None |
| User logs out | client calls deregister | None | None |
| Client deregister request fails (token still listed) | next push 410 → soft-delete | None | None |
| memory audit emit fails | push still delivered; audit lost | SEV-2 logged | Operator restores |
| Concurrent push for same recipient (rapid messages) | suppress cache 1s window | only first delivers; suppressed counter | None |
| Push payload exceeds platform limit (4KB APNS / 4KB FCM) | truncate title to fit | SEV-3 warning | None |
| Tenant_id leak via payload | data field only (not visible to user) | safe | None |
| User changes locale mid-session | next register updates locale | next push uses new locale | None |
| User changes timezone mid-DnD | DnD honours latest props | None | None |
| DnD queue exceeds 24h retention | drop oldest; SEV-3 warning | None | None |
| DnD queue size > 1000 per user | drop oldest; warn | None | None |
| Badge fetch from MM API times out | badge=0 fallback | minor UX degradation | Investigate MM |
| Badge count > 999 | clamp to "999+" in title | None | None |
| Mention detection regex misses (e.g. @user-name with hyphen) | falls back to "no mention" | mention-only user misses | Operator extends regex |
| Sender display_name contains special chars (emoji, RTL) | passed through | client renders correctly | None |
| Channel name empty (rare) | locale-aware fallback ("New message" / "Tin nhắn mới") | None | None |
| Tenant settings query fails | fall back to default template | None | None |
| Suppression cache memory pressure (millions of entries) | LRU eviction | older entries evicted | None |
| Rate limit DB query fails | fail-OPEN (allow push) + log | None | None |
| Silent push delivered as banner (client misconfigured) | client-side issue | not detectable server-side | None |
| Notification displayed but client app crashes on tap | client-side | not detectable server-side | None |
| APNS topic mismatch (e.g. operator changed config) | APNS 400 | SEV-1; chat.push_failed | Operator fixes |
| APNS sandbox vs production confusion | token mismatch (silent failure) | no error from APNS; push silently dropped | Operator inspects logs + retests |
| FCM Android-only token used on iOS | FCM 400 | SEV-3; soft-delete | None |
| FCM iOS token used on Android | platform field disagrees | filter at register | None |
| Plugin can't reach push service (network) | retry 3× | post still saved; push skipped | Investigate |
| Push service can't reach plugin (no-op) | n/a (one-way) | None | None |
| MM user deletion while push in flight | recipient_subject_id orphan | push delivered to last-known device | Operator |
| MM channel deletion while push in flight | reference orphan in payload data | client handles 404 on tap | None |
| Tenant deletion while push in flight | RLS query returns empty | no push sent | None |
| .p8 key rotation in flight | brief auth failures | retry succeeds with new key | None |
| Locale not in supported list (e.g. zz-ZZ) | fallback to en-US | None | None |
| Push title template malformed (placeholder typo) | passed through; literal {channel} appears in title | SEV-3 warning | Operator fixes |
| User has many devices (10+) | fan-out parallel | None | None |
| Device count > 100 per user | rejected at register | client handles error | None |
| Same notification sent to wrong tenant device (RLS bypass) | RLS prevents | safe | None |
| Apple Apple/Google API rate-limit on shared topic | back off + retry | brief degradation | None |

---

## §11 — Implementation notes

- APNS HTTP/2 client uses `h2` crate via `reqwest`; keepalive connection pool across many devices reduces handshake cost ~50ms per push.
- FCM v1 uses service-account JSON via `google-cloud-auth` crate; alternatives (raw OAuth flow) add boilerplate without gain.
- Device dedup at fan-out: in-process map (subject_id, channel_id) → last push timestamp; suppress if < 1s. Memory bounded by LRU eviction at 10k entries (~3MB).
- Push topic per tenant (APNS BundleID + tenant suffix) chosen for: (a) per-tenant audit trail in Apple Developer Console; (b) ability to suspend one tenant without affecting others; (c) easier compliance attribution.
- Plugin emits one webhook per recipient (NOT one webhook with N recipients) because: (a) per-recipient retry independence; (b) per-recipient rate limit easier to enforce; (c) clearer per-recipient audit trail.
- DnD queue uses Postgres table (`push_dnd_queue`) rather than Redis because deliveries are infrequent (a few per user per quiet period); Postgres is simpler operationally.
- We chose APNS HTTP/2 over MQTT (the older APNS protocol) because: (a) Apple deprecated MQTT in 2024; (b) HTTP/2 multiplexing means one connection serves all devices per pod; (c) better error semantics (status codes vs binary frames).
- FCM v1 over legacy server-key because: (a) Google deprecated server-key 2024 (phased shutdown); (b) v1 supports per-message options; (c) v1 errors are structured JSON.
- Rate limit 60/min/recipient calibrated against Apple's published topic-wide rate ceilings; our 60/min ensures we never trip Apple's automatic suspension.
- Suppression 1s window calibrated against typical typing patterns: human messages arrive at 1-3 per minute peak; 1s suppression collapses the typing burst without dropping legitimate consecutive messages.
- Locale-aware title fallback handles only common locales (en-US, vi-VN, others fall to en-US). Adding more is a slice-4+ enhancement; for MVP, the two primary user locales suffice.
- The `apns-id` header carries our trace_id; APNS echoes it in delivery receipts (the `apns-id` response header), enabling end-to-end correlation in APNS logs.
- Badge count fetched from MM API per push; caching at 30s avoids hammering MM during bursts. We considered caching badge counts in chat-push state but the consistency risk (stale badge) outweighs the API-call savings.
- Silent push (`content-available: 1`) is used sparingly: only for badge updates during background app refresh. Sending too many silent pushes triggers iOS throttling.
- We don't currently support web push (PWA) — slice 4+ adds VAPID + Web Push API. PWA users get pushes via the native app on the same device.
- The `last_delivered_at` field is updated on EVERY successful delivery, which is a hot write. We use UNLOGGED Postgres updates would be faster but risk data loss on crash; standard logged updates are the safer default.
- The push service does NOT modify post content; it's read-only against `posts` (only for context). This simplifies failure modes (no risk of corrupting messages).
- We chose a single FCM project (shared across tenants) for MVP because per-tenant projects require operational overhead (Google quota per project, billing splits, etc.). Slice-4+ may migrate to per-tenant for compliance.
- The `apns_environment` config (production vs sandbox) is per-tenant rather than per-device. Mixing modes per device caused production bugs in alpha testing; per-tenant is simpler.
- Push payload size limit is 4KB on both APNS + FCM. Our payload (title + body + ~10 data fields) is ~600 bytes, well under. Operators adding more `data` fields should test size before deploying.
- Why not use a third-party push provider (OneSignal, Pusher, etc.): (a) extra trust boundary; (b) PII concerns (we'd be sending sender names to a third party); (c) cost at scale; (d) MM ecosystem doesn't require it. Direct integration is straightforward.
- The `cyberos` namespace in the APNS payload data is for our custom fields; standard `aps` fields use Apple's namespace. Separating prevents collision with Apple's reserved keys.
- The mention regex for `@user` includes hyphen + underscore + digit (Mattermost username spec): `@[a-zA-Z0-9._-]+`. `@channel` / `@here` / `@all` are special-cased.
- DnD queue redelivery is collapsed: 50 queued pushes for one user → 1 delivery `"You have 50 new messages while you were away"` rather than 50 pushes at 07:00.
- Per-tenant push title template defaults to `{channel}` for simplicity; operators of multi-tenant device users set `{tenant} · {channel}`.
- The plugin filter for mute (channel-level, user-level) is at MM API; the push service trusts the plugin's filter. This avoids double-querying MM preferences from the push service.
- The chat-push service is stateless except for in-process suppression cache and Redis rate-limit state; restart is safe (no in-flight pushes lost, only in-flight ack-pending).
- For privacy auditability, we considered emitting memory row containing the payload bytes; rejected as too verbose. Operators can re-derive the payload from `(post_id, recipient_subject_id)` + the build_payload function.
- The `chat_push_registered_devices` gauge updates on register/deregister + nightly recount; provides operator visibility into device-base growth.

---

*End of FR-CHAT-011.*
