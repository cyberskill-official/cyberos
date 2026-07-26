---
batch: batch/ten-inv-host-b
members:
  - TASK-TEN-205
started: 2026-07-26T08:40:00Z
ended: 2026-07-26T09:31:00Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---

# batch/ten-inv-host-b — auth api_calls metering emit

## Context

After #165 (host-a), deferred queue next item: auth middleware → `api_calls` WAL emit.
Wise host remains for host-c.

## Shipped (target)

### TASK-TEN-205
- `cyberos-auth` → `cyberos-metering`
- `metering_emit::emit_api_call` from `verify_jwt` after successful response
- Process-local WalQueue + InMemoryRecorder; non-blocking

## Out of scope

- Overage 402 / plan-cap admission
- Pg metering drain / CI migrate
- INV Wise HTTP host (host-c)
- Status hub regen after done-flips
