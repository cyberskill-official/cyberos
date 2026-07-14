---
id: TASK-MEMORY-215
title: "PII sidecar (Presidio + VN recognizers) in the cloud ingest path"
module: memory
priority: MUST
status: draft
class: improvement
phase: P1
refs: [R78, F20]
depends_on: [TASK-MEMORY-207]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-215: PII sidecar (Presidio + VN recognizers) in the cloud ingest path

## 1. Description

recall >=99.5% on labeled VN+EN PII set per TASK-MEMORY-111; pii_flags stored per row; ingest fails closed if sidecar down

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R78, F20.

## Acceptance criteria

- [ ] recall >=99.5% on labeled VN+EN PII set per TASK-MEMORY-111; pii_flags stored per row; ingest fails closed if sidecar down
