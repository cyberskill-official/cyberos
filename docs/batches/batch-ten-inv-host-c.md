---
batch: batch/ten-inv-host-c
members:
  - TASK-INV-012
started: 2026-07-26T10:00:00Z
ended: 2026-07-26T11:12:00Z
route_backs: 0
gate_reasks: 0
tokens: unknown
---

# batch/ten-inv-host-c — Wise webhook HTTP host

## Context

After #166 (host-b api_calls), deferred queue next: INV Wise HTTP host.

## Shipped (target)

### TASK-INV-012
- `POST /v1/webhooks/wise/:profile_id` (signature auth, no JWT)
- `CachedPublicKeys` 24h TTL + force_refresh once on verify fail
- `WiseWalQueue` + background `WiseProcessor` (in-memory receipts)
- `CashAppStub` until TASK-INV-006
- Binary `cyberos-inv` (`INV_LISTEN_ADDR`, `WISE_PUBLIC_KEY_PEM`)

## Out of scope

- Live Pg persist / migrate CI
- TASK-INV-006 cash-app
- Memory audit kinds / admin restore
