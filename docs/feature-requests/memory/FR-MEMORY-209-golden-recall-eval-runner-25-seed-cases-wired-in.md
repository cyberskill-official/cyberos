---
id: FR-MEMORY-209
title: "Golden recall eval runner + 25 seed cases wired into CI"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: [R45, F32]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-209: Golden recall eval runner + 25 seed cases wired into CI

## 1. Description

memory-eval binary scores recall@10 and MRR against seed set; CI gate fails on >3% drop

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R45, F32.

## Acceptance criteria

- [ ] memory-eval binary scores recall@10 and MRR against seed set; CI gate fails on >3% drop
