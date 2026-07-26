---
batch: batch/ten-inv-ready
members:
  - TASK-TEN-002
  - TASK-TEN-004
  - TASK-INV-004
started: 2026-07-26T04:45:00Z
ended: null
route_backs: 0
gate_reasks: 0
tokens: unknown
---

# batch/ten-inv-ready — greenfield substrate for TEN + INV ready queue

## Context

After merging TASK-OBS-004 (#162), the operator instructed: **Approve, merge then continue in batches**.
The remaining ready queue (TEN-002, TEN-004, INV-004) had no service crates — this batch stands up
library substrates with migrations and unit/integration tests.

## Shipped

### TASK-TEN-002 (`cyberos-ten`)
- Compile-time caps (DEC-778..781) + plan_tier / effective enums
- Pure plan-change decision (upgrade/downgrade/founder/violation/proration)
- Migrations `0004_plan_tier.sql`, `0005_plan_history.sql`
- `docs/tasks/ten/PLAN_CAPS.md`
- Tests: plan_change / downgrade_violation / founder

### TASK-TEN-004 (`cyberos-metering`)
- 4-axis enum + quantity validation + idempotent in-memory recorder
- Overage policy evaluate (block/warn/allow)
- Bounded WAL queue with 90% back-pressure
- Migration `0001_metering_events.sql`
- Cardinality tests

### TASK-INV-004 (`cyberos-inv`)
- Wise event-type + receipt-state enums
- Staleness + event-type parser
- RSA-SHA256 signature verify (DEC-840)
- Migration `0001_wise_webhook_events.sql` (+ unmatched_receipts)
- Signature + idempotency tests

## Deferred (honest Out of scope for this slice)

- Full axum HTTP servers / JWT auth wiring for TEN plan endpoints and INV webhook route
- Auth middleware emit → metering (api_calls) and cost_reconcile → ai_tokens hooks
- Live Postgres integration tests / sqlx migrate in CI for the new crates
- INV cash-app (TASK-INV-006) and TEN-001 provisioning cutover (auth `plan_tier` TEXT→enum)
- Public-key cache + background WAL processor for Wise

## Verification

```sh
cd services
cargo test -p cyberos-ten -p cyberos-metering -p cyberos-inv -- --test-threads=1
```
