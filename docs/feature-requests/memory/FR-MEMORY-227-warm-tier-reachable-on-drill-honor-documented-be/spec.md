---
id: FR-MEMORY-227
title: "Warm-tier reachable on drill (honor documented behavior)"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R20, F11]
depends_on: [FR-MEMORY-226]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-227: Warm-tier reachable on drill (honor documented behavior)

## 1. Description

drill=true searches warm; cost bounded (quantized or seq-scan budget); docs and code agree

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R20, F11.

## Acceptance criteria

- [ ] drill=true searches warm; cost bounded (quantized or seq-scan budget); docs and code agree
