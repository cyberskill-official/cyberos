# `coverage-gate-author` - pipeline

Run the suite + measure coverage on the FR's touched files; the artefact certifies (or blocks) the testing -> done transition.
Artefact: `coverage-gate@1`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| implementation + observability phases complete | Default chain | git diff since implementing |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| coverage-gate-audit | Default chain | artefact path |
| debugging-cycle-author | Default chain | fires when tests_failed > 0 or thresholds missed |

## HALT points

- Coverage tooling absent in target repo and no gates.env COVERAGE_CMD -> HALT: operator wires the command (reduced-profile floor).

*Added by TASK-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
