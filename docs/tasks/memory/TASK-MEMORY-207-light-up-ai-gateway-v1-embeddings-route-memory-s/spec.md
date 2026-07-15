---
id: TASK-MEMORY-207
title: "Light up ai-gateway /v1/embeddings route + memory-side contract test"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p0
status: draft
phase: P0
refs: [R44, F34]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-207: Light up ai-gateway /v1/embeddings route + memory-side contract test

## 1. Description

real gateway serves bge-m3 embeddings under tenant policy; memory CI runs a contract test against a gateway stub

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R44, F34.

## Acceptance criteria

- [ ] real gateway serves bge-m3 embeddings under tenant policy; memory CI runs a contract test against a gateway stub
