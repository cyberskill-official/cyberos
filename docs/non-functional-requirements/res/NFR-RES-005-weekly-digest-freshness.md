---
id: NFR-RES-005
title: "RES weekly digest freshness — Monday digest MUST include data through Sunday EOD"
module: RES
category: observability
priority: SHOULD
verification: T
phase: P1
slo: "100% of Monday digests include data current through prior Sunday 23:59 local tenant time"
owner: COO
created: 2026-05-18
related_tasks: [TASK-RES-001, TASK-RES-003]
---

## §1 — Statement (BCP-14 normative)

1. The Monday capacity digest **MUST** include allocation data current through the prior Sunday 23:59 (tenant local time).
2. The digest **MUST** be delivered by 09:00 Monday tenant-local; later than 09:30 triggers sev-3.
3. The digest **MUST** include: over/under flags, hiring memo status, recent OT consent activity, next-week forecast.
4. Recipients are configurable per tenant; default = COO + team leads.
5. Late digests **MUST** still send (don't suppress) — operator visibility matters more than punctuality.

## §2 — Why this constraint

The Monday digest is the week-starter ritual. Stale or missing data undermines its planning value. The 09:00 delivery is the "starts the week" cue. The "send even if late" rule preserves operational continuity.

## §3 — Measurement

- Counter `res_digest_send_total{tenant, on_time}`.
- Histogram `res_digest_data_age_hours`.
- Counter `res_digest_recipient_optout_total`.

## §4 — Verification

- Integration test (T) — clock to Monday 09:00; assert digest delivered with Sunday data.
- Snapshot test (T) — digest content includes required sections.

## §5 — Failure handling

- Late digest → send anyway + sev-3.
- Stale data > 24h → sev-3; data pipeline issue.
- Mass opt-out → product feedback on digest value.

---

*End of NFR-RES-005.*
