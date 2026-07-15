---
id: NFR-PROJ-004
title: "PROJ cycle-window enforcement — work added mid-cycle MUST be flagged + tracked"
module: PROJ
category: maintainability
priority: SHOULD
verification: T
phase: P1
slo: "100% of mid-cycle additions/removals tracked + visible in the cycle review draft"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-PROJ-012, TASK-PROJ-004]
---

## §1 — Statement (BCP-14 normative)

1. Cycle windows (default 2-week sprint, configurable) **MUST** be persisted with `{start_at, end_at, frozen_at}` where `frozen_at` is the moment scope is closed.
2. Issues added to a cycle after `frozen_at` **MUST** carry a `cycle_added_post_freeze=true` flag — visible in the cycle review draft.
3. Issues removed from a cycle after `frozen_at` **MUST** retain a stub row showing they were removed, with `removed_at + remover_id + reason`.
4. The cycle review draft (`TASK-PROJ-012`) **MUST** summarise additions/removals separately from completed work — operators see scope drift clearly.
5. Mid-cycle frozen-scope mutations **MUST NOT** retroactively change a closed (historical) cycle.

## §2 — Why this constraint

Mid-cycle scope changes are inevitable but corrosive when invisible: completion rates look healthy because items that didn't ship were silently dropped. The flagged/stubbed pattern keeps the scope-drift visible without forbidding the mutation outright (sometimes scope must change). The historical-immutability rule preserves trustworthy retrospectives.

## §3 — Measurement

- Per-cycle counters: `proj_cycle_added_post_freeze_total`, `proj_cycle_removed_post_freeze_total`.
- Cycle-review draft completeness — every scope mutation in the cycle appears.
- Gauge `proj_cycle_scope_drift_percent` per cycle.

## §4 — Verification

- Unit test (T) — add issue post-freeze; assert flag set.
- Integration test (T) — generate cycle review draft; assert additions/removals listed.
- Snapshot test (T) — closed cycle is read-only.

## §5 — Failure handling

- Flag missing on post-freeze addition → sev-3 data integrity gap.
- Closed-cycle mutation detected → sev-2; CTO investigates.
- High drift cycle-over-cycle → product retrospective.

---

*End of NFR-PROJ-004.*
