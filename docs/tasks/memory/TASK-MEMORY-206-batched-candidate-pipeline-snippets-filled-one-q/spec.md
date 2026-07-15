---
id: TASK-MEMORY-206
title: "Batched candidate pipeline - snippets filled, one-query verify, set-based access check"
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
refs: [R10, R18, R19, F8, F9]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-206: Batched candidate pipeline - snippets filled, one-query verify, set-based access check

## 1. Description

event hits return non-empty snippets; recall issues O(1) queries per stage, not per candidate; p95 measured before/after

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R10, R18, R19, F8, F9.

## Acceptance criteria

- [ ] event hits return non-empty snippets; recall issues O(1) queries per stage, not per candidate; p95 measured before/after
