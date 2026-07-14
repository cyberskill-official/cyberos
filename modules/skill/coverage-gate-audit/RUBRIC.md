# coverage_gate_rubric@1.0

constants: TOTAL_ROWS_MIN=8 (MUST FRs) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable via .cyberos/config.yaml coverage_threshold -> env CYBEROS_COVERAGE_THRESHOLD, TASK-CUO-207)
families: COV-GATE | COV-STRUCT | COV-TRACE
verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `COV-GATE-001` | tests_failed == 0 | SKILL.md gate clause |
| `COV-GATE-002` | files_below_90pct is empty (COVERAGE_THRESHOLD, config-overridable once TASK-CUO-207 lands; default 90 preserved) | SKILL.md threshold clause |
| `COV-GATE-003` | ecm_rows_uncovered is empty (every edge-case-matrix row has a covering test) | SKILL.md ecm-closure clause |
| `COV-STRUCT-001` | raw terminal output of the coverage tool present and non-truncated | SKILL.md raw-terminal clause |
| `COV-TRACE-001` | every §1 clause's cited test from the FR is `passed` in the coverage report (TRACE-004 closure) | SKILL.md §1-closure clause / audit TRACE-004 |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail).
Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- coverage_gate_rubric@1.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
