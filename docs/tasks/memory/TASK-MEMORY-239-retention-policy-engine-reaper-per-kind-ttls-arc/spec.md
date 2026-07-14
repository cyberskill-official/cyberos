---
id: TASK-MEMORY-239
title: "Retention policy engine + reaper (per-kind TTLs, archive not delete)"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R81, R32, F22]
depends_on: [TASK-MEMORY-212]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-239: Retention policy engine + reaper (per-kind TTLs, archive not delete)

## 1. Description

versioned+chained policy table; reaper archives below-threshold rows out of serving indexes; L1 untouched inside legal window

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R81, R32, F22.

## Acceptance criteria

- [ ] versioned+chained policy table; reaper archives below-threshold rows out of serving indexes; L1 untouched inside legal window
