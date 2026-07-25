---
artefact: architecture-decision-record@1
adr_id: ADR-004
task_id: TASK-IMP-058
status: accepted
created: 2026-07-25
dec_crosslinks: []
---
# ADR-004: `rules_sha` content cone

## Context

Installed consumers need a content fingerprint of the vendored rules/machine so
post-install drift is detectable. An incomplete cone (or grepping a stored hash
without recompute) false-greens drift (TASK-IMP-122 / related).

## Options considered

1. Fingerprint only a hand-picked file list that drifts from `build.sh` —
   rejected: cone and fingerprint diverge.
2. Cone = exactly what the payload build vendors for rules/machine content;
   installed side recomputes with the same cone helper — CHOSEN.

## Decision

`rules_sha` is defined over the **build cone** (shared helper), and consumers
**recompute** rather than trusting a stale stamped value alone.

## Consequences

- Cone changes are deliberate version bumps, not silent.
- Fleet audit / update-check use the same recompute path.
- Agents changing vendored paths must update the cone helper and tests together.
