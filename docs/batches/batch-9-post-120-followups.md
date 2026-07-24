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
ended: 2026-07-25T17:40:00Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch 9 — post-1.2.0 follow-ups

Implements the post–batch/8 plan: Wave 0 chores, MMR sync, MEMORY-302, ship-tasks evolution, MCP/OBS schedule, and 1.4.x / 1.5.0 draft tasks (stay on CyberOS 1.x — not a major bump).

## Gate-1 (2026-07-23)

Operator: **all-accept** for MEMORY-302, IMP-141, CUO-305, IMP-142 → advanced to `testing` (evidence `batch-9-gate1-acceptance.md`). Halted before `done`.

## Gate-2 (2026-07-23)

Operator: **all-accept** for MEMORY-302, IMP-141, CUO-305, IMP-142 → `done` (evidence `batch-9-gate2-acceptance.md`). Drafts IMP-143/144 unchanged. MCP/OBS resume waves 9a–9c not started.

## Wave 0 (chores — this PR)

- [x] Delete remote `ship/batch-8f-entrypoint`
- [x] Close parent ledger `batch-8-audit-hardening.md`
- [x] Delete `scripts/awh_finalize.sh`
- [x] Branch-protection probe / decision #7: **closed 2026-07-25** — no existing ruleset / nothing to scrub (operator session note). Stub workflow files remain absent under `.github/workflows/`.

## Wave 1–4 members

| ID | Task | Wave | Intent |
|----|------|------|--------|
| TASK-IMP-141 | MMR sync for memory-append | 1 | doctor stays READY after gated flips |
| TASK-MEMORY-302 | applier raw-writes → put() | 1 | stop BRAIN re-contamination |
| (chore) | rollout.sh checksum chooser | 1 | match bootstrap.sh |
| TASK-CUO-305 | ship-tasks evolution from batch/8 friction | 2 | doctrine + checklists |
| TASK-IMP-142 | MCP/OBS + APP-001 resume schedule | 3 | schedule only (this batch) |
| TASK-IMP-143 | 1.4.x stuck-WIP hub + signed HITL | 4 | shipped (batch/9d) |
| TASK-IMP-144 | 1.5.0 transition-locked state engine | 4 | shipped (batch/9d) |

## MCP/OBS ship schedule (Wave 3)

Gate-2 of IMP-139 routed these to `ready_to_implement` (except APP-001 resume). Suggested ship order:

1. **batch/9a-mcp** — TASK-MCP-003, 005, 006, 007, 008 (re-spec/adopt under `services/mcp-gateway/`) — **merged** PR #139; statuses `done` via session HITL override 2026-07-24 (evidence `batch-9b-obs-session-hitl.md`).
2. **batch/9b-obs** — TASK-OBS-001, 003, 005, 007, 008, 009 (re-spec against as-built obs + `services/shared/cyberos-obs-sdk/`) — **merged** PR #140; statuses `done` via session HITL override (evidence `docs/batches/batch-9b-obs-session-hitl.md`).
3. **batch/9c-app** — TASK-APP-001 (resume; process hygiene) — **shipped** on branch `batch/9c-app` (2026-07-24): task@1 adopt against `services/memory/desktop` Ops; residual guard tests; statuses `done` via session HITL override (evidence `docs/batches/batch-9c-app-session-hitl.md`). Operator merges PR when ready.

Preconditions met: IMP-141 + MEMORY-302 are `done` (doctor floor trustworthy).
