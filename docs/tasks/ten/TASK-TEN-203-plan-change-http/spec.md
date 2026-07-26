---
id: TASK-TEN-203
title: "TEN plan-change HTTP host — POST/GET plan + history TX trigger + plan_tier TEXT→enum"
template: task@1
type: feature
module: ten
status: reviewing
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-26T05:49:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-TEN-002]
blocks: [TASK-TEN-003]
related_tasks: [TASK-TEN-002, TASK-TEN-004, TASK-TEN-001]
routed_back_count: 0
verify: T
phase: P2
milestone: "P2 · billing-substrate"
slice: 2
owner: Stephen Cheng
created: 2026-07-26
effort_hours: 8
service: services/auth
new_files:
  - services/auth/src/plan_admin.rs
  - services/auth/migrations/0034_plan_tier_enum_and_history.sql
  - services/auth/tests/plan_change_http_test.rs
modified_files:
  - services/auth/Cargo.toml
  - services/auth/src/lib.rs
  - services/auth/src/handlers.rs
  - services/auth/src/models.rs
  - services/ten/migrations/0004_plan_tier.sql
  - services/ten/migrations/0005_plan_history.sql
source_pages:
  - docs/batches/batch-ten-inv-ready.md
  - docs/tasks/ten/TASK-TEN-002-plan-tiers/spec.md
source_decisions:
  - DEC-771 closed plan_tier enum cardinality 3
  - DEC-775 plan change single TX
  - DEC-776 append-only history + same-TX trigger
  - DEC-777 founder tenant immutable via non-founder path
  - "2026-07-26 operator: option A residual batch ten-inv-host-a"
---

# TASK-TEN-203: TEN plan-change HTTP host

## Summary

Wire the shipped `cyberos-ten` pure decision substrate into the **auth** admin axum host:
`POST` / `GET /v1/admin/tenants/{id}/plan`, persist plan changes with a same-TX history row
enforced by a Postgres trigger, and cut over auth `tenants.plan_tier` from TEXT (+sandbox CHECK)
to the closed `plan_tier` enum.

## Problem

TASK-TEN-002 landed library + additive migrations only (`decide_plan_change`, `PlanShow`,
enum CREATE TYPE). There is no HTTP route, no history TX trigger, and auth still stores
`plan_tier` as TEXT with a CHECK that includes `sandbox`. Operators cannot change plans via
API; billing (TASK-TEN-003) has no host surface to call.

## Proposed Solution

1. Add `services/auth/src/plan_admin.rs` with:
   - `GET /v1/admin/tenants/:tenant_id/plan` → JSON from `PlanShow` (+ `effective_since` when column present, else `created_at`)
   - `POST /v1/admin/tenants/:tenant_id/plan` body `{ target_tier, effective, acknowledge_data_loss?, reason? }` → build `PlanChangeRequest` from JWT claims + DB → `decide_plan_change` → one TX: INSERT `tenant_plan_history` then UPDATE `tenants.plan_tier`
2. Role gate: `tenant-admin` for own tenant, or founder role for any; founder-tenant short-circuit already in `decide_plan_change`.
3. Auth migration `0034_plan_tier_enum_and_history.sql`: create enums if missing; relocate any `sandbox` rows to `starter`; drop TEXT CHECK; cast column to `plan_tier`; ensure `is_founder_tenant` / history table; install `require_plan_history` BEFORE UPDATE trigger (P0301).
4. Auth models: validate create-tenant `plan_tier` against the closed 3-value set (reject `sandbox` at API).
5. Depend on `cyberos-ten` from `cyberos-auth`.

Error mapping: `Forbidden`/`FounderImmutable` → 403; `SameTier` → 409 `no_change`; `DowngradeViolation` → 409 `downgrade_violation`.

## Alternatives Considered

- **New `cyberos-ten` HTTP binary.** Rejected for this slice: tenants + JWT already live in auth; travel-policy is the established admin nest pattern.
- **Defer TEXT→enum to TASK-TEN-001.** Rejected: host UPDATE must write the enum type; leaving TEXT would block the trigger and sqlx typing.
- **Full TEN-002 residual (rate-limit, dry-run, history GET, founder override route, metering plan_change event).** Rejected: explicit Out of scope for host-a; ledgered below.

## Success Metrics

- Primary: authenticated `tenant-admin` can upgrade Starter→Team via POST and see Team caps on GET; history row exists; bare UPDATE without history fails at DB.
- Guardrail: existing auth create/list tenant tests still pass; no `sandbox` accepted at create API.

## Scope

### In scope

- Auth routes GET/POST plan; TX persist; P0301 trigger; TEXT→enum cutover; create-path allowlist; unit/integration tests for decision wiring + HTTP error mapping (Postgres tests when `DATABASE_URL` available, otherwise library + handler unit tests with mocked decide path).

### Out of scope / Non-Goals

- 24h plan-change rate limit, dry-run query, `GET …/plan/history`, `DELETE …/plan/scheduled`, founder override separate route (TEN-002 §1 #14–#23 residuals).
- Memory audit emit `ten.plan_changed` (follow-on once memory bridge pattern chosen for auth plan path).
- Metering synthetic `plan_change` event (DEC-783) — needs TEN-204 recorder surface.
- Auth middleware `api_calls` emit; INV Wise host; status hub regen.

## Dependencies

- `TASK-TEN-002` done (library substrate).
- Auth JWT + `tenant-admin` role catalogue (TASK-AUTH-101).

## AI Authorship Disclosure

Generated then reviewed against as-built `services/ten` + `services/auth` deep map (2026-07-26).

## Acceptance Criteria

1. **GET plan** — JWT `tenant-admin` for tenant T receives 200 with `tier`, `caps` matching `caps_for(tier)`, and `is_founder_tenant`.
2. **POST upgrade** — Starter→Team with `effective: immediate` returns 200; `tenants.plan_tier` is `team`; one `tenant_plan_history` row with positive `proration_amount_cents` when mid-period.
3. **Downgrade violation** — usage above target seats without `acknowledge_data_loss` → 409 `downgrade_violation`.
4. **Founder lock** — non-founder actor against `is_founder_tenant=true` → 403 `founder_tenant_plan_immutable`.
5. **Same tier** — POST with current tier → 409 `no_change`.
6. **History trigger** — `UPDATE tenants SET plan_tier=…` without prior history INSERT in same TX → SQLSTATE matching P0301 / raise exception.
7. **Enum cutover** — `plan_tier` column type is enum; create-tenant rejecting `sandbox` with 400; cardinality of enum = 3.
8. **Auth dep** — `cyberos-auth` depends on `cyberos-ten`; handlers call `decide_plan_change` (not a reimplemented decision tree).

## Verification

```sh
cd services
cargo test -p cyberos-auth --test plan_change_http_test -- --test-threads=1
cargo test -p cyberos-ten -- --test-threads=1
bash .cyberos/cuo/gates/run-gates.sh
```

## Failure Modes

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Missing history INSERT | P0301 trigger | TX abort | Handler always INSERT then UPDATE |
| Sandbox row at migrate | Migrate relocate to starter | Enum cast succeeds | Log count relocated |
| Stale TEXT in Rust | Compile / sqlx | Build fail | Use PlanTier / String as_str |
| Concurrent plan POSTs | Unique race on history | Last writer wins per TX | Rate-limit later |
| JWT wrong tenant | Role gate | 403 | Client retry |
| DB down | sqlx error | 500 | Retry |
| Invalid target_tier JSON | Serde / parse | 400 | Client fix |
| Downgrade ack false | decide_plan_change | 409 | Ack or reduce usage |
| Founder immutable | decide_plan_change | 403 | Founder path later |
| Migration order vs TEN 0004 | IF NOT EXISTS / DO blocks | Idempotent | Re-run migrate |

---

*End of TASK-TEN-203.*
