---
id: FR-MEMORY-243
title: "External chain anchoring + nightly chain-integrity walker"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R86]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-243: External chain anchoring + nightly chain-integrity walker

## 1. Description

chain head signature published outside the DB on schedule; nightly walk verifies end-to-end and alerts on divergence

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R86.

## Acceptance criteria

- [ ] chain head signature published outside the DB on schedule; nightly walk verifies end-to-end and alerts on divergence
