---
id: TASK-MEMORY-230
title: "Bi-temporal l2_edge - valid_at/invalid_at/expired_at + invalidation not deletion"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R7, F5]
depends_on: [TASK-MEMORY-203]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-230: Bi-temporal l2_edge - valid_at/invalid_at/expired_at + invalidation not deletion

## 1. Description

point-in-time edge queries work; contradicting edge invalidates prior with audit; nothing hard-deleted

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R7, F5.

## Acceptance criteria

- [ ] point-in-time edge queries work; contradicting edge invalidates prior with audit; nothing hard-deleted
