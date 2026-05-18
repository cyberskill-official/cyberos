---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AUTH-001
title: "Tenant create — root-admin in tenant 0 calls POST /v1/admin/tenants with idempotency + RLS provisioning"
module: AUTH
priority: MUST
status: building
verify: T
phase: P0
milestone: P0 · slice 2
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
brain_chain_hash: null
related_frs: [FR-AUTH-002, FR-AUTH-003, FR-AUTH-004, FR-AUTH-005, FR-AUTH-006]
depends_on: []
blocks: [FR-AUTH-002, FR-AUTH-003, FR-AUTH-005, FR-AUTH-006, FR-PROJ-001, FR-TEN-001]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/auth.html#tenant-create
  - website/docs/legal/multi-tenancy.html
source_decisions:
  - DEC-110 (tenant 0 = root tenant; only root-admin in tenant 0 creates tenants at P0)
  - DEC-111 (every tenant-scoped table MUST have RLS policy applied automatically on tenant create)
  - DEC-112 (idempotency-key pattern; repeat POST returns the existing tenant, not 409)
  - PDPL Art. 15 (data subject rights — tenant_id is the boundary primitive)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/Cargo.toml
  - services/auth/src/lib.rs
  - services/auth/src/admin/mod.rs
  - services/auth/src/admin/tenants.rs
  - services/auth/src/admin/idempotency.rs
  - services/auth/src/rls/mod.rs
  - services/auth/src/rls/templates.rs
  - services/auth/src/brain.rs
  - services/auth/migrations/0001_tenants.sql
  - services/auth/migrations/0002_admin_idempotency.sql
  - services/auth/tests/admin_tenant_create_test.rs
  - services/auth/tests/admin_tenant_idempotency_test.rs
  - services/auth/tests/admin_tenant_rls_test.rs
modified_files: []
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cd services/auth && cargo test
  - bash: cd services/auth && sqlx migrate run
disallowed_tools:
  - create tenant 0 via this endpoint (bootstrap is FR-AUTH-006 CLI per §1 #14)
  - allow non-root-admin to create tenants (per §1 #1)
  - skip RLS policy application on success path (per §1 #7)
  - skip BRAIN audit row emission (audit-before-commit per §1 #6)
  - bypass slug regex validation in API layer (defence in depth with DB CHECK)

# ───── Estimated work ─────
effort_hours: 8
sub_tasks:
  - "0.5h: Cargo.toml + crate skeleton (axum, sqlx, jsonwebtoken, uuid, chrono, thiserror)"
  - "0.5h: 0001_tenants.sql migration (table + UNIQUE + CHECK)"
  - "0.5h: 0002_admin_idempotency.sql migration (idempotency_keys table)"
  - "1.0h: validate_slug + validate_name (regex + length checks; same as DB CHECK)"
  - "1.5h: create_tenant transaction (idempotency lookup → insert → RLS provisioning → BRAIN row → commit)"
  - "1.0h: rls/templates.rs — RLS policy SQL templates (per-table USING expression)"
  - "0.5h: rls::apply_for_tenant(tenant_id) iterates registered tables and applies policies"
  - "0.5h: canonical::tenant_created BRAIN audit row builder"
  - "0.5h: Authorisation middleware (tenant_id == Uuid::nil() + role contains root-admin)"
  - "0.5h: Idempotency-Key header parsing + 24h dedup window"
  - "1.5h: Tests — happy + 401 + 403 + 409 + 400 + idempotent-replay + RLS-applied + audit-emitted + p95 latency"
risk_if_skipped: "Every other AUTH FR depends on tenants existing. AI Gateway slice 2+ uses tenant_id from JWT (FR-AI-006 references AUTH JWT extraction). CHAT can't have a workspace. Multi-tenancy starts at zero usable tenants. Without RLS auto-provisioning, every new tenant requires manual SQL — operator bottleneck on every onboarding. Without idempotency, network retries during tenant create produce duplicates with different UUIDs (one tenant becomes two; data scattered across both)."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** expose `POST /v1/admin/tenants` for creating new tenants. The endpoint and surrounding contract obey the following:

1. **MUST** require caller to have `tenant_id == Uuid::nil()` (the nil-UUID `00000000-0000-0000-0000-000000000000`, conventionally referred to as "tenant 0" — the root tenant) AND `role` includes `"root-admin"`. Both conditions are hard requirements; missing either returns `403 FORBIDDEN`. The root tenant itself is bootstrapped via FR-AUTH-006 CLI (chicken-and-egg avoidance per §1 #14).
2. **MUST** accept request body `{ "name": <string 1..=80>, "slug": <string 1..=40 matching /^[a-z][a-z0-9-]*$/> }`. Validation runs at API layer AND via Postgres CHECK constraint (defence in depth).
3. **MUST** create a row in the `tenants` table with auto-generated UUID id, returning `{ "id": <uuid>, "slug": ..., "name": ..., "created_at": <ISO8601>, "suspended": false }`.
4. **MUST** return `409 CONFLICT` with body `{"error":"slug_taken","slug":<slug>}` if `slug` already exists (UNIQUE constraint violation). The 409 is returned EXCEPT when the request carries an `Idempotency-Key` header matching a prior successful create within 24 hours (per §1 #5) — in which case the prior tenant body is returned with the SAME id.
5. **MUST** support idempotency via `Idempotency-Key` header (UUID or arbitrary string ≤ 64 chars). The handler stores `(idempotency_key, request_body_hash, response_body, created_at)` in `admin_idempotency_keys` table with 24h TTL. Repeat POST with the same key + same body returns the prior response with the same id; same key + different body returns `409 CONFLICT` with `{"error":"idempotency_key_reuse","prior_request_hash":<hex16>}`.
6. **MUST** emit exactly one `auth.tenant_created` BRAIN audit row per new tenant (NOT per idempotent replay). The row carries `tenant_id`, `slug`, `name`, `created_by_subject_id` (from JWT), `idempotency_key` (if present), `request_id`. The row is written WITHIN the same Postgres transaction as the tenants insert; transaction rollback rolls back both.
7. **MUST** initialise RLS for the new tenant: every tenant-scoped table registered in `rls/templates.rs` SHALL have the new tenant's RLS policy applied automatically. The applied policy is the standard `USING (tenant_id = current_setting('app.tenant_id')::uuid)` template; tables that need different filters extend the template registry.
8. **MUST** complete in ≤ 100ms p95 (insert + RLS apply + BRAIN row emit + commit). Latency budget asserted by `admin_tenant_create_test.rs` against a live test database.
9. **MUST** return `401 UNAUTHORIZED` if the caller is unauthenticated (no JWT, expired JWT, invalid signature). The 401 body is `{"error":"unauthenticated","reason":"<missing|expired|invalid_sig>"}`.
10. **MUST** return `403 FORBIDDEN` if the caller is authenticated but does NOT meet the root-admin-in-tenant-0 requirement. The 403 body is `{"error":"forbidden","needed":"root-admin in tenant 0"}` — explicit about WHAT permission is missing so operators can grant it correctly.
11. **MUST** return `400 BAD_REQUEST` for malformed slug (uppercase, special chars, starts with digit) OR malformed name (length out of bounds, contains null bytes). The 400 body identifies which field failed and why: `{"error":"invalid_input","field":"slug","reason":"must match /^[a-z][a-z0-9-]*$/, got 'Foo Bar'"}`.
12. **MUST** atomically apply tenants-insert + RLS-provisioning + audit-row emission inside a SINGLE Postgres transaction. ANY step failure rolls back the entire transaction; partial state (tenant exists but RLS not applied; tenant exists but no audit row) is forbidden by construction.
13. **MUST** emit OTel span `auth.create_tenant` with attributes `slug`, `created_by_subject_id`, `outcome` (created | idempotent_replay | conflict | forbidden | invalid_input). Span propagates W3C TraceContext per FR-AI-022.
14. **MUST NOT** create the root tenant (i.e. a tenant whose `id == Uuid::nil()`) via this endpoint — the bootstrap CLI per FR-AUTH-006 owns that creation. The handler explicitly rejects `slug == "root"` AND any attempt to create a tenant with `id == Uuid::nil()` (which is unreachable in practice since UUIDs are randomly generated server-side, but the rejection is retained as a defence-in-depth check).
15. **SHOULD** emit OTel metrics:
    - `auth_tenant_create_total{outcome}` (counter; outcome ∈ created | idempotent_replay | conflict | forbidden | invalid_input | error).
    - `auth_tenant_create_latency_ms` (histogram; SLO p95 < 100ms).
    - `auth_tenant_count` (gauge; total tenants).

---

## §2 — Why this design (rationale for humans)

**Why tenant create requires root-admin in tenant 0 (§1 #1)?** Multi-tenancy is the load-bearing isolation primitive. Allowing arbitrary tenants to create other tenants would invert the privilege hierarchy — a regular tenant could spawn child tenants with different policies. The single-source-of-creation pattern (only root-admin in tenant 0) keeps the org chart of tenants explicit and auditable. P3 introduces self-serve provisioning via the TEN module; until then, ops creates tenants.

**Why idempotency via header rather than implicit dedup on slug (§1 #5)?** Network retries are the operational reality — a tenant create that times out client-side might succeed server-side; the client retries and gets either a duplicate (bad) or a 409 (also bad — caller doesn't know if it succeeded). Idempotency-Key gives the client EXPLICIT control over retry semantics: same key + same body → same result; different body → loud failure. The 24h window is generous for reasonable retry windows; longer windows risk hash collisions and storage growth. This pattern is standard (Stripe, AWS) and clients understand it.

**Why automatic RLS provisioning (§1 #7)?** Every new tenant needs RLS policies on every tenant-scoped table — without them, queries from the new tenant either see nothing (FORCE policies) or see everything (no policies). Manual provisioning is error-prone (forget one table → cross-tenant data exposure). Automatic provisioning + a registry of tables ensures consistency: the registry IS the contract for "what tables need RLS." Adding a new tenant-scoped table requires updating the registry, which is a PR-reviewed change.

**Why a single transaction for insert + RLS + audit (§1 #12)?** Partial states are catastrophic. If the tenant insert succeeds but RLS apply fails, the tenant exists in the DB but queries against it return all tenants' data — an isolation breach. If insert + RLS succeed but audit row fails, the tenant exists but the chain has no record — auditability gap. The single transaction ensures all-or-nothing: either all three steps commit, or none do. The cost (one transaction span instead of three) is trivial vs. the correctness benefit.

**Why explicit error bodies with `field` and `reason` (§1 #11)?** Generic 400 errors force the client to inspect logs. Explicit error bodies let the client display "the slug must contain only lowercase letters, digits, and hyphens" directly to the user — better UX, fewer support tickets. The error structure is consistent across endpoints (FR-AUTH-002+ inherit the pattern).

**Why explicit `needed` field on 403 (§1 #10)?** "Forbidden" without context forces ops to grep code or read source to understand WHY. `"needed":"root-admin in tenant 0"` tells the operator exactly what role/tenant combination would have succeeded — actionable feedback. The pattern surfaces in the BRAIN audit too, so post-mortem investigations of access denials have full context.

**Why 100ms p95 budget (§1 #8)?** Tenant creation isn't user-facing (operators run it during onboarding); but slow creates indicate a bottleneck — likely the RLS-policy-apply loop. The 100ms ceiling forces the implementation to apply policies efficiently (single SQL statement per table, not multiple). At 50 tables × 1ms per policy = 50ms, the budget is comfortable for slice-2 scope and tightens as we add tables.

**Why explicit reject of `slug == "root"` (§1 #14)?** Even though tenant 0 is bootstrapped via CLI, an operator running `POST /v1/admin/tenants {"slug":"root"}` would create a SECOND tenant with that slug (the slug is unique per row, but "root" is conventionally tenant 0). The defence-in-depth rejection prevents the confusing-state scenario.

**Why audit-row-in-transaction (§1 #6)?** The same audit-before-action principle that AI Gateway uses: the audit IS the truth of "did this happen." If we commit the tenant but the audit row write fails (BRAIN unavailable), we have a tenant nobody can prove was created. Including the audit emit in the transaction ensures the chain has the row IF AND ONLY IF the tenant exists.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Type definitions

```rust
// services/auth/src/admin/tenants.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct CreateTenantResponse {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub suspended: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("unauthenticated: {reason}")]
    Unauthenticated { reason: String },
    #[error("forbidden: needed root-admin in tenant 0")]
    Forbidden,
    #[error("invalid input: field={field} reason={reason}")]
    InvalidInput { field: String, reason: String },
    #[error("slug taken: {slug}")]
    Conflict { slug: String },
    #[error("idempotency key reuse: prior_request_hash={prior_hash}")]
    IdempotencyKeyReuse { prior_hash: String },
    #[error("rls provisioning failed: {0}")]
    RlsFailed(String),
    #[error("brain emit failed: {0}")]
    BrainFailed(String),
    #[error("db error: {0}")]
    Db(#[from] sqlx::Error),
}
```

### Migrations

```sql
-- services/auth/migrations/0001_tenants.sql
CREATE TABLE tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug        TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z][a-z0-9-]{0,39}$'),
    name        TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 80),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    suspended   BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE INDEX tenants_created_at_idx ON tenants(created_at DESC);
-- Tenant 0 (root) inserted by FR-AUTH-006 bootstrap CLI; not by this endpoint.
```

```sql
-- services/auth/migrations/0002_admin_idempotency.sql
CREATE TABLE admin_idempotency_keys (
    key                TEXT PRIMARY KEY,
    endpoint           TEXT NOT NULL,
    request_body_hash  CHAR(64) NOT NULL,         -- SHA-256 hex
    response_body      JSONB NOT NULL,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX admin_idempotency_keys_created_at_idx
    ON admin_idempotency_keys(created_at)
    WHERE created_at > NOW() - INTERVAL '24 hours';

-- Sweeper job (FR-AUTH-006 owns) deletes rows older than 24h.
```

### RLS templates

```rust
// services/auth/src/rls/templates.rs
//! Single source of truth for RLS policies applied per tenant.
//! Adding a new tenant-scoped table requires extending TENANT_SCOPED_TABLES.

pub struct TenantScopedTable { pub name: &'static str, pub tenant_column: &'static str }

pub const TENANT_SCOPED_TABLES: &[TenantScopedTable] = &[
    TenantScopedTable { name: "subjects",        tenant_column: "tenant_id" },
    TenantScopedTable { name: "ai_invocations",  tenant_column: "tenant_id" },
    TenantScopedTable { name: "cost_ledger",     tenant_column: "tenant_id" },
    TenantScopedTable { name: "tenant_policies", tenant_column: "tenant_id" },
    // ... more as modules ship
];

pub fn apply_for_tenant_sql(tenant_id: Uuid) -> String {
    let mut sql = String::new();
    for t in TENANT_SCOPED_TABLES {
        sql.push_str(&format!(
            "ALTER TABLE {} ENABLE ROW LEVEL SECURITY;\n\
             CREATE POLICY tenant_{}_{} ON {} \
                FOR ALL USING ({} = current_setting('app.tenant_id')::uuid);\n",
            t.name, tenant_id, t.name, t.name, t.tenant_column,
        ));
    }
    sql
}
```

### Handler skeleton

```rust
// services/auth/src/admin/tenants.rs
pub async fn create_tenant(
    req: CreateTenantRequest,
    idempotency_key: Option<String>,
    pool: &PgPool,
    claims: &Claims,
    request_id: &str,
) -> Result<CreateTenantResponse, TenantError> {
    // §1 #1 + §1 #10
    if claims.tenant_id != Uuid::nil() || !claims.roles.contains(&"root-admin".to_string()) {
        return Err(TenantError::Forbidden);
    }

    // §1 #2 + §1 #11
    validate_slug(&req.slug)?;
    validate_name(&req.name)?;

    // §1 #14
    if req.slug == "root" {
        return Err(TenantError::InvalidInput {
            field: "slug".into(), reason: "slug 'root' is reserved for tenant 0".into(),
        });
    }

    let body_hash = hex::encode(sha256(serde_json::to_vec(&req).unwrap()));

    let mut tx = pool.begin().await?;

    // §1 #5 idempotency check
    if let Some(key) = &idempotency_key {
        if let Some(prior) = idempotency::lookup(&mut tx, key, "/v1/admin/tenants").await? {
            if prior.request_body_hash != body_hash {
                return Err(TenantError::IdempotencyKeyReuse { prior_hash: prior.request_body_hash[..16].into() });
            }
            return Ok(serde_json::from_value(prior.response_body).unwrap());
        }
    }

    // §1 #3 insert tenant
    let row: CreateTenantResponse = sqlx::query_as(
        "INSERT INTO tenants (slug, name) VALUES ($1, $2) RETURNING *",
    ).bind(&req.slug).bind(&req.name).fetch_one(&mut *tx).await
     .map_err(|e| if is_unique_violation(&e) {
         TenantError::Conflict { slug: req.slug.clone() }
     } else { TenantError::Db(e) })?;

    // §1 #7 RLS provisioning
    let rls_sql = rls::templates::apply_for_tenant_sql(row.id);
    sqlx::query(&rls_sql).execute(&mut *tx).await
        .map_err(|e| TenantError::RlsFailed(e.to_string()))?;

    // §1 #6 audit row (within transaction)
    brain::emit_in_tx(&mut tx, brain::canonical::tenant_created(
        row.id, &req.slug, &req.name, claims.subject_id, idempotency_key.as_deref(), request_id,
    )).await.map_err(|e| TenantError::BrainFailed(e.to_string()))?;

    // §1 #5 idempotency record
    if let Some(key) = &idempotency_key {
        idempotency::insert(&mut tx, key, "/v1/admin/tenants", &body_hash, &row).await?;
    }

    tx.commit().await?;
    metrics::tenant_created();
    Ok(row)
}
```

### Validation helpers

```rust
fn validate_slug(slug: &str) -> Result<(), TenantError> {
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r"^[a-z][a-z0-9-]{0,39}$").unwrap()
    });
    if !RE.is_match(slug) {
        return Err(TenantError::InvalidInput {
            field: "slug".into(),
            reason: format!("must match /^[a-z][a-z0-9-]{{0,39}}$/, got {slug:?}"),
        });
    }
    Ok(())
}

fn validate_name(name: &str) -> Result<(), TenantError> {
    if name.is_empty() || name.len() > 80 || name.contains('\0') {
        return Err(TenantError::InvalidInput {
            field: "name".into(),
            reason: format!("must be 1..=80 chars, no null bytes, got {} chars", name.len()),
        });
    }
    Ok(())
}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Root-admin in tenant 0 creates tenant** — `POST /v1/admin/tenants` with valid `name`+`slug` → `201` with full body; UUID populated; `created_at` is recent; `suspended: false`.
2. **Non-root-admin returns 403** — JWT with role `tenant-admin` (not root-admin) → `403` with `{"error":"forbidden","needed":"root-admin in tenant 0"}`.
3. **Wrong tenant returns 403** — JWT with `tenant_id: <some_uuid>` (not nil/0) + role `root-admin` → `403`.
4. **Unauthenticated returns 401** — No `Authorization` header → `401` with `{"error":"unauthenticated","reason":"missing"}`.
5. **Expired JWT returns 401** — Expired JWT → `401` with `reason: expired`.
6. **Duplicate slug returns 409** — Two creates with same `slug` (no idempotency key) → second returns `409` with `{"error":"slug_taken","slug":"<slug>"}`.
7. **Invalid slug (uppercase) returns 400** — `slug: "Foo"` → `400` with `field: slug`, `reason` mentions regex.
8. **Invalid slug (starts with digit) returns 400** — `slug: "1foo"` → `400`.
9. **Name > 80 chars returns 400** — `name: <81 chars>` → `400` with `field: name`.
10. **Name with null byte returns 400** — `name: "Test\0Co"` → `400`.
11. **Reserved slug 'root' returns 400** — `slug: "root"` → `400` with `reason` mentioning reserved.
12. **BRAIN audit row emitted** — Successful create produces exactly one `auth.tenant_created` row in BRAIN with all required fields populated.
13. **Idempotent replay returns prior body** — Two POSTs with same `Idempotency-Key` + same body → second returns the SAME id, no duplicate tenant in DB, no second BRAIN row.
14. **Idempotency-Key reuse with different body returns 409** — Same key + different body → `409` with `idempotency_key_reuse`.
15. **RLS policies created for new tenant** — After successful create, every table in `TENANT_SCOPED_TABLES` has a policy named `tenant_<id>_<table>`.
16. **Latency p95 < 100ms** — 1000 sequential creates; `percentile(latencies, 0.95) < 100`.
17. **Atomic transaction: RLS failure rolls back** — Inject an RLS apply failure; assert no tenant row exists AND no BRAIN audit row written.
18. **OTel span emitted** — `auth.create_tenant` span with `outcome` attribute set per result.

---

## §5 — Verification

```rust
// services/auth/tests/admin_tenant_create_test.rs
use cyberos_auth::admin::tenants::*;

#[tokio::test]
async fn root_admin_creates_tenant() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let resp = create_tenant(
        CreateTenantRequest { name: "Test Co".into(), slug: "test-co".into() },
        None, &pool, &claims, "req_001",
    ).await.unwrap();
    assert_eq!(resp.slug, "test-co");
    assert!(!resp.suspended);
    assert!(brain_test_helper::has_row("auth.tenant_created", &resp.id.to_string()).await);
}

#[tokio::test]
async fn non_root_admin_returns_forbidden() {
    let pool = test_pool().await;
    let claims = tenant_admin_claims();   // not root-admin
    let err = create_tenant(
        CreateTenantRequest { name: "X".into(), slug: "x".into() }, None, &pool, &claims, "req",
    ).await.expect_err("expected Forbidden");
    assert!(matches!(err, TenantError::Forbidden));
}

#[tokio::test]
async fn duplicate_slug_returns_conflict() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let _ = create_tenant(
        CreateTenantRequest { name: "A".into(), slug: "dupe".into() }, None, &pool, &claims, "req1",
    ).await.unwrap();
    let err = create_tenant(
        CreateTenantRequest { name: "B".into(), slug: "dupe".into() }, None, &pool, &claims, "req2",
    ).await.expect_err("expected Conflict");
    assert!(matches!(err, TenantError::Conflict { .. }));
}

#[tokio::test]
async fn invalid_slug_returns_invalid_input() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let err = create_tenant(
        CreateTenantRequest { name: "X".into(), slug: "Foo".into() }, None, &pool, &claims, "req",
    ).await.expect_err("expected InvalidInput");
    match err {
        TenantError::InvalidInput { field, .. } => assert_eq!(field, "slug"),
        e => panic!("wrong variant: {e:?}"),
    }
}

#[tokio::test]
async fn idempotent_replay_returns_same_id() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let key = "idem-001".to_string();
    let req = CreateTenantRequest { name: "Idem".into(), slug: "idem".into() };

    let r1 = create_tenant(req.clone(), Some(key.clone()), &pool, &claims, "req1").await.unwrap();
    let r2 = create_tenant(req.clone(), Some(key.clone()), &pool, &claims, "req2").await.unwrap();
    assert_eq!(r1.id, r2.id);

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE slug = 'idem'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 1);

    let audit_count = brain_test_helper::count_rows("auth.tenant_created", "idem").await;
    assert_eq!(audit_count, 1);
}

#[tokio::test]
async fn idempotency_key_reuse_with_different_body_returns_409() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let key = "idem-002".to_string();

    let _ = create_tenant(
        CreateTenantRequest { name: "A".into(), slug: "ax".into() }, Some(key.clone()), &pool, &claims, "req1",
    ).await.unwrap();
    let err = create_tenant(
        CreateTenantRequest { name: "B".into(), slug: "bx".into() }, Some(key), &pool, &claims, "req2",
    ).await.expect_err("expected IdempotencyKeyReuse");
    assert!(matches!(err, TenantError::IdempotencyKeyReuse { .. }));
}

#[tokio::test]
async fn rls_policies_created_for_new_tenant() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let tenant = create_tenant(
        CreateTenantRequest { name: "R".into(), slug: "rls".into() }, None, &pool, &claims, "req",
    ).await.unwrap();
    for table in cyberos_auth::rls::templates::TENANT_SCOPED_TABLES {
        let policy_name = format!("tenant_{}_{}", tenant.id, table.name);
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_policies WHERE policyname = $1)"
        ).bind(&policy_name).fetch_one(&pool).await.unwrap();
        assert!(exists, "RLS policy {policy_name} not created for table {}", table.name);
    }
}

#[tokio::test]
async fn rls_failure_rolls_back_transaction() {
    let pool = test_pool().await;
    test_helper::inject_rls_apply_failure();
    let claims = root_admin_claims();
    let _ = create_tenant(
        CreateTenantRequest { name: "Roll".into(), slug: "roll".into() }, None, &pool, &claims, "req",
    ).await.expect_err("expected RlsFailed");
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tenants WHERE slug = 'roll'")
        .fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0, "tenant must not exist after RLS failure");
    assert!(!brain_test_helper::has_row("auth.tenant_created", "roll").await);
}

#[tokio::test]
async fn p95_latency_under_100ms() {
    let pool = test_pool().await;
    let claims = root_admin_claims();
    let mut samples = vec![];
    for i in 0..1000 {
        let t0 = std::time::Instant::now();
        let _ = create_tenant(
            CreateTenantRequest { name: format!("L{i}"), slug: format!("lat-{i}") },
            None, &pool, &claims, &format!("req{i}"),
        ).await.unwrap();
        samples.push(t0.elapsed().as_millis() as u64);
    }
    samples.sort();
    let p95 = samples[(samples.len() as f64 * 0.95) as usize];
    assert!(p95 < 100, "p95 {p95}ms exceeds 100ms budget");
}
```

```bash
cd services/auth
sqlx migrate run
cargo test admin_tenant
```

---

## §6 — Implementation skeleton

See §3. Idempotency module:

```rust
// services/auth/src/admin/idempotency.rs
pub struct PriorRecord { pub request_body_hash: String, pub response_body: serde_json::Value }

pub async fn lookup(tx: &mut PgConnection, key: &str, endpoint: &str)
    -> Result<Option<PriorRecord>, sqlx::Error>
{
    sqlx::query_as::<_, (String, serde_json::Value)>(
        "SELECT request_body_hash, response_body FROM admin_idempotency_keys
         WHERE key = $1 AND endpoint = $2 AND created_at > NOW() - INTERVAL '24 hours'",
    ).bind(key).bind(endpoint).fetch_optional(tx).await
        .map(|opt| opt.map(|(h, b)| PriorRecord { request_body_hash: h, response_body: b }))
}

pub async fn insert<T: Serialize>(
    tx: &mut PgConnection, key: &str, endpoint: &str, body_hash: &str, response: &T,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO admin_idempotency_keys (key, endpoint, request_body_hash, response_body)
         VALUES ($1, $2, $3, $4)",
    ).bind(key).bind(endpoint).bind(body_hash).bind(serde_json::to_value(response).unwrap())
     .execute(tx).await?;
    Ok(())
}
```

BRAIN canonical builder:

```rust
// services/auth/src/brain.rs
pub mod canonical {
    pub fn tenant_created(
        id: Uuid, slug: &str, name: &str, created_by: Uuid,
        idempotency_key: Option<&str>, request_id: &str,
    ) -> AuditRow {
        AuditRow {
            kind: "auth.tenant_created".into(),
            payload: serde_json::json!({
                "tenant_id": id, "slug": slug, "name": name,
                "created_by_subject_id": created_by,
                "idempotency_key": idempotency_key,
                "request_id": request_id,
            }),
            ..Default::default()
        }
    }
}

pub async fn emit_in_tx(tx: &mut PgConnection, row: AuditRow) -> Result<(), brain_writer::Error> {
    // Writes to a Postgres outbox table; brain_writer subprocess polls + emits to BRAIN ledger.
    sqlx::query(
        "INSERT INTO brain_outbox (kind, payload_json, created_at) VALUES ($1, $2, NOW())",
    ).bind(&row.kind).bind(&row.payload).execute(tx).await
     .map_err(brain_writer::Error::OutboxInsertFailed)?;
    Ok(())
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AUTH-006 (upstream)** — Bootstrap CLI creates tenant 0; this FR cannot create tenant 0.
- **FR-AUTH-002 (downstream)** — Subject create scopes subjects to a tenant; uses the tenant_id this FR returns.
- **FR-AUTH-003 (downstream)** — RLS enforcement consumes the policies this FR provisions.
- **FR-AUTH-004 (downstream)** — JWT issuance carries `tenant_id` claim referencing tenants this FR created.
- **FR-AI-006 (downstream)** — AI Gateway alias resolution reads `tenant_id` from JWT, queries tenant_policies (a tenant-scoped table requiring RLS).
- **FR-AI-022 (downstream)** — OTel span `auth.create_tenant` carries W3C TraceContext.

### Concept dependencies (shared types)

- `Uuid` (UUIDv4) is the tenant-id primitive across all CyberOS modules.
- `tenant 0` is conversational shorthand for the root tenant whose `id` equals `Uuid::nil()` (the all-zeros UUID `00000000-0000-0000-0000-000000000000`); reserved for bootstrap. tenant_id is `uuid` everywhere — never an integer — and the literal `0` never appears as a tenant identifier in code.
- `slug` is the human-readable tenant identifier; immutable after creation.
- `TENANT_SCOPED_TABLES` registry in `rls/templates.rs` is the single source of truth for "what tables need RLS."
- `Idempotency-Key` header is the standard idempotency primitive (Stripe, AWS pattern).

### Operational / external

- Rust crates: `axum@0.7`, `sqlx@0.7` (postgres + chrono + uuid features), `jsonwebtoken@9`, `uuid@1`, `chrono@0.4`, `serde@1`, `serde_json@1`, `regex@1`, `sha2@0.10`, `hex@0.4`, `thiserror@1`, `once_cell@1`, `tracing@0.1`.
- Postgres 16+ (RLS + JSONB + UUID extension).
- BRAIN module reachable via outbox table (FR-AUTH-006 sets up the brain_writer subprocess).

---

## §8 — Example payloads

### Successful create

```http
POST /v1/admin/tenants HTTP/1.1
Authorization: Bearer <root-admin-jwt>
Content-Type: application/json
Idempotency-Key: 7e57c0de-1234-5678-9abc-def012345678

{ "name": "CyberSkill JSC", "slug": "cyberskill-jsc" }

→ 201 Created
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "slug": "cyberskill-jsc",
  "name": "CyberSkill JSC",
  "created_at": "2026-05-15T14:00:00Z",
  "suspended": false
}
```

### Conflict (duplicate slug)

```http
POST /v1/admin/tenants
{ "name": "Other", "slug": "cyberskill-jsc" }

→ 409 Conflict
{ "error": "slug_taken", "slug": "cyberskill-jsc" }
```

### Idempotent replay

```http
POST /v1/admin/tenants HTTP/1.1
Idempotency-Key: 7e57c0de-1234-5678-9abc-def012345678
{ "name": "CyberSkill JSC", "slug": "cyberskill-jsc" }

→ 201 Created
(same body as first call; same id)
```

### Idempotency-Key reuse with different body

```http
POST /v1/admin/tenants HTTP/1.1
Idempotency-Key: 7e57c0de-1234-5678-9abc-def012345678
{ "name": "Different Co", "slug": "different-co" }

→ 409 Conflict
{ "error": "idempotency_key_reuse", "prior_request_hash": "4b8c0d2f1a7e9c3b" }
```

### Forbidden

```http
POST /v1/admin/tenants HTTP/1.1
Authorization: Bearer <tenant-admin-jwt>

→ 403 Forbidden
{ "error": "forbidden", "needed": "root-admin in tenant 0" }
```

### Invalid slug

```http
POST /v1/admin/tenants HTTP/1.1
{ "name": "Test", "slug": "Foo Bar" }

→ 400 Bad Request
{ "error": "invalid_input", "field": "slug", "reason": "must match /^[a-z][a-z0-9-]{0,39}$/, got 'Foo Bar'" }
```

### Audit row `auth.tenant_created`

```json
{
  "kind": "auth.tenant_created",
  "ts_ns": 1747526400000000000,
  "payload": {
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000",
    "slug": "cyberskill-jsc",
    "name": "CyberSkill JSC",
    "created_by_subject_id": "...",
    "idempotency_key": "7e57c0de-...",
    "request_id": "req_01HZK..."
  }
}
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Tenant deletion (hard delete vs. soft delete via `suspended`) — slice 5+; no production tenant has been deleted to date.
- Tenant rename (`PATCH /v1/admin/tenants/<id> --name=...`) — slice 3+; current model is immutable name.
- Tenant slug change — explicitly OUT of scope; slug is the durable identifier.
- Self-serve tenant provisioning — P3 via TEN module.
- Tenant suspension (`suspended: true`) workflow — slice 4+; the column exists but no endpoint mutates it yet.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Slug conflict (no idempotency key) | UNIQUE constraint violation | 409 with `slug_taken` | Caller picks different slug |
| Idempotent replay (same key + same body) | idempotency table lookup | 201 with prior id | By design |
| Idempotency-Key reuse with different body | hash mismatch | 409 with `idempotency_key_reuse` + prior_hash16 | Caller uses different key |
| Postgres unreachable | sqlx connect error | 503 with `db_unreachable` | Operator investigates DB |
| RLS apply fails (e.g., table doesn't exist) | SQL error in transaction | Transaction rolls back; tenant NOT created; 500 with `rls_failed` | Operator fixes table; re-attempt |
| BRAIN outbox insert fails | sqlx error in transaction | Transaction rolls back; 500 with `brain_failed` | Operator investigates outbox table |
| Unauthenticated request | JWT middleware rejects | 401 with `reason` | Caller obtains valid JWT |
| Expired JWT | JWT validation | 401 with `reason: expired` | Caller refreshes JWT |
| Invalid JWT signature | JWT validation | 401 with `reason: invalid_sig` | Caller obtains JWT from correct issuer |
| Non-root-admin role | claims check | 403 with `needed: root-admin in tenant 0` | Operator grants role |
| Wrong tenant (not 0) | claims check | 403 with `needed: root-admin in tenant 0` | Operator switches context |
| Reserved slug 'root' | early reject | 400 with `reason: reserved` | Caller uses non-reserved slug |
| Invalid slug (uppercase, special chars) | regex check | 400 with `field: slug, reason: regex` | Caller fixes slug |
| Name too long (>80 chars) | length check | 400 with `field: name, reason: length` | Caller shortens name |
| Name with null bytes | byte check | 400 | Caller cleans name |
| Empty body | parse failure | 400 with `error: invalid_input` | Caller sends valid JSON |
| Latency > 100ms | OTel histogram alarm | sev-3 alarm | Operator investigates DB / RLS apply loop |
| Idempotency table grows unbounded | sweeper job missing | Storage growth | FR-AUTH-006 cron sweeps rows > 24h |
| Concurrent inserts with same slug (race) | UNIQUE constraint serializes | One succeeds; other gets 409 | By design |
| New tenant-scoped table not in `TENANT_SCOPED_TABLES` registry | Tenant created without RLS on that table | Cross-tenant data exposure | Operator MUST update registry in PR; CI lint catches missing registry entries |

---

## §11 — Notes

- Slug regex enforced at API level + DB CHECK constraint. Belt + suspenders — defence in depth in case the API validator is bypassed.
- Bootstrap of tenant 0 happens via FR-AUTH-006 CLI, not via this endpoint (chicken-and-egg: tenant 0 needs to exist before root-admin in tenant 0 can call this endpoint).
- The `TENANT_SCOPED_TABLES` registry in `rls/templates.rs` is operationally critical. Every PR adding a new tenant-scoped table MUST extend the registry; a CI lint (`auth_rls_registry_complete_test`) ensures registry entries match table schemas.
- Idempotency key TTL of 24h matches Stripe's default. Shorter (1h) risks legitimate retries failing; longer (7d) bloats storage.
- The `suspended` column exists for future tenant suspension workflows (e.g., "tenant didn't pay; suspend without delete") but no endpoint mutates it at slice 1. Setting `suspended: true` requires manual SQL until a slice-4 admin endpoint ships.
- The BRAIN audit row is written to a Postgres `brain_outbox` table within the same transaction as the tenant insert. The brain_writer subprocess polls the outbox and emits to the BRAIN ledger asynchronously; the in-transaction outbox insert ensures the audit row is durable iff the tenant is durable.
- The 100ms p95 latency budget includes RLS apply (~50ms for 50 tables) + outbox insert (~5ms) + Postgres commit (~20ms). The remaining ~25ms is for validation + query plans.
- Future tenant deletion (slice 5+) is non-trivial: cascading the tenant_id removal across every tenant-scoped table requires careful FK ordering. The current model is "tenants are immutable + suspendable, never deleted at P0."

---

*End of FR-AUTH-001. Status: draft (10/10 target).*
