---
id: FR-MEMORY-233
title: "Tauri SQLite client - synced/local tables, combining views, changes outbox, sync worker"
module: memory
priority: MUST
status: draft
class: improvement
phase: P2
refs: [R61, R63]
depends_on: [FR-MEMORY-232]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-233: Tauri SQLite client - synced/local tables, combining views, changes outbox, sync worker

## 1. Description

offline reads+writes work; LWW-with-revision conflicts; 409 rebase path tested; supervisor spawns the Rust worker, not python

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R61, R63.

## Acceptance criteria

- [ ] offline reads+writes work; LWW-with-revision conflicts; 409 rebase path tested; supervisor spawns the Rust worker, not python
