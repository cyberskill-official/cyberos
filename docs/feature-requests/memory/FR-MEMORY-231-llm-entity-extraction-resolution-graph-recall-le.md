---
id: FR-MEMORY-231
title: "LLM entity extraction + resolution, graph recall leg, query-shape router"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R8, R16, R17, F6]
depends_on: [FR-MEMORY-230, FR-MEMORY-213]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-231: LLM entity extraction + resolution, graph recall leg, query-shape router

## 1. Description

typed entities/edges from episodes with dedup+aliases; 1-2 hop graph leg in RRF; router logged in explain; multi-hop golden cases improve

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R8, R16, R17, F6.

## Acceptance criteria

- [ ] typed entities/edges from episodes with dedup+aliases; 1-2 hop graph leg in RRF; router logged in explain; multi-hop golden cases improve
