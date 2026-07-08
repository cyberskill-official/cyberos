---
id: FR-MEMORY-244
title: "Rust denylist enforcement at emit/ingest + admin binary hardening"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P2
refs: [R87, R88]
depends_on: [FR-MEMORY-202]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# FR-MEMORY-244: Rust denylist enforcement at emit/ingest + admin binary hardening

## 1. Description

secret-shaped content rejected at validate; admin runs require dedicated role + break-glass flag + chained invocation row

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R87, R88.

## Acceptance criteria

- [ ] secret-shaped content rejected at validate; admin runs require dedicated role + break-glass flag + chained invocation row
