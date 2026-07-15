---
id: TASK-MEMORY-213
title: "Fact write pipeline - extract then ADD/UPDATE/DELETE/NOOP with chained op audit"
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
phase: P1
refs: [R5]
depends_on: [TASK-MEMORY-212, TASK-MEMORY-207]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-213: Fact write pipeline - extract then ADD/UPDATE/DELETE/NOOP with chained op audit

## 1. Description

gateway tool-call chooses op against top-k neighbors; every op writes a chain row; L1 never mutated; replay idempotent

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R5.

## Acceptance criteria

- [ ] gateway tool-call chooses op against top-k neighbors; every op writes a chain row; L1 never mutated; replay idempotent
