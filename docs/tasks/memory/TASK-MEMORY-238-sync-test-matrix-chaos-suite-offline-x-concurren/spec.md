---
id: TASK-MEMORY-238
title: "Sync test matrix + chaos suite (offline x concurrent x crash x replay)"
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
phase: P2
refs: [R72, R98]
depends_on: [TASK-MEMORY-233, TASK-MEMORY-236]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-238: Sync test matrix + chaos suite (offline x concurrent x crash x replay)

## 1. Description

matrix green in CI; convergence proven; zero duplicate chain rows under chaos; ingest idempotency property-tested

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R72, R98.

## Acceptance criteria

- [ ] matrix green in CI; convergence proven; zero duplicate chain rows under chaos; ingest idempotency property-tested
