---
id: TASK-MEMORY-214
title: "Content-aware ingestion - dereference content_ref pointers under RLS, embed redacted content"
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
refs: [R6, F2]
depends_on: [TASK-MEMORY-213, TASK-MEMORY-215]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-214: Content-aware ingestion - dereference content_ref pointers under RLS, embed redacted content

## 1. Description

pointer events embed real content post-PII; no raw body copied into brain tables; consent-gated per subject; flag-gated rollout

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R6, F2.

## Acceptance criteria

- [ ] pointer events embed real content post-PII; no raw body copied into brain tables; consent-gated per subject; flag-gated rollout
