---
id: TASK-MEMORY-248
title: "Promotion + dedup + contradiction jobs, eval-gated with auto-revert"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P3
refs: [R28, R29, R30, R35]
depends_on: [TASK-MEMORY-247]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-248: Promotion + dedup + contradiction jobs, eval-gated with auto-revert

## 1. Description

episodic-to-semantic promotion with derived_from; cosine>0.95 merges via op pipeline; contradictions invalidate bi-temporally; every batch bracketed by golden evals

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R28, R29, R30, R35.

## Acceptance criteria

- [ ] episodic-to-semantic promotion with derived_from; cosine>0.95 merges via op pipeline; contradictions invalidate bi-temporally; every batch bracketed by golden evals
