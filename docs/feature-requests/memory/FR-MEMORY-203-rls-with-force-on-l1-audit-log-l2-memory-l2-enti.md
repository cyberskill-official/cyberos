---
id: FR-MEMORY-203
title: "RLS with FORCE on l1_audit_log, l2_memory, l2_entity, l2_edge + cross-tenant probe"
module: memory
priority: MUST
status: draft
class: improvement
phase: P0
refs: [R75, R76, F17]
depends_on: [FR-MEMORY-202]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-203: RLS with FORCE on l1_audit_log, l2_memory, l2_entity, l2_edge + cross-tenant probe

## 1. Description

RLS property test covers all memory tables; probe reads zero cross-tenant rows; query plans still index-driven

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R75, R76, F17.

## Acceptance criteria

- [ ] RLS property test covers all memory tables; probe reads zero cross-tenant rows; query plans still index-driven
