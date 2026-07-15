---
id: TASK-MEMORY-245
title: "Calibrated LLM judge + golden-set growth + internal LongMemEval-style benchmark"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: memory
priority: p0
status: draft
phase: P3
refs: [R46, R56]
depends_on: [TASK-MEMORY-209]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-245: Calibrated LLM judge + golden-set growth + internal LongMemEval-style benchmark

## 1. Description

judge agreement 75-90% vs human labels; 100+ golden cases; five ability buckets tracked per release

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R46, R56.

## Acceptance criteria

- [ ] judge agreement 75-90% vs human labels; 100+ golden cases; five ability buckets tracked per release
