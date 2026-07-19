---
artefact: observability-injection@1
task_id: TASK-IMP-115
workflow: chief-technology-officer/ship-tasks
step: 11-12
branches_total: 0
branches_instrumented: 0
coverage_pct: n/a
---

# Observability injection — TASK-IMP-115

## Critical-path walk

The deliverable is **one enum key per `skill_chain` line plus a doctrine section**. There
is no new code, no state transition, no external IO, and no error branch:

| Touched file | Kind | Runtime branches added | Log / trace / metric points required |
|---|---|---|---|
| `modules/cuo/chief-technology-officer/workflows/ship-tasks.md` | markdown + YAML frontmatter | 0 | 0 |
| `modules/cuo/tests/test_workflow_evolution.py` | test suite | 0 (test code) | 0 |

`≥80 % of branches carry a log/metric/trace point` (§6 of this workflow) is **vacuously
satisfied at 0/0** — and that is recorded here as the finding, not hidden as a pass. A
zero denominator is not evidence of instrumentation; it is evidence there is nothing to
instrument.

## The observability that DOES exist for this change

The field's only failure mode is drift (edge-case matrix E16/E17): the label stops matching
the work. Drift is not observable at runtime, because **nothing reads the field at runtime**
— that is §1.3's design, not a gap. The detector is therefore the suite, which re-proves on
every CI run that:

- every step still carries a value from the closed enum (`test_every_step_has_judgment`);
- every `mechanical` claim still resolves to a helper that exists on disk and is still the
  helper the payload names for that skill (`test_mechanical_steps_are_helper_backed`);
- no model string, price, or host effort name entered via this field
  (`test_no_host_specific_literals`);
- nothing in the payload started reading the key (`test_judgment_is_advisory_not_read`).

## Audit note (step 12)

No instrumentation added, none required, and the reason is stated rather than asserted. The
existing `audit_hooks` and `memory` rows on the chain are untouched: this task adds no step
and emits no new row.
