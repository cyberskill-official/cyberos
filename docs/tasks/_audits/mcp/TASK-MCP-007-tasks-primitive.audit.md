---
task_id: TASK-MCP-007
audited: 2026-07-24
verdict: PASS
score: 10/10
template: task@1
adopt: batch/9a-mcp
entered_via: rework
---

# TASK-MCP-007 audit — Tasks primitive (batch/9a-mcp adopt)

## Verdict

**PASS 10/10** (2026-07-24). Spec honestly adopts `tasks.rs` / `tasks_pg` / migration 0017 lifecycle. Worker pool, NATS, checkpoints, and tools/call async routing remain Out of scope (matching source comments).

## What was checked

| Check | Result |
|-------|--------|
| No `## §N` headings | Pass |
| task@1 sections + grafted AC/Verification | Pass (7 ACs) |
| Paths under mcp-gateway | Pass |
| Deferred surfaces not claimed as shipped | Pass |

## Findings

None open. Largest prior verification gap (phantom `task_*` tests) closed by citing real in-crate tests only.

## Notes for HITL

- Accepting this adopt does **not** ship long_running tools/call routing; that remains a follow-up.
- Do not flip `done` without the two human-acceptance gates.

**Score = 10/10.**

---

*End of TASK-MCP-007 audit (batch/9a-mcp adopt).*
