# TASK-CUO-302 — testing-phase evidence (Batch 8A)

Branch: `ship/batch-8a-core-locks`  
Phase: `ready_to_test → testing` (machine gates)  
Date: 2026-07-23  
Gate-1 evidence: `docs/batches/batch-8a-gate1-acceptance.md`  
Status after this pass: **testing** (halted at gate-2; not done)

## Cited suite (TRACE-004)

```
$ bash tools/install/tests/test_fail_closed_gates.sh
building scratch payload...
test_fail_closed_gates.sh (TASK-CUO-302)
  ok   t01_empty_floor_exits_red
  ok   t02_red_message_actionable
  ok   t03_ack_line_distinct
  ok   t04_monorepo_fallback_seeds_test_cmd
  ok   t05_header_machine_owned
  ok   t06_changelog_breaking_entry
----
pass=6 fail=0
```

## Module / machine gates

See Batch 8A shared gate transcript in the ship report / `docs/batches/batch-8a-ship-notes.md`.
Final `bash .cyberos/cuo/gates/run-gates.sh` must be GREEN before operator gate-2.

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
