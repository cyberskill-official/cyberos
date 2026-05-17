# `program-charter-author` — pipeline

This document describes how `program-charter-author` chains with upstream and downstream skills.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| `<upstream>-audit` (PASS) | Default chain | Passes `<upstream>` artefact path + audit verdict via input envelope. |
| (none — standalone) | User runs directly | Operator provides `source_files` manually. |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| `program-charter-audit` | Default after every `artefact_written` event | `next_skill_recommendation: program-charter-audit` in output envelope. |
| `<next-stage>-author` | After `program-charter-audit` returns PASS | Supervisor reads the audit's output envelope and queues the next stage. |
| (none — terminal) | User opts out of chaining | `chain_to: []` in input envelope. |

## Event emission

This skill publishes the following NATS subjects (per `cyberos/skill/contracts/nats-subjects/`):

| subject | payload | when |
|---|---|---|
| `program-charter_author.program-charter_written` | `{artefact_id, artefact_path, artefact_hash, source_hash}` | After every successful W3 WRITE. |
| `program-charter_author.batch_complete` | `{batch_run_id, artefacts_written, batch_outcome}` | At the end of a WORKER batch. |
| `program-charter_author.hitl_pause` | `{artefact_id, blocking_issues}` | When the batch halts on HITL. |
| `program-charter_author.amendment_request` | `{amendment_id, risk_class, change_description}` | When the author proposes a plan amendment. |

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

- `cyberos/skill/contracts/program-charter/` — the artefact template this skill generates.
- `cyberos/skill/contracts/nats-subjects/` — the NATS subject naming contract.
- `cyberos/skill/program-charter-audit/` — the sibling audit skill that validates this skill's output.
