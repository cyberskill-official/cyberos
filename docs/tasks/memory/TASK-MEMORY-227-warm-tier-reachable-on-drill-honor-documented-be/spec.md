---
id: TASK-MEMORY-227
title: "Warm-tier reachable on drill (honor documented behavior)"
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
refs: [R20, F11]
depends_on: [TASK-MEMORY-226]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-227: Warm-tier reachable on drill (honor documented behavior)

## 1. Description

drill=true searches warm; cost bounded (quantized or seq-scan budget); docs and code agree

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R20, F11.

## Acceptance criteria

- [ ] drill=true searches warm; cost bounded (quantized or seq-scan budget); docs and code agree
