# debugging_cycle_rubric@1.0

constants: TOTAL_ROWS_MIN=8 (MUST tasks) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207) families: DBG-GATE | DBG-STRUCT | DBG-TRACE verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `DBG-GATE-001` | budget compliance: attempts <= 5 consecutive failures before the circuit breaker trips | SKILL.md circuit-breaker clause |
| `DBG-STRUCT-001` | every attempt-row carries a non-vacuous hypothesis (names a mechanism, not 'maybe fix X') | SKILL.md hypothesis clause |
| `DBG-TRACE-001` | every attempt-row's file:line references resolve | SKILL.md file:line clause |
| `DBG-GATE-002` | circuit-breaker arithmetic correct: consecutive counter resets on a green run | SKILL.md arithmetic clause |
| `DBG-STRUCT-002` | the trace ends in a defined resolution: fixed | circuit_broken | escalated | SKILL.md resolution clause |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail). Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- debugging_cycle_rubric@1.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
