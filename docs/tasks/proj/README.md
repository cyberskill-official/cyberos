# PROJ module — task index

_Generated 2026-05-17 — 18 FRs, 128 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-PROJ-001](TASK-PROJ-001-issue-schema/spec.md) | MUST | 1 | 12 | PROJ Issue + Cycle + Engagement schema — RLS + cross-module linkable + status FSM + audit + assignee |
| [TASK-PROJ-002](TASK-PROJ-002-memory-decision-anchoring/spec.md) | MUST | 1 | 7 | memory-anchored proj.decision row per Issue state change — reason + prior_chain link + cross-module r |
| [TASK-PROJ-003](TASK-PROJ-003-yjs-crdt-collaboration/spec.md) | MUST | 2 | 10 | Yjs CRDT for issue description + comment-body fields; LWW for scalar metadata; reconnection state re |
| [TASK-PROJ-004](TASK-PROJ-004-issue-lifecycle-fsm/spec.md) | MUST | 2 | 5 | Issue lifecycle FSM — backlog → todo → in-progress → in-review → done | cancelled with TASK-PROJ-002 a |
| [TASK-PROJ-005](TASK-PROJ-005-rate-card-schema/spec.md) | MUST | 2 | 4 | Rate-card schema per Engagement — (role × currency × hourly_rate × billable_default) with effective- |
| [TASK-PROJ-006](TASK-PROJ-006-billable-cascade/spec.md) | MUST | 2 | 6 | Billable cascade — Member-override → task-class → role-default → fallback; resolution snapshot at ti |
| [TASK-PROJ-007](TASK-PROJ-007-billing-modes/spec.md) | MUST | 2 | 6 | Three billing modes — Time & Materials, Fixed-Fee, Retainer — with mode-aware rollups and per-mode i |
| [TASK-PROJ-008](TASK-PROJ-008-memory-audit-row-per-mutation/spec.md) | MUST | 2 | 5 | memory audit row per issue mutation — chained to PROJ history_event table with field-level diff and c |
| [TASK-PROJ-009](TASK-PROJ-009-memory-link-schema/spec.md) | MUST | 2 | 5 | MEMORY_LINK schema — Issue ↔ memory memory linkage (cites | implements | supersedes) with bidirectiona |
| [TASK-PROJ-010](TASK-PROJ-010-citation-drift-detector/spec.md) | SHOULD | 3 | 4 | Citation drift detector — nightly sweep flags stale MEMORY_LINKs (deleted target, superseded chain, b |
| [TASK-PROJ-011](TASK-PROJ-011-blocker-detector/spec.md) | MUST | 3 | 6 | Blocker detector from comment stream — `blocked by` parser + dwell-time monitor + CUO Notify on stal |
| [TASK-PROJ-012](TASK-PROJ-012-cycle-review-draft/spec.md) | MUST | 3 | 8 | Cycle-review draft generator — CUO/COO-persona LLM compose at cycle close with completion stats, blo |
| [TASK-PROJ-013](TASK-PROJ-013-estimate-calibration/spec.md) | MUST | 3 | 6 | Estimate calibration snapshot — per-member per-task-class nightly batch with Bayesian update and ope |
| [TASK-PROJ-014](TASK-PROJ-014-kanban-board/spec.md) | MUST | 3 | 10 | Kanban Board view — drag/drop status transition + keyboard-first navigation + 60fps virtualised list |
| [TASK-PROJ-015](TASK-PROJ-015-timeline-view/spec.md) | MUST | 3 | 8 | Timeline view — cycle window × assignee swimlane with day-grid layout, drag-resize for date changes, |
| [TASK-PROJ-016](TASK-PROJ-016-gantt-view/spec.md) | SHOULD | 3 | 10 | Gantt view with dependency arrows — issue-to-issue precedence + critical path highlighting + roll-up |
| [TASK-PROJ-017](TASK-PROJ-017-brief-modal/spec.md) | MUST | 3 | 8 | Brief Modal — issue deep-view with Yjs description editor + threaded comments + LWW meta sidebar + p |
| [TASK-PROJ-018](TASK-PROJ-018-design-tokens-a11y-ci/spec.md) | MUST | 3 | 8 | Liquid-Glass design tokens (tokens.proj.css) + axe-core CI accessibility gate + Storybook visual reg |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-PROJ-002→TASK-AI-003
- **AUTH**: TASK-PROJ-001→TASK-AUTH-001, TASK-PROJ-001→TASK-AUTH-003
- **memory**: TASK-PROJ-008→TASK-MEMORY-101
- **CUO**: TASK-PROJ-011→TASK-CUO-101, TASK-PROJ-012→TASK-CUO-101

**This module is depended on by:**

- **CRM**: TASK-CRM-004→TASK-PROJ-005
- **EMAIL**: TASK-EMAIL-007→TASK-PROJ-001
- **HR**: TASK-HR-008→TASK-PROJ-013
- **LEARN**: TASK-LEARN-003→TASK-PROJ-013
- **RES**: TASK-RES-001→TASK-PROJ-001
- **TIME**: TASK-TIME-004→TASK-PROJ-002, TASK-TIME-005→TASK-PROJ-006

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._