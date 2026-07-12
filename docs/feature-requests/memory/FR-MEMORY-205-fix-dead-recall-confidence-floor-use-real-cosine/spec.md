---
id: FR-MEMORY-205
title: "Fix dead recall confidence floor (use real cosine, not constant 1.0)"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: [R9, F7]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-205: Fix dead recall confidence floor (use real cosine, not constant 1.0)

## 1. Description

weak summary match triggers drill; regression test pins the behavior

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R9, F7.

## Acceptance criteria

- [ ] weak summary match triggers drill; regression test pins the behavior

## Implementation status (reconciled 2026-07-08)

Implemented on branch `auto/memory-enterprise` (commit 33f4ea8, migrated from `MEM-005`). The code exists in `services/memory/`; author this FR's section-1 clauses from the implementation and source report before it moves through the review/test gates.
