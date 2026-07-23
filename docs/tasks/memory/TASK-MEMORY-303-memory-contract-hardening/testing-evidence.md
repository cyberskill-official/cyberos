# TASK-MEMORY-303 — testing-phase evidence (Batch 8C)

Branch: `ship/batch-8c-memory`  
Phase: `ready_to_test → testing` (machine gates)  
Date: 2026-07-23  
Gate-1 evidence: `docs/batches/batch-8c-gate1-acceptance.md`  
Status after this pass: **testing** (halted at gate-2; not done)

## Cited suites (TRACE-004)

```
$ python3 -m pytest modules/memory/tests/test_schema_single_source.py \
    modules/memory/tests/test_interop_doc.py \
    modules/memory/tests/test_walker_sessions_dreams.py \
    modules/memory/tests/test_session_id_stamping.py \
    modules/memory/tests/test_schema_drift.py -q
.............                                                            [100%]
13 passed in 0.74s

$ bash tools/install/tests/test_doctor_gate.sh
test_doctor_gate.sh (TASK-MEMORY-303 #1.6 carve-out)
  ok   t01_doctor_gate_three_states
  ok   t02_cli_absent_skips
  ok   t03_real_doctor_when_available
----
pass=3 fail=0
```

Live store post-repair (also after gate-1 `status_overridden` + MMR catch-up):

```
$ python3 -m cyberos doctor
total: 16  pass: 16  warn: 0  error: 0
overall: OK

$ python3 -m cyberos verify
verified 13 records across 1 segment(s); chain intact
```

## Machine gates

```
suites: pass=49 fail=0 skip=1
PASS  test
PASS  doctor (16/16 OK)
GATES: GREEN (machine gates only).
```

Full transcript: `docs/batches/batch-8c-gates-transcript.txt` (re-run after gate-1; prior repair transcript retained in git history).

## awh + caf (ship-tasks steps 28–29)

```
[eval] 3 tasks x 1 seeds | weighted pass@1=100.0% | mean pass^1=100.0% | fully-consistent 3/3
PASS: no task regressed > 0.0%; aggregate +0.0% (base 100.0% -> 100.0%).
awh_memory_rc=0

[caf] memory -> CLEAN
caf_memory_rc=0
```

Full transcript: `docs/batches/batch-8c-awh-caf-transcript.txt`

**Halt:** awaiting operator gate-2 (`testing → done`). Do not flip done without gated verdict.  
IMP-138 thin-spine not in scope this turn.
