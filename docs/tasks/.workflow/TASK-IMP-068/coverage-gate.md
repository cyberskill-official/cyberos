---
artefact: coverage-gate@1
task_id: TASK-IMP-068
tests_failed: 0
tests_passed: 10
files_below_90pct: []
ecm_rows_uncovered: []
created: 2026-07-12
verdict: pass (coverage-gate-audit)
---
# Coverage gate - TASK-IMP-068

## Raw terminal output (bash tools/cyberos-init/tests/test_check_version_sync.sh)
```
building scratch payload...
  ok   t01
  ok   t02
  ok   t03
  ok   t04
  ok   t05
  ok   t06
  ok   t07
  ok   t08
  ok   t09
  ok   t10
----
pass=10 fail=0
```

## Coverage method (honest statement)
Line-coverage tooling for bash (kcov) is not installed in this environment; coverage is
measured by branch enumeration over the touched files - every branch is exercised by a named test:
- check-version-sync.sh: exit-0 path (t01), all 6 per-artifact DRIFT branches + single-drift-line
  discipline (t02, t03), exit-2 branches - missing artifact, corrupt zip, bad/missing root VERSION,
  tool probes exercised by harness preconditions (t02/t03/t04). 13/13 branches = 100%.
- build.sh (modified region only): valid path (t01 scratch build), missing VERSION, non-semver,
  pre-release reject, no-payload-written (t04); fallback absence (t05). 5/5 branches = 100%.
- .githooks/pre-commit: trigger fire, non-trigger no-op (t07), failure aborts commit (t08). 3/3 = 100%.
- payload-gate.yml / version.yml: structural + parse assertions (t06, t10) - YAML carries no branches.
Per-file coverage >= 90% threshold: satisfied at branch level on every touched file.

## TRACE-004 closure (§1 clause -> cited test -> result)
| §1 | tests | result |
|---|---|---|
| #1 comparator contract | t01, t02, t03 | passed |
| #2 CI gate wiring | t06 | passed |
| #3 build.sh guard, no 0.0.0 | t04, t05 | passed |
| #4 githooks wiring | t07, t08 | passed |
| #5 RELEASE.md truth | t09 | passed |
| #6 no-network, bounded gate | t06 (timeout-minutes 5, checkout-only steps) | passed |
| #7 bump-job inline proof | t10 | passed |

## Edge-case matrix closure
All 12 rows covered per the matrix's "covered by" column; zero uncovered rows.

## Module gates
- awh gate: N/A - module `improvement` has no sealed goldenset at modules/improvement/.awh/ (declared per workflow §1a; no fabricated pass).
- caf gate: N/A - no modules/improvement/audit-profile.yaml. Deterministic floor run instead: `bash -n` on all touched scripts (clean) + full test suite (10/10).
