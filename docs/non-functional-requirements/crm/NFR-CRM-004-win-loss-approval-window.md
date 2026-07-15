---
id: NFR-CRM-004
title: "CRM win-loss approval — deals > threshold MUST be approved by CFO before close"
module: CRM
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of large deals (> threshold) carry CFO signature before close"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-CRM-007]
---

## §1 — Statement (BCP-14 normative)

1. Deals with `amount_vnd > tenant.large_deal_threshold` **MUST** require CFO signature before transitioning to `won` or `lost`.
2. Default threshold = 1 billion VND; tenant-configurable upward (never downward without governance review).
3. The approval row carries `{approver_id, signed_at, deal_hash}`; deal mutation post-approval invalidates.
4. Win-loss memo **MUST** be persisted; auto-generated draft from TASK-CRM-007 is acceptable starting point.
5. Approval timeout: 30 days from request; expired requires resubmission.

## §2 — Why this constraint

Large deals carry outsized revenue + risk impact (commit risk, support load, comp). CFO sign provides finance oversight. The hash-binding prevents bait-and-switch. The 30-day window prevents stale approvals on long sales cycles.

## §3 — Measurement

- Counter `crm_large_deal_no_approval_attempt_total` — must be 0.
- Histogram `crm_approval_latency_days`.
- Audit row per approval.

## §4 — Verification

- Integration test (T) — large deal close without sign → reject.
- Property test (T) — mutate post-approval → invalidate.

## §5 — Failure handling

- No-approval attempt → block + audit.
- Expired approval used → resubmit.
- Threshold misconfigured → sev-3 review.

---

*End of NFR-CRM-004.*
