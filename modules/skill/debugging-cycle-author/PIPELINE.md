# `debugging-cycle-author` - pipeline

Multi-vector debugging when the coverage gate trips: classify the failure vector, state a targeted hypothesis + exact file:line change, re-run, one attempt-row per cycle, circuit-break at 5. Artefact: `debug-trace@1`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| coverage-gate-author (tests_failed > 0 or thresholds missed) | Default chain | coverage artefact |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| debugging-cycle-audit | Default chain | trace path |
| coverage-gate-author (re-run) | Default chain | after each fix attempt |

## HALT points

- 5 consecutive failed cycles -> circuit breaker: revert touched paths, route task back to ready_to_implement, do NOT continue.

*Added by TASK-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
