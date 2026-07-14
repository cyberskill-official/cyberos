---
artefact: implementation-plan@1
task_id: TASK-IMP-068
created: 2026-07-12
estimate_pts: 3
verdict: pass (implementation-plan-audit: every matrix row addressed, patterns from context map respected, estimate sane)
---
# Implementation plan - TASK-IMP-068

Slices (each maps to §1 clauses and matrix rows):
1. check-version-sync.sh - 6 readers, exit contract 0/10/2, tool probes (rows 3,6,7,8,9,12; §1 #1).
2. tests/test_check_version_sync.sh - t01..t10 per FR §5 (all rows).
3. payload-gate.yml - push+PR, 4 path filters, build-into-temp + check (§1 #2, #6; row 11 backstop).
4. build.sh guard - remove `|| echo 0.0.0`, regex-gate VERSION (§1 #3; rows 1,2,4,10).
5. .githooks/pre-commit wrapper - trigger match -> engine rebuild -> check; failure aborts commit (§1 #4).
6. version.yml inline proof step between apply and push (§1 #7).
7. RELEASE.md enforcement wording (§1 #5).

Pattern conformance: exit codes + stderr errors + `cyberos-init:` log prefix per context map. No new dependencies (node, unzip, zip already required by build.sh).
