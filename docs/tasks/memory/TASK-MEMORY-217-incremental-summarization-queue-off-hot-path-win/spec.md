---
id: TASK-MEMORY-217
title: "Incremental summarization - queue off hot path, window-bounded scopes, version-race fix"
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
refs: [R26, R36, R93, F29, F33, F34]
depends_on: [TASK-MEMORY-216]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-217: Incremental summarization - queue off hot path, window-bounded scopes, version-race fix

## 1. Description

re-summarize reads only uncovered seqs; time_window bounded to its ISO week; concurrent supersede safe; 3-COUNT-per-event pattern gone

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R26, R36, R93, F29, F33, F34.

## Acceptance criteria

- [ ] re-summarize reads only uncovered seqs; time_window bounded to its ISO week; concurrent supersede safe; 3-COUNT-per-event pattern gone
