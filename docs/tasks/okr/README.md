# OKR module — task index

_Generated 2026-05-17 — 7 FRs, 42 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-OKR-001](TASK-OKR-001-objective-kr-schema/spec.md) | MUST | 1 | 6 | OKR Objective × Key Result schema — Company → Team → Member cascade + quarterly Cycle + closed align |
| [TASK-OKR-002](TASK-OKR-002-kr-types/spec.md) | MUST | 3 | 4 | OKR 3 KR types — hit_target + improvement + milestone with type-specific progress calculation |
| [TASK-OKR-003](TASK-OKR-003-progress-source-dsl/spec.md) | MUST | 3 | 10 | OKR KR progress_source DSL — declarative query against PROJ / INV / HR / LEARN modules for auto-prog |
| [TASK-OKR-004](TASK-OKR-004-auto-progress-batch/spec.md) | MUST | 3 | 5 | OKR auto-progress nightly batch — resolves all KR progress_sources + updates current_value + emits d |
| [TASK-OKR-005](TASK-OKR-005-weekly-check-in/spec.md) | MUST | 3 | 5 | OKR weekly check-in — 1-10 confidence + rationale per KR with rolling 4-week history + trend visuali |
| [TASK-OKR-006](TASK-OKR-006-monday-digest/spec.md) | MUST | 3 | 6 | OKR Monday-morning CUO digest — auto-progress + check-ins → founder summary delivered via email/chat |
| [TASK-OKR-007](TASK-OKR-007-quarterly-retro-draft/spec.md) | SHOULD | 3 | 6 | OKR quarterly retro CUO draft — auto-generated retro with face-saving Vietnamese framing for honest  |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-OKR-001→TASK-AUTH-003, TASK-OKR-001→TASK-AUTH-101
- **CUO**: TASK-OKR-006→TASK-CUO-101, TASK-OKR-007→TASK-CUO-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._