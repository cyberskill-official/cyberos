---
id: FR-MEMORY-124
title: "memory.awh_gate_result aux audit row - the awh out-of-band gate verdict, emitted into the memory chain by the ship-feature-requests testing->done step (step 28)"
renumbered_from: FR-MEMORY-121
renumbered_note: "Renumbered 121 -> 124 on 2026-06-29 so FR-MEMORY-121/122/123 could carry the BRAIN capture trio (interaction-event schema / capture emitters / ingestion) per docs/strategy/cyberos-brain-evaluation-plan.md. The row-kind string `memory.awh_gate_result` is unchanged; only the FR id moved."
module: MEMORY
priority: SHOULD
status: draft
verify: T
phase: P1
milestone: P1 - awh absorption
slice: 1
owner: Stephen Cheng
created: 2026-06-19
shipped: null
memory_chain_hash: null
gated_on: "APPROVE protocol change P23 §6"
related_frs: [FR-MEMORY-118, FR-CUO-101]
depends_on: [FR-MEMORY-101]
blocks: []
---

## §1 - Description (BCP-14 normative)

The awh out-of-band gate (ship-feature-requests workflow step 28) reruns an FR's §1 cited
tests plus its module suite against a sealed, read-only baseline and returns GREEN or RED.
This FR records that verdict as an aux audit row on the memory chain, so the chain tells the
full story of why an FR reached `done` (independent verification), not just that an agent
claimed it.

This FR is gated on `APPROVE protocol change P23 §6`, which adds the row kind to the audit
ledger. Until P23 is approved and this FR ships, the gate writes verdicts to a side log
(`.awh/gate-results.jsonl`) and emits no chain row.

1. The system MUST define a new aux audit row kind `memory.awh_gate_result`, enumerated in
   `memory.schema.json#/definitions/AuditRecord` alongside the existing aux kinds
   (`memory.precondition_failed`, `memory.acl_denied`, `memory.status_overridden`).
2. The row MUST carry payload `{fr_id, module, outcome, weighted_pass, harness_version,
   sealed_acceptance_hash, tasks}` where `outcome` is the closed enum `GREEN | RED`,
   `weighted_pass` is the awh eval weighted pass@1 in `[0.0, 1.0]`, `harness_version` is the
   vendored awh source sha, and `sealed_acceptance_hash` is the hash of the locked golden set
   plus baseline that the FR was graded against.
3. The row MUST be written through the canonical memory writer (AGENTS.md §14.1), never by
   touching `audit/` directly. The awh-gate workflow step shells to `cyberos` to emit it.
4. On RED, the row MUST be emitted before the FR routes back to `ready_to_implement`
   (STATUS-REFERENCE §1.3), so the failure is on the chain even though the FR did not ship.
5. The row is a pure record. It MUST NOT change any memory file and MUST NOT gate any read.

## §2 - Rationale

The whole point of absorbing awh is that agent self-certification is not trust. Recording the
gate verdict on the immutable chain makes the trust boundary auditable: every `done` carries a
`memory.awh_gate_result{outcome: GREEN}` with the harness version and the sealed-acceptance
hash, and every route-back carries a RED row. This FR is the dogfood: it is the first FR
shipped through the awh-gated workflow, so the gate proves itself by gating its own audit row.

## §3 - Cited tests (held-out)

- `modules/memory/tests/core/test_awh_gate_result_row.py` - row shape, enum validation,
  writer integration, RED-before-route-back ordering, read-only invariant.

Acceptance command (sealed via `awh lock modules/memory/tests`):

```
cd modules/memory && python -m pytest tests/core/test_awh_gate_result_row.py -q
```

## §10 - File writes

- `modules/memory/memory.schema.json` - add the row kind to the AuditRecord enum + payload.
- `modules/memory/cyberos/core/writer.py` - typed builder for the row.
- `modules/memory/cyberos/data/AGENTS.md` - document the row kind under §6 (the P23 amendment).
- `modules/memory/tests/core/test_awh_gate_result_row.py` - the cited test.

## §11 - Notes

This is a `draft`. It enters the normal `draft -> ready_to_implement` audit chain
(`feature-request-author` / `feature-request-audit`) to reach 10/10 before the
`ship-feature-requests` workflow picks it up. It is intentionally the first FR routed through
the new awh gate.
