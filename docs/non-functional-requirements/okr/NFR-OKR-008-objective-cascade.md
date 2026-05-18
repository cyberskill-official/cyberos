---
id: NFR-OKR-008
title: "OKR objective cascade integrity — child objectives MUST chain to parent at all times"
module: OKR
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of child objectives have a resolvable parent; 0 orphan trees"
owner: CEO
created: 2026-05-18
related_frs: [FR-OKR-001]
---

## §1 — Statement (BCP-14 normative)

1. Objectives declaring `parent: <objective_id>` **MUST** reference a real, active objective in the same tenant.
2. Orphan child objectives (parent deleted) **MUST** be flagged immediately; the cascade detector runs hourly.
3. Cycle relationships: child cannot belong to a later cycle than its parent.
4. Cascade visualisation **MUST** be available — operators see the tree.
5. Cyclic parentage **MUST** be prevented at edit time (objective A → B → A).

## §2 — Why this constraint

Objective cascade is what makes OKRs an organisational alignment tool, not just team lists. Orphans break the rollup. Cyclic parentage is impossible by definition. The hourly detector catches edge cases (parent deletion). The cycle-relationship rule prevents temporal inversion (a child finishing before its parent starts).

## §3 — Measurement

- Counter `okr_orphan_child_total` — must be 0.
- Counter `okr_cycle_attempt_total` — must be 0.
- Hourly cascade integrity check.

## §4 — Verification

- Integration test (T) — delete parent; assert child flagged.
- Cycle test (T) — attempt A→B→A; assert blocked.
- Snapshot test (T) — cascade tree renders.

## §5 — Failure handling

- Orphan detected → flag + objective owner notified.
- Cycle attempt → reject + audit.
- Cascade visualisation broken → sev-3 UX bug.

---

*End of NFR-OKR-008.*
