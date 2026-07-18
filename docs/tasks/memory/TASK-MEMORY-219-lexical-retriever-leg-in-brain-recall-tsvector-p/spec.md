---
id: TASK-MEMORY-219
title: "Lexical retriever leg in brain recall (tsvector + pg_trgm) fused via RRF"
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
refs: [R11, F12]
depends_on: [TASK-MEMORY-212]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-219: Lexical retriever leg in brain recall (tsvector + pg_trgm) fused via RRF

## 1. Description

hybrid beats vector-only on golden set; expression indexes in place; degraded modes unchanged

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R11, F12.

## Acceptance criteria

- [ ] hybrid beats vector-only on golden set; expression indexes in place; degraded modes unchanged
