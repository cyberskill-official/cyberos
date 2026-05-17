---
id: FR-PORTAL-007
title: "PORTAL PWA installable — mobile-first Progressive Web App with offline-capable view cache + push notifications + per-tenant manifest"
module: PORTAL
priority: SHOULD
status: draft
verify: T
phase: P4
milestone: P4 · slice 2
slice: 2
owner: Stephen Cheng (CPO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-PORTAL-001, FR-PORTAL-002, FR-PORTAL-003, FR-PORTAL-005, FR-PORTAL-006, FR-EMAIL-001, FR-AI-003, FR-BRAIN-111]
depends_on: [FR-PORTAL-001]
blocks: []

source_pages:
  - website/docs/modules/portal.html#pwa
  - https://www.w3.org/TR/appmanifest/
  - https://w3c.github.io/ServiceWorker/
  - https://w3c.github.io/push-api/

source_decisions:
  - DEC-1260 2026-05-17 — PWA per W3C App Manifest spec; mobile-first responsive design; installable on iOS/Android/desktop browsers
  - DEC-1261 2026-05-17 — Per-tenant manifest at `/manifest.json` dynamically composed from FR-PORTAL-002 brand pack (name, icons, theme_color)
  - DEC-1262 2026-05-17 — Service worker handles offline cache + push notifications; cached view-list responses for last-visited Engagement (read-only)
  - DEC-1263 2026-05-17 — Web Push notifications via VAPID per RFC 8292; per-user subscription stored encrypted; FR-PORTAL-006 status-change events deliver as push
  - DEC-1264 2026-05-17 — Offline mode: read-only access to cached list views (24h cache); writes (workflow submit, Genie query) queued + retried on reconnect
  - DEC-1265 2026-05-17 — Cache strategy per resource: views = network-first with 24h cache fallback; static assets = cache-first 30d; manifest = network-first 1h
  - DEC-1266 2026-05-17 — Closed enum `pwa_notification_kind` = {workflow_status_changed, dsar_ready, genie_mention, channel_mention, billing_alert}; CI cardinality asserts 5
  - DEC-1267 2026-05-17 — User notification preferences stored in `portal_pwa_subscriptions` table; default all-enabled; user can opt out per kind
  - DEC-1268 2026-05-17 — Rate limit: 100 push notifications per user per day (prevent notification spam)
  - DEC-1269 2026-05-17 — Service worker version bumped on every deploy; cache invalidation triggered by version mismatch
  - DEC-1270 2026-05-17 — BRAIN audit kinds: portal.pwa_subscription_created, portal.pwa_subscription_revoked, portal.pwa_notification_sent, portal.pwa_notification_delivery_failed
  - DEC-1271 2026-05-17 — Mobile-first responsive design via Tailwind breakpoints; minimum supported viewport 320×568 (iPhone SE); design targets 360×640 (Android baseline)

build_envelope:
  language: rust 1.81 + typescript 5.5
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0020_portal_pwa_subscriptions.sql
    - services/portal/migrations/0021_portal_pwa_notifications_log.sql
    - services/portal/src/pwa/mod.rs
    - services/portal/src/pwa/manifest_gen.rs
    - services/portal/src/pwa/service_worker_gen.rs
    - services/portal/src/pwa/push_subscribe.rs
    - services/portal/src/pwa/push_dispatcher.rs
    - services/portal/src/pwa/vapid.rs
    - services/portal/src/audit/pwa_events.rs
    - services/portal/src/handlers/pwa_routes.rs
    - services/portal/web/pwa/service-worker.ts
    - services/portal/web/pwa/install-prompt.ts
    - services/portal/web/pwa/notification-handler.ts
    - services/portal/tests/pwa_manifest_per_tenant_test.rs
    - services/portal/tests/pwa_service_worker_test.rs
    - services/portal/tests/pwa_push_subscribe_test.rs
    - services/portal/tests/pwa_push_dispatch_test.rs
    - services/portal/tests/pwa_notification_pref_opt_out_test.rs
    - services/portal/tests/pwa_offline_cache_test.rs
    - services/portal/tests/pwa_rate_limit_test.rs
    - services/portal/tests/pwa_notification_kind_enum_cardinality_test.rs
    - services/portal/tests/pwa_audit_emission_test.rs

  modified_files:
    - services/portal/src/lib.rs
    - services/portal/Cargo.toml                                       # +web-push crate

  allowed_tools:
    - file_read: services/portal/**
    - file_write: services/portal/{src,tests,migrations,web}/**
    - bash: cd services/portal && cargo test pwa

  disallowed_tools:
    - serve manifest.json without per-tenant brand application (per DEC-1261)
    - exceed 100 push/day/user (per DEC-1268)
    - cache write operations offline (per DEC-1264 — read-only)
    - send notifications for opted-out kinds (per DEC-1267)

effort_hours: 6
sub_tasks:
  - "0.3h: 0020_portal_pwa_subscriptions.sql + 0021_portal_pwa_notifications_log.sql"
  - "0.3h: pwa/mod.rs + closed enum"
  - "0.5h: pwa/manifest_gen.rs (per-tenant manifest synthesis)"
  - "0.6h: pwa/service_worker_gen.rs (versioned + cache strategy injection)"
  - "0.4h: pwa/push_subscribe.rs"
  - "0.6h: pwa/push_dispatcher.rs (event → push fan-out)"
  - "0.4h: pwa/vapid.rs (VAPID JWT signing)"
  - "0.3h: audit/pwa_events.rs (4 builders)"
  - "0.4h: handlers/pwa_routes.rs"
  - "0.6h: web/pwa/service-worker.ts"
  - "0.3h: web/pwa/install-prompt.ts + notification-handler.ts"
  - "1.0h: tests — 9 test files"
  - "0.3h: wire-up"

risk_if_skipped: "Without PWA, mobile clients use the standard web view — no install prompt, no offline access, no push notifications. Modern client expectation (especially in VN where mobile-first is dominant) is 'install this app from browser → home screen icon → push alerts'. Without DEC-1262 service worker, lost connectivity = blank portal. Without DEC-1263 push, FR-PORTAL-006 workflow status changes only reach users via email (latency 30s-5min vs push <1s). Without DEC-1261 per-tenant manifest, every installed PWA shows 'CyberOS' as the app name — wrong for white-label. The 6h effort delivers the mobile-first experience that meets 2026 client UX baseline."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship Progressive Web App support at `services/portal/src/pwa/` with per-tenant manifest, service worker for offline-cached views + push notifications, VAPID-signed Web Push, user notification preferences, mobile-first responsive UI, and 4 BRAIN audit kinds.

1. **MUST** define closed `pwa_notification_kind` enum: `('workflow_status_changed','dsar_ready','genie_mention','channel_mention','billing_alert')` per DEC-1266. Cardinality asserts 5.

2. **MUST** define `portal_pwa_subscriptions` at migration `0020`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL, endpoint_url_kms_blob BYTEA NOT NULL, p256dh_key_kms_blob BYTEA NOT NULL, auth_key_kms_blob BYTEA NOT NULL, user_agent TEXT, enabled_kinds JSONB NOT NULL DEFAULT '["workflow_status_changed","dsar_ready","genie_mention","channel_mention","billing_alert"]'::jsonb, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), last_used_at TIMESTAMPTZ, revoked_at TIMESTAMPTZ)`. Partial unique on `(subject_id, endpoint_url_sha256_for_indexing)` to dedupe per device.

3. **MUST** define `portal_pwa_notifications_log` at migration `0021`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, subject_id UUID NOT NULL, kind pwa_notification_kind NOT NULL, payload_sha256 CHAR(64) NOT NULL, delivery_status TEXT NOT NULL CHECK (delivery_status IN ('sent','failed','rate_limited')), failure_reason TEXT, sent_at TIMESTAMPTZ NOT NULL DEFAULT now(), trace_id CHAR(32))`. Append-only.

4. **MUST** enforce RLS scoped to subject_id self-access.

5. **MUST** generate per-tenant manifest at `GET /manifest.json` per DEC-1261 with caller's tenant brand applied:
   ```json
   { "name": "<tenant_display_name>", "short_name": "<tenant_short_name>",
     "icons": [{ "src": "/cdn/brand/<slug>/favicon.png?v=...", "sizes": "192x192", "type": "image/png" }],
     "theme_color": "<brand.primary>", "background_color": "<brand.background>",
     "display": "standalone", "start_url": "/" }
   ```
   Manifest cached 1h client-side; invalidated by brand-pack update via FR-PORTAL-002 NATS event.

6. **MUST** generate versioned service worker at `GET /service-worker.js` per DEC-1262 + DEC-1265 + DEC-1269. Worker code:
   - Cache name `cyberos-portal-v{version}` (version = deploy timestamp).
   - Network-first strategy for `/v1/portal/views/*` with 24h cache fallback.
   - Cache-first for static assets (`/static/*`) with 30d TTL.
   - On activation: deletes stale cache versions.
   - Push event handler: shows notification with brand-themed icon + body from payload.

7. **MUST** expose `POST /v1/portal/pwa/subscribe` body `{ endpoint, p256dh, auth, user_agent }`. Handler:
   - Validates JWT.
   - KMS-encrypts endpoint + keys.
   - UPSERTs subscription row.
   - Emit `portal.pwa_subscription_created`.

8. **MUST** expose `POST /v1/portal/pwa/unsubscribe` body `{ subscription_id }` → revokes subscription. Emit `portal.pwa_subscription_revoked`.

9. **MUST** expose `PATCH /v1/portal/pwa/preferences` body `{ enabled_kinds: [...] }` → updates `enabled_kinds` JSONB.

10. **MUST** dispatch push notifications per DEC-1263 via `pwa/push_dispatcher.rs`. Consumer of NATS events from FR-PORTAL-006 (workflow_status_changed), FR-PORTAL-008 (dsar_ready), FR-PORTAL-005 (genie_mention), FR-CHAT-005 (channel_mention), FR-INV-001 (billing_alert). For each event:
   - Lookup subscriptions for target subject_id WHERE kind IN enabled_kinds AND revoked_at IS NULL.
   - Sign VAPID JWT per RFC 8292.
   - POST to push endpoint with encrypted payload (Web Push encryption per RFC 8291).
   - Log delivery_status; emit `portal.pwa_notification_sent` or `portal.pwa_notification_delivery_failed`.

11. **MUST** rate-limit at 100 push/day/subject per DEC-1268. Excess → log `delivery_status='rate_limited'`; user not notified (avoid notification spam).

12. **MUST** sign push via VAPID per DEC-1263. Per-deployment VAPID keypair generated at deploy time; private key in KMS; public key embedded in service worker.

13. **MUST** mobile-first responsive per DEC-1271. Tailwind breakpoints: `sm: 640px, md: 768px, lg: 1024px, xl: 1280px`. Minimum viewport 320×568 supported.

14. **MUST** apply per-tenant brand to install-prompt + notification icon + push notification body (theme_color from FR-PORTAL-002).

15. **MUST** version service worker on every deploy per DEC-1269. Cache key includes deploy timestamp; old caches deleted on activation.

16. **MUST** emit 4 BRAIN audit kinds per DEC-1270:
   - `portal.pwa_subscription_created` (sev-3)
   - `portal.pwa_subscription_revoked` (sev-3)
   - `portal.pwa_notification_sent` (sev-3 — high-volume, sampled 1%)
   - `portal.pwa_notification_delivery_failed` (sev-2 — security signal could indicate compromised endpoint)

17. **MUST** PII-scrub: payload_sha256 only in audit chain; raw payload in delivery_log only.

18. **MUST** thread trace_id from source event through dispatcher to delivery log.

19. **MUST** cache only READ operations offline per DEC-1264. Write operations (POST/PUT/DELETE) queue in IndexedDB + retry on reconnect; service worker queues but never silently writes.

20. **MUST NOT** include sensitive data in push payload (push provider servers see payload until decrypted by browser). Payload = generic notification text + `workflow_id` opaque reference; user must open PWA to see content.

---

## §2 — Why this design (rationale for humans)

**Why per-tenant manifest (§1 #5, DEC-1261)?** White-label = each tenant's portal looks like THEIR product. Installed app name = "Acme Client Portal" not "CyberOS". Manifest synthesis at request time matches that expectation.

**Why network-first for views with 24h fallback (§1 #6, DEC-1265)?** Views show live data; stale-cached display = confusing. Network-first = always-fresh when online; fallback ensures basic browsing offline.

**Why VAPID per RFC 8292 (§1 #12, DEC-1263)?** Web Push standard; without VAPID, browsers reject the subscription. Per-deployment key = single trust anchor.

**Why rate-limit push at 100/day (§1 #11, DEC-1268)?** Notification fatigue is real — beyond ~10/day users disable. 100 is generous upper bound; sustained high volume = bug or attack.

**Why generic push payload (§1 #20)?** Push providers (Apple, Google, Mozilla, Microsoft) see payload before it reaches the browser. Sensitive content in payload = third-party data exposure. Generic + ref-to-PWA pattern is industry standard.

**Why mobile-first responsive design (§1 #13, DEC-1271)?** Vietnamese market is mobile-first (>70% smartphone-only users); other client markets trending same way. Desktop is the secondary experience.

---

## §3 — API contract

```sql
-- 0020_portal_pwa_subscriptions.sql
CREATE TYPE pwa_notification_kind AS ENUM ('workflow_status_changed','dsar_ready','genie_mention','channel_mention','billing_alert');

CREATE TABLE portal_pwa_subscriptions (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  endpoint_url_kms_blob BYTEA NOT NULL,
  endpoint_url_sha256 CHAR(64) NOT NULL,    -- for dedup index
  p256dh_key_kms_blob BYTEA NOT NULL,
  auth_key_kms_blob BYTEA NOT NULL,
  kms_key_id TEXT NOT NULL,
  user_agent TEXT,
  enabled_kinds JSONB NOT NULL DEFAULT '["workflow_status_changed","dsar_ready","genie_mention","channel_mention","billing_alert"]'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  last_used_at TIMESTAMPTZ,
  revoked_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX uniq_subscription_per_device
  ON portal_pwa_subscriptions(subject_id, endpoint_url_sha256)
  WHERE revoked_at IS NULL;
ALTER TABLE portal_pwa_subscriptions ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_pwa_subscriptions_rls ON portal_pwa_subscriptions
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND subject_id = current_setting('auth.subject_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND subject_id = current_setting('auth.subject_id')::uuid);
REVOKE DELETE ON portal_pwa_subscriptions FROM cyberos_app;
GRANT UPDATE (enabled_kinds, last_used_at, revoked_at) ON portal_pwa_subscriptions TO cyberos_app;

-- 0021_portal_pwa_notifications_log.sql
CREATE TABLE portal_pwa_notifications_log (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  subject_id UUID NOT NULL,
  subscription_id BIGINT REFERENCES portal_pwa_subscriptions(id),
  kind pwa_notification_kind NOT NULL,
  payload_sha256 CHAR(64) NOT NULL,
  delivery_status TEXT NOT NULL CHECK (delivery_status IN ('sent','failed','rate_limited')),
  failure_reason TEXT,
  sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
CREATE INDEX idx_pwa_log_subject ON portal_pwa_notifications_log(subject_id, sent_at DESC);
ALTER TABLE portal_pwa_notifications_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_pwa_notifications_log_rls ON portal_pwa_notifications_log
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_pwa_notifications_log FROM cyberos_app;
```

Endpoints:
```text
GET    /manifest.json                                 (public, per-caller-tenant)
GET    /service-worker.js                             (public, versioned)
POST   /v1/portal/pwa/subscribe                       (caller)
POST   /v1/portal/pwa/unsubscribe                     (caller)
PATCH  /v1/portal/pwa/preferences                     (caller — toggle kinds)
GET    /v1/portal/pwa/subscriptions                   (caller — own subscriptions)
```

---

## §4 — Acceptance criteria

1. **pwa_notification_kind cardinality** — 5 values.
2. **Per-tenant manifest** — GET /manifest.json returns tenant's brand `name`, `theme_color`, icons.
3. **Service worker versioned** — current version differs from prior deploy's; old cache deleted on activation.
4. **Subscribe encrypts endpoint** — endpoint/keys never in plaintext in DB.
5. **Push delivered** — fixture event → push dispatched → notification log row with `sent`.
6. **Opt-out respected** — user disables `genie_mention`; subsequent genie mentions → no push, no log row.
7. **Rate limit 100/day** — 101st push → `rate_limited` log row + no actual push.
8. **VAPID signed** — outgoing push request has valid VAPID JWT header.
9. **Offline cache 24h** — disconnect network; cached views still loadable for 24h.
10. **Static asset 30d cache** — repeated load shows cache-hit until 30d expiry.
11. **Manifest cache 1h** — manifest re-fetched after 1h.
12. **Generic payload** — push payload contains no sensitive data; only ref + generic text.
13. **Mobile-first viewport 320px** — UI renders at 320×568 without horizontal scroll.
14. **Brand pack applied** — install icon + theme color match FR-PORTAL-002 active pack.
15. **NATS event → push** — dispatcher subscribes to events; matching subscription gets push.
16. **Subscription dedup per device** — same endpoint twice → second is no-op.
17. **Failed delivery logged** — push provider 410 (gone) → marks subscription revoked + log failure.
18. **4 BRAIN audit kinds emitted** — subscribe + revoke + sent + failed all emit.
19. **Trace_id threaded** — source event → dispatcher → push → log all share trace_id.
20. **Cache invalidation on deploy** — new SW version triggers old-cache delete on activation.

---

## §5 — Verification

```rust
// 5.1 manifest per-tenant
#[tokio::test]
async fn manifest_returns_tenant_brand() {
    let ctx = TestContext::with_branded_tenant("acme", "#1A73E8").await;
    let r = ctx.get_authenticated("/manifest.json").await;
    let body: serde_json::Value = r.json().await.unwrap();
    assert!(body["name"].as_str().unwrap().contains("Acme"));
    assert_eq!(body["theme_color"], "#1A73E8");
}

// 5.2 push delivered
#[tokio::test]
async fn workflow_status_change_triggers_push() {
    let ctx = TestContext::with_pwa_subscription("alice").await;
    ctx.publish_nats_event("workflow_status_changed", json!({"subject_id": ctx.alice_id, ...})).await;
    tokio::time::sleep(Duration::from_millis(500)).await;
    let r = sqlx::query_as::<_, (String,)>("SELECT delivery_status FROM portal_pwa_notifications_log WHERE subject_id=$1")
        .bind(ctx.alice_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(r.0, "sent");
}

// 5.3 rate limit
// 5.4 opt-out
// 5.5 VAPID signature
// 5.6 offline cache
// 5.7 generic payload (no sensitive data)
// 5.8 mobile viewport
// 5.9 dedup
// 5.10 audit emission
```

---

## §7 — Dependencies

**Upstream:** FR-PORTAL-001 (views to cache offline).
**Cross-module:** FR-PORTAL-002 (brand in manifest + notifications), FR-PORTAL-003 (auth for subscribe), FR-PORTAL-005 (genie_mention event source), FR-PORTAL-006 (workflow_status_changed event), FR-PORTAL-008 (dsar_ready event), FR-CHAT-005 (channel_mention), FR-INV-001 (billing_alert), FR-EMAIL-001 (fallback channel), FR-AI-003, FR-BRAIN-111.
**Downstream:** None.

---

## §8 — Example payload

`portal.pwa_notification_sent`:
```json
{
  "kind": "portal.pwa_notification_sent",
  "severity": 3,
  "tenant_id": "8a2f...",
  "trace_id": "...",
  "occurred_at": "2026-05-17T...",
  "payload": {
    "subject_id_hash16": "f8a1...",
    "kind": "workflow_status_changed",
    "subscription_id": 42,
    "payload_sha256": "9c4e..."
  }
}
```

Manifest:
```json
{
  "name": "Acme Client Portal",
  "short_name": "Acme",
  "icons": [
    { "src": "/cdn/brand/acme/favicon.png?v=f8a1", "sizes": "192x192", "type": "image/png" },
    { "src": "/cdn/brand/acme/splash.png?v=f8a1", "sizes": "512x512", "type": "image/png" }
  ],
  "theme_color": "#1A73E8",
  "background_color": "#FFFFFF",
  "display": "standalone",
  "start_url": "/",
  "scope": "/"
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Notification scheduling (quiet hours, time-zone aware) — slice 3.
- **Deferred:** Rich notifications with action buttons — slice 3.
- **Deferred:** Background sync for write queueing — slice 3.
- **Deferred:** App badges (unread count) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Push endpoint returns 410 Gone | HTTP status | Mark subscription revoked + audit failed | Subscription pruned |
| Push endpoint returns 429 | HTTP status | Retry after Retry-After; if persistent, mark failed | Inherent |
| VAPID JWT signing fails | KMS error | Sev-2 alert; notification dropped | KMS recovery |
| Manifest brand pack lookup fails | KMS or DB | Default brand applied + sev-3 log | Inherent fallback |
| Service worker version mismatch race | install/activate | Old/new SW coexist briefly; client refresh | Inherent SW lifecycle |
| Offline cache > quota | browser eviction | Browser evicts LRU; sev-3 log | Inherent |
| Push payload > 4 KiB | size check | 413 from push provider; truncate to generic | Inherent |
| Subject revoked mid-push-dispatch | subscription RLS | Push not sent; log row | Inherent |
| Cross-tenant push dispatch attempt | RLS | 0 subscriptions matched; no push | Inherent |
| Notification kind disabled | enabled_kinds check | No push; no log row (silent opt-out) | Inherent |
| Rate limit hit | counter | `rate_limited` log; no push delivered | User receives next-day |
| Browser blocks notifications at OS level | subscribe returns Err | Log + user notified in UI | User enables in OS settings |
| Service worker registration fails | client-side error | PWA degraded to standard web; sev-3 log | User upgrades browser |
| Manifest icons 404 | client browser error | Default-icon used | Brand pack uploads fixed |
| Encrypted payload decryption fails on browser | push silently dropped | Sev-2 audit; subscription marked suspect | Re-subscribe |
| Subscribing same device twice | dedup index | Second is no-op | Inherent |
| Subscription leaked (sold/lost device) | manual revoke endpoint | User revokes via /unsubscribe | User-initiated |

---

## §11 — Implementation notes

**§11.1** `web-push` Rust crate handles VAPID signing + Web Push encryption per RFCs.

**§11.2** Manifest generated server-side per request (no cache key collision across tenants).

**§11.3** Service worker source code in `services/portal/web/pwa/service-worker.ts` compiled to JS at deploy time; version stamped via build step.

**§11.4** Push payload format: `{ "kind": "<enum>", "title": "<generic>", "body": "<generic>", "url": "https://<slug>.cyberos.world/..." }`. ≤ 4 KiB total per push provider limits.

**§11.5** VAPID keypair generated once per deployment via `cyberos-portal vapid-gen`; private key in KMS.

**§11.6** Rate limit Redis sliding-window per (subject_id, day).

**§11.7** Tailwind config tuned for mobile-first; default breakpoint chain starts mobile.

**§11.8** IndexedDB queue for offline writes (slice 3 enhancement); slice 2 = network-required for writes.

**§11.9** Notification dedup at client side via service worker check (avoid double-show on push + manual refresh).

**§11.10** Per-tenant Manifest can be heavily cached (1h client + edge); brand changes propagate via service-worker cache invalidation on next visit.

---

*End of FR-PORTAL-007 spec.*
