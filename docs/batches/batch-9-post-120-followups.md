---
batch: batch/9-post-120-followups
members:
  - TASK-IMP-141
  - TASK-MEMORY-302
  - TASK-IMP-142
  - TASK-CUO-305
  - TASK-IMP-143
  - TASK-IMP-144
started: 2026-07-23T18:34:00Z
ended: null
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch 9 — post-1.2.0 follow-ups

Implements the post–batch/8 plan: Wave 0 chores, MMR sync, MEMORY-302, ship-tasks evolution, MCP/OBS schedule, and v3.x/v4.0 draft tasks.

## Gate-1 (2026-07-23)

Operator: **all-accept** for MEMORY-302, IMP-141, CUO-305, IMP-142 → advanced to `testing` (evidence `batch-9-gate1-acceptance.md`). Halted before `done`.

## Wave 0 (chores — this PR)

- [x] Delete remote `ship/batch-8f-entrypoint`
- [x] Close parent ledger `batch-8-audit-hardening.md`
- [x] Delete `scripts/awh_finalize.sh`
- [x] Branch-protection probe: stub workflows absent; Settings API 403 to this token — operator confirm in UI

## Wave 1–4 members

| ID | Task | Wave | Intent |
|----|------|------|--------|
| TASK-IMP-141 | MMR sync for memory-append | 1 | doctor stays READY after gated flips |
| TASK-MEMORY-302 | applier raw-writes → put() | 1 | stop BRAIN re-contamination |
| (chore) | rollout.sh checksum chooser | 1 | match bootstrap.sh |
| TASK-CUO-305 | ship-tasks evolution from batch/8 friction | 2 | doctrine + checklists |
| TASK-IMP-142 | MCP/OBS + APP-001 resume schedule | 3 | schedule only (this batch) |
| TASK-IMP-143 | v3.x stuck-WIP hub + signed HITL | 4 | draft |
| TASK-IMP-144 | v4.0 transition-locked state engine | 4 | draft |

## MCP/OBS ship schedule (Wave 3)

Gate-2 of IMP-139 routed these to `ready_to_implement` (except APP-001 resume). Suggested ship order:

1. **batch/9a-mcp** — TASK-MCP-003, 005, 006, 007, 008 (re-spec/adopt under `services/mcp-gateway/`)
2. **batch/9b-obs** — TASK-OBS-001, 003, 005, 007, 008, 009 (re-spec against `services/shared/` reality)
3. **batch/9c-app** — TASK-APP-001 (resume; process hygiene)

Do not start 9a–9c until IMP-141 + MEMORY-302 are `done` (doctor floor must stay trustworthy).
