---
id: TASK-MEMORY-228
title: "Recall API v1.1 - keyset pagination, scores breakdown in explain, feedback endpoint"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R21, R23, R24, F13]
depends_on: [TASK-MEMORY-221]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-228: Recall API v1.1 - keyset pagination, scores breakdown in explain, feedback endpoint

## 1. Description

stable cursoring past 100; explain shows per-leg ranks and terms; feedback updates access stats and chains an event

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R21, R23, R24, F13.

## Acceptance criteria

- [ ] stable cursoring past 100; explain shows per-leg ranks and terms; feedback updates access stats and chains an event
