# `mock-contract-test-author` - pipeline

When an FR declares a missing external dependency: pin the expected request/response contract, a mock that satisfies it today, and contract tests the real service passes tomorrow.
Artefact: `mock-contract-test@1`. This document binds the skill into the ship chain; the step semantics live in SKILL.md.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| edge-case-matrix-audit (PASS) | Default chain | SECURITY/DEGRADATION rows -> error_modes |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| implementation (ship steps 8-14) | Default chain | mock import path |
| mock-contract-test-audit | Default chain | artefact path |

## HALT points

- Dependency turns out to exist and be reachable -> HALT: the FR's has_external_dependency flag is wrong; operator corrects the FR.

*Added by TASK-SKILL-118 (contract parity). Phases and step prose: SKILL.md is normative.*
