# `repo-context-map-author` - pipeline

Deep-scan the repo before code generation: existing patterns, schemas, blast radius, module-placement check. Artefact: `repo-context-map@1`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| (ship step 0 - task selected) | Default chain | task path via workflow |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| edge-case-matrix-author | Default chain | map informs matrix rows |
| repo-context-map-audit | Default chain | artefact path |

## HALT points

- Module-placement warning non-null and not resolvable from task text -> HALT for operator (escalate, never guess).

*Added by TASK-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
