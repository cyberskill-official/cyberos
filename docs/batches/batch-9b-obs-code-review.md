---
batch: batch/9b-obs
members:
  - TASK-OBS-001
  - TASK-OBS-003
  - TASK-OBS-005
  - TASK-OBS-007
  - TASK-OBS-008
  - TASK-OBS-009
also_covers:
  - TASK-MCP-003
  - TASK-MCP-005
  - TASK-MCP-006
  - TASK-MCP-007
  - TASK-MCP-008
started: 2026-07-24T13:00:00Z
ended: null
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch/9b-obs — code review (session HITL override)

Reviewed: 2026-07-24 on branch `batch/9b-obs` against `main` @ `77ba6adf` (post PR #139).

## Scope

Adopt/re-spec TASK-OBS-001, 003, 005, 007, 008, 009 against as-built obs trees. Also close batch/9a-mcp HITL leftover (`reviewing → done`) under the same session override evidence.

## Diff review

| Area | Verdict |
|------|---------|
| Spec grammar task@1 / no `## §N` | Pass — all six lint clean via `task-lint.mjs` |
| Paths `services/obs-*` + `services/shared/cyberos-obs-sdk/` | Pass |
| OBS-005 drops OBS-004 depends_on | Pass — LangSmith correlation ledgered Out of scope |
| OBS-007 Alertmanager wiring | Pass — `deploy/obs/alertmanager-config.yaml` + wiring test |
| OBS-009 manifest format doc | Pass — `docs/manifest-format.md` |
| Deferred surfaces (PDF, macros, chat RED, otelcol supervision) | Pass — Out of scope |
| Gated flips | Pass — Gate-1 + Gate-2 via `backlog-mutate` + `batch-9b-obs-session-hitl.md` |

## Operator notes

1. Accepting review means accepting narrowed as-built ACs (no Loki-as-claimed flat deploy layout; no PDF compliance export; no LangSmith correlation in OBS-005).
2. Operator residual #7 (branch-protection stub checks) remains open from Wave 0.
3. Do **not** merge this PR to main without an explicit operator merge instruction (session allows push + open PR only).
