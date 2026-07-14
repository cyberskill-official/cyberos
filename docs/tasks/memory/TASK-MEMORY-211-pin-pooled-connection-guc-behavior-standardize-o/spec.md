---
id: TASK-MEMORY-211
title: "Pin pooled-connection GUC behavior; standardize on app.tenant_id"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: [R95, F35]
depends_on: [TASK-MEMORY-202]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-211: Pin pooled-connection GUC behavior; standardize on app.tenant_id

## 1. Description

integration test proves tx-local set_config isolation under transaction pooling; one GUC name across memory+eval

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R95, F35.

## Acceptance criteria

- [ ] integration test proves tx-local set_config isolation under transaction pooling; one GUC name across memory+eval
