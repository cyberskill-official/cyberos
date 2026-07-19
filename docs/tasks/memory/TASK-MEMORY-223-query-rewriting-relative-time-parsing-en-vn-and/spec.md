---
id: TASK-MEMORY-223
title: "Query rewriting - relative-time parsing (EN+VN) and subject-handle expansion"
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
priority: p1
status: draft
phase: P1
refs: [R15]
depends_on: [TASK-MEMORY-219]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-223: Query rewriting - relative-time parsing (EN+VN) and subject-handle expansion

## 1. Description

relative-time phrases (EN+VN) become ts filters; handles resolve to subject UUIDs; rewrite visible in explain

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R15.

## Acceptance criteria

- [ ] relative-time phrases (EN+VN) become ts filters; handles resolve to subject UUIDs; rewrite visible in explain
