# `breach-notification-audit` — pipeline

This document describes how `breach-notification-audit` chains with upstream and downstream skills.

## Upstream

| Upstream skill | Trigger | Hand-off |
|---|---|---|
| `breach-notification-author` (per `artefact_written` event) | Default chain | Passes the just-written artefact path + manifest path via input envelope's `upstream_context`. |
| (none — standalone) | User runs directly | Operator provides `artefact_paths` manually. |

## Downstream

| Downstream skill | Trigger | Hand-off |
|---|---|---|
| `breach-notification-author` (RESUME phase) | When verdict is `needs_human` and operator replies; or when STALE-001 fires with REVERT_TO_MANIFEST | Sets `requires_regen: true` in output envelope; supervisor invokes author. |
| `<next-stage>-author` | After verdict is `pass` | Supervisor reads `next_skill_recommendation` and queues the next stage. |
| (none — terminal) | User opts out of chaining | Empty `next_skill_recommendation`. |

## Event emission

This skill publishes the following NATS subjects (per `cyberos/skill/contracts/nats-subjects/`):

| subject | payload | when |
|---|---|---|
| `breach-notification_audit.audit_written` | `{artefact_path, audit_path, audited_file_sha256, verdict}` | After every successful Step 8 WRITE. |
| `breach-notification_audit.audit_batch_complete` | `{batch_run_id, per_artefact, hitl_required}` | At the end of a batch. |
| `breach-notification_audit.hitl_pause` | `{artefact_path, blocking_issues}` | When the batch halts on HITL. |

## Halting and resuming

Halts on:

- HITL (any `needs_human` verdict).
- Self-audit invariant breach.
- `deterministic_drift` signal (catastrophic — pauses immediately).
- Operator interrupt.

Resumes when:

- Human replies to `HITL_BATCH_REQUEST`; supervisor re-invokes this skill with the operator's resolutions in the input envelope.
- Operator approves a refinement proposal.

## Idempotency

This skill is idempotent on `audited_file_sha256`. Re-running on an unchanged artefact produces byte-identical reports modulo timestamps (per `INVARIANTS.md` INV-006).

## Cross-references

- `cyberos/skill/contracts/breach-notification/` — the artefact template this skill audits.
- `cyberos/skill/contracts/nats-subjects/` — the NATS subject naming contract.
- `cyberos/skill/breach-notification-author/` — the sibling author skill whose output this skill validates.
