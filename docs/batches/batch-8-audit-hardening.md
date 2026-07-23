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
ended: 2026-07-23T12:45:00+07:00
route_backs: 0
gate_reasks: 0
tokens: unknown
---
# batch 8 — audit hardening (final sequential pass)

Closes the gap between CyberOS doctrine and enforcement: fail-closed gates, mechanical HITL, skill quality floor, memory contract hardening (plan only for live-store repair), CI/CAF truth, install portability, corpus hygiene (mechanical half), and benchmark-gate checkers. Phase 1 polish (`d3652a5b`) and task authoring (`be89966b`) were already on the branch; this pass landed the six workers' uncommitted implementations, applied missing ledger items, verified, and advanced statuses to the HITL-safe cells below.

## Member status after this pass

| ID | Task | Status | Why this cell |
|---|---|---|---|
| T1 | TASK-CUO-302 | **done** | Gate-2 all-accept 2026-07-23 (`batch-8a-gate2-acceptance.md`) |
| T2 | TASK-CUO-303 | **done** | Gate-2 all-accept 2026-07-23 (`batch-8a-gate2-acceptance.md`) |
| T3 | TASK-CUO-304 | **done** | Gate-2 all-accept 2026-07-23 (`batch-8a-gate2-acceptance.md`) |
| T4 | TASK-IMP-136 | **reviewing** | CAF root CI + awh hook + stub sweep + `test_ci_truth` |
| T5 | TASK-SKILL-202 | **reviewing** | NFR delist, untrusted backport, stub lint 7/7 |
| T6 | TASK-MEMORY-303 | **implementing** | Code landed; **live store repair is operator-gated** |
| T7 | TASK-IMP-137 | **reviewing** | MCP loopback+token, shasum, atomic vendor, engines |
| T8 | TASK-IMP-138 | **ready_to_implement** | Operator fork — not implemented this wave |
| T9 | TASK-IMP-139 | **implementing** | Mechanical half done; **Gate-1/Gate-2 operator-gated** |
| P3 | TASK-IMP-140 | **implementing** | Checkers + risk rows landed; **BRAIN record deferred on T6** |

Hard boundaries observed: no push/merge/deploy; HITL gates crossed only with recorded verdicts; Batch A (CUO-302/303/304) closed done on 2026-07-23; live BRAIN store repair deferred to MEMORY-303 operator item; no `# UNREVIEWED` marker cleared; TASK-IMP-138 decision pending Batch D; hooks never bypassed.

## Verification (verbatim summary)

| Check | Result |
|---|---|
| `cd modules/cuo && python3 -m pytest -q` | **274 passed**, 2 skipped |
| `cd modules/memory && python3 -m pytest -q` | **522 passed**, 5 skipped (≥519) |
| `bash scripts/tests/run_all.sh` | **suites: pass=49 fail=0 skip=1** |
| `bash tools/install/build.sh` + pair-parity | **skills=52**, parity OK 25/25 |
| `bash scripts/tests/test_benchmark_gates.sh` (no SKIP_HEAVY) | **7 passed, 0 failed** |
| `bash scripts/tests/test_ci_truth.sh` | **6 passed, 0 failed** |
| Scratch install fail-closed | exit **3**, `GATES: RED - EMPTY FLOOR` |
| Scratch `CYBEROS_ALLOW_EMPTY_GATES=1` | exit **0**, `EMPTY-ACKNOWLEDGED` (no GREEN) |
| Scratch doctor SKIP | `SKIP doctor (no memory store…)` then GREEN with seeded test |
| Live `bash .cyberos/cuo/gates/run-gates.sh` (no reinstall) | **GATES: GREEN** |
| `bash scripts/check_doc_anchors.sh` | exit **0** (historical WARNs only) |

Note: the live installed `run-gates.sh` was **not** refreshed (ordering: store repair before doctor wiring on this repo). Scratch payload install proves the new gate script.

## OPERATOR DECISION LIST

Record a dated verdict in chat (or the owning task's `source_decisions`) before any follow-up that mutates markers, statuses past HITL, or the live BRAIN.

1. **TASK-MEMORY-303 — live-store layout repair** — **APPROVED NOW** (operator 2026-07-23: "yes repair after A"; A closed). Execute `store-repair-plan.md` on `ship/batch-8c-memory`; re-measure hashes at execution.

2. **TASK-IMP-139 Gate-1 — UNREVIEWED fork?**  
   Brief: `…/TASK-IMP-139-…/assets/unreviewed-fork-brief.md` (167 non-draft files / 333 marker lines). Choose **clear** (recommended, with TASK-EVAL-001 carve-out) vs **re-audit wave**. Record verdict in the IMP-139 spec **before** any marker-touching commit.

3. **TASK-IMP-139 Gate-2 — 12 stuck-`implementing` verdicts?**  
   Dossiers under `…/assets/reconcile/`. Recommended tally: **11 route_back · 1 resume (APP-001) · 0 on_hold**. Apply only through the standard override path after recording per-task verdicts.

4. **TASK-IMP-138 — entry-point identity fork?**  
   Thin AGENTS.md spine vs explicit dual-identity pointers. Structural; stays `ready_to_implement` until chosen.

5. **TASK-IMP-140 — BRAIN recording after T6 repair?**  
   Run `docs/tasks/improvement/TASK-IMP-140-…/brain-record.sh` only when doctor reports READY. Checklist in the same folder. Final acceptance for IMP-140 includes the executed recording.

6. **CAF B17/B18 regressions (surfaced by TASK-IMP-136)?**  
   `validate.py --all` is RED at HEAD on two expected-fail fixtures that now pass (H9 demonstrating itself). Schedule a CAF self-improvement cycle; do not weaken fixtures. First `caf-evals-gate` CI run will be honestly RED until fixed.

7. **Branch protection before merge?**  
   Confirm the 9 deleted stub workflows are not required checks (`stub-disposition.md` operator steps).

8. **Refresh this repo's `.cyberos/` after store repair?**  
   After (1): `CYBEROS_SYNC_HOST_PLUGINS=0 bash tools/install/build.sh && CYBEROS_OFFLINE=1 bash dist/cyberos/install.sh .` so the vendored gate script + doctor gate activate without freezing sibling runs.

## Final-pass ledger notes

Prior aborted final-pass agents had already applied most of A–H. This pass confirmed and closed remaining gaps: OBS-001 live-token wording softened; OBS-009 dossier dead `docs/manifest-format.md` citation fixed; G5 structural exclusions extended; suite fallout from Unreleased CHANGELOG / NFR delist / FM-117 fixtures repaired. No `# UNREVIEWED` markers touched; no store repair executed.
