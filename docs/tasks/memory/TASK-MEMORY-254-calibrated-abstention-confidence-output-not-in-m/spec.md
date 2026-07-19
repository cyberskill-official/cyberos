---
id: TASK-MEMORY-254
title: "Calibrated abstention - confidence output + not-in-memory behavior + false-memory metric"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p1
status: draft
phase: P3
refs: [R57]
depends_on: [TASK-MEMORY-245]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-254: Calibrated abstention - confidence output + not-in-memory behavior + false-memory metric

## 1. Description

recall exposes calibrated confidence; benchmark tracks abstention bucket; false-memory rate on the ops tile

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R57.

## Acceptance criteria

- [ ] recall exposes calibrated confidence; benchmark tracks abstention bucket; false-memory rate on the ops tile
