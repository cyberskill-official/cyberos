---
id: TASK-MEMORY-253
title: "Poisoning defenses - source trust scores, quarantine, quoted-data prompt wrapping, red-team suite in CI"
module: memory
priority: MUST
status: draft
class: improvement
phase: P3
refs: [R79, R80, R58, F24]
depends_on: [TASK-MEMORY-213]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-253: Poisoning defenses - source trust scores, quarantine, quoted-data prompt wrapping, red-team suite in CI

## 1. Description

MINJA-style CI cases end quarantined/de-ranked; retrieved memory never interpolated as instructions; trust decays ranking

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R79, R80, R58, F24.

## Acceptance criteria

- [ ] MINJA-style CI cases end quarantined/de-ranked; retrieved memory never interpolated as instructions; trust decays ranking
