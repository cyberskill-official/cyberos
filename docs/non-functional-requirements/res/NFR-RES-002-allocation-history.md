---
id: NFR-RES-002
title: "RES allocation-change history — every allocation mutation MUST emit an audit row"
module: RES
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of allocation mutations produce an audit row with before/after delta"
owner: COO
created: 2026-05-18
related_frs: [FR-RES-002]
---

## §1 — Statement (BCP-14 normative)

1. Every change to a member's allocation (create, update, deactivate) **MUST** emit a BRAIN audit row carrying `{member_id, project_id, before_pct, after_pct, actor_id, reason?, changed_at}`.
2. Allocation rows themselves are append-only; corrections take the form of a new period overriding the prior.
3. Bulk allocation changes (e.g., team rebalance) **MUST** emit one row per affected member; bulk operations are not allowed to collapse into a single audit row.
4. Allocation history **MUST** be retrievable as a per-member timeline.
5. Retention: ≥ 3 years for historical analysis + dispute resolution.

## §2 — Why this constraint

Allocation disputes are common ("when was I moved off project X?"). Per-mutation audit rows + per-member timeline make these queries cheap. Append-only prevents post-hoc rewriting of who-was-on-what-when. Per-member rows during bulk operations preserve forensic granularity.

## §3 — Measurement

- Counter `res_allocation_audit_row_total{operation=create|update|deactivate}`.
- Audit-row count = mutation-counter (reconciliation).
- Retention checker.

## §4 — Verification

- Integration test (T) — mutation; assert audit row + content.
- Property test (T) — bulk; assert per-member rows.
- Retention test (T) — old rows still present.

## §5 — Failure handling

- Audit row missing → sev-2 audit gap.
- Bulk collapsed into one row → sev-2; granularity broken.
- Retention < 3y → sev-3.

---

*End of NFR-RES-002.*
