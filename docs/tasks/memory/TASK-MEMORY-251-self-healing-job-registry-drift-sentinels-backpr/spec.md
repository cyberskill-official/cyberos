---
id: TASK-MEMORY-251
title: "Self-healing job registry + drift sentinels + backpressure/DLQ"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p1
status: draft
phase: P3
refs: [R52, R43, R97]
depends_on: [TASK-MEMORY-226, TASK-MEMORY-245]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-251: Self-healing job registry + drift sentinels + backpressure/DLQ

## 1. Description

jobs (re-embed, re-summarize, re-dedupe, reindex) emit before/after metrics and auto-revert; sentinel corpus alerts on drift; poison events park in DLQ

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R52, R43, R97.

## Acceptance criteria

- [ ] jobs (re-embed, re-summarize, re-dedupe, reindex) emit before/after metrics and auto-revert; sentinel corpus alerts on drift; poison events park in DLQ
