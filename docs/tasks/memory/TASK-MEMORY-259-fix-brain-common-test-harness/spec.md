---
id: TASK-MEMORY-259
title: "brain_common test harness fix so the brain DB suite runs (raw_sql + once-per-process migrate)"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: []
depends_on: []
created: 2026-07-08
verify: N
---
# TASK-MEMORY-259: brain_common test harness fix so the brain DB suite runs (raw_sql + once-per-process migrate)

## 1. Description

brain_common test harness fix so the brain DB suite runs (raw_sql + once-per-process migrate). Implemented on branch `auto/memory-enterprise` (commit 9808d0b). Filed during the memory hardening session as `MEM-059`, which post-dated the backlog migration and so had no migrated FR. Author the normative clauses from the implementation before this FR leaves draft.

## Acceptance criteria

- [ ] the fix is covered by a passing test (see the implementing commit)
