---
id: TASK-MEMORY-225
title: "Day-1 emitters (chat/auth/proj/obs) + real consent gate wired to TASK-EVAL-001 ledger"
module: memory
priority: MUST
status: draft
class: improvement
phase: P1
refs: [R103, F19]
depends_on: [TASK-MEMORY-201]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-225: Day-1 emitters (chat/auth/proj/obs) + real consent gate wired to TASK-EVAL-001 ledger

## 1. Description

modules emit with content_ref pointers; DenyAll replaced by ledger-backed gate with 60s cache; unacknowledged subjects never captured

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R103, F19.

## Acceptance criteria

- [ ] modules emit with content_ref pointers; DenyAll replaced by ledger-backed gate with 60s cache; unacknowledged subjects never captured
