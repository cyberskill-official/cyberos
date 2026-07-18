---
id: TASK-MEMORY-225
title: "Day-1 emitters (chat/auth/proj/obs) + real consent gate wired to TASK-EVAL-001 ledger"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: memory
priority: p0
status: draft
phase: P1
refs: [R103, F19]
depends_on: [TASK-MEMORY-201]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-225: Day-1 emitters (chat/auth/proj/obs) + real consent gate wired to TASK-EVAL-001 ledger

## 1. Description

modules emit with content_ref pointers; DenyAll replaced by ledger-backed gate with 60s cache; unacknowledged subjects never captured

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R103, F19.

## Acceptance criteria

- [ ] modules emit with content_ref pointers; DenyAll replaced by ledger-backed gate with 60s cache; unacknowledged subjects never captured
