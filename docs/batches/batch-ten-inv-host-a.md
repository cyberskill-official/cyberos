---
batch: batch/ten-inv-host-a
members:
  - TASK-TEN-203
  - TASK-TEN-204
started: 2026-07-26T05:49:00Z
ended: 2026-07-26T08:15:00Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---

# batch/ten-inv-host-a — TEN plan HTTP + ai_tokens metering emit

## Context

Residual host wiring deferred from `batch/ten-inv-ready` (#163). Operator chose
option A: new residual tasks TEN-203 / TEN-204 (next-id stems, not 002b/004b).

## Shipped

### TASK-TEN-203
- Auth routes `GET|POST /v1/admin/tenants/:id/plan` calling `decide_plan_change`
- Migration `0034_plan_tier_enum_and_history.sql` — TEXT→enum, sandbox→starter, P0301 trigger
- Create-tenant rejects `sandbox`
- Unit tests: error map + decide wiring; ignored Postgres P0301 test

### TASK-TEN-204
- `cyberos-ai-gateway` → `cyberos-metering`
- `metering_emit::emit_ai_tokens` from `cost_reconcile` Success + Cancelled(Some)
- Process-local WalQueue + InMemoryRecorder; non-blocking on overflow
- Integration tests for quantity / idempotency / zero-skip

## Out of scope (honest)

- Auth middleware `api_calls` emit + overage 402
- Pg metering drain / sqlx migrate CI for metering
- INV Wise HTTP host / pubkey cache / WAL processor / cash-app
- TEN-002 residuals: rate-limit, dry-run, history GET, founder override route, memory `ten.plan_changed`
- Status hub regen (after done-flips)

## Verify

```sh
cd services
cargo test -p cyberos-auth --lib plan_admin -- --test-threads=1
cargo test -p cyberos-auth --test plan_change_http_test -- --test-threads=1
cargo test -p cyberos-ai-gateway --test metering_ai_tokens_emit_test -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```
