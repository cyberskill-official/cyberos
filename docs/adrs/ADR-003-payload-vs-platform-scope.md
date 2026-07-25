---
artefact: architecture-decision-record@1
adr_id: ADR-003
task_id: TASK-IMP-058
status: accepted
created: 2026-07-25
dec_crosslinks: []
---
# ADR-003: Payload vs platform scope for CyberOS 1.x

## Context

The monorepo holds both the **install payload** (workflows, skills, docs-tools,
MCP helpers, BRAIN protocol) and **platform** code (`services/*`, deploy, native
stores). Deep-audit improvement stubs mixed both. Shipping 1.x required a hard
scope fork (batch/10e).

## Options considered

1. Treat every monorepo path as CyberOS 1.x — rejected: never finishes; blockers
   invent themselves.
2. Payload-only 1.x; platform/go-live/research closed or deferred — CHOSEN.

## Decision

CyberOS 1.x = the consumer install payload and its gates. Platform services,
VPS deploy runbooks, and research pilots are out of 1.x acceptance unless a task
explicitly says otherwise.

## Consequences

- Improvement grooming closes platform stubs as won't-do for 1.x with reasons.
- New IMP tasks must declare whether they touch payload or platform.
- See `docs/batches/batch-10e-imp-stub-wont-do.md` for the applied ruling.
