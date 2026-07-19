---
id: TASK-MEMORY-233
title: "Tauri SQLite client - synced/local tables, combining views, changes outbox, sync worker"
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
refs: [R61, R63]
depends_on: [TASK-MEMORY-232]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-233: Tauri SQLite client - synced/local tables, combining views, changes outbox, sync worker

## 1. Description

offline reads+writes work; LWW-with-revision conflicts; 409 rebase path tested; supervisor spawns the Rust worker, not python

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R61, R63.

## Acceptance criteria

- [ ] offline reads+writes work; LWW-with-revision conflicts; 409 rebase path tested; supervisor spawns the Rust worker, not python
