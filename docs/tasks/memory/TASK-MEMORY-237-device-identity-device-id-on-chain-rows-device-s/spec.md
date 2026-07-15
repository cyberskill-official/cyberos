---
id: TASK-MEMORY-237
title: "Device identity - device_id on chain rows, device-scoped JWTs, per-device limits"
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
phase: P2
refs: [R70, R71]
depends_on: [TASK-MEMORY-232]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-237: Device identity - device_id on chain rows, device-scoped JWTs, per-device limits

## 1. Description

rows attributable per device; short-expiry device tokens with refresh; rate limits enforced

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R70, R71.

## Acceptance criteria

- [ ] rows attributable per device; short-expiry device tokens with refresh; rate limits enforced
