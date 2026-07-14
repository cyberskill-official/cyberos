---
id: TASK-MEMORY-238
title: "Sync test matrix + chaos suite (offline x concurrent x crash x replay)"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R72, R98]
depends_on: [TASK-MEMORY-233, TASK-MEMORY-236]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-238: Sync test matrix + chaos suite (offline x concurrent x crash x replay)

## 1. Description

matrix green in CI; convergence proven; zero duplicate chain rows under chaos; ingest idempotency property-tested

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R72, R98.

## Acceptance criteria

- [ ] matrix green in CI; convergence proven; zero duplicate chain rows under chaos; ingest idempotency property-tested
