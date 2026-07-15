---
id: TASK-MEMORY-234
title: "Derived-data policy - server recomputes embeddings/summaries; optional embeddings-down + sqlite-vec"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: memory
priority: p1
status: draft
phase: P2
refs: [R62]
depends_on: [TASK-MEMORY-233]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-234: Derived-data policy - server recomputes embeddings/summaries; optional embeddings-down + sqlite-vec

## 1. Description

clients never upload derived artifacts; optional on-device search keyed by embed_model_version

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R62.

## Acceptance criteria

- [ ] clients never upload derived artifacts; optional on-device search keyed by embed_model_version
