# `edge-case-matrix-author` - pipeline

Enumerate null/bounds/malformed/race/security/degradation edge cases before implementation, one row per category-and-trigger with a covering-test pointer.
Artefact: `edge-case-matrix@1`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| repo-context-map-audit (PASS) | Default chain | map path via envelope |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| mock-contract-test-author | Default chain | SECURITY/DEGRADATION rows feed error_modes |
| edge-case-matrix-audit | Default chain | artefact path |

## HALT points

- Task declares MUST priority but fewer than 8 total rows derivable -> HALT rather than pad with vacuous rows.

*Added by TASK-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
