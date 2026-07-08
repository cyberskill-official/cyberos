---
id: FR-MEMORY-202
title: "Fail-closed RLS on brain tables (drop NULL-GUC arm + nil-uuid bypass)"
module: memory
priority: MUST
status: draft
class: improvement
phase: P0
refs: [R74, F16]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-202: Fail-closed RLS on brain tables (drop NULL-GUC arm + nil-uuid bypass)

## 1. Description

query without app.tenant_id returns zero rows; admin paths use dedicated role, not magic uuid

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R74, F16.

## Acceptance criteria

- [ ] query without app.tenant_id returns zero rows; admin paths use dedicated role, not magic uuid
