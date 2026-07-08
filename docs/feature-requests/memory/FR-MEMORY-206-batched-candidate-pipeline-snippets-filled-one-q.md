---
id: FR-MEMORY-206
title: "Batched candidate pipeline - snippets filled, one-query verify, set-based access check"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: [R10, R18, R19, F8, F9]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-206: Batched candidate pipeline - snippets filled, one-query verify, set-based access check

## 1. Description

event hits return non-empty snippets; recall issues O(1) queries per stage, not per candidate; p95 measured before/after

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R10, R18, R19, F8, F9.

## Acceptance criteria

- [ ] event hits return non-empty snippets; recall issues O(1) queries per stage, not per candidate; p95 measured before/after
