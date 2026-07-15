---
id: TASK-MEMORY-205
title: "Fix dead recall confidence floor (use real cosine, not constant 1.0)"
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
refs: [R9, F7]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-205: Fix dead recall confidence floor (use real cosine, not constant 1.0)

## 1. Description

weak summary match triggers drill; regression test pins the behavior

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R9, F7.

## Acceptance criteria

- [ ] weak summary match triggers drill; regression test pins the behavior

## Implementation status (reconciled 2026-07-08)

Implemented on branch `auto/memory-enterprise` (commit 33f4ea8, migrated from `MEM-005`). The code exists in `services/memory/`; author this task's section-1 clauses from the implementation and source report before it moves through the review/test gates.
