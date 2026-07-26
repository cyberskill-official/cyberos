---
id: TASK-TEN-205
title: "Metering api_calls emit at auth verify_jwt — WalQueue push on success"
template: task@1
type: feature
module: ten
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T08:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-TEN-004, TASK-TEN-204]
blocks: []
related_tasks: [TASK-TEN-004, TASK-TEN-204, TASK-TEN-203]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 3
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 4
service: services/auth
new_files:
  - services/auth/src/metering_emit.rs
  - services/auth/tests/metering_api_calls_emit_test.rs
modified_files:
  - services/auth/Cargo.toml
  - services/auth/src/lib.rs
  - services/auth/src/middleware.rs
source_pages:
  - docs/batches/batch-ten-inv-ready.md
  - docs/batches/batch-ten-inv-host-a.md
  - docs/tasks/ten/TASK-TEN-004-four-axis-metering/spec.md
source_decisions:
  - DEC-700 four metering axes
  - DEC-715 idempotent (tenant, axis, idempotency_key)
  - "2026-07-26 operator: continue after #165 — host-b api_calls emit"
---

# TASK-TEN-205: api_calls metering emit at verify_jwt

## Summary

Wire the second live metering emit path: after a successful JWT-gated request
(`verify_jwt` → handler → 2xx/3xx), push one `MeteringEvent` with `axis = api_calls`,
`quantity = 1` onto a process-local WalQueue / InMemoryRecorder. Failed auth never bills.

## Problem

TASK-TEN-004 §1 #6 requires auth-middleware `api_calls` emit. TEN-204 shipped
`ai_tokens` at cost_reconcile only. Auth middleware has the tenant on the response
path and is the single inheritance point for every JWT-gated route.

## Proposed Solution

1. Add `cyberos-metering` dep to `cyberos-auth` (alongside existing `cyberos-ten`).
2. `services/auth/src/metering_emit.rs` mirroring ai-gateway: OnceLock WalQueue +
   InMemoryRecorder; `emit_api_call(tenant_id, idempotency_key, method, path)`.
3. In `verify_jwt`, after `next.run(request)`, if `response.status().is_success()`,
   call emit with `idempotency_key = format!("{jti}:{method}:{path}:{uuid}")` (unique
   per request; jti alone is session-scoped).
4. Never emit on early 401 paths (missing/invalid/revoked JWT).
5. WAL overflow / lock poison: log; never change the response.
6. Unit + integration tests for emit helper; middleware success-path covered by helper
   contract + a unit test that documents the call site.

## Alternatives Considered

- **Overage 402 in the same slice.** Rejected: needs period usage MV + plan caps admission;
  ledger Out of scope.
- **Emit on request path before handler.** Rejected: TEN-004 ≤200µs doctrine + metering
  must not block; response path after success matches spec.
- **Shared crate for emit helpers.** Deferred; copy the TEN-204 shape for speed.

## Success Metrics

- Primary: one successful JWT-gated call → one ApiCalls event qty=1.
- Guardrail: 401 paths add zero events; existing auth middleware tests still pass.

## Scope

### In scope

- Dep + emit helper + verify_jwt success-path hook + tests.

### Out of scope / Non-Goals

- Overage evaluate / `402 PAYMENT_REQUIRED`
- Pg drain / sqlx migrate CI for metering
- INV Wise host (next deferred batch)
- Seats/storage snapshot jobs

## Dependencies

- TASK-TEN-004 done; TASK-TEN-204 done (emit pattern).

## AI Authorship Disclosure

Generated then reviewed against as-built auth middleware + TEN-204 emit (2026-07-26).

## Acceptance Criteria

1. **Success emit** — `emit_api_call` with qty=1 axis api_calls records once.
2. **Idempotency** — duplicate same key → recorder length unchanged.
3. **Non-blocking** — overflow does not panic / does not alter return contract.
4. **Auth fail** — early unauthorized path does not call emit (documented + unit coverage
   of helper not invoked when status would be 401).
5. **source_service** — `"auth"`.
6. **extra** — includes `method` and `path`.
7. **Dep** — `Cargo.toml` lists `cyberos-metering`.
8. **verify_jwt hook** — success response path calls emit (source citation).

## Verification

```sh
cd services
cargo test -p cyberos-auth --lib metering_emit -- --test-threads=1
cargo test -p cyberos-auth --test metering_api_calls_emit_test -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

## Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| WAL overflow | WalError | log; request OK | Drain later |
| Lock poison | lock err | log; request OK | Restart |
| Bad quantity | validate | skip | N/A (qty=1) |
| Missing jti | UUID fallback | still emit | — |
| Double middleware | unique key | two events if two success | OK |
| Path PII | path only | no query string | Strip query |
| Test isolation | unique keys | — | — |
| Dep missing | cargo | build fail | Add dep |
| Emit before next.run | code review | fail AC | Move after |
| 5xx billed | is_success false | no emit | OK |

---

*End of TASK-TEN-205.*
