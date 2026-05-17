# `knowledge-asset-author` — pipeline

This document describes how `knowledge-asset-author` chains with upstream and downstream skills.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| `<upstream>-audit` (PASS) | Default chain | Passes `<upstream>` artefact path + audit verdict via input envelope. |
| (none — standalone) | User runs directly | Operator provides `source_files` manually. |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| `knowledge-asset-audit` | Default after every `artefact_written` event | `next_skill_recommendation: knowledge-asset-audit` in output envelope. |
| `<next-stage>-author` | After `knowledge-asset-audit` returns PASS | Supervisor reads the audit's output envelope and queues the next stage. |
| (none — terminal) | User opts out of chaining | `chain_to: []` in input envelope. |

## Event emission

This skill publishes the following NATS subjects (per `cyberos/skill/contracts/nats-subjects/`):

| subject | payload | when |
|---|---|---|
| `knowledge-asset_author.knowledge-asset_written` | `{artefact_id, artefact_path, artefact_hash, source_hash}` | After every successful W3 WRITE. |
| `knowledge-asset_author.batch_complete` | `{batch_run_id, artefacts_written, batch_outcome}` | At the end of a WORKER batch. |
| `knowledge-asset_author.hitl_pause` | `{artefact_id, blocking_issues}` | When the batch halts on HITL. |
| `knowledge-asset_author.amendment_request` | `{amendment_id, risk_class, change_description}` | When the author proposes a plan amendment. |

## Halting and resuming

The chain halts on:

- HITL (any `needs_human` issue from the audit).
- Self-audit invariant breach (emits `refinement_proposal`).
- Operator interrupt.

The chain resumes when:

- A human replies to a `HITL_BATCH_REQUEST` and the supervisor invokes this skill in RESUME phase.
- An operator approves a refinement proposal and the supervisor invokes this skill with `--refinement-run-id <id>`.

## Idempotency

This skill is idempotent on manifest state. Re-running on a fully settled manifest is a no-op except for the `last_audit_at` timestamp refresh.

## Cross-references

- `cyberos/skill/contracts/knowledge-asset/` — the artefact template this skill generates.
- `cyberos/skill/contracts/nats-subjects/` — the NATS subject naming contract.
- `cyberos/skill/knowledge-asset-audit/` — the sibling audit skill that validates this skill's output.
