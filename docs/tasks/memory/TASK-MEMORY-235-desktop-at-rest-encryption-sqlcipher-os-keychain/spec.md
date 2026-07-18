---
id: TASK-MEMORY-235
title: "Desktop at-rest encryption - SQLCipher + OS keychain + file perms"
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
phase: P2
refs: [R66, F27]
depends_on: [TASK-MEMORY-233]
created: 2026-07-08
# awh N/A until a goldenset is sealed for this area
verify: N
---
# TASK-MEMORY-235: Desktop at-rest encryption - SQLCipher + OS keychain + file perms

## 1. Description

DB+WAL encrypted, key in keychain, 0600 perms; key-loss recovery = re-hydrate documented and tested

Migrated 2026-07-08 from the memory improvement backlog, folded into the task system as `class: improvement`. Source report refs: R66, F27.

## Acceptance criteria

- [ ] DB+WAL encrypted, key in keychain, 0600 perms; key-loss recovery = re-hydrate documented and tested
