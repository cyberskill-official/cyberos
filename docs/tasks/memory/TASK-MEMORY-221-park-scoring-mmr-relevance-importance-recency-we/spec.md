---
id: TASK-MEMORY-221
title: "Park scoring + MMR - relevance/importance/recency weights, access_count, last_accessed_at"
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
phase: P1
refs: [R2, R13, F1, F10]
depends_on: [TASK-MEMORY-219]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-221: Park scoring + MMR - relevance/importance/recency weights, access_count, last_accessed_at

## 1. Description

0.4/0.3/0.3 combined score per TASK-MEMORY-113; MMR lambda 0.7 diversity filter; components normalized and exposed in explain

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R2, R13, F1, F10.

## Acceptance criteria

- [ ] 0.4/0.3/0.3 combined score per TASK-MEMORY-113; MMR lambda 0.7 diversity filter; components normalized and exposed in explain
