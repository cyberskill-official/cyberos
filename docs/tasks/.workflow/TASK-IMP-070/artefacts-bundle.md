---
artefacts: repo-context-map@1 + edge-case-matrix@1 + implementation-plan@1 + observability-injection@1 + coverage-gate@1 + code-review@1 (bundled)
task_id: TASK-IMP-070
tests_failed: 0
tests_passed: 8
files_below_90pct: []
ecm_rows_uncovered: []
created: 2026-07-12
verdicts: all pass - human verdict pending at HITL gate 1
---
# Ship artefact bundle - TASK-IMP-070

## Context map
Patterns per TASK-IMP-068/069 lineage. files_outside_immediate_domain: 0 -> no ADR. has_external_dependency: false (GitHub API consumed best-effort at runtime with a hermetic file-path override for tests; never a hard dependency - offline is first-class).

## Edge-case matrix (9 rows, all covered)
null/empty: empty endpoint response -> unknown (t03); repo without .cyberos -> installed=none, repo_stale (matrix in t04B family). bounds: 1.10.0 vs 1.9.0 numeric (t05); installed > latest never advises downgrade (up_to_date fall-through, t04A logic). malformed: HTML/garbage endpoint -> regex-gated unknown (t03); pre-release tag rejected by X.Y.Z regex (code path, same gate as t01/t02). concurrency: none (read-only, single line out). SECURITY: endpoint value echoed verbatim but never executed or eval'd; offline flag cannot be overridden by endpoint (t06). DEGRADATION: curl timeout 3s -> unknown -> local-only verdict + note (t03, t04D).

## Coverage (branch enumeration)
check-latest.sh: offline-early, file vs curl branch, bare/JSON/garbage parse, found/unknown output. 7/7 (t01, t02, t03, t06). install.sh --check region: latest resolution + skip, three-value output, all 4 verdict branches + note, is_ver/ver_lt comparator, resolver-absent fallback ([-f check-latest.sh] guard). 9/9 (t03, t04A-D, t05). Docs: t07, t08. All prior suites green as regression (10/10, 6/6, 9/9).

## §1 -> tests
#1 resolver contract: t01, t02, t03, t06 | #2 three values + verdicts + next: t04 | #3 numeric semver: t05 | #4 update.md: t07 | #5 changelog.md: t08 | #6 offline first-class: t03, t04D, t06 | #7 read-only: t04 fixtures unmodified (mkrepo-only writes).

## Deviations
1. build.sh gains one copy line (check-latest.sh vendored beside install.sh) so payload-run checks can resolve latest - added to keep the resolver available where install.sh actually runs; older payloads degrade gracefully via the [-f] guard.
2. Legacy `CyberOS: installed=X available=Y` line replaced by the machine-parseable three-value format (spec-mandated). TASK-APP-001's Ops tab displays raw output per its own clause 3, so it inherits the richer report.

## Verdict requested
Review acceptance (HITL gate 1).
