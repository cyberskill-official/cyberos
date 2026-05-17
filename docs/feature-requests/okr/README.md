# OKR module — feature request index

_Generated 2026-05-17 — 7 FRs, 42 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-OKR-001](FR-OKR-001-objective-kr-schema.md) | MUST | 1 | 6 | OKR Objective × Key Result schema — Company → Team → Member cascade + quarterly Cycle + closed align |
| [FR-OKR-002](FR-OKR-002-kr-types.md) | MUST | 3 | 4 | OKR 3 KR types — hit_target + improvement + milestone with type-specific progress calculation |
| [FR-OKR-003](FR-OKR-003-progress-source-dsl.md) | MUST | 3 | 10 | OKR KR progress_source DSL — declarative query against PROJ / INV / HR / LEARN modules for auto-prog |
| [FR-OKR-004](FR-OKR-004-auto-progress-batch.md) | MUST | 3 | 5 | OKR auto-progress nightly batch — resolves all KR progress_sources + updates current_value + emits d |
| [FR-OKR-005](FR-OKR-005-weekly-check-in.md) | MUST | 3 | 5 | OKR weekly check-in — 1-10 confidence + rationale per KR with rolling 4-week history + trend visuali |
| [FR-OKR-006](FR-OKR-006-monday-digest.md) | MUST | 3 | 6 | OKR Monday-morning CUO digest — auto-progress + check-ins → founder summary delivered via email/chat |
| [FR-OKR-007](FR-OKR-007-quarterly-retro-draft.md) | SHOULD | 3 | 6 | OKR quarterly retro CUO draft — auto-generated retro with face-saving Vietnamese framing for honest  |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-OKR-001→FR-AUTH-003, FR-OKR-001→FR-AUTH-101
- **CUO**: FR-OKR-006→FR-CUO-101, FR-OKR-007→FR-CUO-101

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._