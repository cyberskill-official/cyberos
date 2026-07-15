---
id: TASK-MEMORY-249
title: "Retrieval config A/B + shadow evaluation on sampled live traffic"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: memory
priority: p1
status: draft
phase: P3
refs: [R48, R54]
depends_on: [TASK-MEMORY-245, TASK-MEMORY-221]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-249: Retrieval config A/B + shadow evaluation on sampled live traffic

## 1. Description

config table with hash assignment; judge-scored outcomes promote winners by flag flip; shadow deltas logged

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R48, R54.

## Acceptance criteria

- [ ] config table with hash assignment; judge-scored outcomes promote winners by flag flip; shadow deltas logged
