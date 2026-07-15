---
id: TASK-MEMORY-203
title: "RLS with FORCE on l1_audit_log, l2_memory, l2_entity, l2_edge + cross-tenant probe"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
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
refs: [R75, R76, F17]
depends_on: [TASK-MEMORY-202]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-203: RLS with FORCE on l1_audit_log, l2_memory, l2_entity, l2_edge + cross-tenant probe

## 1. Description

RLS property test covers all memory tables; probe reads zero cross-tenant rows; query plans still index-driven

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R75, R76, F17.

## Acceptance criteria

- [ ] RLS property test covers all memory tables; probe reads zero cross-tenant rows; query plans still index-driven
