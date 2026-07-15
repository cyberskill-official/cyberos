# Turning on brain capture (the chat-to-brain link)

This is the ordered, governance-first way to start recording work interactions into the brain. The pipeline
is already built and ships OFF; this runbook is the sequence to turn it on responsibly. Do the steps in order.
Skipping straight to the env flip records nobody (the consent gate denies by default), so the order below is
the only one that actually works.

## What is already built

- Capture: one interaction-event schema (`cyberos-capture` / `services/memory` TASK-MEMORY-121) and the chat
  emitters (`services/chat/src/capture.rs`, TASK-MEMORY-122). Message bodies are never copied into the brain -
  events carry a pointer to `chat_messages`, not the text.
- Consent gate: `SqlConsentGate` reads the acknowledgement ledger and denies by default. An unacknowledged
  person yields zero rows, so no one is recorded until they have acknowledged the current notice.
- Governance: `services/eval` (TASK-EVAL-001) - publish and read the monitoring notice, the
  `subject_acknowledgment` ledger, `POST /v1/eval/ack`, access control, data-subject requests, and retention
  categories.
- Deploy wiring: the P0 compose already carries `CAPTURE_ENABLED` (default `false`) and
  `CHAT_AUDIT_DATABASE_URL` (default blank) on the chat service - safely dormant.

## Safety properties (why this is not surveillance)

- Default deny: nothing is captured for a person until they acknowledge the notice.
- Pointers, not content: the brain stores who/what/when/where and a reference to the chat row, not the message
  body.
- Tamper-evident: events chain into the hash-linked `l1_audit_log`.
- Human in the loop: any evaluation is drafted for a person to review and decide; the model does not decide
  pay, progression, or employment.

## The sequence

1. Governance (Phase 0). Finalize the existing bilingual notice `docs/legal/data-monitoring-and-evaluation-notice.md`
   (co-located with the three signed contracts) with counsel: set the lawful-basis wording, the retention
   periods, and the data contact. This is the gate for everything below.
2. Confirm the services are deployed. The activation needs `services/eval` and `services/memory` running in
   the P0 stack, with their migrations applied (eval `0001_governance.sql`; memory `0001..0008`). Recall
   (Phase 2) also needs the embed sidecar, which still needs its Dockerfile - capture (Phase 1) does not, so
   capture can go on before recall.
3. Publish the notice. `POST /v1/eval/notice` (founder only) with the finalized text. Confirm with
   `GET /v1/eval/notice`.
4. Collect acknowledgements. Each employee acknowledges - the code defaults `ack_source = 'signed_contract'`,
   so the intended path is a signed addendum to the employment agreement, recorded per person via
   `POST /v1/eval/ack`. The governance status endpoint shows acknowledged vs not.
5. Flip the link. In the P0 env (`deploy/vps/.env.p0`), set `CHAT_AUDIT_DATABASE_URL` to the brain database
   and `CAPTURE_ENABLED=true`, then roll. From this point, acknowledged people's chat interactions chain into
   the brain; unacknowledged people are still skipped.
6. Verify. Confirm interaction-event rows appear for an acknowledged test user and none for an unacknowledged
   one, and that the Memory and Audit view shows the new `memory.interaction_event` rows.

## Turning it back off

Set `CAPTURE_ENABLED=false` and roll. Capture stops immediately; the recorded history remains in the
tamper-evident log under the retention policy.
