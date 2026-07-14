---
id: TASK-MEMORY-223
title: "Query rewriting - relative-time parsing (EN+VN) and subject-handle expansion"
module: memory
priority: SHOULD
status: draft
class: improvement
phase: P1
refs: [R15]
depends_on: [TASK-MEMORY-219]
created: 2026-07-08
verify: N   # awh N/A until a goldenset is sealed for this area
---
# TASK-MEMORY-223: Query rewriting - relative-time parsing (EN+VN) and subject-handle expansion

## 1. Description

relative-time phrases (EN+VN) become ts filters; handles resolve to subject UUIDs; rewrite visible in explain

Migrated 2026-07-08 from the memory improvement backlog, folded into the FR system as `class: improvement`. Source report refs: R15.

## Acceptance criteria

- [ ] relative-time phrases (EN+VN) become ts filters; handles resolve to subject UUIDs; rewrite visible in explain
