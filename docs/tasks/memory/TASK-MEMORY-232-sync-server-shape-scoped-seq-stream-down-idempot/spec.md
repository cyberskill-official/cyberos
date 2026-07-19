---
id: TASK-MEMORY-232
title: "Sync server - shape-scoped seq stream down + idempotent outbox endpoint up"
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
phase: P2
refs: [R60, R65, F25, F26]
depends_on: [TASK-MEMORY-201, TASK-MEMORY-210]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-232: Sync server - shape-scoped seq stream down + idempotent outbox endpoint up

## 1. Description

device pulls only permitted shapes (own subject + grants, shareable sync_class); uploads dedup on event_id; contract documented

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R60, R65, F25, F26.

## Acceptance criteria

- [ ] device pulls only permitted shapes (own subject + grants, shareable sync_class); uploads dedup on event_id; contract documented
