---
id: FR-MEMORY-232
title: "Sync server - shape-scoped seq stream down + idempotent outbox endpoint up"
module: memory
priority: MUST
status: draft
class: improvement
phase: P2
refs: [R60, R65, F25, F26]
depends_on: [FR-MEMORY-201, FR-MEMORY-210]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-232: Sync server - shape-scoped seq stream down + idempotent outbox endpoint up

## 1. Description

device pulls only permitted shapes (own subject + grants, shareable sync_class); uploads dedup on event_id; contract documented

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R60, R65, F25, F26.

## Acceptance criteria

- [ ] device pulls only permitted shapes (own subject + grants, shareable sync_class); uploads dedup on event_id; contract documented
