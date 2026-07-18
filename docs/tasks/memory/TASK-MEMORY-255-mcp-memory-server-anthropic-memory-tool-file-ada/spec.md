---
id: TASK-MEMORY-255
title: "MCP memory server + Anthropic memory-tool file adapter"
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
phase: P3
refs: [R101, R102]
depends_on: [TASK-MEMORY-228]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-255: MCP memory server + Anthropic memory-tool file adapter

## 1. Description

recall/remember/feedback tools served over MCP with JWT auth; /memories path CRUD adapter backed by facts+profiles

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R101, R102.

## Acceptance criteria

- [ ] recall/remember/feedback tools served over MCP with JWT auth; /memories path CRUD adapter backed by facts+profiles
