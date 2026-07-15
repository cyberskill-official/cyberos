---
id: TASK-MEMORY-239
title: "Retention policy engine + reaper (per-kind TTLs, archive not delete)"
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
phase: P2
refs: [R81, R32, F22]
depends_on: [TASK-MEMORY-212]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-239: Retention policy engine + reaper (per-kind TTLs, archive not delete)

## 1. Description

versioned+chained policy table; reaper archives below-threshold rows out of serving indexes; L1 untouched inside legal window

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R81, R32, F22.

## Acceptance criteria

- [ ] versioned+chained policy table; reaper archives below-threshold rows out of serving indexes; L1 untouched inside legal window
