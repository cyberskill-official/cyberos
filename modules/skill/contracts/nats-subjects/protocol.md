# `nats_subjects@1` — protocol notes

> Companion document to `CONTRACT.md`. The contract pins names, payloads, QoS, and durability. This file describes the *operational* protocol — connection management, ack semantics, dedup, and error handling — that every publisher and subscriber MUST follow.

## Connection management

### Skills (publishers)

A skill that publishes to NATS MUST:

1. Connect to the NATS cluster URL declared in its runtime config (`nats://nats.cyberos.<tenant>.internal:4222` or equivalent).
2. Authenticate with the skill's NKey or JWT (issued at skill-deployment time by the supervisor).
3. Publish synchronously — `await js.publish(subject, payload)` — and check the returned `PubAck` before logging success. A skill that "fires and forgets" without inspecting the ack breaks at-least-once durability.
4. On `PubAck` failure, retry with exponential backoff (100ms, 200ms, 400ms, 800ms, fail). After fail, write a `op:rejected reason:nats-publish-failed:<subject>` row to `genie.action_log` and surface to the supervisor.

### Supervisor (subscribers)

The LangGraph supervisor's classify-act node MUST:

1. Subscribe with a **durable consumer name** (`cuo-supervisor-classify-act-v<n>`). Ephemeral subscribers lose messages on restart.
2. Use `AckPolicy: Explicit` — call `msg.ack()` only after the classify-act node finishes processing AND the next skill (if any) has been invoked. Acking before invocation breaks at-least-once chaining on supervisor crash.
3. Use a **bounded ack-wait** (default 30s). If the supervisor crashes mid-classification, the message redelivers after the wait and the new supervisor instance picks up where the old one left off.
4. Use **max-deliver = 5**. After 5 failed deliveries, the message lands on the DLQ subject `_dlq.<original-subject>` for human review.

## Ack semantics

| QoS | Publisher gets | Subscriber must |
| --- | --- | --- |
| `at-most-once` | No ack — fire-and-forget | No `msg.ack()` call needed; messages auto-discard. |
| `at-least-once` | `PubAck` confirming JetStream persistence | `msg.ack()` after side-effects complete. Failure → redelivery. |
| `exactly-once` | `PubAck` + dedup window | `msg.ack()` AND the consumer must implement an idempotent side-effect (e.g. UPSERT on a unique constraint). |

## Dedup window

JetStream's `Stream.duplicate_window` is configured per stream. Default: 2 minutes. Skills publishing under at-least-once MAY set a `Nats-Msg-Id` header equal to the payload's `trace_id + event_name` to opt into dedup within the window. This is RECOMMENDED for `refinement_proposed` (where a flapping invariant could publish many duplicate proposals in quick succession) and OPTIONAL for the rest.

## Subject discovery

Subscribers MUST NOT discover subjects dynamically (e.g., via NATS server `$JS.STREAM.LIST`). They MUST subscribe only to subjects declared in `CONTRACT.md`'s inventory table. The registry validator enforces this at skill-deployment time by parsing the skill's source for `nats.subscribe(...)` calls and confirming each token matches a contract-declared subject.

## Error semantics

| Error | Meaning | Action |
| --- | --- | --- |
| `NoResponders` | No subscriber is listening | If publisher requires sync ack: retry; after 5 retries, fall back to writing the payload directly to `genie.action_log` for the supervisor to pick up. |
| `MaxBytesExceeded` | Payload exceeded stream's `max_msg_size` (default 1 MiB) | Reject the publish; surface as `op:rejected reason:nats-payload-too-large`; the skill should split the payload (e.g., reference a file written elsewhere instead of inlining). |
| `Unauthenticated` | Skill's NKey/JWT expired or revoked | Stop publishing; surface to the supervisor; the supervisor coordinates credential rotation with the platform team. |
| `WrongStream` | Subject doesn't match any configured stream | Skill is publishing to a subject not in `CONTRACT.md`. Reject + surface to the user as a contract violation. |

## Retention + replay

A subscriber MAY replay historical messages by creating a fresh durable consumer with `DeliverPolicy: All` (or `ByStartTime`). This is useful for:

- **Backfill** — a new skill subscribed after some events were published; replay them to catch up.
- **Audit reconstruction** — given a trace_id, walk every event published under it.
- **Debugging** — replay a problematic batch through a debug subscriber.

Replay does NOT bypass ack semantics. Subscribers replaying historical messages MUST still call `msg.ack()` for at-least-once subjects; failure to do so causes infinite redelivery within the bounded ack-wait until the consumer is deleted.

## Cross-tenant isolation

NATS subjects are tenant-scoped by virtue of the cluster URL. Each tenant runs its own NATS cluster (or a logically-isolated leaf-node attached to a shared cluster with import/export rules). Subjects do NOT cross tenant boundaries. A skill in tenant A cannot subscribe to `cuo.task_author.task_written` in tenant B even if the subject string is identical — the cluster URLs differ and the NKeys/JWTs do not authenticate across.

## Observability

Every publish + ack emits an OBS metric:

- `nats_publish_total{subject, skill_id, success}` — counter
- `nats_publish_latency_ms{subject, skill_id}` — histogram
- `nats_consumer_lag_msgs{subject, consumer_name}` — gauge (subscriber side)
- `nats_dlq_total{subject}` — counter

Dashboards live under the operations runbook in `cuo/cseco/observability/`.

## Citations

- **DEC-029** — NATS event bus baseline.
- **DEC-090..093** — registry v0.2.0 contract expansion.
- **`CONTRACT.md`** — subject inventory + payload references.
- **`schema.json`** — canonical payload shapes.
