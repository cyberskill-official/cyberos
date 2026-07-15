---
id: NFR-EMAIL-001
title: "EMAIL inbound delivery latency — SMTP receipt to inbox visible < 30s p95"
module: EMAIL
category: performance
priority: MUST
verification: T
phase: P0
slo: "p95 < 30s from SMTP MAIL FROM to inbox-row visible in user UI"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-EMAIL-001, TASK-EMAIL-003]
---

## §1 — Statement (BCP-14 normative)

1. Inbound emails reaching the Stalwart MTA **MUST** be parsed, classified, indexed, and visible in the recipient's inbox UI within 30s p95 and 90s p99.
2. The pipeline stages (`receive → SPF/DKIM/DMARC check → CaMeL classify → index → notify`) each have sub-budgets; aggregate must stay within ceiling.
3. Bulk inbound surges (> 100 msg/min/tenant) **MUST NOT** cause unbounded queuing — backpressure surfaces via gauge `email_inbound_queue_depth_seconds`.
4. Messages held by CaMeL (suspected injection) **MUST** still produce a queue-row visible to the user as "held for review" within the SLA.
5. Delivery to user UI **MUST** be over WebSocket or SSE so no polling latency contributes to the budget.

## §2 — Why this constraint

Inbound mail is a primary user activity. Delays > 30s create the perception of broken delivery, and users start looking elsewhere. The 90s p99 tolerates burst load. The CaMeL hold visibility rule is the trust-restoration: even when we block a message, the user knows it arrived. The WebSocket/SSE rule keeps the UI in real-time mode.

## §3 — Measurement

- Histogram `email_inbound_delivery_latency_seconds{stage=receive|auth_check|camel|index|notify}`.
- Gauge `email_inbound_queue_depth_seconds`.
- Counter `email_camel_hold_total{reason}`.

## §4 — Verification

- Integration test (T) — synthetic SMTP receive; assert visible < 30s.
- Load test (T) — 100 msg/min surge; assert SLO holds.
- Chaos test (T) — CaMeL slow; assert held messages still surface in queue UI.

## §5 — Failure handling

- p95 > 30s → sev-3; identify bottleneck stage.
- Queue depth > 60s → sev-2; meaningful backlog.
- CaMeL hold not surfaced in UI → sev-2; transparency broken.

---

*End of NFR-EMAIL-001.*
