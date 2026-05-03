---
name: Feature Request pickup
about: Claim an existing FR from docs/tasks/ to start implementing
title: "[FR-XXX-NNN] <short title>"
labels: ["fr-pickup"]
assignees: []
---

## FR reference

- File: `docs/tasks/batch-NN/FR-XXX-NNN-<slug>.md`
- Module: `<module>`
- Phase: P0 / P1 / P2 / P3 / P4
- Owner role: <role>

## Plan

<!-- Sub-tasks. Each one is a commit / PR. -->

- [ ] Migration: `migrations/<module>/NNNN_*.sql`
- [ ] Service: resolvers + domain logic in `services/<module>/src/`
- [ ] Frontend: remote in `frontends/<module>-views/src/` (if user-visible)
- [ ] Tests: unit + integration + Gherkin acceptance
- [ ] Anti-regression: extend suite if cross-cutting

## Compliance gate

- [ ] DPIA refresh required? <yes/no — link to FR-CP-001 row>
- [ ] BRAIN denylist additions?
- [ ] Audit events catalog additions?
- [ ] Persona Skill changes?

## Branch

`feat/FR-XXX-NNN-<short-slug>`
