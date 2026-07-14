---
id: TASK-MEMORY-217
title: "Incremental summarization - queue off hot path, window-bounded scopes, version-race fix"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R26, R36, R93, F29, F33, F34]
depends_on: [TASK-MEMORY-216]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-217: Incremental summarization - queue off hot path, window-bounded scopes, version-race fix

## 1. Description

re-summarize reads only uncovered seqs; time_window bounded to its ISO week; concurrent supersede safe; 3-COUNT-per-event pattern gone

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R26, R36, R93, F29, F33, F34.

## Acceptance criteria

- [ ] re-summarize reads only uncovered seqs; time_window bounded to its ISO week; concurrent supersede safe; 3-COUNT-per-event pattern gone
