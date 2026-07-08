---
id: FR-MEMORY-240
title: "Erasure - crypto-shredding (per-subject DEKs), lineage cascade, ghost-vector reindex, backup re-deletion"
module: memory
priority: MUST
status: draft
class: improvement
phase: P2
refs: [R82, R84, R89, F21]
depends_on: [FR-MEMORY-239, FR-MEMORY-214]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-240: Erasure - crypto-shredding (per-subject DEKs), lineage cascade, ghost-vector reindex, backup re-deletion

## 1. Description

erasure drill removes plaintext+vectors for a test subject, chain stays verifiable, restore replays erasure ledger; field-level crypto for sensitive kinds

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R82, R84, R89, F21.

## Acceptance criteria

- [ ] erasure drill removes plaintext+vectors for a test subject, chain stays verifiable, restore replays erasure ledger; field-level crypto for sensitive kinds
