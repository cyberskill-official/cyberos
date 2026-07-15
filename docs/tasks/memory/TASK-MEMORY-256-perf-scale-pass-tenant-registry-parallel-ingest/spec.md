---
id: TASK-MEMORY-256
title: "Perf + scale pass - tenant registry, parallel ingest, l2 HNSW decision, SLO smoke, partitioning plan"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: improvement
created_at: 2026-07-08T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: memory
priority: p1
status: draft
phase: P3
refs: [R91, R92, R94, R96, R99, F30]
depends_on: [TASK-MEMORY-226]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-256: Perf + scale pass - tenant registry, parallel ingest, l2 HNSW decision, SLO smoke, partitioning plan

## 1. Description

DISTINCT-scan discovery gone; bounded-concurrency ingest; recall p95 <300ms at 1M hot rows in nightly smoke; partition plan written with triggers

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R91, R92, R94, R96, R99, F30.

## Acceptance criteria

- [ ] DISTINCT-scan discovery gone; bounded-concurrency ingest; recall p95 <300ms at 1M hot rows in nightly smoke; partition plan written with triggers
