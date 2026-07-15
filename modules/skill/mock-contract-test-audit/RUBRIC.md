# mock_contract_test_rubric@1.0

constants: TOTAL_ROWS_MIN=8 (MUST tasks) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207)
families: MCT-GATE | MCT-STRUCT | MCT-TRACE
verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `MCT-STRUCT-001` | artefact carries >= 1 request_response_pair | SKILL.md contract clause |
| `MCT-GATE-001` | error_modes cover every SECURITY and DEGRADATION matrix row of the task | SKILL.md error-modes clause |
| `MCT-TRACE-001` | swap_target is a real symbol in the codebase | SKILL.md swap-target clause |
| `MCT-GATE-002` | sunset_criterion has an observable trigger | SKILL.md sunset clause |
| `MCT-GATE-003` | contract tests pass against the Mock at authoring time | SKILL.md tests-pass clause |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail).
Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- mock_contract_test_rubric@1.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
