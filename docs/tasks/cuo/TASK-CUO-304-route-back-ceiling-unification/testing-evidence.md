# TASK-CUO-304 — testing-phase evidence (Batch 8A)

Branch: `ship/batch-8a-core-locks`  
Phase: `ready_to_test → testing` (machine gates)  
Date: 2026-07-23  
Gate-1 evidence: `docs/batches/batch-8a-gate1-acceptance.md`  
Status after this pass: **testing** (halted at gate-2; not done)

## Cited suite (TRACE-004)

```
$ cd modules/cuo && python3 -m pytest -q tests/test_doctrine_constants.py
.....                                                                    [100%]
5 passed in 0.15s
```

Full CUO module suite (guardrail):

```
$ cd modules/cuo && python3 -m pytest -q
274 passed, 2 skipped in 6.76s
```

## Module / machine gates

See Batch 8A shared gate transcript. Final `run-gates.sh` must be GREEN before operator gate-2.

## Machine gates (verbatim summary)

```
suites: pass=49 fail=0 skip=1
PASS  test
GATES: GREEN (machine gates only).
```

Full transcript: `docs/batches/batch-8a-gates-transcript.txt`

## awh + caf (ship-tasks steps 28–29)

```
[eval] 2 tasks x 1 seeds | weighted pass@1=100.0% | mean pass^1=100.0% | fully-consistent 2/2
PASS: no task regressed > 0.0%; aggregate +0.0% (base 100.0% -> 100.0%).
awh_rc=0

[caf] cuo -> CLEAN
caf_rc=0
```

Full transcript: `docs/batches/batch-8a-awh-caf-transcript.txt`

**HALT:** gate-2 (`testing → done`) requires operator ACCEPT. Agent will not flip done.
