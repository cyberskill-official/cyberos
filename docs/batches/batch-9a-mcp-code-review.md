---
batch: batch/9a-mcp
members:
  - TASK-MCP-003
  - TASK-MCP-005
  - TASK-MCP-006
  - TASK-MCP-007
  - TASK-MCP-008
started: 2026-07-24T07:00:00Z
ended: null
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch/9a-mcp — code review (pre-HITL)

Reviewed: 2026-07-24 on branch `batch/9a-mcp` against `main` @ `d140b267`.

## Scope

Adopt/re-spec TASK-MCP-003, 005, 006, 007, 008 against as-built `services/mcp-gateway/`. Code mostly pre-existed; this PR closes process drift (FM-004), residual MCP-003 verification, and bash-3.2 CI portability.

## Diff review (section checklist)

| Area | Verdict |
|------|---------|
| Spec grammar task@1 / no `## §N` | Pass — all five lint clean via `task-lint.mjs` |
| Paths `services/mcp-gateway/` | Pass |
| Deferred surfaces ledgered Out of scope | Pass — worker/NATS/drift/EdDSA/rate-limit explicitly not claimed |
| MCP-003 CI script bash 3.2 | Pass — no `mapfile`; live + planted tests green |
| Residual tests | Pass — `sep986_ci_grep_test`, `sep986_audit_emission_test` |
| Status honesty | Pass — tasks at `reviewing`; HITL required before `ready_to_test` / `done` |

## Risks / operator notes

1. **MCP-007/008 adopt does not ship** long_running tools/call async routing, worker pools, NATS, TTL sweepers. Accepting review means accepting the narrowed as-built ACs.
2. **MCP-005** advertises RS256 only (as-built verifier); multi-issuer residency PRM remains Out of scope.
3. **Operator residual #7** (branch-protection stub checks) remains open from Wave 0; not blocking this PR unless Settings CI requires the stubs.

## Gate-1 ask

Operator: accept review for TASK-MCP-003/005/006/007/008 → `reviewing → ready_to_test` (with `--verdict-by` + evidence), then testing gates, then Gate-2 for `done`.
