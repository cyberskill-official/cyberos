---
artefact: architecture-decision-record@1
adr_id: ADR-002
task_id: TASK-IMP-058
status: accepted
created: 2026-07-25
dec_crosslinks: []
---
# ADR-002: AGE removal — relational `l2_edge` graph

## Context

Apache AGE was evaluated for graph traversal over memory. Managed Postgres
providers (RDS, Cloud SQL, Supabase, Neon) do not uniformly support AGE, and the
ops cost of a custom extension blocked portability.

## Options considered

1. Require Apache AGE everywhere — rejected: locks CyberOS off common managed
   Postgres; raises restore/drill cost.
2. Dual-path (AGE when present, SQL fallback) — rejected: two semantics, two
   test matrices.
3. Relational adjacency (`l2_edge`) + recursive CTEs / bi-temporal columns —
   CHOSEN (recorded in `docs/architecture/tech-stack.md` and the memory
   enterprise plan).

## Decision

Graph traversal for BRAIN/memory uses relational edges, not AGE.

## Consequences

- CyberOS runs on stock managed Postgres with pgvector (+ PGroonga where needed).
- Agents must not reintroduce AGE as a hard dependency without a superseding ADR.
- Portability enables the chat PDPL / residency options that assumed a portable DB.
