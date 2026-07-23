# TASK-IMP-136 — testing-phase evidence (Batch 8B)

Branch: `ship/batch-8b-install-ci-skills`  
Phase: `ready_to_test → testing` (machine gates)  
Date: 2026-07-23  
Gate-1 evidence: `docs/batches/batch-8b-gate1-acceptance.md`  
Status after this pass: **testing** (halted at gate-2; not done)

## Cited suite (TRACE-004)

```
$ bash scripts/tests/test_ci_truth.sh
  ok   t01_caf_gate_wired
  ok   t02_awh_hook_wired_safely
  ok   t03_dead_config_gone
  ok   t04_no_stub_survives
  ok   t05_self_test_negative_paths
  ok   t06_changelog_records_sweep
benchmark-ci-truth: 6 passed, 0 failed
```

## Machine gates

```
suites: pass=49 fail=0 skip=1
PASS  test
PASS  doctor (16/16 OK after MMR rebuild — see batch-8b-ship-notes.md)
GATES: GREEN (machine gates only).
```

Full transcript: `docs/batches/batch-8b-gates-transcript.txt`

## awh + caf

Improvement module has no `.awh/` goldenset — awh N/A per ship-tasks §1a.  
Cross-cutting machine gates + cited suite cover this task. caf for `improvement` N/A (no module audit-profile).

**Halt:** awaiting operator gate-2 (`testing → done`). Do not flip done without gated verdict.
