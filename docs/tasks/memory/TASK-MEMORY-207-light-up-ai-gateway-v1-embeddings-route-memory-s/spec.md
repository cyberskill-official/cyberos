---
id: TASK-MEMORY-207
title: "Light up ai-gateway /v1/embeddings route + memory-side contract test"
module: memory
priority: MUST
status: draft
class: improvement
phase: P0
refs: [R44, F34]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-207: Light up ai-gateway /v1/embeddings route + memory-side contract test

## 1. Description

real gateway serves bge-m3 embeddings under tenant policy; memory CI runs a contract test against a gateway stub

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R44, F34.

## Acceptance criteria

- [ ] real gateway serves bge-m3 embeddings under tenant policy; memory CI runs a contract test against a gateway stub
