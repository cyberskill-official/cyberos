---
batch: batch/8-audit-hardening
members:
  - TASK-CUO-302
  - TASK-CUO-303
  - TASK-CUO-304
  - TASK-IMP-136
  - TASK-SKILL-202
  - TASK-MEMORY-303
  - TASK-IMP-137
  - TASK-IMP-138
  - TASK-IMP-139
  - TASK-IMP-140
started: 2026-07-23T00:00:00Z
ended: 2026-07-23T18:23:36Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch 8 — audit hardening (CLOSED — Wave 0 complete)

Closes the gap between CyberOS doctrine and enforcement: fail-closed gates, mechanical HITL, skill quality floor, memory contract hardening, CI/CAF truth, install portability, corpus hygiene, benchmark gates G1–G16, and the thin-spine entry point. Merged to `main` as [PR #132](https://github.com/cyberskill-official/cyberos/pull/132) → commit `1d8da66e` (released as **1.2.0**). Wave 0 chores complete 2026-07-25 (decision #7 closed; see operator decision list).

## Member status (final)

| ID | Task | Status | Evidence |
|---|---|---|---|
| T1 | TASK-CUO-302 | **done** | `batch-8a-gate2-acceptance.md` |
| T2 | TASK-CUO-303 | **done** | `batch-8a-gate2-acceptance.md` |
| T3 | TASK-CUO-304 | **done** | `batch-8a-gate2-acceptance.md` |
| T4 | TASK-IMP-136 | **done** | `batch-8b-gate2-acceptance.md` |
| T5 | TASK-SKILL-202 | **done** | `batch-8b-gate2-acceptance.md` |
| T6 | TASK-MEMORY-303 | **done** | `batch-8c-gate2-acceptance.md` |
| T7 | TASK-IMP-137 | **done** | `batch-8b-gate2-acceptance.md` |
| T8 | TASK-IMP-138 | **done** | Batch F / Branch A thin spine |
| T9 | TASK-IMP-139 | **done** | Batch D (bulk-clear + reconcile) |
| P3 | TASK-IMP-140 | **done** | Batch E (checkers + BRAIN record) |

## Sub-batch branches (merged via `ship/batch-8f-entrypoint`)

| Branch | Scope |
|--------|--------|
| `ship/batch-8a-core-locks` | CUO-302/303/304 |
| `ship/batch-8b-install-ci-skills` | IMP-136/137, SKILL-202 |
| `ship/batch-8c-memory` | MEMORY-303 |
| `ship/batch-8-integrate` | merge A+B+C |
| `ship/batch-8d-corpus` | IMP-139 |
| `ship/batch-8e-benchmarks` | IMP-140 |
| `ship/batch-8f-entrypoint` | IMP-138 (PR tip) |

Remote tip `origin/ship/batch-8f-entrypoint` deleted 2026-07-23 post-merge (Wave 0 cleanup).

## Operator decision list (1–8 executed)

1. MEMORY-303 live-store repair — **done**
2. IMP-139 UNREVIEWED bulk-clear (+ EVAL-001 carve-out) — **done**
3. IMP-139 Gate-2 reconcile (11 route_back + APP-001 resume) — **done**
4. IMP-138 Branch A thin spine — **done**
5. IMP-140 BRAIN recording — **done**
6. CAF B17/B18 — **fixed on main** (`46ceb8b4`); historical ledger warning obsolete
7. Branch protection vs deleted stubs — **closed 2026-07-25** (owner: Stephen Cheng; re-verified 2026-07-25). Classic `/branches/main/protection` is absent (404). Active ruleset `AWH gate` (id 17883269) requires only context `awh-gate`; none of the nine deleted stub workflow contexts are required. Stub workflow files remain absent under `.github/workflows/`. Nothing to scrub; do not invent a ruleset.
8. `.cyberos/` refresh after store repair — **done** on batch-8c

## Follow-ups (post-1.2.0)

See plan `post-1.2.0_next_steps` / batch-9 schedule: MMR sync for `memory-append`, TASK-MEMORY-302, ship-tasks evolution, MCP/OBS resume wave, 1.4.x / 1.5.0 drafts (stay on 1.x). Wave 0 complete 2026-07-25: `ship/batch-8*` pruned, decision #7 confirmed (ruleset requires only `awh-gate`), ledger closed, `scripts/awh_finalize.sh` absent.
