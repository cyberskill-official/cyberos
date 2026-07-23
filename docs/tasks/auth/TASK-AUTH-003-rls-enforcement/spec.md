---
id: TASK-AUTH-003
title: "RLS enforcement at every tenant-scoped table — USING + WITH CHECK + per-connection app.tenant_id + property test"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: auth
priority: p0
status: done
verify: T
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: 2026-05-19
memory_chain_hash: null
related_tasks: [TASK-AUTH-001, TASK-AUTH-002, TASK-AUTH-004, TASK-AUTH-005, TASK-AI-018]
depends_on: [TASK-AUTH-001]
blocks: [TASK-AUTH-004, TASK-AUTH-005, TASK-PROJ-001, TASK-MEMORY-101, TASK-HR-001, TASK-TIME-001, TASK-KB-001, TASK-CRM-001, TASK-OKR-001, TASK-TEN-004]

source_pages:
  - website/docs/modules/auth.html#rls
  - website/docs/legal/multi-tenancy-isolation.html
source_decisions:
  - DEC-058 (cross-tenant data leakage = 0; ANY leak = sev-1)
  - DEC-118 (RLS USING + WITH CHECK both required; USING alone permits silent wrong-tenant inserts)
  - DEC-119 (cyberos_app vs cyberos_ops role separation; superuser NEVER used by application)
  - PDPL Art. 7 + GDPR Art. 32 (multi-tenant isolation as fundamental security control)

language: rust 1.81 + postgres SQL
service: cyberos/services/auth/
new_files:
  - services/auth/src/rls/mod.rs
  - services/auth/src/rls/with_tenant.rs
  - services/auth/src/rls/registry.rs
  - services/auth/migrations/0004_rls_roles.sql
  - services/auth/migrations/0005_rls_enable_on_tables.sql
  - services/auth/tests/rls_isolation_test.rs
  - services/auth/tests/rls_isolation_test.rs
  - services/auth/tests/rls_property_test.rs
  - services/auth/tests/rls_isolation_test.rs
  - services/auth/tests/rls_registry_completeness_test.rs
  - .github/workflows/rls-property-gate.yml
modified_files:
  # canonical policy template
  - services/auth/src/rls/templates.rs
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - file_write: .github/workflows/rls-property-gate.yml
  - bash: cd services/auth && cargo test rls
disallowed_tools:
  - use parameter interpolation in SET LOCAL (must use SET LOCAL with sqlx bind, NOT format!)
  #4 — superuser bypasses RLS)
  - run application queries as `postgres` superuser (per §1
  #2 — silent wrong-tenant inserts must be blocked)
  - omit WITH CHECK on policies (per §1
  #11)
  - skip the rls-property-gate CI workflow (per §1

effort_hours: 12
subtasks:
  - "0.5h: 0004_rls_roles.sql (cyberos_app NOSUPERUSER + cyberos_ops with bypass + grants)"
  - "1.0h: rls/templates.rs canonical USING + WITH CHECK template"
  - "1.0h: 0005_rls_enable_on_tables.sql (apply policies to ALL existing tenant-scoped tables)"
  - "1.0h: rls/registry.rs — TENANT_SCOPED_TABLES list + completeness check at boot"
  - "1.0h: rls/with_tenant.rs — `with_tenant(pool, tenant_id, async fn)` helper using bound parameter"
  - "1.0h: rls_basic_test — SELECT/INSERT under different app.tenant_id contexts"
  - "1.0h: rls_with_check_test — INSERT with wrong tenant_id rejected"
  - "2.0h: rls_property_test — 1000 tenant pairs × 10K queries, zero cross-tenant reads"
  - "0.5h: rls_role_separation_test — cyberos_app cannot bypass; cyberos_ops can"
  - "0.5h: rls_registry_completeness_test — every tenant-scoped table is in registry"
  - "0.5h: .github/workflows/rls-property-gate.yml CI workflow"
  - "0.5h: with_tenant_secure helper (sqlx bind, NEVER format!)"
risk_if_skipped: "DEC-058 multi-tenancy invariant unenforced. ANY query touching a tenant-scoped table without RLS context could return another tenant's data — and silently. The single most catastrophic failure class in CyberOS. Without WITH CHECK, an INSERT with the wrong tenant_id silently writes into the wrong tenant's space (USING clause makes it invisible to subsequent SELECTs; the data is there but the writer doesn't know). Without the property test, regressions ship undetected. Without role separation, application code with a SQL injection vulnerability can pivot to superuser-equivalent access."
---

## §1 — Description (BCP-14 normative)

Every tenant-scoped Postgres table in the CyberOS schema **MUST** have RLS enabled with both USING and WITH CHECK policies. The enforcement contract:

1. **MUST** enable RLS on every table in `rls/registry.rs::TENANT_SCOPED_TABLES`. Slice 1 list: `subjects`, `sessions`, `ai_invocations`, `cost_ledger`, `cost_ledger_hold`, `tenant_policies`, `chat_workspaces`, `chat_messages`, `kb_articles`, `proj_issues`, `audit_outbox`, `admin_idempotency_keys`. New tenant-scoped tables added to the schema MUST be added to the registry; CI lint (`rls_registry_completeness_test`) blocks PRs that introduce a tenant-scoped table without registry entry.
2. **MUST** apply BOTH `USING (tenant_id = current_setting('app.tenant_id', true)::uuid)` AND `WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid)` clauses on every policy. USING filters reads; WITH CHECK filters writes. Policies with only USING permit silent wrong-tenant INSERTs (the row writes successfully but is invisible to subsequent SELECTs); WITH CHECK rejects the INSERT outright. Both clauses MUST exist on every policy.
3. **MUST** set `app.tenant_id` per-transaction at session start via `SET LOCAL app.tenant_id = $1` with `$1` bound through sqlx's parameter binding (NEVER via `format!` string interpolation — that would be SQL injection). The `with_tenant` helper in `rls/with_tenant.rs` enforces this pattern; direct SQL writes must use the helper.
4. **MUST** use a dedicated DB role `cyberos_app` (with `NOSUPERUSER`, `NOBYPASSRLS`) for all application queries. The `postgres` superuser is reserved for migrations only; application code connects as `cyberos_app` exclusively. The connection-string config enforces this — application boot fails if connecting as superuser.
5. **MUST** define a separate `cyberos_ops` role (with `BYPASSRLS`) for operational queries that legitimately need cross-tenant reads (compliance reports, ops investigations). Use of `cyberos_ops` MUST be logged via PgAudit + emit a `auth.rls_bypass_used` memory audit row per query; sev-2 alarm on increment beyond a baseline (any unexpected use is investigated).
6. **MUST** be tested via property-based test: 1000 random tenant pairs × 10K queries × ZERO cross-tenant reads. The test mirrors TASK-AI-018's pattern but covers all tenant-scoped tables, not just the cache. Pattern: insert rows under tenant A; switch context to tenant B; assert queries return zero rows from A. Repeated 1000× with random tenant ids.
7. **MUST** apply RLS to NEW tenants automatically via TASK-AUTH-001's `rls::apply_for_tenant` hook. Adding a new tenant adds `tenant_<id>_<table>` policy entries on every registered table. The registry IS the contract; missing entries cause cross-tenant exposure on the new tenant's first call.
8. **MUST** include WITH CHECK rejection in error responses — when a SQL INSERT violates the WITH CHECK clause, the application catches the postgres error code `42501` (insufficient privilege) and returns `403 FORBIDDEN` with `{"error":"rls_check_violation","table":"<table>","attempted_tenant":"<uuid>","actual_tenant":"<uuid>"}`. Without this surfacing, RLS rejections look like generic SQL errors.
9. **MUST** verify at boot that every table in `TENANT_SCOPED_TABLES` actually has RLS enabled (`pg_tables.rowsecurity = true`) AND has both USING + WITH CHECK policies. Boot fails if any registered table is missing RLS — this catches "operator added the table to the registry but forgot to apply the migration" before the gateway accepts traffic.
10. **MUST** be deterministic: same `(app.tenant_id, query)` pair always returns the same result. Race conditions in setting `app.tenant_id` (e.g., a connection returned to the pool with stale setting) are catastrophic — `with_tenant` uses `SET LOCAL` (transaction-scoped) to ensure no leakage across requests on the same connection.
11. **MUST** be CI-gated via `.github/workflows/rls-property-gate.yml` on every PR touching `services/auth/migrations/**`, `services/auth/src/rls/**`, OR any file matching `services/*/migrations/*.sql` (any module's migrations might add a tenant-scoped table). The workflow runs property test + registry completeness test + role-separation test; non-skip enforcement per TASK-AI-018 §1 #13 pattern.
12. **SHOULD** emit OTel metrics:
- `auth_rls_policy_count{table}` (gauge; per-table policy count, should equal tenant count).
- `auth_rls_check_violations_total{table}` (counter; sev-1 — expected to be near-zero in production).
- `auth_rls_bypass_used_total{role}` (counter; sev-2 alarm on increment).
- `auth_rls_with_tenant_calls_total{tenant_id}` (counter; tracks helper usage for context-validation audits).

---

## §2 — Why this design (rationale for humans)

**Why USING + WITH CHECK, not just USING (§1 #2)?** USING only filters READS — `SELECT * FROM subjects` returns only the current tenant's subjects. But INSERT with a hand-crafted `tenant_id` (e.g., from a SQL injection or a buggy code path) succeeds; the row writes successfully and disappears from the writer's subsequent SELECTs (because USING filters them out). The writer thinks the row failed; the row actually exists in some other tenant's space. Silent failure mode. WITH CHECK rejects the INSERT at write time with a clear error. Both clauses together are defense-in-depth.

**Why dedicated `cyberos_app` role (§1 #4)?** Superuser bypasses RLS by Postgres design — RLS only applies to non-superuser roles. If the application connects as `postgres`, every query bypasses every policy. The dedicated `cyberos_app` role with `NOSUPERUSER NOBYPASSRLS` ensures RLS always applies. The boot-time check (refuse to start if connecting as superuser) prevents the misconfiguration class.

**Why a separate `cyberos_ops` BYPASSRLS role (§1 #5)?** Some operations legitimately need cross-tenant reads — compliance reports ("show me all subjects across all tenants where..."), ops investigations ("which tenant has the runaway request?"), regulator audits. A separate role makes these legitimate uses explicit AND auditable: every query as `cyberos_ops` is logged via PgAudit + emits a memory audit row. Without the separate role, the only options would be "use superuser" (no isolation between legitimate ops and emergency root) or "do without" (legitimate ops fail).

**Why `SET LOCAL` instead of `SET` (§1 #3)?** `SET` persists for the connection's lifetime; if the connection returns to the pool with `app.tenant_id = A` set, the next user of that connection (potentially tenant B's request) starts with A's context — catastrophic cross-tenant leak. `SET LOCAL` is transaction-scoped: it resets at COMMIT/ROLLBACK. Connection pools recycle connections without state contamination.

**Why parameter binding for SET LOCAL (§1 #3)?** `format!("SET LOCAL app.tenant_id = '{}'", tenant_id)` is SQL injection if `tenant_id` is attacker-controlled. Even though it's typed as Uuid (which can't contain injection chars), defense in depth says: never interpolate, always bind. The `with_tenant` helper uses `sqlx::query("SET LOCAL app.tenant_id = $1").bind(tenant_id)` which is safe by construction.

**Why a registry of tenant-scoped tables (§1 #1)?** Without a registry, "is this table tenant-scoped?" is a code-archaeology question. A PR adding a new table might forget to add RLS — and the omission is invisible until production data lands in the wrong tenant. The registry is the explicit contract; the boot-time check (§1 #9) catches drift; the CI test (`rls_registry_completeness_test`) catches PRs that introduce un-registered tables.

**Why surface RLS violations as 403 Forbidden (§1 #8)?** Postgres returns error code `42501 (insufficient_privilege)` for WITH CHECK rejections. Bubbled up as a 500 error, this looks like a server bug. Catching the specific code and returning 403 with detail tells the operator EXACTLY what happened (which table, which tenants involved) — actionable feedback for debugging.

**Why mirror TASK-AI-018's property-test pattern (§1 #6)?** TASK-AI-018 proved the pattern works for cache cross-tenant isolation (200K random ops, 7 regression scenarios, adversarial inputs). RLS is the same invariant ("zero cross-tenant reads") at the DB layer. Reusing the proven pattern reduces risk and keeps the test infrastructure consistent.

**Why CI-gate against ANY module's migrations (§1 #11)?** A new tenant-scoped table might be introduced by TASK-CHAT-007 or TASK-PROJ-014 — modules that don't touch `services/auth/`. Gating only on `services/auth/migrations/**` would miss these. The broader path filter (`services/*/migrations/*.sql`) catches every migration; the registry-completeness test asserts the registry was updated.

**Why `auth.rls_bypass_used` audit row on every cyberos_ops query (§1 #5)?** A regulator asking "did anyone access cross-tenant data during the period under review?" gets a positive answer (rows showing every legitimate cross-tenant query) rather than absence-of-evidence. The sev-2 alarm on baseline drift catches abuse: if cyberos_ops usage suddenly 10x's, an investigation starts.

---

## §3 — API contract

### Roles + base RLS migration

```sql
-- services/auth/migrations/0004_rls_roles.sql
CREATE ROLE cyberos_app NOSUPERUSER NOBYPASSRLS LOGIN PASSWORD '<env>';
CREATE ROLE cyberos_ops NOSUPERUSER BYPASSRLS LOGIN PASSWORD '<env>';

GRANT USAGE ON SCHEMA public TO cyberos_app, cyberos_ops;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO cyberos_app, cyberos_ops;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO cyberos_app, cyberos_ops;
```

```sql
-- services/auth/migrations/0005_rls_enable_on_tables.sql
-- Apply policy to every tenant-scoped table (registry mirror).

ALTER TABLE subjects ENABLE ROW LEVEL SECURITY;
ALTER TABLE subjects FORCE ROW LEVEL SECURITY;   -- applies to table owner too
CREATE POLICY subjects_tenant_isolation ON subjects
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

ALTER TABLE sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE sessions FORCE ROW LEVEL SECURITY;
CREATE POLICY sessions_tenant_isolation ON sessions
    USING      (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

-- ... same pattern for: ai_invocations, cost_ledger, cost_ledger_hold, tenant_policies,
-- chat_workspaces, chat_messages, kb_articles, proj_issues, audit_outbox, admin_idempotency_keys
```

### Registry

```rust
// services/auth/src/rls/registry.rs
pub struct TenantScopedTable { pub name: &'static str, pub tenant_column: &'static str }

pub const TENANT_SCOPED_TABLES: &[TenantScopedTable] = &[
    TenantScopedTable { name: "subjects",                 tenant_column: "tenant_id" },
    TenantScopedTable { name: "sessions",                 tenant_column: "tenant_id" },
    TenantScopedTable { name: "ai_invocations",           tenant_column: "tenant_id" },
    TenantScopedTable { name: "cost_ledger",              tenant_column: "tenant_id" },
    TenantScopedTable { name: "cost_ledger_hold",         tenant_column: "tenant_id" },
    TenantScopedTable { name: "tenant_policies",          tenant_column: "tenant_id" },
    TenantScopedTable { name: "chat_workspaces",          tenant_column: "tenant_id" },
    TenantScopedTable { name: "chat_messages",            tenant_column: "tenant_id" },
    TenantScopedTable { name: "kb_articles",              tenant_column: "tenant_id" },
    TenantScopedTable { name: "proj_issues",              tenant_column: "tenant_id" },
    TenantScopedTable { name: "audit_outbox",             tenant_column: "tenant_id" },
    TenantScopedTable { name: "admin_idempotency_keys",   tenant_column: "tenant_id" },
];

/// §1 #9: boot-time check that every registered table has RLS enabled with USING + WITH CHECK.
pub async fn verify_rls_at_boot(pool: &PgPool) -> Result<(), RlsBootError> {
    for t in TENANT_SCOPED_TABLES {
        let enabled: bool = sqlx::query_scalar(
            "SELECT rowsecurity FROM pg_tables WHERE schemaname='public' AND tablename=$1",
        ).bind(t.name).fetch_one(pool).await?;
        if !enabled {
            return Err(RlsBootError::NotEnabled { table: t.name.into() });
        }

        let policies: Vec<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT qual, with_check FROM pg_policies WHERE tablename = $1",
        ).bind(t.name).fetch_all(pool).await?;
        if policies.is_empty() {
            return Err(RlsBootError::NoPolicy { table: t.name.into() });
        }
        for (using_clause, with_check_clause) in policies {
            if using_clause.is_none() {
                return Err(RlsBootError::MissingUsing { table: t.name.into() });
            }
            if with_check_clause.is_none() {
                return Err(RlsBootError::MissingWithCheck { table: t.name.into() });
            }
        }
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum RlsBootError {
    #[error("RLS not enabled on table {table}")]
    NotEnabled { table: String },
    #[error("no RLS policy on table {table}")]
    NoPolicy { table: String },
    #[error("policy on {table} missing USING clause")]
    MissingUsing { table: String },
    #[error("policy on {table} missing WITH CHECK clause")]
    MissingWithCheck { table: String },
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}
```

### `with_tenant` helper

```rust
// services/auth/src/rls/with_tenant.rs
use uuid::Uuid;
use sqlx::{PgPool, Postgres, Transaction};
use std::future::Future;

pub async fn with_tenant<F, Fut, T>(pool: &PgPool, tenant_id: Uuid, f: F) -> Result<T, sqlx::Error>
where
    F: for<'c> FnOnce(&'c mut Transaction<'_, Postgres>) -> Fut,
    Fut: Future<Output = Result<T, sqlx::Error>>,
{
    let mut tx = pool.begin().await?;
    // §1 #3: bound parameter, NEVER format!.
    sqlx::query("SET LOCAL app.tenant_id = $1::text").bind(tenant_id.to_string())
        .execute(&mut *tx).await?;
    let result = f(&mut tx).await?;
    tx.commit().await?;
    Ok(result)
}

/// Surface postgres error code 42501 as a typed RlsCheckViolation.
pub fn classify_pg_error(err: &sqlx::Error) -> Option<RlsCheckViolation> {
    if let sqlx::Error::Database(db) = err {
        if db.code().as_deref() == Some("42501") {
            return Some(RlsCheckViolation { detail: db.message().into() });
        }
    }
    None
}

#[derive(Debug)]
pub struct RlsCheckViolation { pub detail: String }
```

---

## §4 — Acceptance criteria

1. `app.tenant_id = A`: `SELECT * FROM subjects` returns only A's rows (zero from other tenants).
2. `app.tenant_id = B`: same SELECT returns only B's rows.
3. `INSERT INTO subjects (tenant_id, ...) VALUES ('<other-tenant-uuid>', ...)` under `app.tenant_id = A` returns postgres error 42501; row NOT inserted.
4. Property test: 1000 random tenant pairs × 10K queries → 0 cross-tenant reads (mirrors TASK-AI-018).
5. New tenant from TASK-AUTH-001 → `tenant_<id>_<table>` policies exist on every registered table.
6. `cyberos_app` role cannot bypass RLS (`SELECT current_setting('is_superuser')` returns `'off'`; queries respect policies).
7. `cyberos_ops` role bypasses RLS — sees all tenants' rows; emits `auth.rls_bypass_used` audit row per query.
8. Boot-time check fails if a registered table doesn't have RLS enabled.
9. Boot-time check fails if a policy is missing WITH CHECK clause.
10. CI workflow `rls-property-gate.yml` blocks merge on cross-leak detection.
11. Every table introduced in any module's migration MUST be added to TENANT_SCOPED_TABLES OR explicitly excluded with a `// non-tenant-scoped` comment in the schema file (CI lint enforces).
12. RLS check violations surface as HTTP 403 with table + attempted_tenant + actual_tenant fields.
13. `with_tenant` helper uses bound parameter (NEVER format!) — code-search lint asserts.
14. `SET LOCAL` (not `SET`) used in helper — connection pool returned with no residual context.
15. `cyberos_app` boot connection MUST be NOSUPERUSER + NOBYPASSRLS — boot fails if connected as superuser.
16. Operator queries via `cyberos_ops` emit memory audit rows + sev-2 alarm on >baseline rate.

---

## §5 — Verification

```rust
// services/auth/tests/rls_isolation_test.rs
#[tokio::test]
async fn select_returns_only_current_tenant_rows() {
    let pool = test_pool_as_cyberos_app().await;
    let tenant_a = test_helper::create_tenant().await;
    let tenant_b = test_helper::create_tenant().await;
    test_helper::insert_subject(tenant_a, "a@x.com").await;
    test_helper::insert_subject(tenant_b, "b@x.com").await;

    rls::with_tenant(&pool, tenant_a, |tx| async move {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subjects").fetch_one(&mut **tx).await?;
        assert_eq!(count, 1);
        Ok(())
    }).await.unwrap();

    rls::with_tenant(&pool, tenant_b, |tx| async move {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subjects").fetch_one(&mut **tx).await?;
        assert_eq!(count, 1);
        Ok(())
    }).await.unwrap();
}
```

```rust
// services/auth/tests/rls_isolation_test.rs
#[tokio::test]
async fn insert_with_wrong_tenant_id_blocked_by_with_check() {
    let pool = test_pool_as_cyberos_app().await;
    let a = test_helper::create_tenant().await;
    let b = test_helper::create_tenant().await;

    let err = rls::with_tenant(&pool, a, |tx| async move {
        sqlx::query("INSERT INTO subjects (tenant_id, email, password_hash) VALUES ($1, $2, $3)")
            .bind(b)                           // Wrong tenant!
            .bind("evil@x.com")
            .bind("$2b$12$...")
            .execute(&mut **tx).await
    }).await.expect_err("expected 42501");

    let violation = rls::with_tenant::classify_pg_error(&err).expect("expected RlsCheckViolation");
    assert!(violation.detail.contains("policy"));

    // Confirm row NOT inserted.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subjects WHERE email='evil@x.com'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);
}
```

```rust
// services/auth/tests/rls_property_test.rs
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn no_cross_tenant_reads_under_random_pairs(
        ops_per_pair in 5..50_usize,
    ) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let pool = test_pool_as_cyberos_app().await;
            let a = test_helper::create_tenant().await;
            let b = test_helper::create_tenant().await;

            // Insert ops_per_pair rows under tenant A.
            for i in 0..ops_per_pair {
                rls::with_tenant(&pool, a, |tx| async move {
                    sqlx::query("INSERT INTO subjects (tenant_id, email, password_hash) VALUES ($1, $2, $3)")
                        .bind(a).bind(format!("a{i}@x.com")).bind("$2b$12$...")
                        .execute(&mut **tx).await.map(|_| ())
                }).await.unwrap();
            }

            // From tenant B context, assert none of A's rows visible.
            rls::with_tenant(&pool, b, |tx| async move {
                let leaked: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM subjects WHERE email LIKE 'a%@x.com'"
                ).fetch_one(&mut **tx).await?;
                prop_assert_eq!(leaked, 0);
                Ok(())
            }).await.unwrap();
        });
    }
}
```

```rust
// services/auth/tests/rls_isolation_test.rs
#[tokio::test]
async fn cyberos_app_cannot_bypass_rls() {
    let pool = test_pool_as_cyberos_app().await;
    let is_super: String = sqlx::query_scalar("SELECT current_setting('is_superuser')")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(is_super, "off");

    let bypass: String = sqlx::query_scalar(
        "SELECT rolbypassrls::text FROM pg_roles WHERE rolname=current_user"
    ).fetch_one(&pool).await.unwrap();
    assert_eq!(bypass, "false");
}

#[tokio::test]
async fn cyberos_ops_can_bypass_rls_and_emits_audit() {
    let pool = test_pool_as_cyberos_ops().await;
    let _ = test_helper::insert_subjects_in_multiple_tenants(3).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM subjects").fetch_one(&pool).await.unwrap();
    assert!(count >= 3, "ops role should see all tenants");

    let audit_count = memory_test_helper::count_rows_since("auth.rls_bypass_used", recently()).await;
    assert!(audit_count >= 1, "ops bypass MUST emit audit row");
}
```

```rust
// services/auth/tests/rls_registry_completeness_test.rs
#[tokio::test]
async fn every_tenant_scoped_table_in_registry() {
    let pool = test_pool_as_postgres_for_introspection().await;
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT tablename FROM pg_tables WHERE schemaname='public'
         AND tablename NOT IN (SELECT tablename FROM pg_tables WHERE schemaname='public' AND tablename LIKE '_sqlx%')"
    ).fetch_all(&pool).await.unwrap();

    let mut missing = vec![];
    for t in &tables {
        let has_tenant_id: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM information_schema.columns
                           WHERE table_name = $1 AND column_name = 'tenant_id')",
        ).bind(t).fetch_one(&pool).await.unwrap();
        if has_tenant_id && !rls::registry::TENANT_SCOPED_TABLES.iter().any(|r| r.name == t) {
            missing.push(t.clone());
        }
    }
    assert!(missing.is_empty(),
        "tables with tenant_id column not in TENANT_SCOPED_TABLES registry: {missing:?}");
}
```

CI workflow:

```yaml
# .github/workflows/rls-property-gate.yml
name: RLS Property Gate
on:
  pull_request:
    paths:
      - 'services/auth/migrations/**'
      - 'services/auth/src/rls/**'
      - 'services/*/migrations/*.sql'
      - '.github/workflows/rls-property-gate.yml'
jobs:
  rls-gate:
    runs-on: ubuntu-22.04
    timeout-minutes: 10
    services:
      postgres:
        image: postgres:16
        env: { POSTGRES_PASSWORD: pass }
        ports: ['5432:5432']
        options: --health-cmd "pg_isready"
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - run: |
          cd services/auth
          sqlx migrate run
          cargo test rls -- --test-threads=1 --format=json | tee report.json
          if grep -c '"event":"ignored"' report.json | xargs test 0 -lt; then exit 1; fi
```

---

## §6 — Implementation skeleton

See §3 + §5 for full code. Boot integration:

```rust
// services/auth/src/lib.rs (additions)
pub async fn run() -> Result<(), Error> {
    let pool_app = PgPool::connect_with(connect_opts_for_role("cyberos_app")).await?;
    rls::registry::verify_rls_at_boot(&pool_app).await?;
    // Refuse to bind if RLS not enabled on registered tables.
}
```

---

## §7 — Dependencies

- **TASK-AUTH-001** — Tenant create invokes `rls::apply_for_tenant` which uses the canonical template.
- **TASK-AUTH-004 (downstream)** — JWT issuance reads from `subjects` table; relies on RLS context being set.
- **TASK-AI-018** — Cache cross-tenant test pattern; reused here for DB-layer testing.
- Crates: `proptest@1`, `sqlx@0.7` (postgres), `uuid@1`, `thiserror@1`.
- Postgres 16+ (RLS + `pg_policies` view).
- PgAudit extension for `cyberos_ops` query logging.

---

## §8 — Example payloads

### RLS check violation surfaced as HTTP 403

```http
POST /v1/admin/subjects HTTP/1.1
{ "tenant_id": "<other-tenant>", ... }

→ 403 Forbidden
{
  "error": "rls_check_violation",
  "table": "subjects",
  "attempted_tenant": "<other-tenant>",
  "actual_tenant": "<caller-tenant>"
}
```

### Audit row `auth.rls_bypass_used`

```json
{
  "kind": "auth.rls_bypass_used",
  "payload": {
    "operator_id": "stephen@cyberos.world",
    "role": "cyberos_ops",
    "query_sha256": "...",
    "tables_accessed": ["subjects", "ai_invocations"],
    "request_id": "ops_..."
  }
}
```

### Boot failure

```text
ERROR rls_boot_check_failed: RLS not enabled on table 'kb_articles'
ERROR refusing to bind; operator must run migrations OR remove kb_articles from TENANT_SCOPED_TABLES
```

---

## §9 — Open questions

All resolved. Deferred:
- Cross-region RLS (per-region database with replication) — slice 6+.
- Per-row ACL beyond tenant_id (e.g., subject-scoped rows visible only to creator) — slice 5+.
- Time-limited RLS bypass token for emergency ops (auto-expires in 1 hour) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| RLS policy missing on a new tenant-scoped table | boot-time check via `verify_rls_at_boot` | Gateway refuses to bind | Operator runs migration; redeploy |
| Property test finds cross-leak | proptest panics in CI | PR blocked | Engineer fixes policy or query |
| INSERT with wrong tenant_id under cyberos_app | WITH CHECK violation (postgres 42501) | 403 RLS_CHECK_VIOLATION | Caller fixes tenant_id |
| `cyberos_ops` query without audit emit | integration test asserts | Test fails → PR blocked | Add memory_writer call |
| Application connects as `postgres` (superuser) | boot check `is_superuser='on'` | Gateway refuses to bind | Operator fixes connection-string config |
| `SET` used instead of `SET LOCAL` | code-grep lint | PR blocked | Use `with_tenant` helper exclusively |
| `format!` interpolation in SET LOCAL | code-grep lint for `format!.*SET LOCAL` | PR blocked | Use bound parameter |
| New tenant created but RLS apply fails | TASK-AUTH-001 transaction rollback | Tenant NOT created | By design (TASK-AUTH-001 §1 #12) |
| `TENANT_SCOPED_TABLES` registry incomplete (table has tenant_id but not registered) | `rls_registry_completeness_test` | PR blocked | Add to registry |
| `pg_policies` query returns missing WITH CHECK | boot check | Gateway refuses to bind | Operator updates migration to add WITH CHECK |
| Connection pool returns connection with stale `app.tenant_id` | impossible by design (SET LOCAL transaction-scoped) | N/A | By design |
| Race: SET LOCAL + concurrent query on same connection | sqlx serializes within tx | No race | By design |
| Sev-2 alarm on cyberos_ops baseline drift | metric `auth_rls_bypass_used_total` increment > baseline | Operator investigates | Standard ops process |
| pg_audit not installed | `cyberos_ops` queries not logged | Compliance gap | Operator installs extension |
| Migration adds tenant_id column but doesn't enable RLS | boot check + registry test | Both checks fail | Operator updates migration + registry |
| Replica server runs without RLS sync | replica boot check fails | Replica refuses promotion | Operator investigates replication |
| Operator manually grants superuser to cyberos_app | boot check catches `is_superuser='on'` | Refuse to bind | Revoke superuser |
| `cyberos_ops` token leaked to non-ops user | abuse via audit-log review | sev-1 incident | Rotate role password; audit recent queries |
| FORCE ROW LEVEL SECURITY missing on a table | table owner could bypass | possibly catch via ad-hoc test | Add to migration |

---

## §11 — Notes

- USING + WITH CHECK is the load-bearing pattern. Without WITH CHECK, malicious or buggy INSERTs can write into the wrong tenant silently. The §10 row "INSERT with wrong tenant_id under cyberos_app" is the proof that WITH CHECK actually catches the case.
- `SET LOCAL` (transaction-scoped) is the only safe way to set `app.tenant_id`. `SET` (session-scoped) leaves residual state on connection-pool return — a connection that handled tenant A's request would serve tenant B's next request with A's context.
- The `cyberos_app` vs `cyberos_ops` role split is the operational primitive. Application code never needs cross-tenant reads; ops occasionally does. The split makes legitimate cross-tenant queries explicit + auditable.
- The boot-time check (§1 #9) is the structural defence against "operator forgot to run the migration." It refuses to bind rather than serve traffic with broken RLS — loud failure beats silent leak.
- The property test pattern is borrowed from TASK-AI-018. Keeping the test infrastructure consistent across modules reduces cognitive load and shares the proven tooling.
- `FORCE ROW LEVEL SECURITY` (in §3 migration) ensures the table OWNER also respects RLS. Without it, the role that owns the table (typically `postgres`) bypasses policies even if NOSUPERUSER. Defence in depth.
- `rls_registry_completeness_test` is a CI-time check that catches "operator added a tenant_id column but forgot to register the table." Without this test, RLS gaps ship to production silently.
- The `with_tenant` helper is the single sanctioned way to set RLS context. Direct sqlx queries that don't use the helper risk forgetting `SET LOCAL` — a code-grep lint (`grep -r 'app.tenant_id' --exclude-dir=rls`) flags any such bypass.
- Sev-2 alarm on `cyberos_ops` baseline drift catches abuse early. A 10x increase in cross-tenant queries is investigated; either it's legitimate (regulator audit underway) or it's not (incident).

---

*End of TASK-AUTH-003. Status: draft (10/10 target).*
