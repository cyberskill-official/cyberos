# `observability-injection-author` - invariants

Lifted from SKILL.md's normative prose (TASK-SKILL-118 AC 2 discipline: no invariant without a prose source).

1. Instrumentation is injected on the critical paths of THIS FR's diff, not repo-wide.
2. Log lines are structured (key=value / JSON), never bare prints.
3. No PII in log payloads without an explicit redaction policy.
4. The coverage estimate is computed from the artefact's own point list vs branch list - self-consistent.

Enforced at audit time by `observability-injection-audit` per RUBRIC.md (observability_injection_rubric@1.0).
