---
id: TASK-MEMORY-229
title: "Query-embed LRU cache + EmbedClient in AppState"
module: memory
priority: COULD
status: draft
class: improvement
phase: P1
refs: [R22, F14]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-229: Query-embed LRU cache + EmbedClient in AppState

## 1. Description

repeat queries skip gateway; client constructed once; cache TTL 5m keyed by (model, hash)

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R22, F14.

## Acceptance criteria

- [ ] repeat queries skip gateway; client constructed once; cache TTL 5m keyed by (model, hash)
