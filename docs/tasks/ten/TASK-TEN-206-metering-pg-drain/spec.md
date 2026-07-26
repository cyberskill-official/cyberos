---
id: TASK-TEN-206
title: "Metering Pg Recorder + WAL drain + sqlx migrate CI"
template: task@1
type: feature
module: ten
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T11:20:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-TEN-004, TASK-TEN-204, TASK-TEN-205]
blocks: [TASK-TEN-207]
related_tasks: [TASK-TEN-004, TASK-TEN-205, TASK-INV-004]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 2
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 6
service: services/metering
new_files:
  - services/metering/src/pg.rs
  - services/metering/src/drain.rs
  - services/metering/src/usage.rs
  - services/metering/tests/pg_drain_test.rs
modified_files:
  - services/metering/Cargo.toml
  - services/metering/src/lib.rs
  - .github/workflows/awh-gate.yml
source_pages:
  - docs/batches/batch-ten-inv-ready.md
  - docs/tasks/ten/TASK-TEN-205-api-calls-metering-emit/spec.md
source_decisions:
  - DEC-702 append-only metering_events
  - DEC-715 UNIQUE idempotency
  - "2026-07-26 operator: continue after host-c — host-d Pg drain"
---

# TASK-TEN-206: Metering Pg Recorder + WAL drain

## Summary

Persist WAL / recorder events into `metering_events` via sqlx, expose period
usage SUM for admission, and apply metering migrations in awh-gate CI.

## Problem

TEN-204/205 emit into process-local InMemoryRecorder only. No durable floor;
overage 402 cannot trust period counts.

## Proposed Solution

1. `pg.rs`: async INSERT … ON CONFLICT DO NOTHING matching UNIQUE
   `(tenant_id, axis, idempotency_key)`.
2. `usage.rs`: `period_sum(pool, tenant, axis, period_start)` → u64.
3. `drain.rs`: pop WalQueue → record via sync `Recorder` (tests) and async Pg path.
4. awh-gate: `cd services/metering && sqlx migrate run` after auth migrations.
5. Tests: drain→InMemory; Pg tests skip without `DATABASE_URL`.

## Alternatives Considered

- **Full MV + 5-min refresh.** Deferred; SUM over `occurred_at >= period_start` is enough for host-d admission.

## Success Metrics

- Primary: drain N WAL items → N Pg rows (dup keys → no second row).
- Guardrail: migrate applies cleanly in CI.

## Scope

### In scope

- Pg insert + period sum + WAL drain + CI migrate + tests.

### Out of scope / Non-Goals

- 402 admission (TASK-TEN-207)
- Memory audit chain per event
- Seats/storage snapshots / GET /v1/usage
- INV Pg persist

## Dependencies

- TASK-TEN-004 migration `0001_metering_events.sql`.

## Acceptance Criteria

1. **Pg insert** idempotent on unique key.
2. **Drain** empties WAL into recorder/Pg.
3. **period_sum** returns aggregate quantity for axis in window.
4. **CI** awh-gate runs metering `sqlx migrate run`.
5. **No DATABASE_URL** — unit/drain tests still pass offline.

## Verification

```sh
cd services && cargo test -p cyberos-metering -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

---

*End of TASK-TEN-206.*
