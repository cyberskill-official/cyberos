# edge_case_matrix_rubric@1.0

constants: TOTAL_ROWS_MIN=8 (MUST tasks) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207) families: ECM-GATE | ECM-STRUCT | ECM-TRACE verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `ECM-GATE-001` | at least 1 row per category (null/empty, bounds, malformed, race, security, degradation) | SKILL.md category clause |
| `ECM-TRACE-001` | every SECURITY row points at a real test path | SKILL.md security-row clause |
| `ECM-GATE-002` | every DEGRADATION row specifies detection + recovery | SKILL.md degradation clause |
| `ECM-GATE-003` | total_rows >= 8 when the task priority is MUST (TOTAL_ROWS_MIN) | SKILL.md MUST-priority clause |
| `ECM-STRUCT-001` | each row = category + trigger + covering-test pointer; no test-less rows | SKILL.md row grammar |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail). Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- edge_case_matrix_rubric@1.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
