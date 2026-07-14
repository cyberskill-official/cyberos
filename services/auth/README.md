# cyberos-auth — AUTH module runtime

Implements **tenant + subject create + RLS** as the Wave 2 first-slice. Spec source: [`docs/tasks/auth/TASK-AUTH-001…109`](../../docs/tasks/auth/).

## Production deploy

The canonical Fargate runbook lives in the **root README §4 — AUTH deploy** ([`../../README.md`](../../README.md#4--auth-deploy)). It walks: Rust build (`cargo build --release -p cyberos-auth`), the 20-migration sequence (idempotent post-2026-05-19 fix to migration 0004), JWK keygen for production, the `bootstrap` CLI for root-tenant + first-admin (with TOTP setup), HTTP-server boot with all flags, smoke tests via JWKS + OIDC discovery + TOTP flow, ECR push + ECS `update-service`, rollback (including the `sqlx migrate revert` requirement for cross-migration rollbacks), Grafana dashboard, and AWS Secrets Manager layout.

Use the **Quick start** below for local dev only.

## Quick start

```bash
# 1. Boot Postgres + Redis (from services/dev/)
cd ../dev && docker compose up -d

# 2. Apply migrations + start the service
cd ../auth
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos
sqlx migrate run                    # applies 0001..0005
cargo run

# 3. Smoke-test — create a tenant
curl -X POST http://localhost:7700/v1/admin/tenants \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: smoke-test-1' \
  -d '{"slug": "acme-corp", "display_name": "Acme Corp"}'

# 4. Create a subject in that tenant
curl -X POST http://localhost:7700/v1/admin/subjects \
  -H 'Content-Type: application/json' \
  -H 'X-Tenant-Id: <the-uuid-returned-above>' \
  -d '{"handle": "@admin", "password": "hunter2!", "email": "admin@acme.test"}'
```

## What ships in this slice

| FR | Component | State |
|---|---|---|
| TASK-AUTH-001 | `POST /v1/admin/tenants` + idempotency table | ✓ shipped |
| TASK-AUTH-002 | `POST /v1/admin/subjects` + bcrypt | ✓ shipped |
| TASK-AUTH-003 | RLS USING + WITH CHECK on tenants/subjects/admin_idempotency | ✓ shipped (migrations 0004 + 0005) |
| TASK-AUTH-003 | RLS isolation integration test | ✓ shipped (`tests/rls_isolation_test.rs`, `#[ignore]` until CI sets up Postgres) |
| TASK-AUTH-004 | JWT issuance + JWKS endpoint | not yet |
| TASK-AUTH-005 | Admin REST (list/revoke/unrevoke + cursor pagination) | not yet |
| TASK-AUTH-006 | `cyberos-auth bootstrap` CLI | not yet |
| TASK-AUTH-101 | 22-role RBAC catalogue | not yet |
| TASK-AUTH-102 | TOTP + WebAuthn MFA | not yet |
| TASK-AUTH-103 | SAML 2.0 SSO | not yet |
| TASK-AUTH-104 | OIDC SSO | not yet |
| TASK-AUTH-105 | Passkey enrolment + login | not yet |
| TASK-AUTH-106 | Impossible-travel detection | not yet |
| TASK-AUTH-107 | HIBP breach check | not yet |
| TASK-AUTH-108 | Lumi tenant identity JWT | not yet |
| TASK-AUTH-109 | Stub-to-full migration tooling | not yet |

## Layout

```
auth/
├── Cargo.toml
├── migrations/
│   ├── 0001_tenants.sql               # tenants table + root seed
│   ├── 0002_admin_idempotency.sql     # Idempotency-Key dedupe
│   ├── 0003_subjects.sql              # subjects table + constraints
│   ├── 0004_rls_roles.sql             # cyberos_app + cyberos_ro roles
│   └── 0005_rls_enable_on_tables.sql  # ENABLE RLS + policies (USING + WITH CHECK)
├── src/
│   ├── main.rs                # axum binary entry
│   ├── lib.rs
│   ├── state.rs               # AppState — connects + auto-SET ROLE cyberos_app
│   ├── handlers.rs            # /v1/admin/tenants + /v1/admin/subjects + /healthz
│   ├── idempotency.rs         # admin_idempotency CRUD
│   └── models.rs              # Tenant / Subject / Create*Request structs
└── tests/
    └── rls_isolation_test.rs  # property test: cross-tenant SELECT returns 0 rows
```

## Open invariants (per task-audit skill)

- §3.1 rule 1 — root tenant is `Uuid::nil()`. ✓ enforced (`tenants` seed row + `cyberos_types::TenantId::ROOT`).
- §3.4 rule 13 — RLS MUST have BOTH USING and WITH CHECK. ✓ enforced (migration 0005).
- §3.4 rule 12 — append-only tables must `REVOKE UPDATE, DELETE`. Not applicable yet — `tenants` and `subjects` are mutable; `admin_idempotency` is effectively append-only (PK + 24h TTL).
- §3.7 rule 22 — every outbound RPC carries W3C `traceparent`. Not applicable yet (no outbound RPCs in this slice).

## Next implementation steps

Per the BACKLOG `§0.6` deploy roadmap, Wave 2 advances in this order: TASK-AUTH-001/002/003 (this slice) → 004 (JWT) → 005 (admin REST) → 006 (bootstrap CLI) → 101 (RBAC) → 102 (MFA) → 103/104 (SAML/OIDC) → 105 (Passkey) → 106 (impossible-travel) → 107 (HIBP) → 108 (Lumi) → 109 (migration).

## License

Apache-2.0. See repo root `LICENSE`.
