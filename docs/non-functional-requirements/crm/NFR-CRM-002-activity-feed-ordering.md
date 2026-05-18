---
id: NFR-CRM-002
title: "CRM activity-feed ordering — feed MUST be strictly chronologically consistent"
module: CRM
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of activity-feed reads return events in strict committed-at order; no out-of-order"
owner: CSO-Sales
created: 2026-05-18
related_frs: [FR-CRM-002]
---

## §1 — Statement (BCP-14 normative)

1. Activity events (call, email, meeting, note, stage-change) for an account/contact/deal **MUST** be displayed in the feed in strict `committed_at` order (newest first).
2. The `committed_at` timestamp is server-generated; client clocks are ignored to prevent skew.
3. Events with identical committed_at use the `event_id` as tiebreaker (monotonic per-tenant).
4. Late-arriving events (e.g., async email delivery) appear in the feed at their committed_at, not insertion order.
5. The feed **MUST** be paginated; pagination tokens encode the (committed_at, event_id) cursor for consistent pagination.

## §2 — Why this constraint

Sales reps build trust based on the feed: "what happened last with this account?" Out-of-order events break that trust. Server-generated timestamps eliminate clock-skew bugs. Cursor-based pagination prevents the "scroll and miss" race condition on newly-added events.

## §3 — Measurement

- Counter `crm_feed_out_of_order_total` — must be 0.
- Histogram `crm_feed_pagination_latency_ms`.
- Property test in CI.

## §4 — Verification

- Integration test (T) — insert events with varied committed_at; assert order.
- Property test (T) — random insertion sequences; assert order invariant.
- Pagination test (T) — concurrent insert + paginate; assert no skipped/dup events.

## §5 — Failure handling

- Out-of-order detected → sev-2; investigate timestamp source.
- Pagination dup → sev-3 cursor bug.
- Client-clock used by accident → sev-3 lint failure.

---

*End of NFR-CRM-002.*
