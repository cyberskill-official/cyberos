---
id: TASK-MEMORY-240
title: "Erasure - crypto-shredding (per-subject DEKs), lineage cascade, ghost-vector reindex, backup re-deletion"
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
phase: P2
refs: [R82, R84, R89, F21]
depends_on: [TASK-MEMORY-239, TASK-MEMORY-214]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-240: Erasure - crypto-shredding (per-subject DEKs), lineage cascade, ghost-vector reindex, backup re-deletion

## 1. Description

erasure drill removes plaintext+vectors for a test subject, chain stays verifiable, restore replays erasure ledger; field-level crypto for sensitive kinds

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R82, R84, R89, F21.

## Acceptance criteria

- [ ] erasure drill removes plaintext+vectors for a test subject, chain stays verifiable, restore replays erasure ledger; field-level crypto for sensitive kinds
