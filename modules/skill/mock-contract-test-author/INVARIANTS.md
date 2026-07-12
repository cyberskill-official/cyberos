# `mock-contract-test-author` - invariants

Lifted from SKILL.md's normative prose (FR-SKILL-118 AC 2 discipline: no invariant without a prose source).

1. The mock satisfies the SAME contract tests the real service must pass - one suite, two implementations.
2. Swap is a one-line import change at swap_target; anything wider is a design failure.
3. Every mocked ship carries the `shipped + mocked-dependency` BACKLOG tag until sunset.
4. Sunset criteria are observable events, never dates alone.

Enforced at audit time by `mock-contract-test-audit` per RUBRIC.md (mock_contract_test_rubric@1.0).
