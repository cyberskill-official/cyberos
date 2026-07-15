---
id: NFR-TEN-008
title: "TEN audit-event durability — tenant lifecycle events MUST never be lost"
module: TEN
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of tenant lifecycle events (create/upgrade/offboard/delete) reach the durable audit store"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-TEN-001, TASK-TEN-104, TASK-TEN-106]
---

## §1 — Statement (BCP-14 normative)

1. Tenant lifecycle events **MUST** land in the durable audit store with at-least-once + idempotent semantics.
2. The audit store **MUST** be append-only; no UPDATE or DELETE in any code path.
3. Retention **MUST** be ≥ 7 years (regulatory).
4. Audit-store unavailability **MUST NOT** silently swallow events — the producer spools locally and retries.
5. Reconciliation: count of lifecycle counter increments ≡ count of audit rows; drift > 0.

## §2 — Why this constraint

Tenant lifecycle events are the platform's legal-evidence backbone. Losing them invalidates downstream audit chains. The local-spool + at-least-once + idempotent pattern is the standard durability triad. 7-year retention matches tax-record statutory floor. The append-only invariant prevents post-hoc rewriting of history.

## §3 — Measurement

- Counter `ten_lifecycle_event_emit_total{event}`.
- Counter `ten_lifecycle_audit_drift_total` — must be 0.
- Gauge `ten_audit_spool_depth` — surfaces transient outages.

## §4 — Verification

- Integration test (T) — emit + verify durable.
- Chaos test (T) — kill audit store; assert spool + replay.
- Reconciliation (T) — daily.

## §5 — Failure handling

- Audit store down → spool + retry.
- Drift > 0 → sev-1; durability promise broken.
- Append-only violation → sev-1; halt; investigate.

---

*End of NFR-TEN-008.*
