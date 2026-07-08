---
id: FR-MEMORY-220
title: "Cross-encoder rerank stage (BGE via embed-sidecar) over fused top-50"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R12, F10]
depends_on: [FR-MEMORY-219]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-220: Cross-encoder rerank stage (BGE via embed-sidecar) over fused top-50

## 1. Description

one batched sidecar call per recall; golden-set lift recorded; skipped when degraded

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R12, F10.

## Acceptance criteria

- [ ] one batched sidecar call per recall; golden-set lift recorded; skipped when degraded
