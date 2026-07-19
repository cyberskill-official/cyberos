---
id: TASK-MEMORY-202
title: "Fail-closed RLS on brain tables (drop NULL-GUC arm + nil-uuid bypass)"
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
priority: p0
status: draft
phase: P0
refs: [R74, F16]
depends_on: []
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-202: Fail-closed RLS on brain tables (drop NULL-GUC arm + nil-uuid bypass)

## 1. Description

query without app.tenant_id returns zero rows; admin paths use dedicated role, not magic uuid

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R74, F16.

## Acceptance criteria

- [ ] query without app.tenant_id returns zero rows; admin paths use dedicated role, not magic uuid

## Implementation status (reconciled 2026-07-08)

Implemented on branch `auto/memory-enterprise` (commit 2441825, migrated from `MEM-002`). The code exists in `services/memory/`; author this task's section-1 clauses from the implementation and source report before it moves through the review/test gates.
