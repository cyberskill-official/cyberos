---
id: TASK-MEMORY-259
title: "brain_common test harness fix so the brain DB suite runs (raw_sql + once-per-process migrate)"
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
phase: P0
refs: []
depends_on: []
created: 2026-07-08
verify: N
---
# TASK-MEMORY-259: brain_common test harness fix so the brain DB suite runs (raw_sql + once-per-process migrate)

## 1. Description

brain_common test harness fix so the brain DB suite runs (raw_sql + once-per-process migrate). Implemented on branch `auto/memory-enterprise` (commit 9808d0b). Filed during the memory hardening session as `MEM-059`, which post-dated the backlog migration and so had no migrated task. Author the normative clauses from the implementation before this task leaves draft.

## Acceptance criteria

- [ ] the fix is covered by a passing test (see the implementing commit)
