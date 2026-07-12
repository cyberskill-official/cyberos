---
id: FR-MEMORY-214
title: "Content-aware ingestion - dereference content_ref pointers under RLS, embed redacted content"
module: memory
priority: MUST
status: draft
class: improvement
phase: P1
refs: [R6, F2]
depends_on: [FR-MEMORY-213, FR-MEMORY-215]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-214: Content-aware ingestion - dereference content_ref pointers under RLS, embed redacted content

## 1. Description

pointer events embed real content post-PII; no raw body copied into brain tables; consent-gated per subject; flag-gated rollout

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R6, F2.

## Acceptance criteria

- [ ] pointer events embed real content post-PII; no raw body copied into brain tables; consent-gated per subject; flag-gated rollout
