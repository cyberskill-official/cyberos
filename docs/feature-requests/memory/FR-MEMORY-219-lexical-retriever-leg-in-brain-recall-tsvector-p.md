---
id: FR-MEMORY-219
title: "Lexical retriever leg in brain recall (tsvector + pg_trgm) fused via RRF"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R11, F12]
depends_on: [FR-MEMORY-212]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-219: Lexical retriever leg in brain recall (tsvector + pg_trgm) fused via RRF

## 1. Description

hybrid beats vector-only on golden set; expression indexes in place; degraded modes unchanged

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R11, F12.

## Acceptance criteria

- [ ] hybrid beats vector-only on golden set; expression indexes in place; degraded modes unchanged
