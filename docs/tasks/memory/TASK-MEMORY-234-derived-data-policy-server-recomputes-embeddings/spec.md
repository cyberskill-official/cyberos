---
id: TASK-MEMORY-234
title: "Derived-data policy - server recomputes embeddings/summaries; optional embeddings-down + sqlite-vec"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R62]
depends_on: [TASK-MEMORY-233]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-234: Derived-data policy - server recomputes embeddings/summaries; optional embeddings-down + sqlite-vec

## 1. Description

clients never upload derived artifacts; optional on-device search keyed by embed_model_version

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R62.

## Acceptance criteria

- [ ] clients never upload derived artifacts; optional on-device search keyed by embed_model_version
