---
id: TASK-MEMORY-236
title: "Offline capture outbox + hydration (hot window snapshot, cold on demand)"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R67, R68, R69]
depends_on: [TASK-MEMORY-233]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-236: Offline capture outbox + hydration (hot window snapshot, cold on demand)

## 1. Description

offline-created events chain exactly once after reconnect; fresh device hydrates 30d + profiles fast; conflicts.py marked legacy

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R67, R68, R69.

## Acceptance criteria

- [ ] offline-created events chain exactly once after reconnect; fresh device hydrates 30d + profiles fast; conflicts.py marked legacy
