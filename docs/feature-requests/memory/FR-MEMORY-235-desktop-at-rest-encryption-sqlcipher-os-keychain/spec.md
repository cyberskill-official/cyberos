---
id: FR-MEMORY-235
title: "Desktop at-rest encryption - SQLCipher + OS keychain + file perms"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R66, F27]
depends_on: [FR-MEMORY-233]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-235: Desktop at-rest encryption - SQLCipher + OS keychain + file perms

## 1. Description

DB+WAL encrypted, key in keychain, 0600 perms; key-loss recovery = re-hydrate documented and tested

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R66, F27.

## Acceptance criteria

- [ ] DB+WAL encrypted, key in keychain, 0600 perms; key-loss recovery = re-hydrate documented and tested
