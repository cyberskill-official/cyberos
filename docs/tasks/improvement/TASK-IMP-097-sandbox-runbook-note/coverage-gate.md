---
artefact: coverage-gate@1
task: TASK-IMP-097
phase: testing
tests_failed: 0
files_below_90pct: []
ecm_rows_uncovered: []
---
# TASK-IMP-097 coverage gate

Raw terminal: `bash tools/install/tests/test_full_sdp_payload.sh` -> `pass=9 fail=0` (t09_sandbox_runbook_guide: five greps - heading, local-clone line, local-ref-move clause, hook-replay line, --no-verify - against the scratch payload's GUIDE.md).

| File | Covered by | Coverage |
|---|---|---|
| tools/install/docs/index.md (GUIDE section) | t09_sandbox_runbook_guide | 5 gated phrases |
| modules/cuo/chief-technology-officer/workflows/ship-tasks.md (1 xref line) | AC 2 recorded grep -c = 1 | n/a - single prose line, rationale in spec |

TRACE-004: 1.1+1.3 -> t09_sandbox_runbook_guide passed; 1.2 -> AC 2 ops (grep recorded in gate log; t12's doctrine gate pins the file's normative content). Payload proof: GUIDE.md:129 carries the section; no version bump in this task (099 carries the round's bump). ECM rows uncovered: none.
