---
id: TASK-MEMORY-208
title: "Metrics correctness - embed_malformed label, per-leg recall latency spans"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P0
refs: [R100, F31]
depends_on: []
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-208: Metrics correctness - embed_malformed label, per-leg recall latency spans

## 1. Description

Malformed no longer counted as postgres_error; recall legs individually timed in traces

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R100, F31.

## Acceptance criteria

- [ ] Malformed no longer counted as postgres_error; recall legs individually timed in traces
