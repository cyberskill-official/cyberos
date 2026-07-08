---
id: FR-MEMORY-249
title: "Retrieval config A/B + shadow evaluation on sampled live traffic"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P3
refs: [R48, R54]
depends_on: [FR-MEMORY-245, FR-MEMORY-221]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-249: Retrieval config A/B + shadow evaluation on sampled live traffic

## 1. Description

config table with hash assignment; judge-scored outcomes promote winners by flag flip; shadow deltas logged

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R48, R54.

## Acceptance criteria

- [ ] config table with hash assignment; judge-scored outcomes promote winners by flag flip; shadow deltas logged
