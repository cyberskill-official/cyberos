---
id: NFR-EMAIL-005
title: "EMAIL thread merge correctness — In-Reply-To + References MUST group messages correctly"
module: EMAIL
category: reliability
priority: MUST
verification: T
phase: P0
slo: "≥ 99% of incoming messages with In-Reply-To are correctly merged into the parent thread"
owner: CTO
created: 2026-05-18
related_frs: [FR-EMAIL-003]
---

## §1 — Statement (BCP-14 normative)

1. Inbound messages carrying `In-Reply-To` or `References` headers **MUST** be merged into the parent thread when the parent exists in the tenant's inbox.
2. Missing parent (referenced message-id not in inbox) **MUST** create a new thread; the orphaned reply is NOT silently dropped.
3. Subject-based merging (no In-Reply-To, but matching subject `Re:`) **MUST** be a fallback strategy with explicit precedence — header-based first, subject-based only when headers absent.
4. Thread merge decisions **MUST** be auditable; every message row carries `{merged_into_thread_id, merge_method=header|subject|new}`.
5. Mistaken merges (user reports "wrong thread") **MUST** be reversible via a UI action that creates a new thread + moves the message.

## §2 — Why this constraint

Thread integrity is foundational to conversation UX. A mismerged message (wrong conversation) is confusing; a missed merge fragments a conversation into pieces. The hybrid strategy + explicit precedence makes the algorithm predictable. The audit trail + reversal action lets users + ops debug rare cases.

## §3 — Measurement

- Counter `email_thread_merge_total{method=header|subject|new}`.
- Counter `email_thread_merge_user_reversal_total` — user-reported errors.
- Daily reconciliation: assert thread membership consistent.

## §4 — Verification

- Integration test (T) — fixtures with proper headers; assert merged.
- Integration test (T) — orphaned reply; assert new thread.
- Property test (T) — random mail sequences; assert thread invariants.

## §5 — Failure handling

- User reversal > 1% of messages → sev-3; merging logic needs review.
- Orphan rate > 5% → sev-3; investigate (possibly real but flag for review).
- Mismerge silently fixed → audit row preserves history.

---

*End of NFR-EMAIL-005.*
