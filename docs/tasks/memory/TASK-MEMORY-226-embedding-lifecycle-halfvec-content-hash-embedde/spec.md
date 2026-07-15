---
id: TASK-MEMORY-226
title: "Embedding lifecycle - halfvec, content_hash+embedded_at, batch embeds, version-pinned indexes, migration runbook"
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
phase: P1
refs: [R37, R38, R39, R40, R41, F28]
depends_on: [TASK-MEMORY-207]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-226: Embedding lifecycle - halfvec, content_hash+embedded_at, batch embeds, version-pinned indexes, migration runbook

## 1. Description

halfvec indexes live; batch input[] used by worker+backfill; partial indexes pin embed_model_version; dual-column runbook documented and rehearsed on dev

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R37, R38, R39, R40, R41, F28.

## Acceptance criteria

- [ ] halfvec indexes live; batch input[] used by worker+backfill; partial indexes pin embed_model_version; dual-column runbook documented and rehearsed on dev
