---
id: NFR-AUTH-002
title: "AUTH RLS cross-tenant leak — zero rows leaked under property test sweep of 1k random tenants"
module: AUTH
category: security
priority: MUST
verification: T
phase: P0
slo: "Property test: 0 cross-tenant rows returned across 1000 random tenant_id pairings on every RLS-protected table"
owner: CSO
created: 2026-05-18
related_tasks: [TASK-AUTH-003, TASK-AUTH-001]
---

## §1 — Statement (BCP-14 normative)

1. Every tenant-scoped Postgres table **MUST** carry a Row-Level Security (RLS) policy `tenant_id = current_setting('app.tenant_id')::uuid` applied to SELECT, UPDATE, DELETE.
2. The `app.tenant_id` setting **MUST** be set by the AUTH-issued connection middleware on every connection check-out from the pool, before any tenant-scoped query is issued.
3. A property-based test **MUST** generate ≥ 1000 random `(tenant_a_jwt, tenant_b_data, table_name)` triples and verify that a query under tenant_a's JWT never returns any tenant_b row. Zero false hits permitted.
4. RLS policies **MUST** be applied automatically by `services/auth/src/rls/templates.rs` on tenant create — operators **MUST NOT** apply RLS manually per tenant.
5. The CI gate **MUST** scan every new migration for tables with `tenant_id` column and fail if RLS policy is not applied in the same migration.

## §2 — Why this constraint

RLS is the platform's load-bearing multi-tenancy primitive. A single cross-tenant row leak is sufficient to break SOC 2 multi-tenancy assertions, PDPL Art. 22 tenant-isolation requirements, and the "tenant_id is the boundary primitive" contract (DEC-110, DEC-111). The property test gives statistical confidence; the auto-apply rule of §1 #4 prevents the operator error mode of "forgot to apply RLS for tenant N"; the CI gate of §1 #5 prevents the developer error mode of "forgot to add RLS to a new migration."

## §3 — Measurement

- Counter `auth_rls_unscoped_query_total` — should always be zero. Sev-0 alarm on any non-zero.
- memory audit query `view kind=auth.rls.policy_applied` — every tenant create should produce one row per registered table.
- pg_stat_statements query analyzer — any SELECT/UPDATE/DELETE on a tenant-scoped table without `tenant_id` in the predicate → flagged.

## §4 — Verification

- Property test `services/auth/tests/admin_tenant_rls_test.rs` (T) — generates 1000 (tenant_a, tenant_b, table) triples; asserts no cross-tenant rows returned.
- Migration audit (I) — quarterly DB review confirms every tenant-scoped table has RLS applied.
- Pen test (A) — quarterly external test attempts cross-tenant row read via crafted JWT.

## §5 — Failure handling

- Property test fails → sev-0 PR block; the breaking change reverted, root cause investigated before any other change merges.
- `auth_rls_unscoped_query_total > 0` in prod → sev-0; halt new tenant onboarding; emergency CSO + CTO call.
- New migration missing RLS → CI gate blocks; developer adds the policy.

---

*End of NFR-AUTH-002.*
