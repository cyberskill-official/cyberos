---
id: TASK-MEMORY-212
title: "brain_fact table + memory_kind taxonomy (episodic/semantic/procedural/profile/resource)"
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
refs: [R3, R4, F4]
depends_on: [TASK-MEMORY-202, TASK-MEMORY-203]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-212: brain_fact table + memory_kind taxonomy (episodic/semantic/procedural/profile/resource)

## 1. Description

migration + Rust model with importance, trust, valid_at/invalid_at, revision lineage, derived_from provenance, RLS fail-closed

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R3, R4, F4.

## Acceptance criteria

- [ ] migration + Rust model with importance, trust, valid_at/invalid_at, revision lineage, derived_from provenance, RLS fail-closed
