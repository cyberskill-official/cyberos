---
id: TASK-INV-012
title: "Wise webhook HTTP host — POST /v1/webhooks/wise/{profile_id} + 24h key cache + WAL processor"
template: task@1
type: feature
module: inv
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T10:00:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-INV-004]
blocks: []
related_tasks: [TASK-INV-004, TASK-INV-006, TASK-TEN-205]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 2
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 8
service: services/inv
new_files:
  - services/inv/src/wise/public_key.rs
  - services/inv/src/wise/wal.rs
  - services/inv/src/wise/handler.rs
  - services/inv/src/wise/processor.rs
  - services/inv/src/bin/cyberos-inv.rs
  - services/inv/tests/wise_host_test.rs
modified_files:
  - services/inv/Cargo.toml
  - services/inv/src/lib.rs
  - services/inv/src/wise/mod.rs
source_pages:
  - docs/batches/batch-ten-inv-ready.md
  - docs/tasks/inv/TASK-INV-004-wise-webhook/spec.md
source_decisions:
  - DEC-840 RSA-SHA256 verify
  - DEC-841 24h public key cache
  - DEC-848 rotation retry once
  - DEC-850 200 empty body ≤5s via WAL offload
  - "2026-07-26 operator: continue after #166 — host-c Wise HTTP"
---

# TASK-INV-012: Wise webhook HTTP host

## Summary

Wire the shipped `cyberos-inv` Wise library into an axum host:
`POST /v1/webhooks/wise/{profile_id}` with 24h PEM cache + one rotation re-fetch on
verify failure, in-memory WAL push for fast 200, and a background processor that
accepts events into an in-memory append log (cash-app stubbed until TASK-INV-006).

## Problem

TASK-INV-004 landed verify + parse + migrations as a library only. No HTTP route,
no key cache, no WAL processor — Wise cannot deliver webhooks.

## Proposed Solution

1. `public_key.rs`: `PublicKeySource` trait + `CachedPublicKeys` (24h TTL);
   on verify fail → `force_refresh` once → retry verify (DEC-848).
2. `wal.rs`: bounded queue of `WiseWalItem { profile_id, event_id, event_type, body }`.
3. `handler.rs`: extract raw body + `X-Signature-SHA256` → verify (with rotation) →
   parse → reject unknown type / stale / profile mismatch (200 for stale per DEC-844) →
   WAL push → empty 200.
4. `processor.rs`: pop WAL → idempotent in-memory receipt set → state `received`;
   cash-app call is a documented stub (`CashAppStub`).
5. Binary `cyberos-inv`: axum router, `WISE_PUBLIC_KEY_PEM` / fetch URL env for key
   source in dev; listen `INV_LISTEN_ADDR` default `:7710`.
6. Tests: cache TTL/refresh, rotation retry accepts new key, handler happy path with
   generated RSA keypair, stale → 200 without WAL growth.

## Alternatives Considered

- **Mount on auth binary.** Rejected: webhook is signature-auth, not JWT; obs-router pattern is a dedicated host.
- **Full sqlx persist + INV-006 match.** Rejected for host-c size; in-memory append + stub ledgered; migration SQL already shipped in INV-004.

## Success Metrics

- Primary: signed valid event → 200 + one WAL/processor receipt.
- Guardrail: bad signature → 401; no cash-app panic.

## Scope

### In scope

- Host route, cache+rotation, WAL, processor stub, binary, unit/integration tests.

### Out of scope / Non-Goals

- Live Postgres INSERT / migrate CI for inv
- TASK-INV-006 cash-app cascade (stub only)
- Memory audit kinds / admin restore route
- Real Wise HTTP fetch in CI (trait + static PEM)

## Dependencies

- TASK-INV-004 done (library).

## AI Authorship Disclosure

Generated then reviewed against as-built inv crate + INV-004 deferred host list (2026-07-26).

## Acceptance Criteria

1. **POST route** exists at `/v1/webhooks/wise/{profile_id}`.
2. **Valid signature** → 200 empty body; event recorded once.
3. **Invalid signature** (no refresh help) → 401.
4. **Rotation retry** — stale cache PEM fails, refreshed PEM succeeds → 200.
5. **Stale event** → 200 without processor receipt.
6. **Unknown event_type** → not processed as receipt (dead-letter or drop with test).
7. **Cash-app stub** — processor does not call INV-006; documents stub.
8. **24h cache** — second fetch not called within TTL for same profile.

## Verification

```sh
cd services
cargo test -p cyberos-inv -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

## Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Bad sig | verify err | 401 | Operator check key |
| Key fetch fail | source err | 401 after retry | Retry later |
| WAL overflow | push err | 503 or 200+drop | Raise capacity |
| Stale | is_stale | 200 no process | — |
| Dup event | HashSet | 200 no reprocess | — |
| Unknown type | parse None | dead-letter | Ops |
| Profile mismatch | check | 401/200 policy | Spec: treat as bad |
| Cash-app missing | stub | received only | INV-006 |
| Body too large | limit 1MiB | 413 | Client |
| Panic in processor | catch/log | event retried | DLQ later |

---

*End of TASK-INV-012.*
