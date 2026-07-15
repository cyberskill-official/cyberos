# `coverage-gate-author` - invariants

Lifted from SKILL.md's normative prose (TASK-SKILL-118 AC 2 discipline: no invariant without a prose source).

1. The gate certifies testing -> done; spec correctness stays task-audit's job at draft -> ready_to_implement (deliberate phase split, RUBRIC.md §9).
2. Coverage is measured on the task's touched files, never repo-wide averages.
3. Raw tool output ships in the artefact - summaries without the terminal text are unauditable.
4. A pass at <10/10 does not exist; the audit refuses partial credit.

Enforced at audit time by `coverage-gate-audit` per RUBRIC.md (coverage_gate_rubric@1.0).
