---
id: FR-TEN-001
title: "TEN tenant provisioning CLI — `cyberos-ten provision` ops-driven flow with schema namespace + NATS subject + S3 prefix + initial root-admin subject + memory audit"
module: TEN
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-AUTH-001, FR-AUTH-101, FR-AI-016, FR-AI-003, FR-MEMORY-101, FR-TEN-002, FR-TEN-104, FR-TEN-103]
depends_on: [FR-AUTH-001]
blocks: [FR-TEN-002, FR-TEN-004, FR-TEN-101, FR-TEN-103, FR-TEN-104]

source_pages:
  - website/docs/modules/ten.html#what
  - website/docs/modules/ten.html#tenant-provisioning
  - website/docs/modules/ten.html#isolation
source_decisions:
  - DEC-320 (ops-driven provisioning CLI is slice 1; self-serve signup FR-TEN-101 ships at P3)
  - DEC-321 (each tenant gets a unique slug-derived Postgres schema namespace: `tenant_<slug>` with `<slug>` matching `^[a-z][a-z0-9-]{2,40}[a-z0-9]$`)
  - DEC-322 (NATS subject namespace: every cross-service event is prefixed `tenant.<slug>.<topic>` — guarantees no cross-tenant subscription leakage)
  - DEC-323 (S3 bucket prefix: per-residency bucket + `<tenant_id>/` prefix — composes with FR-DOC-001 + FR-EMAIL-001 residency routing)
  - DEC-324 (default residency = vn-1 at slice 1; explicit `--residency <sg-1|eu-1|us-1|vn-1>` flag override; FR-TEN-103 ships full 4-residency provisioning)
  - DEC-325 (initial root-admin subject created with tenant-admin role via FR-AUTH-001's POST /v1/admin/tenants flow; password auto-generated + emitted to operator stdout once)
  - DEC-326 (tenant.status closed enum: provisioning, active, suspended, terminating, terminated — slice 1 ships all 5; transitions in FR-TEN-104)
  - DEC-327 (memory audit kinds: ten.tenant_provisioned, ten.tenant_suspended, ten.tenant_resumed, ten.tenant_terminating, ten.tenant_terminated, ten.tenant_branding_updated, ten.tenant_residency_changed — slice 1 emits ten.tenant_provisioned; others are FR-TEN-104)
  - DEC-328 (REVOKE UPDATE, DELETE on tenant + tenant_status_history from cyberos_app — append-only enforced at SQL grant; status changes via FR-TEN-104)
  - DEC-329 (`cyberos-ten provision` is idempotent on slug — re-running with same slug returns existing tenant; collision on different residency or plan_tier → 409)
  - DEC-330 (exit codes follow cyberos-cli-exit shared crate per feature-request-audit skill rule 9: 0 success, 1 already-exists-idempotent-match, 64 invalid-arg, 65 invalid-data, 73 cant-create, 75 temp-fail, 77 permission-denied)
  - DEC-331 (the initial root-admin password is printed ONCE to the operator's terminal + immediately scrubbed from CLI memory via zeroize; never logged, never persisted in plaintext)
  - DEC-332 (provisioning generates a tenant-bootstrap audit-chain anchor — first chain row that bootstraps the tenant's memory audit chain for FR-AI-003)
  - DEC-058 (tenant-as-degenerate-tenant: until FR-TEN-001 ships, system runs as single-tenant; this FR activates the second-and-beyond tenants)
  - PDPL Art. 4 (data minimisation — operator's name + email recorded as `provisioned_by_subject_id` but the tenant's customer-facing data is empty at creation)

language: rust 1.81
service: cyberos/services/ten/
new_files:
  - services/ten/migrations/0001_tenants.sql                          # tenants table + tenant_status enum + RLS + REVOKE writes
  - services/ten/migrations/0002_tenant_status_history.sql            # append-only status transition log
  - services/ten/migrations/0003_tenant_residency_map.sql             # per-tenant residency tag (consumed by FR-AI-016 + FR-DOC-001 + FR-EMAIL-001)
  - services/ten/src/lib.rs                                           # crate root
  - services/ten/src/types.rs                                         # Tenant struct, TenantStatus enum (5), Residency enum (4 placeholders for full FR-TEN-103)
  - services/ten/src/repo/tenants.rs                                  # CRUD: create, get, list, update_status
  - services/ten/src/repo/status_history.rs                           # append-only writer
  - services/ten/src/provisioning/orchestrator.rs                     # the provision flow: schema → NATS → S3 → AUTH tenant + root-admin
  - services/ten/src/provisioning/schema_namespace.rs                 # Postgres schema namespace creator
  - services/ten/src/provisioning/nats_namespace.rs                   # NATS subject namespace + permission ACL writer
  - services/ten/src/provisioning/s3_prefix.rs                        # S3 bucket prefix initialiser (creates tenant marker object)
  - services/ten/src/provisioning/auth_bootstrap.rs                   # delegates to FR-AUTH-001 POST /v1/admin/tenants + creates initial root-admin
  - services/ten/src/audit/tenant_events.rs                           # canonical ten.tenant_* memory row builders
  - services/ten/src/cli/provision.rs                                 # cyberos-ten provision command
  - services/ten/src/cli/mod.rs                                       # CLI scaffold + subcommand registry
  - services/ten/Cargo.toml                                           # +sqlx, +uuid, +serde, +clap, +zeroize, +rand, +cyberos-cli-exit
  - services/ten/tests/provision_happy_test.rs                        # happy + idempotent + namespace creation
  - services/ten/tests/provision_slug_validation_test.rs              # invalid slugs rejected
  - services/ten/tests/provision_idempotency_test.rs                  # same slug + same residency → existing tenant; different residency → 409
  - services/ten/tests/provision_memory_audit_test.rs                  # ten.tenant_provisioned memory row emitted; carries tenant_id + slug + residency + provisioned_by
  - services/ten/tests/provision_root_admin_test.rs                   # root-admin subject created with tenant-admin role; password printed once
  - services/ten/tests/provision_namespace_isolation_test.rs          # provisioning produces distinct schema/NATS/S3 prefixes per tenant
  - services/ten/tests/provision_residency_default_test.rs            # default residency = vn-1; --residency flag overrides
  - services/ten/tests/append_only_test.rs                            # UPDATE/DELETE rejected on tenants + tenant_status_history
  - services/ten/tests/exit_codes_test.rs                             # 0/1/64/65/73/75/77 codes correct
modified_files:
  - services/auth/src/admin/tenants.rs                                # expose internal helper for FR-TEN-001 to drive tenant + root-admin in one tx

allowed_tools:
  - file_read: services/ten/**
  - file_read: services/auth/src/admin/**
  - file_write: services/ten/{src,tests,migrations}/**
  - bash: cd services/ten && cargo test
  - bash: cd services/ten && cargo run --bin cyberos-ten -- provision --slug ... --display-name "..." --root-admin-email ... --residency vn-1
  - bash: psql -f services/ten/migrations/0001_tenants.sql (local Postgres only)

disallowed_tools:
  - log the initial root-admin password to file or memory audit (per DEC-331 — printed once + zeroised)
  - allow UPDATE on tenants or tenant_status_history (per DEC-328)
  - allow cross-tenant slug collision (per DEC-321 — UNIQUE constraint)
  - allow provisioning to proceed if any namespace allocator fails (transactional all-or-nothing)
  - hard-code residency to vn-1 (per DEC-324 — `--residency` flag default but overridable)
  - ship self-serve signup at slice 1 (per DEC-320 — FR-TEN-101 ships at P3)

effort_hours: 5
sub_tasks:
  - "0.5h: 0001_tenants.sql — tenants table + TenantStatus enum + UNIQUE(slug) + RLS-as-superuser (root-only) + REVOKE writes"
  - "0.3h: 0002_tenant_status_history.sql — append-only log"
  - "0.3h: 0003_tenant_residency_map.sql — per-tenant residency tag"
  - "0.3h: types.rs — Tenant struct + 2 enums"
  - "0.4h: provisioning/orchestrator.rs — transactional 5-step flow"
  - "0.3h: provisioning/schema_namespace.rs — Postgres CREATE SCHEMA tenant_<slug>"
  - "0.3h: provisioning/nats_namespace.rs — NATS account + subject ACL writer"
  - "0.3h: provisioning/s3_prefix.rs — S3 bucket marker object"
  - "0.4h: provisioning/auth_bootstrap.rs — call FR-AUTH-001 + create root-admin"
  - "0.3h: audit/tenant_events.rs — 7 row builders (only ten.tenant_provisioned wired at slice 1)"
  - "0.5h: cli/provision.rs — clap subcommand + flag parsing + exit code mapping"
  - "1.1h: tests — 9 test files covering happy path, slug validation, idempotency, memory audit, root-admin, namespace isolation, residency default, append-only, exit codes"

risk_if_skipped: "CyberOS is multi-tenant by construction (every table has `tenant_id`, every NATS subject is namespaced, every S3 bucket key is prefixed), but until FR-TEN-001 ships, the system runs as a degenerate single tenant (DEC-058). Every downstream TEN FR (FR-TEN-002 plan tiers, FR-TEN-003 Stripe billing, FR-TEN-004 metering, FR-TEN-101 self-serve signup, FR-TEN-103 4-residency provisioning, FR-TEN-104 90-day offboarding, FR-TEN-105 signed-bundle export) depends on the tenant lifecycle primitive. Without DEC-321's per-tenant schema namespace, RLS becomes the ONLY isolation layer — a single SQL bug leaks across tenants. Without DEC-322's NATS namespace, message subscriptions can leak across tenants. Without DEC-323's S3 prefix, body storage can race across tenants. Without DEC-331's password-printed-once policy, ops accidentally pastes root credentials into logs. The 5h effort lands the foundational ops-driven flow + creates the namespace invariants that every other module trusts."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship the `cyberos-ten provision` CLI as the canonical ops-driven tenant provisioning flow. Each requirement:

1. **MUST** define the `tenants` table with: `id UUID PRIMARY KEY`, `slug TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z][a-z0-9-]{2,40}[a-z0-9]$')`, `display_name TEXT NOT NULL CHECK (length(display_name) BETWEEN 1 AND 200)`, `status tenant_status NOT NULL DEFAULT 'provisioning'`, `plan_tier TEXT NOT NULL DEFAULT 'starter'` (placeholder until FR-TEN-002 ships full enum), `residency residency_code NOT NULL DEFAULT 'vn-1'`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `provisioned_at TIMESTAMPTZ`, `terminated_at TIMESTAMPTZ`, `provisioned_by_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`. Full DDL in §3.1.

2. **MUST** declare the closed `tenant_status` Postgres enum with exactly 5 values (per DEC-326): `'provisioning'`, `'active'`, `'suspended'`, `'terminating'`, `'terminated'`. State transitions land in FR-TEN-104; slice 1 sets `provisioning` → `active` at end of successful provisioning.

3. **MUST** declare the closed `residency_code` Postgres enum with exactly 4 values: `'vn-1'`, `'sg-1'`, `'eu-1'`, `'us-1'`. Adding a 5th is an ADR (mirrors the global-residency pattern).

4. **MUST** ship the `cyberos-ten provision` CLI subcommand accepting flags:
    - `--slug <slug>` (required) — kebab-case identifier per the regex.
    - `--display-name <text>` (required) — human-readable name.
    - `--root-admin-email <email>` (required) — initial root-admin subject's email.
    - `--root-admin-display-name <text>` (required) — initial root-admin display name.
    - `--residency <vn-1|sg-1|eu-1|us-1>` (optional; default `vn-1` per DEC-324).
    - `--plan-tier <starter|team|enterprise>` (optional; default `starter`).
    - `--json` (optional; print result as JSON instead of human-readable).

5. **MUST** be **idempotent on slug** (per DEC-329). Re-running with same `--slug` AND same `--residency` AND same `--plan-tier` returns the existing tenant + exit code 1 (idempotent-match). Same slug + different residency or plan_tier → exit code 65 with error `slug_collision_different_attrs`.

6. **MUST** execute the provisioning flow as a **transactional 5-step orchestration** (per DEC-321 + DEC-322 + DEC-323 + DEC-325):
    - Step 1: validate inputs (slug regex, email format, residency in enum); fail-fast.
    - Step 2: INSERT into `tenants` with `status='provisioning'` + create initial `tenant_status_history` row.
    - Step 3: create Postgres schema `tenant_<slug>` via `CREATE SCHEMA` (idempotent — `IF NOT EXISTS`).
    - Step 4: create NATS account + subject ACL: subject namespace = `tenant.<slug>.>` (NATS multi-level wildcard); ACL grants the tenant-admin role pub/sub on the namespace only.
    - Step 5: write S3 bucket marker objects: `s3://cyberos-doc-<residency>-<scope>/<tenant_id>/.cyberos-tenant-marker` for each scope (per FR-DOC-001 + FR-EMAIL-001 bucket layout); marker contains tenant slug + provisioned_at + provisioned_by; existence of marker is the lookup contract.
    - Step 6: delegate to FR-AUTH-001 `POST /v1/admin/tenants` (internal helper) to register tenant in AUTH + create initial root-admin subject with `tenant-admin` role via FR-AUTH-002 + FR-AUTH-101.
    - Step 7: UPDATE tenant status `provisioning` → `active`; UPDATE `provisioned_at = now()`; INSERT a `tenant_status_history` row.
    - Step 8: emit `ten.tenant_provisioned` memory audit row with full provenance.
   Any step failure rolls back the entire flow (Postgres transactions for steps 2/3/6/7; compensating actions for NATS + S3).

7. **MUST** generate the initial root-admin **password server-side** using a cryptographically secure RNG (per DEC-331). Password shape: 32-char base62 (alphanumerics + safe symbols). The password is:
   - Hashed (bcrypt cost 12 per FR-AUTH-002) before persistence.
   - Printed ONCE to the operator's terminal (stdout, with a clear marker `=== ROOT ADMIN PASSWORD (RECORD IMMEDIATELY) ===`).
   - **Immediately zeroised** from CLI memory via the `zeroize` crate after print + hash.
   - NEVER logged to any file, NEVER included in any memory audit row.

8. **MUST** emit `ten.tenant_provisioned` memory audit row at the end of successful provisioning. The row carries `{tenant_id, slug, display_name, status: 'active', plan_tier, residency, provisioned_by_subject_id_hash16, root_admin_subject_id_hash16, ts_ns}`. The row does NOT carry the root-admin password (per DEC-331) or the email (PII-scrubbed to hash16 per FR-MEMORY-111).

9. **MUST** enforce RLS on `tenants` AND `tenant_status_history` with the **superuser-only** policy: `USING (current_setting('auth.is_root_admin', true) = 'true')`. Tenant management is a CyberSkill-operator privilege, not a tenant-admin privilege. Tenant-admins read their own tenant via FR-TEN-107's SPA (slice 3+).

10. **MUST** be **append-only** on `tenants` AND `tenant_status_history` at the SQL-grant layer (per DEC-328). `REVOKE UPDATE, DELETE ON tenants, tenant_status_history FROM cyberos_app;`. Status transitions create new rows in history table; the `tenants.status` column is updated via the elevated `cyberos_provisioner` role (used by this FR's CLI + FR-TEN-104's offboarding orchestrator).

11. **MUST** use the `cyberos_provisioner` SQL role for the CLI's writes — distinct from `cyberos_app` (which has REVOKE'd UPDATE/DELETE). The provisioner role:
    - Has INSERT on `tenants`, `tenant_status_history`, `tenant_residency_map`.
    - Has UPDATE on `tenants.status`, `tenants.provisioned_at`, `tenants.terminated_at` (only those columns).
    - Cannot DELETE anything.
    - Is granted to the CLI's connection role at startup.

12. **MUST** validate slug uniqueness at INSERT time via the `UNIQUE(slug)` constraint. Conflict → 23505 → handler maps to either exit 1 (idempotent match per §1 #5) or exit 65 (collision_different_attrs).

13. **MUST** emit exit codes per feature-request-audit skill rule 9 + DEC-330:
    - 0 = success (new tenant provisioned).
    - 1 = success-idempotent (existing tenant returned).
    - 64 = invalid argument (missing required flag).
    - 65 = invalid data (slug regex fail; email format fail; residency unknown; slug_collision_different_attrs).
    - 73 = cant-create (Postgres / NATS / S3 / AUTH step failed; reversible — operator should investigate + retry).
    - 75 = temp-fail (transient infra failure; retry advised).
    - 77 = permission-denied (operator lacks root-admin role).

14. **MUST** require the operator running the CLI to have the `root-admin` role per FR-AUTH-101 (cross-tenant superuser). The CLI verifies via the operator's JWT before any state mutation. Missing role → exit 77 immediately.

15. **MUST** complete the full provisioning flow in ≤ 30 seconds p95 (excludes any human input — slack for Postgres + NATS + S3 + AUTH round-trips). `provision_perf_test` asserts on a local stack.

16. **MUST** print a structured success block to stdout on completion:
   ```
   ✓ Tenant provisioned
     tenant_id:        <uuid>
     slug:             <slug>
     display_name:     <text>
     residency:        <code>
     plan_tier:        <code>
     postgres_schema:  tenant_<slug>
     nats_namespace:   tenant.<slug>.>
     s3_prefix:        <tenant_id>/
     root_admin_email: <email>
     === ROOT ADMIN PASSWORD (RECORD IMMEDIATELY) ===
     <32-char-password>
     === END ===
   ```
   With `--json`, output is a single JSON object (omitting the password — it's printed in a dedicated stderr block).

17. **MUST** emit OTel span `ten.provision` with attributes: `slug`, `residency`, `plan_tier`, `outcome` (success | idempotent | slug_collision | invalid_input | step_failed_<step> | permission_denied | timeout).

18. **MUST** emit OTel metrics:
    - `ten_provision_total{outcome, residency}` (counter).
    - `ten_provision_duration_ms{outcome}` (histogram).
    - `ten_tenant_count{status, residency}` (gauge — periodic compute).
    - `ten_active_tenants_total` (gauge — count of `status='active'`).

19. **MUST** ship the `tenant_residency_map` table for downstream consumers: `(tenant_id UUID PRIMARY KEY REFERENCES tenants(id), residency residency_code NOT NULL, set_at TIMESTAMPTZ, set_by_subject_id UUID)`. FR-DOC-001's `residency::resolve()` + FR-EMAIL-001's `residency::resolve()` + FR-AI-016's residency policy all consume this table. Slice 1 ships the table; provisioning writes the initial row; FR-TEN-103 ships per-tenant residency change.

20. **MUST** create the tenant's memory audit-chain anchor (per DEC-332): emit a special chain-bootstrap row that becomes the genesis row for the tenant's audit chain. FR-AI-003's memory_writer initialises the chain head from this row; subsequent rows chain to it.

21. **MUST** create the AUTH side via FR-AUTH-001's internal helper `auth::admin::tenants::provision_tenant(slug, display_name, root_admin_email, root_admin_display_name, root_admin_password_hash, operator_subject_id)` returning `(tenant_id, root_admin_subject_id)`. This helper is exposed from FR-AUTH-001 specifically for FR-TEN-001's use; it is NOT exposed via REST (operator privilege only).

22. **MUST** support `cyberos-ten list` returning `id, slug, status, residency, plan_tier, created_at, provisioned_at` for all tenants. Caller MUST have `root-admin` role.

23. **MUST** support `cyberos-ten get --slug <slug>` returning full tenant detail (excluding root-admin password — never retrievable).

24. **MUST** validate `--root-admin-email` against the standard email regex (same as FR-AUTH-002 §1 #2) AND check it's NOT already an existing AUTH subject in any tenant.

25. **MUST** treat slug as **canonical and immutable** post-provisioning. There is no `--rename` flag; slug rename requires manual SQL + ADR + downstream impact analysis (out of scope for slice 1).

26. **MUST** support `--dry-run` flag: validates all inputs + checks slug availability + checks residency policy + simulates each step, but DOES NOT write anything. Returns exit 0 if all checks pass; exit 65/73/77 on issues. Used by FR-TEN-101's signup form preflight (slice 3+).

---

## §2 — Why this design (rationale for humans)

**Why ops-driven CLI at slice 1, not self-serve signup (DEC-320)?** Self-serve signup (FR-TEN-101) needs a polished web form + Stripe payment + fraud detection + email-verification — substantial. Ops-driven CLI is 5h of work and gets the core lifecycle primitive landed. CyberSkill at P2 has ~10 vertical-pack customers; CLI is the right tool for the volume. P3 graduates to self-serve when volume justifies the additional surface.

**Why per-tenant Postgres schema namespace (DEC-321, §1 #6)?** RLS alone is a "shared schema, isolated by predicate" pattern — a single SQL bug (forgetting `WHERE tenant_id = ?`) leaks across tenants. Per-tenant schema namespace adds defence in depth: the `tenant_<slug>` schema becomes the search_path; tables in other tenants' schemas aren't even visible without explicit qualification. Combined with RLS on shared metadata tables (where multi-tenant queries are legitimate), the leak surface narrows significantly.

**Why NATS subject namespace per tenant (DEC-322)?** NATS subscriptions are subject-pattern based. A subscriber listening on `>` (catch-all) would receive every tenant's events — catastrophic data leak. Per-tenant prefix `tenant.<slug>.>` limits what a tenant's subscribers can listen to (enforced by NATS ACLs at the account level). Cross-tenant pub/sub requires explicit gateway routing — auditable + intentional.

**Why S3 per-tenant prefix (DEC-323)?** S3 bucket-level isolation is too coarse (would need a bucket per tenant — operationally painful). Prefix isolation (`<tenant_id>/`) lets all tenants share a bucket while bucket policies + IAM enforce per-prefix access. FR-DOC-001 + FR-EMAIL-001 use this prefix in their S3 keys; the marker object at `<tenant_id>/.cyberos-tenant-marker` is the prefix-creation evidence.

**Why password printed once + zeroised (DEC-331, §1 #7)?** Ops staff need to deliver the root-admin password to the new tenant's admin (out-of-band; e.g. via in-person handoff or encrypted channel). Persisting the password anywhere (file, log, memory audit) creates a leak surface. Printing once + immediate zeroise from CLI memory means the only persisted form is the bcrypt hash (per FR-AUTH-002).

**Why operator role check at root-admin (§1 #14)?** Provisioning a tenant is a cross-tenant operation — the operator must be in tenant 0 (CyberSkill itself) with the `root-admin` role per FR-AUTH-101. Allowing any tenant-admin would let any tenant operator create new tenants — privilege escalation.

**Why exit code 1 for idempotent match (DEC-329, §1 #5, §1 #13)?** Exit 0 = "I did the work"; exit 1 = "I didn't do the work because it was already done correctly". Operators scripting CLI invocations need to distinguish (e.g. provisioning script in CI should treat both as success but log differently). Standard "well-known" idiom matches `git pull` behaviour.

**Why slug regex `^[a-z][a-z0-9-]{2,40}[a-z0-9]$` (§1 #1)?** Slug is the canonical identifier: appears in Postgres schema name (`tenant_<slug>`), NATS namespace (`tenant.<slug>.>`), S3 marker, URLs. All these systems have different identifier rules — the intersection is lowercase alphanumeric + hyphens, starting alphabetic, no trailing hyphen. 4-42 char length covers typical company shortnames.

**Why default residency vn-1 (DEC-324)?** CyberSkill's home market is Vietnam; default to the home jurisdiction so operators don't accidentally provision EU/US tenants in VN-residency. The `--residency` flag is explicit override; FR-TEN-103 will ship per-tenant residency change for migrations.

**Why transactional 5-step orchestration (§1 #6)?** Each step touches a different system (Postgres, NATS, S3, AUTH); partial failures leave orphan state (NATS account without DB row, etc.). Compensating actions (delete NATS account, delete S3 marker, delete AUTH tenant) on any step's failure ensures cleanup. The Postgres parts use a single transaction; NATS + S3 + AUTH have their own commit points but rollback via compensating action.

**Why `cyberos_provisioner` SQL role distinct from `cyberos_app` (§1 #11)?** App code (the runtime) MUST NOT mutate tenants — that's a privileged operation. Splitting roles enforces: the provisioner role is granted only to the CLI's connection; the app role REVOKE's UPDATE/DELETE. A bug in app code can't accidentally suspend a tenant.

**Why memory chain anchor at provisioning (§1 #20, DEC-332)?** Every tenant has its own audit chain (per AGENTS.md §6 — chained rows with prev_chain). The genesis row is the bootstrap anchor that memory_writer initialises from. Without it, the first audit row for the tenant has no `prev_chain` value and chain validation fails.

**Why dry-run mode (§1 #26)?** FR-TEN-101's self-serve signup form needs to validate inputs in real-time (before committing payment). Dry-run lets the form check everything without state mutation — fast feedback, clean separation.

**Why slug immutable post-provisioning (§1 #25)?** Slug is in Postgres schema name + NATS namespace + URL paths + S3 prefixes. Renaming requires coordinated mutation across all systems + downstream consumer awareness. The 1% of cases needing rename are deliberate operator events with ADRs; the API doesn't offer a shortcut.

**Why list + get subcommands (§1 #22, #23)?** Operators need to inspect existing tenants. `list` for inventory; `get` for detail. Both are read-only and require `root-admin` role.

**Why email uniqueness check across all tenants (§1 #24)?** A subject's email is the cross-tenant join key for "I'm the same human in two tenants". Allowing the same email in two new tenants creates ambiguity. Slice 1 enforces single-tenant-email; FR-AUTH-2xx may add multi-tenant binding later (out of scope).

**Why 30s p95 perf budget (§1 #15)?** Provisioning touches 4 external systems + creates ~10 rows. 30s is generous — typical happy path is 5-10s. The p95 budget catches infrastructure outliers; outliers > 30s = sev-3 alarm for ops investigation.

**Why TenantStatus enum has 5 values (DEC-326)?** Mirrors the conventional SaaS lifecycle: provisioning (mid-flight) → active → suspended (non-payment, etc.) → terminating (90-day offboarding contract) → terminated (irreversible wipe attestation). Slice 1 ships all 5 values; transitions implemented in FR-TEN-104.

**Why `tenant_residency_map` separate table (§1 #19)?** Residency may change (rare but possible: tenant graduates from sg-1 to vn-1 for VN expansion). Separate table lets FR-TEN-103 ship the change-residency workflow without touching the `tenants` table. The map is the canonical lookup for FR-DOC-001 + FR-EMAIL-001 + FR-AI-016.

**Why `provisioned_by_subject_id` recorded (§1 #1)?** Accountability — every tenant has a creator operator. PDPL Art. 4 data minimisation is satisfied because the field stores the operator's UUID (already in the AUTH cluster), not their personal data.

**Why no DELETE on tenants (§1 #10)?** Tenants enter `terminated` state via FR-TEN-104's 90-day offboarding contract; the row persists with `status='terminated'` and `terminated_at` set. Hard delete would lose the forensic record + would orphan FK references (audit rows, billing records).

**Why JSON mode omits password (§1 #16)?** JSON mode is intended for automation (scripts piping output to other systems). Including the password in JSON means it could end up in logs, command history, CI artifacts. Stderr block forces operators to handle it deliberately.

---

## §3 — API contract

### 3.1 — Migration 0001 — tenants table

```sql
-- services/ten/migrations/0001_tenants.sql

BEGIN;

CREATE TYPE tenant_status AS ENUM ('provisioning', 'active', 'suspended', 'terminating', 'terminated');
CREATE TYPE residency_code AS ENUM ('vn-1', 'sg-1', 'eu-1', 'us-1');

CREATE TABLE tenants (
    id                          UUID         PRIMARY KEY,
    slug                        TEXT         NOT NULL UNIQUE
                                CHECK (slug ~ '^[a-z][a-z0-9-]{2,40}[a-z0-9]$'),
    display_name                TEXT         NOT NULL CHECK (length(display_name) BETWEEN 1 AND 200),
    status                      tenant_status NOT NULL DEFAULT 'provisioning',
    plan_tier                   TEXT         NOT NULL DEFAULT 'starter',
    residency                   residency_code NOT NULL DEFAULT 'vn-1',
    created_at                  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    provisioned_at              TIMESTAMPTZ,
    terminated_at               TIMESTAMPTZ,
    provisioned_by_subject_id   UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE INDEX tenants_status_idx ON tenants (status);
CREATE INDEX tenants_residency_idx ON tenants (residency);

ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;

-- Only superuser / root-admin can see this table; tenant-admins read their own tenant via FR-TEN-107.
CREATE POLICY tenants_superuser_only ON tenants
    USING (current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (current_setting('auth.is_root_admin', true) = 'true');

-- Provisioner role for the CLI; distinct from cyberos_app.
CREATE ROLE cyberos_provisioner;
GRANT INSERT ON tenants TO cyberos_provisioner;
GRANT UPDATE (status, provisioned_at, terminated_at) ON tenants TO cyberos_provisioner;
GRANT SELECT ON tenants TO cyberos_provisioner;

REVOKE UPDATE, DELETE ON tenants FROM cyberos_app;

COMMIT;
```

### 3.2 — Migration 0002 — tenant_status_history (append-only)

```sql
-- services/ten/migrations/0002_tenant_status_history.sql

BEGIN;

CREATE TABLE tenant_status_history (
    id                     BIGSERIAL    PRIMARY KEY,
    tenant_id              UUID         NOT NULL REFERENCES tenants(id),
    from_status            tenant_status,                              -- NULL on initial create
    to_status              tenant_status NOT NULL,
    changed_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id  UUID         NOT NULL,
    reason                 TEXT
);

CREATE INDEX tenant_status_history_tenant_idx ON tenant_status_history (tenant_id, changed_at DESC);

ALTER TABLE tenant_status_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_status_history_superuser_only ON tenant_status_history
    USING (current_setting('auth.is_root_admin', true) = 'true')
    WITH CHECK (current_setting('auth.is_root_admin', true) = 'true');

GRANT INSERT, SELECT ON tenant_status_history TO cyberos_provisioner;
REVOKE UPDATE, DELETE ON tenant_status_history FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration 0003 — tenant_residency_map

```sql
-- services/ten/migrations/0003_tenant_residency_map.sql

BEGIN;

CREATE TABLE tenant_residency_map (
    tenant_id              UUID         PRIMARY KEY REFERENCES tenants(id),
    residency              residency_code NOT NULL,
    set_at                 TIMESTAMPTZ  NOT NULL DEFAULT now(),
    set_by_subject_id      UUID         NOT NULL
);

-- Allow cyberos_app read access — FR-DOC-001, FR-EMAIL-001, FR-AI-016 all consume.
GRANT SELECT ON tenant_residency_map TO cyberos_app;
GRANT INSERT ON tenant_residency_map TO cyberos_provisioner;
REVOKE UPDATE, DELETE ON tenant_residency_map FROM cyberos_app;
-- UPDATE is granted to cyberos_provisioner for FR-TEN-103's residency-change flow.
GRANT UPDATE (residency, set_at, set_by_subject_id) ON tenant_residency_map TO cyberos_provisioner;

COMMIT;
```

### 3.4 — Rust types

```rust
// services/ten/src/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "tenant_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TenantStatus { Provisioning, Active, Suspended, Terminating, Terminated }

impl TenantStatus {
    pub const ALL: &'static [TenantStatus] = &[
        TenantStatus::Provisioning, TenantStatus::Active, TenantStatus::Suspended,
        TenantStatus::Terminating, TenantStatus::Terminated,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "residency_code", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ResidencyCode { Vn1, Sg1, Eu1, Us1 }

impl ResidencyCode {
    pub const ALL: &'static [ResidencyCode] = &[
        ResidencyCode::Vn1, ResidencyCode::Sg1, ResidencyCode::Eu1, ResidencyCode::Us1,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            ResidencyCode::Vn1 => "vn-1",
            ResidencyCode::Sg1 => "sg-1",
            ResidencyCode::Eu1 => "eu-1",
            ResidencyCode::Us1 => "us-1",
        }
    }
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Tenant {
    pub id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub status: TenantStatus,
    pub plan_tier: String,
    pub residency: ResidencyCode,
    pub created_at: DateTime<Utc>,
    pub provisioned_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub provisioned_by_subject_id: Uuid,
}
```

### 3.5 — Provisioning orchestrator

```rust
// services/ten/src/provisioning/orchestrator.rs
use crate::types::*;
use uuid::Uuid;
use zeroize::Zeroizing;

pub struct ProvisionRequest {
    pub slug: String,
    pub display_name: String,
    pub root_admin_email: String,
    pub root_admin_display_name: String,
    pub residency: ResidencyCode,
    pub plan_tier: String,
    pub operator_subject_id: Uuid,
    pub dry_run: bool,
}

pub struct ProvisionResult {
    pub tenant_id: Uuid,
    pub slug: String,
    pub display_name: String,
    pub residency: ResidencyCode,
    pub plan_tier: String,
    pub postgres_schema: String,
    pub nats_namespace: String,
    pub s3_prefix: String,
    pub root_admin_subject_id: Uuid,
    pub root_admin_email: String,
    pub root_admin_password: Zeroizing<String>,   // printed once + zeroised
    pub idempotent_match: bool,
}

pub async fn provision(req: ProvisionRequest, ctx: &Ctx) -> Result<ProvisionResult, ProvisionError> {
    // Step 1: validate (fail-fast)
    crate::validation::slug(&req.slug)?;
    crate::validation::email(&req.root_admin_email)?;

    if req.dry_run {
        return crate::dry_run::simulate(req, ctx).await;
    }

    // Step 2: idempotency check
    if let Some(existing) = ctx.repo.find_by_slug(&req.slug).await? {
        if existing.residency != req.residency || existing.plan_tier != req.plan_tier {
            return Err(ProvisionError::SlugCollisionDifferentAttrs);
        }
        return Ok(ProvisionResult::from_existing(existing));
    }

    let mut tx = ctx.db.begin().await?;
    let tenant_id = Uuid::new_v4();

    // Step 3: INSERT tenant (status='provisioning')
    sqlx::query("INSERT INTO tenants (id, slug, display_name, status, plan_tier, residency, provisioned_by_subject_id) VALUES ($1, $2, $3, 'provisioning'::tenant_status, $4, $5::residency_code, $6)")
        .bind(tenant_id).bind(&req.slug).bind(&req.display_name)
        .bind(&req.plan_tier).bind(req.residency.as_str()).bind(req.operator_subject_id)
        .execute(&mut *tx).await?;
    sqlx::query("INSERT INTO tenant_status_history (tenant_id, from_status, to_status, changed_by_subject_id, reason) VALUES ($1, NULL, 'provisioning'::tenant_status, $2, 'initial provisioning')")
        .bind(tenant_id).bind(req.operator_subject_id).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO tenant_residency_map (tenant_id, residency, set_by_subject_id) VALUES ($1, $2::residency_code, $3)")
        .bind(tenant_id).bind(req.residency.as_str()).bind(req.operator_subject_id)
        .execute(&mut *tx).await?;

    // Step 4: create Postgres schema namespace
    crate::provisioning::schema_namespace::create(&req.slug, &mut tx).await?;

    tx.commit().await?;

    // Step 5: create NATS namespace (compensating action on failure)
    let nats_namespace = format!("tenant.{}.>", req.slug);
    crate::provisioning::nats_namespace::create(&req.slug, &ctx.nats).await
        .map_err(|e| {
            // Compensate: delete Postgres schema + tenant rows.
            tokio::spawn(crate::provisioning::compensate::rollback(tenant_id, req.slug.clone(), ctx.clone()));
            e
        })?;

    // Step 6: create S3 prefix markers (compensating action on failure)
    let s3_prefix = format!("{tenant_id}/");
    crate::provisioning::s3_prefix::initialise(&tenant_id, req.residency, &ctx.s3).await
        .map_err(|e| {
            tokio::spawn(crate::provisioning::compensate::rollback_with_nats(tenant_id, req.slug.clone(), ctx.clone()));
            e
        })?;

    // Step 7: AUTH bootstrap — create AUTH tenant + initial root-admin subject
    let password = generate_password();
    let bootstrap = crate::provisioning::auth_bootstrap::call(
        tenant_id, &req.slug, &req.display_name,
        &req.root_admin_email, &req.root_admin_display_name,
        &password, req.operator_subject_id, &ctx.auth_client,
    ).await.map_err(|e| {
        tokio::spawn(crate::provisioning::compensate::rollback_with_nats_and_s3(tenant_id, req.slug.clone(), ctx.clone()));
        e
    })?;

    // Step 8: UPDATE tenant status to 'active'
    let mut tx2 = ctx.db.begin().await?;
    sqlx::query("UPDATE tenants SET status = 'active'::tenant_status, provisioned_at = now() WHERE id = $1")
        .bind(tenant_id).execute(&mut *tx2).await?;
    sqlx::query("INSERT INTO tenant_status_history (tenant_id, from_status, to_status, changed_by_subject_id, reason) VALUES ($1, 'provisioning'::tenant_status, 'active'::tenant_status, $2, 'provisioning complete')")
        .bind(tenant_id).bind(req.operator_subject_id).execute(&mut *tx2).await?;

    // Step 9: emit memory audit row
    crate::audit::tenant_events::emit_tenant_provisioned(
        &mut tx2, tenant_id, &req.slug, &req.display_name,
        req.plan_tier.clone(), req.residency, req.operator_subject_id, bootstrap.root_admin_subject_id,
    ).await?;

    tx2.commit().await?;

    Ok(ProvisionResult {
        tenant_id,
        slug: req.slug,
        display_name: req.display_name,
        residency: req.residency,
        plan_tier: req.plan_tier,
        postgres_schema: format!("tenant_{}", &req.slug),
        nats_namespace,
        s3_prefix,
        root_admin_subject_id: bootstrap.root_admin_subject_id,
        root_admin_email: req.root_admin_email,
        root_admin_password: password,
        idempotent_match: false,
    })
}

fn generate_password() -> Zeroizing<String> {
    use rand::distributions::{Alphanumeric, DistString};
    Zeroizing::new(Alphanumeric.sample_string(&mut rand::thread_rng(), 32))
}
```

### 3.6 — CLI command

```rust
// services/ten/src/cli/provision.rs
use clap::Parser;
use cyberos_cli_exit::ExitCode;
use crate::provisioning::orchestrator::{provision, ProvisionRequest, ProvisionError};
use crate::types::ResidencyCode;
use std::str::FromStr;

#[derive(Parser)]
pub struct ProvisionCmd {
    #[arg(long)] pub slug: String,
    #[arg(long)] pub display_name: String,
    #[arg(long)] pub root_admin_email: String,
    #[arg(long)] pub root_admin_display_name: String,
    #[arg(long, default_value = "vn-1")] pub residency: String,
    #[arg(long, default_value = "starter")] pub plan_tier: String,
    #[arg(long, default_value_t = false)] pub json: bool,
    #[arg(long, default_value_t = false)] pub dry_run: bool,
}

pub async fn run(cmd: ProvisionCmd, ctx: AppCtx) -> ExitCode {
    // Operator role check
    let operator = match ctx.current_operator().await {
        Ok(op) if op.has_role(Role::RootAdmin) => op,
        Ok(_) => { eprintln!("ERROR: caller must be root-admin"); return ExitCode::PermissionDenied; }
        Err(_) => return ExitCode::PermissionDenied,
    };

    let residency = match ResidencyCode::from_str(&cmd.residency) {
        Ok(r) => r,
        Err(_) => { eprintln!("ERROR: unknown residency: {}", cmd.residency); return ExitCode::InvalidData; }
    };

    let req = ProvisionRequest {
        slug: cmd.slug,
        display_name: cmd.display_name,
        root_admin_email: cmd.root_admin_email,
        root_admin_display_name: cmd.root_admin_display_name,
        residency,
        plan_tier: cmd.plan_tier,
        operator_subject_id: operator.subject_id,
        dry_run: cmd.dry_run,
    };

    match provision(req, &ctx).await {
        Ok(result) => {
            if cmd.json {
                let json = serde_json::to_string_pretty(&result.public_view()).unwrap();
                println!("{json}");
                eprintln!("\n=== ROOT ADMIN PASSWORD (RECORD IMMEDIATELY) ===");
                eprintln!("{}", *result.root_admin_password);
                eprintln!("=== END ===");
            } else {
                println!("✓ Tenant provisioned");
                println!("  tenant_id:        {}", result.tenant_id);
                println!("  slug:             {}", result.slug);
                println!("  display_name:     {}", result.display_name);
                println!("  residency:        {}", result.residency.as_str());
                println!("  plan_tier:        {}", result.plan_tier);
                println!("  postgres_schema:  {}", result.postgres_schema);
                println!("  nats_namespace:   {}", result.nats_namespace);
                println!("  s3_prefix:        {}", result.s3_prefix);
                println!("  root_admin_email: {}", result.root_admin_email);
                println!("  === ROOT ADMIN PASSWORD (RECORD IMMEDIATELY) ===");
                println!("  {}", *result.root_admin_password);
                println!("  === END ===");
            }
            if result.idempotent_match { ExitCode::IdempotentMatch } else { ExitCode::Success }
        }
        Err(ProvisionError::SlugCollisionDifferentAttrs) => {
            eprintln!("ERROR: slug exists with different residency or plan_tier");
            ExitCode::InvalidData
        }
        Err(ProvisionError::InvalidSlug) => { eprintln!("ERROR: invalid slug (must match ^[a-z][a-z0-9-]{{2,40}}[a-z0-9]$)"); ExitCode::InvalidData }
        Err(ProvisionError::InvalidEmail) => { eprintln!("ERROR: invalid email format"); ExitCode::InvalidData }
        Err(ProvisionError::StepFailed(step)) => { eprintln!("ERROR: step {step} failed; rollback in progress"); ExitCode::CantCreate }
        Err(ProvisionError::Transient) => { eprintln!("ERROR: transient infrastructure failure; retry advised"); ExitCode::TempFail }
        Err(e) => { eprintln!("ERROR: {e:?}"); ExitCode::CantCreate }
    }
}
```

---

## §4 — Acceptance criteria

1. **Tenant status enum closed at 5** — `TenantStatus::ALL.len() == 5`; Postgres enum has exactly 5 labels.
2. **Residency code enum closed at 4** — same shape.
3. **POST provision happy path** — valid input → exit 0; tenant row created with `status=active`; `tenant_<slug>` Postgres schema exists; NATS namespace registered; S3 marker objects written; root-admin subject created; `ten.tenant_provisioned` memory row emitted; password printed once.
4. **Idempotent on slug** — re-run with same slug + same residency + same plan_tier → exit 1; same tenant row returned; NO duplicate memory row.
5. **Slug collision different residency** → exit 65 with `slug_collision_different_attrs`.
6. **Invalid slug regex** — slug with uppercase or trailing hyphen → exit 65.
7. **Invalid email format** → exit 65.
8. **Unknown residency** → exit 65.
9. **Operator without root-admin role** → exit 77 immediately (no state mutation).
10. **Missing required flag** → exit 64.
11. **UPDATE on tenants blocked from cyberos_app** — `UPDATE tenants SET status='terminated'` as cyberos_app → permission denied.
12. **DELETE on tenants blocked from cyberos_app** — same.
13. **tenant_status_history append-only** — UPDATE/DELETE blocked.
14. **`cyberos-ten get` returns tenant detail** — exit 0; root_admin password NOT in output.
15. **`cyberos-ten list` returns all tenants** — root-admin only.
16. **Root admin password printed once + zeroised** — memory inspection after CLI exit (test harness) shows the password bytes overwritten.
17. **Root admin password NOT in memory row** — `ten.tenant_provisioned` row JSON contains no password-shaped field.
18. **Default residency vn-1** — `--residency` omitted → tenant created with residency='vn-1'.
19. **--residency flag overrides** — `--residency eu-1` → tenant created with residency='eu-1'.
20. **--dry-run validates without writing** — exit 0; no tenant row created; no schema/NATS/S3 mutation.
21. **--dry-run reports validation failure** — bad slug → exit 65; no state.
22. **tenant_residency_map row written** — readable by cyberos_app.
23. **NATS namespace ACL applied** — subscriber on `tenant.<other-slug>.>` cannot receive messages on `tenant.<slug>.>`.
24. **S3 marker object exists** — `s3://cyberos-doc-vn-1-generic/<tenant_id>/.cyberos-tenant-marker` returns 200 on HEAD.
25. **AUTH side created** — POST /v1/admin/tenants on AUTH returns same tenant_id + root_admin_subject_id; root-admin subject has `tenant-admin` role per FR-AUTH-101.
26. **Provision rolls back on AUTH failure** — mock AUTH to return 500 → tenant row + NATS + S3 marker cleaned up via compensating actions; exit 73.
27. **OTel span `ten.provision` emitted** — `outcome=success`.
28. **Counter `ten_provision_total{outcome=success, residency=vn-1}` increments** — per provisioning.
29. **Perf budget < 30s p95** — `provision_perf_test` 50 iterations.
30. **`--json` mode omits password from stdout** — password in stderr block only.

---

## §5 — Verification

```rust
// services/ten/tests/provision_happy_test.rs
#[tokio::test]
async fn happy_path_creates_everything() {
    let ctx = TestStack::up().await;
    let result = ctx.run_cli(&[
        "provision",
        "--slug", "acme-corp",
        "--display-name", "ACME Corporation",
        "--root-admin-email", "root@acme.example",
        "--root-admin-display-name", "ACME Root Admin",
        "--residency", "vn-1",
    ]).await;
    assert_eq!(result.exit_code, 0);

    // Check tenant row
    let tenant = ctx.db.fetch_tenant_by_slug("acme-corp").await.unwrap();
    assert_eq!(tenant.status, TenantStatus::Active);
    assert!(tenant.provisioned_at.is_some());

    // Check Postgres schema
    let schemas = ctx.db.list_schemas().await;
    assert!(schemas.contains(&"tenant_acme-corp".to_string()));

    // Check NATS namespace
    let nats_accounts = ctx.nats.list_accounts().await;
    assert!(nats_accounts.iter().any(|a| a.subject_namespace == "tenant.acme-corp.>"));

    // Check S3 markers
    let marker = ctx.s3.head_object("cyberos-doc-vn-1-generic", &format!("{}/.cyberos-tenant-marker", tenant.id)).await;
    assert!(marker.is_ok());

    // Check AUTH-side root admin
    let auth_subject = ctx.auth.get_subject_by_email("root@acme.example").await.unwrap();
    assert!(auth_subject.roles.contains(&"tenant-admin".to_string()));

    // Check memory audit row
    let rows = ctx.memory_audit_rows("ten.tenant_provisioned").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["slug"], "acme-corp");
}

#[tokio::test]
async fn idempotent_match_returns_existing() {
    let ctx = TestStack::up().await;
    let _first = ctx.run_cli(&["provision", "--slug", "acme", "--display-name", "ACME", "--root-admin-email", "r@acme", "--root-admin-display-name", "R", "--residency", "vn-1"]).await;
    let second = ctx.run_cli(&["provision", "--slug", "acme", "--display-name", "ACME", "--root-admin-email", "r@acme", "--root-admin-display-name", "R", "--residency", "vn-1"]).await;
    assert_eq!(second.exit_code, 1);
    let rows = ctx.memory_audit_rows("ten.tenant_provisioned").await;
    assert_eq!(rows.len(), 1, "no duplicate memory row on idempotent match");
}
```

```rust
// services/ten/tests/provision_root_admin_test.rs
#[tokio::test]
async fn password_printed_once_and_zeroised() {
    let ctx = TestStack::up().await;
    let result = ctx.run_cli_capture(&["provision", "--slug", "acme", /* ... */]).await;
    assert_eq!(result.exit_code, 0);

    // Password is in stdout (with marker block)
    assert!(result.stdout.contains("=== ROOT ADMIN PASSWORD"));
    let password_line = result.stdout.lines().find(|l| l.trim().len() == 32 && l.trim().chars().all(|c| c.is_alphanumeric())).unwrap();
    let password = password_line.trim().to_string();

    // Password is NOT in memory row
    let rows = ctx.memory_audit_rows("ten.tenant_provisioned").await;
    let row_json = serde_json::to_string(&rows[0]).unwrap();
    assert!(!row_json.contains(&password), "password leaked into memory row");

    // Password is NOT in log files
    let logs = ctx.read_log_files().await;
    assert!(!logs.contains(&password), "password leaked into logs");
}
```

```rust
// services/ten/tests/append_only_test.rs
#[sqlx::test]
async fn tenants_update_blocked_from_app(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_tenant_as_provisioner(&pool).await;
    let err = sqlx::query("UPDATE tenants SET status = 'terminated'::tenant_status WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}

#[sqlx::test]
async fn tenants_update_allowed_from_provisioner(pool: sqlx::PgPool) {
    set_role_provisioner(&pool).await;
    let id = seed_tenant_as_provisioner(&pool).await;
    sqlx::query("UPDATE tenants SET status = 'terminated'::tenant_status, terminated_at = now() WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap();
}
```

```rust
// services/ten/tests/provision_namespace_isolation_test.rs
#[tokio::test]
async fn two_tenants_get_distinct_namespaces() {
    let ctx = TestStack::up().await;
    let a = ctx.run_cli(&["provision", "--slug", "acme", /* ... */]).await;
    let b = ctx.run_cli(&["provision", "--slug", "biko", /* ... */]).await;
    assert_eq!(a.exit_code, 0);
    assert_eq!(b.exit_code, 0);

    let schemas = ctx.db.list_schemas().await;
    assert!(schemas.contains(&"tenant_acme".to_string()));
    assert!(schemas.contains(&"tenant_biko".to_string()));

    let nats = ctx.nats.list_accounts().await;
    assert!(nats.iter().any(|a| a.subject_namespace == "tenant.acme.>"));
    assert!(nats.iter().any(|a| a.subject_namespace == "tenant.biko.>"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; compensating-action functions follow the standard rollback pattern: delete the Postgres rows + NATS account + S3 marker if a later step fails.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-001** — tenant + subject create (this FR delegates via internal helper).

**Downstream (2 placeholders):**
- **FR-TEN-002** — 3 plan tiers (consumes `plan_tier` column).
- **FR-TEN-104** — 90-day offboarding contract (uses the closed TenantStatus enum).

**Cross-module:**
- **FR-AUTH-002** — subject create (consumed via the AUTH bootstrap helper for root-admin creation).
- **FR-AUTH-101** — RBAC catalogue (root-admin role + tenant-admin role assignment).
- **FR-AI-016** — residency policy (consumes `tenant_residency_map` table).
- **FR-DOC-001** — document repository (consumes `tenant_residency_map` for body bucket lookup).
- **FR-EMAIL-001** — Stalwart mail server (consumes `tenant_residency_map` for body bucket lookup).
- **FR-AI-003** — memory audit bridge (receives `ten.tenant_provisioned` row + chain anchor).

---

## §8 — Example payloads

### 8.1 — `cyberos-ten provision` happy invocation

```bash
$ cyberos-ten provision \
    --slug acme-corp \
    --display-name "ACME Corporation" \
    --root-admin-email root@acme.example \
    --root-admin-display-name "ACME Root Admin" \
    --residency vn-1
✓ Tenant provisioned
  tenant_id:        01HG7V8B0K8M4Z8Z8M8M8M8M8M
  slug:             acme-corp
  display_name:     ACME Corporation
  residency:        vn-1
  plan_tier:        starter
  postgres_schema:  tenant_acme-corp
  nats_namespace:   tenant.acme-corp.>
  s3_prefix:        01HG7V8B0K8M4Z8Z8M8M8M8M8M/
  root_admin_email: root@acme.example
  === ROOT ADMIN PASSWORD (RECORD IMMEDIATELY) ===
  Kj7Lp9MzQrTyVwBxCdEfGhJ3Kl5Nm6Op
  === END ===
```

### 8.2 — ten.tenant_provisioned memory row

```json
{
  "kind": "ten.tenant_provisioned",
  "tenant_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "slug": "acme-corp",
  "display_name": "ACME Corporation",
  "status": "active",
  "plan_tier": "starter",
  "residency": "vn-1",
  "provisioned_by_subject_id_hash16": "8a7c8c8012344567",
  "root_admin_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

### 8.3 — JSON-mode output

```bash
$ cyberos-ten provision --slug acme-corp --display-name "ACME" --root-admin-email r@a --root-admin-display-name R --residency vn-1 --json
{
  "tenant_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "slug": "acme-corp",
  "display_name": "ACME",
  "residency": "vn-1",
  "plan_tier": "starter",
  "postgres_schema": "tenant_acme-corp",
  "nats_namespace": "tenant.acme-corp.>",
  "s3_prefix": "01HG7V8B0K8M4Z8Z8M8M8M8M8M/",
  "root_admin_email": "r@a",
  "root_admin_subject_id": "9b1deb4d-..."
}
=== ROOT ADMIN PASSWORD (RECORD IMMEDIATELY) ===
Kj7Lp9MzQrTyVwBxCdEfGhJ3Kl5Nm6Op
=== END ===
```

### 8.4 — Idempotent-match exit

```bash
$ cyberos-ten provision --slug acme-corp --display-name "ACME" --root-admin-email r@a --root-admin-display-name R --residency vn-1
✓ Tenant already exists (idempotent match)
  tenant_id:        01HG7V8B0K8M4Z8Z8M8M8M8M8M
  slug:             acme-corp
  status:           active
$ echo $?
1
```

---

## §9 — Open questions

Deferred:
- **Plan tier closed enum** — FR-TEN-002 ships full 3-tier validation.
- **Self-serve signup form** — FR-TEN-101 (P3).
- **Per-tenant residency change workflow** — FR-TEN-103.
- **90-day offboarding** — FR-TEN-104.
- **Plan downgrade with quota violation** — FR-TEN-2xx.
- **Branding config** — FR-TEN-2xx.
- **Slug rename** — out of scope; manual SQL + ADR.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Slug regex fail | DB CHECK + handler | exit 65 invalid_data | Use valid slug |
| Slug collision same attrs | UNIQUE + handler | exit 1 idempotent_match | None — designed |
| Slug collision different attrs | UNIQUE + handler diff check | exit 65 collision_different_attrs | Use different slug or align attrs |
| Invalid email | handler | exit 65 | Use valid email |
| Unknown residency | enum parse | exit 65 | Use vn-1 / sg-1 / eu-1 / us-1 |
| Operator not root-admin | JWT role check | exit 77 | Re-auth with root-admin role |
| Missing required flag | clap | exit 64 | Provide flag |
| Postgres schema create fails (permission) | step 3 error | exit 73 + compensating rollback | Check cyberos_provisioner role perms |
| NATS account create fails | step 4 error | exit 73 + rollback Postgres | Check NATS API health |
| S3 marker write fails | step 5 error | exit 73 + rollback Postgres + NATS | Check S3 perms + KMS |
| AUTH bootstrap fails | step 6 error | exit 73 + full rollback | Investigate AUTH service health |
| Status transition `provisioning → active` fails | step 7 tx | exit 73; tenant left in `provisioning` state | Operator runs cleanup script |
| memory audit emit fails | step 8 tx rollback | exit 73; full rollback | memory_writer diagnosis |
| Password generation fails | step 7 internal | exit 75 transient | Retry |
| RNG entropy starvation | step 7 | exit 75 | Restart process |
| Operator deletes the printed password before saving | None — operator responsibility | Tenant unusable; force password-reset via AUTH | FR-AUTH-2xx password reset flow |
| Postgres connection pool exhausted | sqlx error | exit 75 | Wait + retry |
| Cross-tenant slug-derived schema collision (e.g. SQL injection in slug) | regex check + parameterised query | None | Regex prevents |
| `cyberos_provisioner` role not granted to CLI's connection | startup check fails | Service refuses to start | Grant role |
| `tenant_residency_map` INSERT fails | step 3 tx rollback | exit 73 | Investigate cluster health |
| Idempotent re-run with different display_name | OK (idempotency only checks slug+residency+plan_tier) | exit 1; display_name from existing row | Use `cyberos-ten update display-name` (FR-TEN-2xx) |
| NATS compensating action fails | logged; sev-1 alarm | Orphan NATS account | Operator manual cleanup |
| S3 compensating action fails | logged; sev-1 alarm | Orphan marker object (low cost) | S3 lifecycle cleanup |
| AUTH compensating action fails | logged; sev-1 alarm | Orphan AUTH tenant + subject | Operator manual cleanup via AUTH CLI |
| Dry-run side-effect leak | test asserts | CI fails | Fix orchestrator branching |
| OTel span attribute missing | otel_attrs_test | CI fails | Fix span builder |
| password length not 32 chars | unit test | CI fails | Fix RNG |
| password contains spaces or non-alphanumerics | unit test | CI fails | Fix RNG generator |
| concurrent provision same slug | first wins; second hits UNIQUE | exit 1 (if attrs match) or 65 | Designed |
| schema_namespace creator forgets to use IF NOT EXISTS | rerun fails | CI test catches | Fix |
| AUTH internal helper signature drift | type error at compile | Build fails | Coordinate FR-AUTH-001 + FR-TEN-001 changes |
| Stale `tenant_residency_map` row after rollback | compensating action covers | Designed | None |

---

## §11 — Implementation notes

- **Ops-driven at slice 1** — self-serve signup (FR-TEN-101) lands at P3. CLI covers our P2 vertical-pack volume.
- **Per-tenant schema namespace + NATS namespace + S3 prefix** combine for defense-in-depth isolation; RLS alone is insufficient.
- **`cyberos_provisioner` SQL role distinct from `cyberos_app`** — privileged operations route through provisioner; app code can't touch tenants table.
- **Password printed once + zeroise via `zeroize` crate** — ops must save the password manually; subsequent retrieval requires password reset.
- **Compensating actions on each step's failure** — Postgres uses transactions; NATS + S3 + AUTH have their own commit points + manual rollback.
- **Exit code 1 for idempotent match** — distinguishes from exit 0 (did the work) for scripting; matches `git pull` idiom.
- **Slug regex** — kebab-case, 4-42 chars, no trailing hyphen; safe for Postgres schema name + NATS subject + S3 prefix + URL.
- **`auth.is_root_admin` GUC** — set by the AUTH JWT middleware when operator's JWT carries the root-admin role; RLS policy on `tenants` consults this.
- **`tenant_residency_map` is the lookup contract** — FR-DOC-001 + FR-EMAIL-001 + FR-AI-016 all read from this table. Centralising prevents drift.
- **memory chain anchor at provisioning** — first audit row's `prev_chain` is the genesis hash; memory_writer initialises chain head from this.
- **`--dry-run` mode** — validates everything, mutates nothing. Powers FR-TEN-101's signup-form preflight.
- **Slug immutable post-provisioning** — schema name + NATS subject + S3 prefix all derive from slug; renaming requires coordinated multi-system mutation.
- **`provisioned_by_subject_id` is the operator's UUID** — accountability without storing operator PII (PDPL Art. 4 satisfied).
- **`tenant_status_history` append-only** — full lifecycle trace for forensics + compliance reviews.
- **TenantStatus 5 values** — ships all at slice 1; transitions to suspended/terminating/terminated land in FR-TEN-104.
- **`cyberos-ten get` excludes password** — there is no path to retrieve the original password; password reset is via AUTH (FR-AUTH-2xx).
- **`--json` mode separates password to stderr** — automation-friendly stdout JSON + force-attention stderr password block.
- **AUTH internal helper exposed only for FR-TEN-001** — not a public REST endpoint; operator-privilege only.
- **`tenant.slug.>` NATS subject pattern** — Multi-level wildcard captures all events for the tenant; ACLs limit pub/sub to this namespace.
- **S3 marker object pattern** — `<tenant_id>/.cyberos-tenant-marker` is a small ~100-byte JSON; existence is the prefix-creation evidence.
- **Operator JWT verification** — slice 1 reads `auth.is_root_admin` GUC; slice 2 (FR-TEN-101) adds full OAuth flow.
- **30s p95 budget** — generous; happy path is 5-10s.
- **Closed enum cardinality tests** at CI prevent drift between Rust + SQL enum sizes.
- **`cyberos-ten list` + `get`** — read-only inventory; useful for operators auditing the fleet.

---

*End of FR-TEN-001.*
