# `observability-injection-author` - pipeline

Walk the implementation's critical paths and emit structured log points, trace spans, and error counters with a branch-coverage estimate.
Artefact: `observability-injection@1`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| implementation (ship steps 8-14) | Default chain | touched-files diff |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| observability-injection-audit | Default chain | artefact path |
| coverage-gate-author | Default chain | instrumented branches inform coverage read |

## HALT points

- PII in scope but no redaction policy derivable from repo conventions -> HALT for operator policy decision.

*Added by FR-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
