---
artefact: architecture-decision-record@1
adr_id: ADR-005
task_id: TASK-IMP-058
status: accepted
created: 2026-07-25
dec_crosslinks: []
---
# ADR-005: HITL acceptance gates are non-bypassable

## Context

Ship-tasks has two human-acceptance gates: `reviewing → ready_to_test` and
`testing → done` (STATUS-REFERENCE §1.4). Agents that self-accept produce green
theatre. TASK-CUO-303 / IMP-143 bind flips to `--verdict-by` + evidence (+
verdict artefact).

## Options considered

1. Allow agents to self-set `done` when gates are green — rejected: removes the
   only human stop.
2. Require recorded human verdict (actor + evidence file) on both gates; machine
   gates remain necessary but never sufficient — CHOSEN.

## Decision

HITL gates stay mandatory. Session operators may pre-authorise a named actor for
a batch via a verdict-evidence artefact, but the flip still records that actor
and evidence hash — there is no silent bypass flag.

## Consequences

- `backlog-mutate` / `task-state` refuse gated flips without verdict (exit 8).
- MCP `ship_task` never auto-accepts; it only returns the trigger.
- Temporary session overrides must still write evidence under `docs/batches/`.
