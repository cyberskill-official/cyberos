# PROJ module — feature request index

_Generated 2026-05-17 — 18 FRs, 128 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-PROJ-001](FR-PROJ-001-issue-schema.md) | MUST | 1 | 12 | PROJ Issue + Cycle + Engagement schema — RLS + cross-module linkable + status FSM + audit + assignee |
| [FR-PROJ-002](FR-PROJ-002-memory-decision-anchoring.md) | MUST | 1 | 7 | memory-anchored proj.decision row per Issue state change — reason + prior_chain link + cross-module r |
| [FR-PROJ-003](FR-PROJ-003-yjs-crdt-collaboration.md) | MUST | 2 | 10 | Yjs CRDT for issue description + comment-body fields; LWW for scalar metadata; reconnection state re |
| [FR-PROJ-004](FR-PROJ-004-issue-lifecycle-fsm.md) | MUST | 2 | 5 | Issue lifecycle FSM — backlog → todo → in-progress → in-review → done | cancelled with FR-PROJ-002 a |
| [FR-PROJ-005](FR-PROJ-005-rate-card-schema.md) | MUST | 2 | 4 | Rate-card schema per Engagement — (role × currency × hourly_rate × billable_default) with effective- |
| [FR-PROJ-006](FR-PROJ-006-billable-cascade.md) | MUST | 2 | 6 | Billable cascade — Member-override → task-class → role-default → fallback; resolution snapshot at ti |
| [FR-PROJ-007](FR-PROJ-007-billing-modes.md) | MUST | 2 | 6 | Three billing modes — Time & Materials, Fixed-Fee, Retainer — with mode-aware rollups and per-mode i |
| [FR-PROJ-008](FR-PROJ-008-memory-audit-row-per-mutation.md) | MUST | 2 | 5 | memory audit row per issue mutation — chained to PROJ history_event table with field-level diff and c |
| [FR-PROJ-009](FR-PROJ-009-memory-link-schema.md) | MUST | 2 | 5 | MEMORY_LINK schema — Issue ↔ memory memory linkage (cites | implements | supersedes) with bidirectiona |
| [FR-PROJ-010](FR-PROJ-010-citation-drift-detector.md) | SHOULD | 3 | 4 | Citation drift detector — nightly sweep flags stale MEMORY_LINKs (deleted target, superseded chain, b |
| [FR-PROJ-011](FR-PROJ-011-blocker-detector.md) | MUST | 3 | 6 | Blocker detector from comment stream — `blocked by` parser + dwell-time monitor + CUO Notify on stal |
| [FR-PROJ-012](FR-PROJ-012-cycle-review-draft.md) | MUST | 3 | 8 | Cycle-review draft generator — CUO/COO-persona LLM compose at cycle close with completion stats, blo |
| [FR-PROJ-013](FR-PROJ-013-estimate-calibration.md) | MUST | 3 | 6 | Estimate calibration snapshot — per-member per-task-class nightly batch with Bayesian update and ope |
| [FR-PROJ-014](FR-PROJ-014-kanban-board.md) | MUST | 3 | 10 | Kanban Board view — drag/drop status transition + keyboard-first navigation + 60fps virtualised list |
| [FR-PROJ-015](FR-PROJ-015-timeline-view.md) | MUST | 3 | 8 | Timeline view — cycle window × assignee swimlane with day-grid layout, drag-resize for date changes, |
| [FR-PROJ-016](FR-PROJ-016-gantt-view.md) | SHOULD | 3 | 10 | Gantt view with dependency arrows — issue-to-issue precedence + critical path highlighting + roll-up |
| [FR-PROJ-017](FR-PROJ-017-brief-modal.md) | MUST | 3 | 8 | Brief Modal — issue deep-view with Yjs description editor + threaded comments + LWW meta sidebar + p |
| [FR-PROJ-018](FR-PROJ-018-design-tokens-a11y-ci.md) | MUST | 3 | 8 | Liquid-Glass design tokens (tokens.proj.css) + axe-core CI accessibility gate + Storybook visual reg |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-PROJ-002→FR-AI-003
- **AUTH**: FR-PROJ-001→FR-AUTH-001, FR-PROJ-001→FR-AUTH-003
- **memory**: FR-PROJ-008→FR-MEMORY-101
- **CUO**: FR-PROJ-011→FR-CUO-101, FR-PROJ-012→FR-CUO-101

**This module is depended on by:**

- **CRM**: FR-CRM-004→FR-PROJ-005
- **EMAIL**: FR-EMAIL-007→FR-PROJ-001
- **HR**: FR-HR-008→FR-PROJ-013
- **LEARN**: FR-LEARN-003→FR-PROJ-013
- **RES**: FR-RES-001→FR-PROJ-001
- **TIME**: FR-TIME-004→FR-PROJ-002, FR-TIME-005→FR-PROJ-006

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._