# observability_injection_rubric@1.0

constants: TOTAL_ROWS_MIN=8 (MUST tasks) | BRANCH_COVERAGE_MIN=80 | COVERAGE_THRESHOLD=90 (config-overridable, TASK-CUO-207)
families: OBS-GATE
verdict: pass requires 10/10; any family failure -> fail; ambiguity -> needs_human

## Rules (prose -> rule mapping, TASK-SKILL-118 AC 2)

Every rule cites the prose gate it encodes. A rule without a prose source is itself a review finding (TASK-SKILL-118 §10 #1).

| rule_id | gate | prose source |
|---|---|---|
| `OBS-GATE-001` | every state transition carries >= 1 structured log point (tenant_id + subject_id when in scope) | SKILL.md log-point clause |
| `OBS-GATE-002` | every external IO call is wrapped by >= 1 trace span | SKILL.md span clause |
| `OBS-GATE-003` | every error branch increments >= 1 error counter | SKILL.md counter clause |
| `OBS-GATE-004` | branch_coverage estimate >= 80% (BRANCH_COVERAGE_MIN) | SKILL.md coverage clause |
| `OBS-GATE-005` | redaction_policy present whenever PII is in scope | SKILL.md redaction clause |

## Scoring

/10 overall. Start at 10; each open finding subtracts per severity (blocker -2, major -1, minor -0.5, rounded toward fail).
Only 10/10 passes. `needs_human` on structural ambiguity (unparseable artefact, contradictory sources) - never a guessed verdict.

## Changelog

- observability_injection_rubric@1.0: initial file-form of the gates already normative in SKILL.md prose (TASK-SKILL-118; no bar raised, no bar lowered).
