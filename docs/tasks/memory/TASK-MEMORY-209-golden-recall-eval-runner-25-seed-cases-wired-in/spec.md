---
id: TASK-MEMORY-209
title: "Golden recall eval runner + 25 seed cases wired into CI"
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
phase: P0
refs: [R45, F32]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-209: Golden recall eval runner + 25 seed cases wired into CI

## 1. Description

memory-eval binary scores recall@10 and MRR against seed set; CI gate fails on >3% drop

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R45, F32.

## Acceptance criteria

- [ ] memory-eval binary scores recall@10 and MRR against seed set; CI gate fails on >3% drop
