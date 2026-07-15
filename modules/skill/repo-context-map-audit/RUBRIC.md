# repo_context_map_rubric@1.0

constants: TOTAL_ROWS_MIN=8 (MUST tasks) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207)
families: RCM-GATE | RCM-STRUCT | RCM-TRACE
verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `RCM-STRUCT-001` | artefact carries the three baseline patterns: error_type, logging, test_framework | SKILL.md baseline-patterns clause |
| `RCM-TRACE-001` | every `pinned_in` reference resolves to a real file | SKILL.md pinned_in clause |
| `RCM-GATE-001` | database schemas + type interfaces present when the task declares migrations | SKILL.md schemas clause |
| `RCM-GATE-002` | module-placement warning is null OR escalated to the operator - never silently swallowed | SKILL.md placement-warning clause |
| `RCM-STRUCT-002` | blast-radius estimate present: file count + module count + cross-module edges | SKILL.md blast-radius clause |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail).
Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- repo_context_map_rubric@1.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
