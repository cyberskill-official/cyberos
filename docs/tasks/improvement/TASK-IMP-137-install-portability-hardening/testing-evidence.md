# TASK-IMP-137 — testing-phase evidence (Batch 8B)

Branch: `ship/batch-8b-install-ci-skills`  
Phase: `ready_to_test → testing` (machine gates)  
Date: 2026-07-23  
Gate-1 evidence: `docs/batches/batch-8b-gate1-acceptance.md`  
Status after this pass: **testing** (halted at gate-2; not done)

## Cited suite (TRACE-004)

```
$ bash tools/install/tests/test_install_portability.sh
test_install_portability.sh (TASK-IMP-137)
  ok   t01_loopback_default
  ok   t02_bearer_token_enforced
  ok   t03_shasum_fallback_verifies
  ok   t04_engines_unified
  ok   t05_ci_channel_real
  ok   t06_atomic_swap_no_reader_gap
  ok   t07_changelog_five_changes
----
pass=7 fail=0
```

## Machine gates

```
suites: pass=49 fail=0 skip=1
PASS  test
PASS  doctor (16/16 OK)
GATES: GREEN (machine gates only).
```

Full transcript: `docs/batches/batch-8b-gates-transcript.txt`

## awh + caf

Improvement module has no `.awh/` goldenset — awh N/A per ship-tasks §1a.

**Halt:** awaiting operator gate-2 (`testing → done`). Do not flip done without gated verdict.
