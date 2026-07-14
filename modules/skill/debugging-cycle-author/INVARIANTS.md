# `debugging-cycle-author` - invariants

Lifted from SKILL.md's normative prose (TASK-SKILL-118 AC 2 discipline: no invariant without a prose source).

1. One attempt-row per cycle: classify vector (state/network/memory/logic/flake) -> hypothesis -> exact change -> re-run.
2. Hypotheses name mechanisms; 'retry and see' is not a hypothesis.
3. The circuit breaker is not advisory - trip means revert + route back, mechanically.
4. Flake classification requires evidence (same test green on unchanged code), not convenience.

Enforced at audit time by `debugging-cycle-audit` per RUBRIC.md (debugging_cycle_rubric@1.0).
