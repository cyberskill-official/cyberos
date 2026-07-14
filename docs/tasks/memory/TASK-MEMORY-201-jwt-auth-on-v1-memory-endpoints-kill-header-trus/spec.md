---
id: TASK-MEMORY-201
title: "JWT auth on /v1/memory endpoints (kill header-trust identity)"
module: memory
priority: MUST
status: draft
class: improvement
phase: P0
refs: [R73, F15]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-201: JWT auth on /v1/memory endpoints (kill header-trust identity)

## 1. Description

forged x-tenant-id/x-subject-id cannot cross tenants; tenant+subject come from verified JWT claims; negative test proves it

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R73, F15.

## Acceptance criteria

- [ ] forged x-tenant-id/x-subject-id cannot cross tenants; tenant+subject come from verified JWT claims; negative test proves it

## Implementation status (reconciled 2026-07-08)

Implemented on branch `auto/memory-enterprise` (commit f05d406, migrated from `MEM-001`). The code exists in `services/memory/`; author this FR's section-1 clauses from the implementation and source report before it moves through the review/test gates.
