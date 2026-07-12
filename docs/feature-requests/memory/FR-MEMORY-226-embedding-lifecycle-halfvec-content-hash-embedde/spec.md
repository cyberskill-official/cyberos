---
id: FR-MEMORY-226
title: "Embedding lifecycle - halfvec, content_hash+embedded_at, batch embeds, version-pinned indexes, migration runbook"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R37, R38, R39, R40, R41, F28]
depends_on: [FR-MEMORY-207]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-226: Embedding lifecycle - halfvec, content_hash+embedded_at, batch embeds, version-pinned indexes, migration runbook

## 1. Description

halfvec indexes live; batch input[] used by worker+backfill; partial indexes pin embed_model_version; dual-column runbook documented and rehearsed on dev

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R37, R38, R39, R40, R41, F28.

## Acceptance criteria

- [ ] halfvec indexes live; batch input[] used by worker+backfill; partial indexes pin embed_model_version; dual-column runbook documented and rehearsed on dev
