---
id: NFR-CHAT-001
title: "CHAT message persist latency — p95 < 100ms from send to DB ack"
module: CHAT
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 100ms, p99 < 250ms from CHAT POST /api/v4/posts to DB commit ack"
owner: CTO
created: 2026-05-18
related_frs: [FR-CHAT-001, FR-CHAT-003]
---

## §1 — Statement (BCP-14 normative)

1. From the moment a CHAT POST request lands on the Mattermost fork, the message **MUST** be committed to durable Postgres storage and the DB ack returned at **p95 < 100ms** and **p99 < 250ms**, measured at the application span.
2. The persist path **MUST** be synchronous — the API response **MUST NOT** return success until the DB commit acks. No "fire-and-forget" queueing.
3. WebSocket fanout (NFR-CHAT-002) **MAY** happen post-ack on a background task — fanout latency is separately budgeted.
4. The 100ms target assumes the Mattermost fork retains upstream's `WriteAhead` patterns; deviations from upstream that materially regress this SLO require CTO sign-off.
5. The SLO **MUST** hold under steady-state slice-2 load (~1000 messages/sec across the fleet); load tests verify quarterly.

## §2 — Why this constraint

100ms persist latency is the "did my message send" UX threshold — beyond it, users start clicking send twice. The synchronous rule is critical for trust: if the API returns 200 OK and the message later fails to persist (queue overflow, DB crash), users believe their message was sent when it wasn't. The 250ms p99 ceiling accommodates occasional GC or DB-checkpoint spikes without dropping below the perceptual threshold.

## §3 — Measurement

- Histogram `chat_message_persist_seconds{channel_type, has_attachments}` per POST.
- p95 alarm at > 100ms; p99 alarm at > 250ms.
- The OBS dashboard `chat-message-flow` plots persist + fanout + brain-bridge stacked.

## §4 — Verification

- Load test `deploy/loadtest/chat-message-flow.k6.js` (T) — 1000 msg/sec for 10 minutes; asserts p95 < 100ms.
- Integration test (T) — drives 200 sequential POSTs; asserts p95 < 100ms in steady state.

## §5 — Failure handling

- p95 > 100ms sustained → sev-3; investigate DB write contention or Mattermost fork drift.
- p99 > 500ms → sev-2; possible disk I/O degradation; vertical scale or DB tier investigation.
- API returns 200 but row not persisted (data loss) → sev-0 immediate halt of message ingestion.

---

*End of NFR-CHAT-001.*
