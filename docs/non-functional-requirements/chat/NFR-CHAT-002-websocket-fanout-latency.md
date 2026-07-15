---
id: NFR-CHAT-002
title: "CHAT WebSocket fanout latency — p95 < 200ms from persist to all subscribed clients"
module: CHAT
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 200ms from DB persist ack to message delivered to every connected WebSocket subscriber on the channel"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-CHAT-001]
---

## §1 — Statement (BCP-14 normative)

1. After a message is committed to Postgres (NFR-CHAT-001), the WebSocket fanout **MUST** deliver it to every currently-connected subscriber on the channel at **p95 < 200ms** and **p99 < 500ms**.
2. Fanout latency is measured as `(websocket_send_ts - db_commit_ts)` per subscriber per message; the metric aggregates the worst-case subscriber per message.
3. The fanout **MUST** be tenant-scoped — a tenant A message **MUST NOT** be delivered to any tenant B subscriber, even on a misconfigured shared channel.
4. Subscribers connected to a different fork pod than the writer **MUST** still receive the message within the 200ms budget; cross-pod fanout uses the Mattermost cluster gossip mechanism (or equivalent in the fork).
5. Disconnected subscribers **MUST** receive the message on next reconnect via the channel backfill API (no real-time delivery for offline clients; backfill is separate).

## §2 — Why this constraint

200ms fanout makes CHAT feel "instant" for active conversations — the message appears on peer screens within human-perceptible delay. Combined with the 100ms persist (NFR-CHAT-001), total send-to-peer-render is < 300ms p95 — matching the rendered NFR catalog's "CHAT message deliver p95 ≤ 200ms" claim (the 200ms there counts the WebSocket leg only). The tenant-scoping rule is the load-bearing multi-tenancy guarantee for CHAT.

## §3 — Measurement

- Histogram `chat_ws_fanout_latency_seconds{cross_pod}` per (message, subscriber) pair; rolled up to worst-case-per-message.
- p95 alarm at > 200ms; p99 alarm at > 500ms.
- Counter `chat_ws_cross_tenant_delivery_total` — should always be zero; sev-0 on > 0.

## §4 — Verification

- Load test (T) — 1000 channels × 50 subscribers each, 10 msg/sec/channel; asserts p95 < 200ms.
- Property test (T) — multi-tenant subscriber matrix, drives 10k messages; asserts zero cross-tenant deliveries.

## §5 — Failure handling

- p95 > 200ms → sev-3; investigate cluster gossip health, WebSocket connection pool size.
- Cross-tenant delivery → sev-0; halt CHAT immediately; emergency CSO + CTO call.
- Fanout to disconnected subscribers (impossible by design) — should not happen; if seen, sev-2.

---

*End of NFR-CHAT-002.*
