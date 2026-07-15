---
id: TASK-MEMORY-231
title: "LLM entity extraction + resolution, graph recall leg, query-shape router"
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
phase: P1
refs: [R8, R16, R17, F6]
depends_on: [TASK-MEMORY-230, TASK-MEMORY-213]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-231: LLM entity extraction + resolution, graph recall leg, query-shape router

## 1. Description

typed entities/edges from episodes with dedup+aliases; 1-2 hop graph leg in RRF; router logged in explain; multi-hop golden cases improve

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R8, R16, R17, F6.

## Acceptance criteria

- [ ] typed entities/edges from episodes with dedup+aliases; 1-2 hop graph leg in RRF; router logged in explain; multi-hop golden cases improve
