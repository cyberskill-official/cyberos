---
id: TASK-MEMORY-230
title: "Bi-temporal l2_edge - valid_at/invalid_at/expired_at + invalidation not deletion"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p1
status: draft
phase: P1
refs: [R7, F5]
depends_on: [TASK-MEMORY-203]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-230: Bi-temporal l2_edge - valid_at/invalid_at/expired_at + invalidation not deletion

## 1. Description

point-in-time edge queries work; contradicting edge invalidates prior with audit; nothing hard-deleted

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R7, F5.

## Acceptance criteria

- [ ] point-in-time edge queries work; contradicting edge invalidates prior with audit; nothing hard-deleted
