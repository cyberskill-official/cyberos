---
id: FR-MEMORY-204
title: "Per-principal and per-tenant rate limits on recall/search"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: [R77, F18]
depends_on: [FR-MEMORY-201]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-204: Per-principal and per-tenant rate limits on recall/search

## 1. Description

burst over limit returns 429 with retry-after; limits configurable per tenant

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R77, F18.

## Acceptance criteria

- [ ] burst over limit returns 429 with retry-after; limits configurable per tenant
