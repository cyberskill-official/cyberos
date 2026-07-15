---
id: TASK-MEMORY-243
title: "External chain anchoring + nightly chain-integrity walker"
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
refs: [R86]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-243: External chain anchoring + nightly chain-integrity walker

## 1. Description

chain head signature published outside the DB on schedule; nightly walk verifies end-to-end and alerts on divergence

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R86.

## Acceptance criteria

- [ ] chain head signature published outside the DB on schedule; nightly walk verifies end-to-end and alerts on divergence
