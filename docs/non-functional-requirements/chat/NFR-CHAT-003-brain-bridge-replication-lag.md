---
id: NFR-CHAT-003
title: "CHAT BRAIN bridge replication lag — p95 < 2s from chat message to BRAIN put"
module: CHAT
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 2s, p99 < 5s from chat message persist to BRAIN bridge put committed"
owner: CTO
created: 2026-05-18
related_frs: [FR-CHAT-005, FR-BRAIN-101]
---

## §1 — Statement (BCP-14 normative)

1. After a chat message is committed to CHAT Postgres (NFR-CHAT-001), the BRAIN bridge plugin **MUST** replicate the message to BRAIN at **p95 < 2s** and **p99 < 5s**, measured from chat commit to BRAIN bridge put ack.
2. The bridge **MUST** be async (not on the CHAT critical path) — bridge failure or slowness **MUST NOT** block CHAT delivery.
3. Bridge failures **MUST** retry with exponential backoff (1s, 2s, 4s, 8s, 16s; then dead-letter to `chat_brain_bridge_dlq` after 5 attempts).
4. The bridge **MUST** preserve message ordering per channel — chronological order in CHAT must match chronological order in BRAIN.
5. Every bridged message **MUST** carry a tenant_id and channel_id on the BRAIN row; the row is queryable by both keys.

## §2 — Why this constraint

The 2s replication lag is the bound for "CUO can answer questions about this morning's chat." Beyond ~5s, the lag becomes perceptible when a user @-mentions CUO immediately after a key message ("CUO, what did Alice just say?"). The async rule decouples CHAT availability from BRAIN availability — a BRAIN outage doesn't degrade CHAT. The DLQ is the safety net for replication backlog; ordering preservation is the assertion CUO can rely on for chronological answers.

## §3 — Measurement

- Histogram `chat_brain_bridge_lag_seconds` per bridged message.
- Counter `chat_brain_bridge_retries_total{attempt_n}` per retry; sev-3 alarm on sustained high attempt-n.
- Gauge `chat_brain_bridge_dlq_depth` — should be near-zero; sev-2 at > 100.

## §4 — Verification

- Integration test `services/chat-plugins/brain-bridge/tests/lag_test.rs` (T) — drives 200 messages; asserts p95 < 2s.
- Retry test (T) — simulates BRAIN unavailable for 30s; asserts retries succeed and messages land within budget after recovery.

## §5 — Failure handling

- p95 > 2s sustained → sev-3; investigate BRAIN ingest pipeline (NFR-BRAIN-001) health.
- DLQ depth > 100 → sev-2; replay DLQ once root cause identified.
- Ordering violation in BRAIN → sev-2; per-channel ordering invariant broken; investigate parallel-replay edge case.

---

*End of NFR-CHAT-003.*
