# cyberos-auth — AUTH module runtime

Implements **tenant + subject create + RLS** as the Wave 2 first-slice. Spec source: [`docs/feature-requests/auth/FR-AUTH-001…109`](../../docs/feature-requests/auth/).

## Production deploy

The canonical Fargate runbook lives in the **root README §4 — AUTH deploy** ([`../../README.md`](../../README.md#4--auth-deploy)). It walks: Rust build (`cargo build --release -p cyberos-auth`), the 20-migration sequence (idempotent post-2026-05-19 fix to migration 0004), JWK keygen for production, the `bootstrap` CLI for root-tenant + first-admin (with TOTP setup), HTTP-server boot with all flags, smoke tests via JWKS + OIDC discovery + TOTP flow, ECR push + ECS `update-service`, rollback (including the `sqlx migrate revert` requirement for cross-migration rollbacks), Grafana dashboard, and AWS Secrets Manager layout.

Use the **Quick start** below for local dev only.

## Quick start

```bash
# 1. Boot Postgres + Redis (from services/dev/)
cd ../dev && docker compose up -d

# 2. Apply migrations + start the service
cd ../auth
export DATABASE_URL=postgres://cyberos:cyberos@localhost:5432/cyberos_auth
sqlx migrate run                    # applies 0001..0026
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
| FR-AUTH-001 | `POST /v1/admin/tenants` + idempotency table | ✓ shipped |
| FR-AUTH-002 | `POST /v1/admin/subjects` + bcrypt | ✓ shipped |
| FR-AUTH-003 | RLS USING + WITH CHECK on tenants/subjects/admin_idempotency | ✓ shipped (migrations 0004 + 0005) |
| FR-AUTH-003 | RLS isolation integration test | ✓ shipped (`tests/rls_isolation_test.rs`, `#[ignore]` until CI sets up Postgres) |
| FR-AUTH-004 | JWT issuance + JWKS endpoint | ✓ shipped |
| FR-AUTH-005 | Admin REST (list/revoke + cursor pagination) | ✓ shipped |
| FR-AUTH-006 | `cyberos-auth bootstrap` CLI | ✓ shipped |
| FR-AUTH-101 | 22-role RBAC catalogue | ✓ shipped |
| FR-AUTH-102 | TOTP + WebAuthn MFA | ✓ shipped |
| FR-AUTH-103 | SAML 2.0 SSO | ✓ shipped |
| FR-AUTH-104 | OIDC SSO | ✓ shipped |
| FR-AUTH-105 | Passkey enrolment + login | ✓ shipped |
| FR-AUTH-106 | Impossible-travel detection | ✓ shipped |
| FR-AUTH-107 | HIBP breach check | ✓ shipped |
| FR-AUTH-108 | Lumi tenant identity JWT | ✓ shipped |
| FR-AUTH-109 | Stub-to-full migration tooling | ✓ shipped |

## Layout

```
auth/
├── Cargo.toml
├── migrations/
│   ├── 0001_tenants.sql               # tenants table + root seed
│   ├── 0002_admin_idempotency.sql     # Idempotency-Key dedupe
│   ├── 0003_subjects.sql              # subjects table + constraints
│   ├── 0004_rls_roles.sql             # cyberos_app + cyberos_ro roles
│   ├── 0005_rls_enable_on_tables.sql  # ENABLE RLS + policies (USING + WITH CHECK)
│   └── 0006..0026_*.sql               # JWT/JWKS, RBAC, MFA, SSO, travel, sessions
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

## Open invariants (per feature-request-audit skill)

- §3.1 rule 1 — root tenant is `Uuid::nil()`. ✓ enforced (`tenants` seed row + `cyberos_types::TenantId::ROOT`).
- §3.4 rule 13 — RLS MUST have BOTH USING and WITH CHECK. ✓ enforced (migration 0005).
- §3.4 rule 12 — append-only tables must `REVOKE UPDATE, DELETE`. Not applicable yet — `tenants` and `subjects` are mutable; `admin_idempotency` is effectively append-only (PK + 24h TTL).
- §3.7 rule 22 — every outbound RPC carries W3C `traceparent`. Not applicable yet (no outbound RPCs in this slice).

## Next implementation steps

AUTH's FR-AUTH-001..006 and FR-AUTH-101..109 are implemented. Before live
testing, boot Postgres/Redis, apply all migrations, and run the Postgres-gated
tests with `cargo test -p cyberos-auth -- --ignored`.

## License

Apache-2.0. See repo root `LICENSE`.
