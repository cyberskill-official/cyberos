# Pilot batch 1 — operator runbook (2026-06-11)

Three repos, all prepared in **runner mode**: each carries only `audit-profile.yaml` (the protocol stays single-source in code-audit-framework — every run uses the current release). CONFIG preflights verified clean from the profile path; v1-era artifacts archived as `docs/BACKLOG-2026-06-09-v1era.md` (kymondongiap and 3d-preriodic-table are the ORIGINAL FAILURE_LOG production repos — full circle).

| Repo | Stack | BENCHMARK_MODE | Why |
|---|---|---|---|
| kymondongiap | Python+ephemeris / Vite TS / Supabase | none | the original "no comparator exists" lesson, encoded |
| 3d-preriodic-table | TS / Vite / Three.js / Vitest | none | internal targets only |
| claude-certified-architect-mock-exam | TS / Next.js / Vitest / Playwright | auto | deliberately exercises the R2 cited-benchmark path |

## Per repo (TESTING-PROTOCOL tiers)

1. **Launch** (from the code-audit-framework checkout): `./core/evals/run-audit.sh /path/to/target claude -p` (or run it without an agent command to print the kickoff prompt for any CLI)
2. **Gate**: agent stops after Phase 2 → review backlog → write `Approved: <IDs>` (or `Approved: none`) under the loop heading → relaunch the same prompt (R4 resumes).
3. **T1 validate** after each session: `code-audit-validate --run . --report json > ../code-audit-field-data/reports/<run_id>.json`
4. **T2 rubric** (~10 min): score via core/improve/RETROSPECTIVE.md.
5. **T3 sample**: re-run 3 random MEASURED verify commands, diff outputs.
6. **Record**: `code-audit-validate --run . --emit-feedback --run-id <run_id>` → fill adjudication fields → save to `records/<run_id>.yaml`, commit here.

## T4 (once, this batch)
Run 3d-preriodic-table a second time with a different CLI (e.g. codex exec), same CONFIG; note divergence in the record's `cross_model` field — this is the evidence gate for the queued DEPTH/R2-offline protocol candidates.

## After the batch
From the framework checkout: aggregate (`python3 core/evals/validate.py --aggregate ../code-audit-field-data/reports/*.json`), run the calibration pipeline per core/evals/TESTING-PROTOCOL.md — at most ONE CRITIC cycle.
