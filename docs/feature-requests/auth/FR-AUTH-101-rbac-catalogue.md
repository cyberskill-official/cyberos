---
id: FR-AUTH-101
title: "AUTH 22-role RBAC catalogue — closed enum + permission matrix + role-assignment REST + JWT claims + ADR gate + stub→full migration"
module: AUTH
priority: MUST
status: building
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-16
shipped: null
brain_chain_hash: null
related_frs: [FR-AUTH-002, FR-AUTH-003, FR-AUTH-004, FR-AUTH-005, FR-AUTH-006, FR-AUTH-108, FR-AUTH-109, FR-CRM-001, FR-HR-001, FR-KB-001, FR-REW-001, FR-DOC-001, FR-OKR-001]
depends_on: [FR-AUTH-005]
blocks: [FR-AUTH-108, FR-AUTH-109, FR-CRM-001, FR-DOC-001, FR-HR-001, FR-KB-001, FR-OKR-001, FR-REW-001, FR-TIME-001, FR-INV-005, FR-AUTH-104, FR-INV-003, FR-INV-004]

source_pages:
  - website/docs/modules/auth.html#rbac-catalogue
  - website/docs/modules/auth.html#stub-stack
source_decisions:
  - DEC-121 (closed 22-role catalogue — ADR-gated, no code-only role additions)
  - DEC-122 (permission matrix is roles × resources × actions; never ABAC)
  - DEC-123 (5-role stub from FR-AUTH-002 §1 #5 is strict prefix of the 22-role full catalogue — same role names, additive only)
  - DEC-124 (role + scope-grant layered: role gives base privilege; scope-grant narrows to specific resources)
  - DEC-125 (existing FR-AUTH-004 access tokens remain valid for a 30-day grace window after this FR ships — see FR-AUTH-109 for the migration enforcer)
  - DEC-126 (role checks via in-memory `RoleMatrix` snapshot, refreshed every 60s; never per-request DB lookup)
  - DEC-127 (root-admin and external roles `client-portal-user / auditor / regulator / billing-system` MUST NOT be self-assignable — only granted via dedicated REST endpoints with elevated approval)
  - DEC-128 (founder role REQUIRES WebAuthn enrolment per FR-AUTH-105; assignment endpoint refuses if MFA factor missing)
  - DEC-129 (role catalogue version is monotonic integer; bumped on every ADR-approved schema change; embedded in JWT as `rbac_v` claim for replay-resistance)
  - PDPL Art. 6 (data minimisation: role claim in JWT carries names only, no permission matrix copy)
  - NIST SP 800-162 (RBAC reference architecture; role = job function, permission = action on resource)
  - ISO/IEC 27001:2022 A.5.18 (access rights provisioned via documented roles)

language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/src/rbac/catalogue.rs                                 # closed Role enum + 22 variants + name parse + Display
  - services/auth/src/rbac/permissions.rs                               # closed Resource × Action enums + permission-matrix loader
  - services/auth/src/rbac/matrix.rs                                    # in-memory RoleMatrix snapshot + 60s refresher
  - services/auth/src/rbac/check.rs                                     # HasRole / HasAnyRole / HasPermission middleware extractors
  - services/auth/src/rbac/scope_grant.rs                               # narrowing layer: role gives base; grant restricts to {resource_id}
  - services/auth/src/rbac/migrate.rs                                   # stub→full migration helper (used by FR-AUTH-109)
  - services/auth/src/rbac/adr.rs                                       # ADR-file validator — refuses catalogue change without matching ADR-NNN.md
  - services/auth/src/admin/roles_rest.rs                               # POST/DELETE /v1/admin/subjects/{id}/roles + GET /v1/admin/roles
  - services/auth/src/audit/role_events.rs                              # canonical auth.role_assigned / auth.role_revoked / auth.role_catalogue_changed builders
  - services/auth/migrations/0005_roles_permissions.sql                 # roles + permissions + role_permissions + subject_roles tables; seeded with 22 roles + matrix
  - services/auth/migrations/0006_role_catalogue_version.sql            # role_catalogue_version singleton table + bump trigger
  - services/auth/tests/rbac_catalogue_test.rs                          # closed-enum invariants (no extra strings parseable, no missing roles)
  - services/auth/tests/rbac_permission_matrix_test.rs                  # matrix lookups, in-memory cache, refresh semantics
  - services/auth/tests/rbac_check_test.rs                              # HasRole / HasAnyRole / HasPermission middleware tests
  - services/auth/tests/rbac_assignment_test.rs                         # POST roles handler — happy + 401 + 403 + 409 + invalid-role + reserved-role + idempotent
  - services/auth/tests/rbac_jwt_claim_test.rs                          # roles + rbac_v claims present, parseable, downstream-checkable
  - services/auth/tests/rbac_stub_migration_test.rs                     # FR-AUTH-002 5-role stub tokens still valid; new tokens carry rbac_v=2+
  - services/auth/tests/rbac_adr_gate_test.rs                           # CI gate: migration touches roles table without matching ADR → fail
  - services/auth/tests/rbac_scope_grant_test.rs                        # role + scope-grant intersection; revoked grant blocks access
  - services/auth/tests/rbac_founder_webauthn_gate_test.rs              # founder assignment fails without WebAuthn factor
  - services/auth/tests/rbac_reserved_role_self_assign_test.rs          # root-admin / auditor / regulator / billing-system / client-portal-user — refuse self-assign
  - services/auth/adr/ADR-101-rbac-22-role-catalogue.md                 # the catalogue's own ADR (this FR ships the ADR)
modified_files:
  - services/auth/src/admin/subjects.rs                                 # role allow-list switches from {tenant-admin, tenant-member} to full RoleMatrix
  - services/auth/src/admin/password.rs                                 # founder role triggers MFA-required side-condition
  - services/auth/src/jwt.rs                                            # add `roles` (array) + `rbac_v` (integer) claims; preserve `tenant_id` shape
  - services/auth/src/rls/templates.rs                                  # RLS now consults role membership for sensitive tables (subjects, audit, billing)
  - services/auth/src/lib.rs                                            # pub mod rbac
  - services/auth/Cargo.toml                                            # +tokio-stream (for 60s refresher), +sha2 (matrix hash)
  - services/auth/RFC.md                                                # update §6 (role catalogue source-of-truth) — point at FR-AUTH-101

allowed_tools:
  - file_read: services/auth/**
  - file_read: docs/feature-requests/auth/**
  - file_write: services/auth/{src,tests,migrations,adr}/**
  - bash: cd services/auth && cargo test rbac_
  - bash: cd services/auth && cargo test rbac_adr_gate_test
  - bash: psql -f services/auth/migrations/0005_roles_permissions.sql (local Postgres only)

disallowed_tools:
  - introduce a 23rd role via a code change without a matching ADR-NNN.md file (per DEC-121; the ADR-gate test enforces this)
  - allow ABAC (attribute-based) checks anywhere in the rbac module (per DEC-122 — closed catalogue is the design assertion)
  - allow self-assignment of root-admin / auditor / regulator / billing-system / client-portal-user (per DEC-127)
  - assign founder role to a subject without a registered WebAuthn factor (per DEC-128 + FR-AUTH-105)
  - issue a JWT with a `roles` claim referencing a role not in the closed catalogue (per §1 #13)
  - perform per-request permission lookup against the database (per DEC-126 — must use the in-memory RoleMatrix)
  - mutate the role catalogue at runtime via REST (the catalogue is migration-defined; runtime mutation is forbidden)

effort_hours: 12
sub_tasks:
  - "0.5h: ADR-101 — document the closed catalogue rationale, scope-creep assessment, deprecation policy"
  - "1.0h: 0005_roles_permissions.sql — roles (22 seeded) + resources (≈40 seeded) + permissions matrix (~280 rows) + subject_roles table"
  - "0.5h: 0006_role_catalogue_version.sql — singleton row + AFTER INSERT/UPDATE/DELETE trigger on roles/permissions"
  - "1.0h: catalogue.rs — Role enum (22 variants) + From<&str> + Display + Iter + reserved-role classifier"
  - "1.0h: permissions.rs — Resource (40) × Action (read/write/admin/approve/sign) enums + matrix loader"
  - "1.0h: matrix.rs — RoleMatrix snapshot struct + 60s tokio refresher + hash-stamp for change detection"
  - "0.5h: check.rs — HasRole / HasAnyRole / HasAllRoles / HasPermission axum extractors with 401/403 semantics"
  - "1.0h: scope_grant.rs — narrowing layer (subject_id + resource_id + grant) + RLS-style intersection"
  - "0.5h: migrate.rs — stub→full helper used by FR-AUTH-109 + grace-window validator"
  - "1.0h: adr.rs — CI-callable validator: any migration touching roles/permissions tables requires a matching ADR-NNN.md"
  - "1.0h: roles_rest.rs — POST /v1/admin/subjects/{id}/roles + DELETE /v1/admin/subjects/{id}/roles/{role} + GET /v1/admin/roles"
  - "0.5h: role_events.rs — canonical auth.role_assigned / auth.role_revoked / auth.role_catalogue_changed BRAIN builders"
  - "0.5h: jwt.rs update — emit roles array + rbac_v integer claim"
  - "0.5h: subjects.rs + rls/templates.rs — switch role allow-list to RoleMatrix; RLS consults membership for sensitive tables"
  - "2.5h: Tests — 11 test files (catalogue / matrix / check / assignment / jwt-claim / stub-migration / adr-gate / scope-grant / founder-webauthn / reserved-role-self-assign / perf)"

risk_if_skipped: "Every downstream module that needs more than a 2-role gate (tenant-admin/tenant-member) is blocked. DOC, KB, HR, REW, CRM, OKR all assume specialist roles (cfo, dpo, chro, cseco, founder, auditor) for sensitive operations. Without this FR, those modules ship with either (a) ad-hoc string-typed role checks that drift across services, or (b) over-broad permissions that grant tenant-admin too much power. FR-AUTH-108 (Lumi tenant-identity JWT) needs agent-persona to be in the production catalogue; FR-AUTH-109 (stub→full migration) needs this FR's matrix to point to. Compliance auditors (ISO 27001 A.5.18 + SOC 2 CC6.1) reject ad-hoc role checks — they expect a documented, ADR-gated catalogue with traceable assignment and revocation. Without ADR-101, the next time someone adds 'super-admin' to AUTH-005's allow-list there is no documented stop-sign — and the 22-role boundary is the only thing preventing the slide into ABAC complexity."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** ship the closed 22-role RBAC catalogue, the permission matrix, the role-assignment REST API, the JWT role-claim shape, the ADR gate, and the stub→full migration path. Each component:

1. **MUST** define exactly 22 roles in a closed Rust `enum Role` at `services/auth/src/rbac/catalogue.rs`. The variants are (in catalogue order, matching `website/docs/modules/auth.html#rbac-catalogue` and DEC-121): `RootAdmin`, `TenantAdmin`, `TenantMember`, `ServiceAccount`, `AgentPersona`, `Founder`, `Cfo`, `Cto`, `Coo`, `Chro`, `Cmo`, `Cpo`, `Cso`, `Cseco`, `Clo`, `Cdo`, `Dpo`, `Caio`, `ClientPortalUser`, `Auditor`, `Regulator`, `BillingSystem`. The string form is kebab-case (e.g. `tenant-admin`, `client-portal-user`). The enum **MUST NOT** carry a `_Other(String)` variant — unknown strings parse to `RoleParseError::UnknownRole(input)` and are rejected at every API boundary.

2. **MUST** define exactly **40 resources** in a closed `enum Resource` (one per cross-module surface: `subject`, `tenant`, `role_assignment`, `jwt_jwks`, `audit_row`, `crm_account`, `crm_contact`, `crm_deal`, `proj_issue`, `proj_engagement`, `proj_rate_card`, `proj_timeline`, `time_entry`, `time_expense`, `inv_invoice`, `inv_payment`, `inv_hoa_don`, `kb_document`, `kb_runbook`, `hr_member`, `hr_contract`, `hr_leave`, `hr_cccd_photo`, `rew_payslip`, `rew_bp_ledger`, `esop_grant`, `esop_valuation`, `learn_skill`, `learn_certification`, `okr_objective`, `okr_kr`, `res_allocation`, `doc_document`, `doc_signature`, `email_thread`, `chat_channel`, `chat_message`, `cuo_chain`, `brain_memory`, `obs_alert`). Adding a 41st resource is an ADR.

3. **MUST** define exactly 5 actions in a closed `enum Action`: `Read`, `Write`, `Admin`, `Approve`, `Sign`. `Approve` is reserved for dual-signoff workflows (CFO+CEO co-sign, etc.); `Sign` is reserved for e-signature on DOC + hóa đơn emissions. Adding a 6th action is an ADR.

4. **MUST** seed the permission matrix in migration `0005_roles_permissions.sql` with one row per `(role, resource, action)` tuple that is allowed. Disallowed tuples are absent — there is no explicit deny row. Total seeded rows: ~280 (varies by audit; the migration's row count is asserted in the migration test). The matrix is loaded into the `RoleMatrix` snapshot at service start and refreshed every 60 s by a tokio task.

5. **MUST** expose `POST /v1/admin/subjects/{subject_id}/roles` with body `{"role": "<kebab-name>"}` to grant a single role to a subject. The handler:
   - Validates the caller has `Resource::RoleAssignment + Action::Admin` (typically `tenant-admin` or `root-admin`).
   - Parses the role string via `Role::from_str` — unknown name → `400 BAD_REQUEST {"error":"unknown_role","role":"<x>","allowed":[...22 names...]}`.
   - Refuses self-assignment of reserved roles (per §1 #11). Reserved-role assignment requires a dedicated elevated-privilege endpoint (out of scope for slice 1; see §9).
   - Refuses `founder` assignment to a subject without a registered WebAuthn factor (per DEC-128 + FR-AUTH-105). Missing factor → `409 CONFLICT {"error":"webauthn_required","role":"founder"}`.
   - Inserts into `subject_roles` (tenant-scoped; RLS-protected) with `(subject_id, role, granted_by, granted_at)`. Duplicate grant → `409 CONFLICT {"error":"already_granted"}` (idempotent on the (subject_id, role) PK).
   - Emits exactly one `auth.role_assigned` BRAIN audit row before commit (audit-before-action per AUTHORING.md rule 25).
   - Returns `201 CREATED` with `{"subject_id", "role", "granted_by", "granted_at"}`.

6. **MUST** expose `DELETE /v1/admin/subjects/{subject_id}/roles/{role}` with the same caller-permission gate. Deletion:
   - Is hard-delete (no soft-delete; the audit row is the record).
   - Emits exactly one `auth.role_revoked` BRAIN audit row before commit.
   - Returns `204 NO CONTENT`. Already-absent → `204` (idempotent — `DELETE` matches REST semantics).

7. **MUST** expose `GET /v1/admin/roles` returning the catalogue: `{"version": <rbac_v>, "roles": [{"name":"<kebab>","display":"<Human>","reserved":<bool>,"requires_webauthn":<bool>,"scope_summary":"<one-line>"}, ...]}`. The handler reads from the in-memory `RoleMatrix`; never hits the DB. RBAC-version-aware caching: ETag is `W/"rbac-v<n>"`.

8. **MUST** issue access tokens (FR-AUTH-004 path) with two new claims:
   - `roles: ["<role-name>", ...]` — array of kebab-case role names the subject holds at issuance time.
   - `rbac_v: <integer>` — the catalogue version embedded for replay-resistance. Verifiers MAY compare against the live `RoleMatrix.version` and challenge tokens issued under stale catalogue versions (acceptable lag: 1 minor version; rejection threshold: > 2 versions behind).

9. **MUST** check role membership via the in-memory `RoleMatrix` only. The matrix is loaded at boot from `subject_roles` join `role_permissions` and refreshed every 60 s. Per-request DB lookups for role/permission are **MUST NOT** (per DEC-126 — performance budget is < 50 µs per check at p95).

10. **MUST** consult the `RoleMatrix` from RLS via the `auth.has_role(role_name)` SQL function (created in migration `0005_roles_permissions.sql`). The function reads the per-session GUC `auth.roles` (set by the JWT middleware on every connection). RLS policies for sensitive tables (`subjects`, `audit_row`, `billing_*`, `hr_cccd_photo`, `rew_payslip`) **MUST** include a `auth.has_role(<role>)` check in addition to the existing tenant-id RLS.

11. **MUST** classify the following 5 roles as **reserved** (`Role::is_reserved() == true`): `root-admin`, `client-portal-user`, `auditor`, `regulator`, `billing-system`. Reserved roles **MUST NOT** be assignable via the standard `POST /v1/admin/subjects/{id}/roles` endpoint. Attempts return `403 FORBIDDEN {"error":"reserved_role","role":"<x>","required_endpoint":"<path or 'not yet specified'>"}`. The dedicated reserved-role-assignment endpoints land in slice 2 (out of scope for this FR; see §9).

12. **MUST** classify `founder` as **WebAuthn-required** (`Role::requires_webauthn() == true`). All other roles return `false` from this method. The classification is intrinsic to the role (closed enum match), not driven by configuration — preventing accidental relaxation.

13. **MUST** reject any JWT whose `roles` claim contains a string that does not parse to a `Role` variant. The verifier (used by every consuming service per FR-AUTH-004 §1 #7) treats this as a tampered token: `401 UNAUTHORIZED {"error":"invalid_token","reason":"unknown_role_in_claim"}`. The check is at the verifier, not downstream — failing closed.

14. **MUST** emit exactly one `auth.role_catalogue_changed` BRAIN audit row whenever the catalogue version bumps (via the trigger on `role_catalogue_version`). The row carries `{old_version, new_version, changed_at, migration_id, adr_id}`. Catalogue version bumps without a recorded `adr_id` (e.g. ad-hoc INSERT in a manual psql session) fail the audit row's `NOT NULL adr_id` constraint and roll back the change.

15. **MUST** ship migration `0005_roles_permissions.sql` that creates:
    - `roles(name TEXT PRIMARY KEY, display TEXT NOT NULL, reserved BOOLEAN NOT NULL, requires_webauthn BOOLEAN NOT NULL, scope_summary TEXT NOT NULL, lands_in_slice INT NOT NULL)` — 22 seeded rows matching the closed enum.
    - `resources(name TEXT PRIMARY KEY, module TEXT NOT NULL)` — 40 seeded rows.
    - `actions(name TEXT PRIMARY KEY)` — 5 seeded rows.
    - `role_permissions(role TEXT REFERENCES roles, resource TEXT REFERENCES resources, action TEXT REFERENCES actions, PRIMARY KEY (role, resource, action))` — ~280 seeded rows.
    - `subject_roles(tenant_id UUID, subject_id UUID, role TEXT REFERENCES roles, granted_by UUID, granted_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY (subject_id, role))` — tenant-scoped; RLS enabled with `USING + WITH CHECK` on `tenant_id = current_setting('auth.tenant_id')::uuid`.
    - SQL function `auth.has_role(role_name TEXT) RETURNS BOOLEAN` — reads session GUC `auth.roles` (comma-separated).

16. **MUST** ship migration `0006_role_catalogue_version.sql` that creates:
    - `role_catalogue_version(id INT PRIMARY KEY CHECK (id = 1), version INT NOT NULL, updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), adr_id TEXT NOT NULL)` — singleton (id=1).
    - AFTER INSERT/UPDATE/DELETE trigger on `roles` + `role_permissions` that bumps `version` and inserts an `auth.role_catalogue_changed` BRAIN audit row via the brain_writer bridge (FR-AI-003).

17. **MUST** ship the ADR gate at `services/auth/src/rbac/adr.rs` as a CI-callable validator. Invoked as `cargo test rbac_adr_gate_test`, it:
    - Parses each migration file under `services/auth/migrations/` for changes to `roles` or `role_permissions`.
    - For each such migration, requires a matching `services/auth/adr/ADR-NNN-*.md` file referenced in a SQL comment `-- ADR: ADR-NNN`.
    - Fails the test if any role-touching migration lacks an ADR reference, OR if the referenced ADR-NNN.md file does not exist.

18. **MUST** support the 5-role stub→full migration path **without invalidating existing access tokens issued under `rbac_v = 1`**. Existing tokens (issued by FR-AUTH-002's 5-role allow-list, before this FR ships) carry no `rbac_v` claim. The verifier treats the absence of `rbac_v` as implicit `rbac_v = 1` and accepts the token for a 30-day grace window. After grace, missing-claim tokens are rejected (`401 UNAUTHORIZED {"error":"rbac_version_required"}`). The grace-window enforcer is FR-AUTH-109.

19. **MUST** classify the 5 stub roles (root-admin, tenant-admin, tenant-member, service-account, agent-persona) as a strict prefix of the 22-role catalogue — same names, same string form, additive permissions only. The matrix migration MUST NOT change the permission matrix for any stub role except to add new (resource, action) tuples. A regression test (`rbac_stub_compat_test`) asserts that every (stub_role, resource, action) tuple present in the prior FR-AUTH-002 catalogue is still present after migration.

20. **MUST** support layered narrowing via the `scope_grant` table: `scope_grants(tenant_id UUID, subject_id UUID, resource TEXT, resource_id UUID, action TEXT, granted_by UUID, granted_at TIMESTAMPTZ, expires_at TIMESTAMPTZ)`. The grant **NARROWS** a role's base privilege to specific resource_ids. Example: `cfo` has `inv_invoice + Read` matrix-wide; a `scope_grants` row with `resource=inv_invoice, resource_id=<invoice-A>, action=Read, expires_at=2026-12-31` for a `tenant-member` subject grants read access to invoice-A only. The `HasPermission` extractor checks both layers: role matrix MUST permit OR scope-grant MUST cover. Grants are not standalone privileges — they only narrow/extend within a tenant's scope.

21. **MUST** complete role check (`has_role` / `has_any_role` / `has_permission`) in ≤ 50 µs p99 against an in-memory matrix of 22 roles × 200 permissions × 1000 active subjects. The performance test (`rbac_perf_test`) asserts this on every CI run.

22. **MUST** emit OTel span `auth.rbac_check` with attributes `subject_id_hash16`, `role`, `resource`, `action`, `outcome` (allow | deny | reserved_role | webauthn_required | unknown_role) on every check. Sampling: 1% under steady state, 100% on `outcome != allow` (deny + error paths fully captured).

23. **MUST** emit OTel metrics:
    - `auth_rbac_check_total{outcome, role}` (counter).
    - `auth_rbac_check_latency_us` (histogram; SLO p99 < 50 µs).
    - `auth_rbac_matrix_refresh_total{outcome}` (counter; outcome ∈ {success, db_unreachable, hash_unchanged}).
    - `auth_rbac_subject_role_count{tenant_id}` (gauge — number of (subject, role) pairs).
    - `auth_rbac_catalogue_version` (gauge — current `rbac_v`).

24. **MUST** include `roles` and `rbac_v` claims in every JWT issued by FR-AUTH-004 after this FR ships. The JWT header `typ` MUST remain `JWT`; the claim shape change is additive only (no breaking changes to existing claim names). FR-AUTH-004's verifier is updated to surface both claims via `Claims::roles()` and `Claims::rbac_v()`.

25. **MUST** ship the ADR file `services/auth/adr/ADR-101-rbac-22-role-catalogue.md` as part of this FR. The ADR documents: business rationale (closed catalogue prevents ABAC slide), scope-creep risk assessment (each new role costs 1 ADR + DPO + CSEC review), deprecation policy (90-day shadow-monitoring window), audit-trail implications (every assignment + revocation chained into BRAIN), and the explicit DPO + CSEC sign-off block.

---

## §2 — Why this design (rationale for humans)

**Why a closed 22-role catalogue and not ABAC (§1 #1, DEC-121, DEC-122)?** ABAC (attribute-based access control) sounds flexible — "if subject.department == finance AND resource.classification <= confidential, allow" — but in practice it produces a debugging nightmare. Every access decision becomes a logic puzzle; tracing why a request was denied requires reconstructing the attribute graph at decision time. RBAC with a closed catalogue is auditable by construction: every subject's privileges are a finite, enumerable set, and changes are ADR-gated. The 22-role boundary is a deliberate ceiling — when we want a 23rd, we either (a) realise an existing role covers the case, or (b) write an ADR that forces explicit consideration of scope creep. The website docs (§2.6) describe this as "a design assertion."

**Why the 5-role stub is a strict prefix (§1 #19, DEC-123)?** FR-AUTH-002 shipped a 2-name allow-list (`tenant-admin`, `tenant-member`); the wider stub used in FR-AUTH-005/006 included `root-admin`, `service-account`, `agent-persona`. Those 5 names are not "temporary placeholder names" — they ARE the production names. This FR adds 17 more without renaming any of the 5. The `rbac_stub_compat_test` regression test makes this guarantee enforceable: prior subjects holding `tenant-admin` keep exactly the same matrix-permission set + any new (resource, action) tuples that were absent before. This is the property that lets existing tokens (without `rbac_v`) keep working — the underlying matrix is additive only.

**Why a permission matrix layer instead of role checks directly in code (§1 #4, DEC-122)?** Role checks scattered across services produce drift: service A might use `caller.has_role("chief-financial-officer")`, service B might write `caller.role == "cfo" || caller.role == "founder"`. The matrix centralises the truth — `caller.has_permission(Resource::InvInvoice, Action::Approve)` is the canonical check, and the matrix decides which roles satisfy it. Adding a new role only requires updating the matrix; consuming services don't change. This is the ISO 27001:2022 A.5.18 recommendation (access rights provisioned via documented roles), not via inline string comparisons.

**Why role + scope-grant layered (§1 #20, DEC-124)?** Pure RBAC over-grants: giving an external auditor the `auditor` role grants matrix-wide read; we only want them to see the specific audit window they were engaged for. The scope-grant layer narrows: `auditor` role provides base privilege; the `scope_grants` row narrows it to `resource_id IN (engagement-2026-q3)`. Without scope-grants, we'd be forced to invent per-audit roles (`auditor-2026-q3`) — and that's the ABAC slide the closed catalogue is designed to prevent. Scope grants are NOT standalone privileges (they cannot grant what the role does not already permit); they only narrow/restrict. This preserves the matrix as the source of truth for "what is allowed in principle."

**Why in-memory RoleMatrix with 60s refresh and not per-request DB lookup (§1 #9, §1 #21, DEC-126)?** Role checks happen on every authenticated request — typically 5–20 per request lifetime (auth, RLS context, scope checks, audit-row emission). At 1k RPS with 10 checks each, that's 10k role lookups per second; at 1 ms per DB hop, that's 10 cores burning on RBAC alone. The in-memory matrix collapses this to a single hashmap lookup at < 50 µs. The 60-second refresh tolerates revocation latency in exchange for performance: a revoked role is honoured for up to 60s after revocation; the OTel `auth_rbac_matrix_refresh_total{outcome=success}` metric tracks freshness. Time-critical revocations (e.g. terminated employee) go via the per-tenant CRL flush endpoint that targets the cache directly (out of scope for this FR; see FR-AUTH-111 placeholder).

**Why reserved roles cannot be self-assigned (§1 #11, DEC-127)?** `root-admin` is the cross-tenant superuser — its assignment is a CyberSkill operator action, not a tenant action. `client-portal-user` belongs to PORTAL's JIT provisioning flow (FR-PORTAL-003). `auditor` and `regulator` are external identities granted through a vetted intake — never by a tenant-admin clicking a UI button. `billing-system` is the Stripe/VietQR webhook identity — assigned via a setup script, not a REST call. Routing reserved-role assignment through the standard endpoint would invite operator mistakes; refusing at the API boundary makes the elevated path explicit.

**Why founder requires WebAuthn (§1 #12, DEC-128)?** The founder role grants cross-module privileged read (financial overview, OKR-cascade override, strategic-document signoff). A compromised founder credential is catastrophic — the entire company's executive view is exposed. WebAuthn (passkey) eliminates phishability and provides hardware-bound assurance. The check is intrinsic to the role (`Role::Founder.requires_webauthn() == true`) rather than configuration-driven because we never want this gate accidentally turned off via a config typo.

**Why `rbac_v` in the JWT (§1 #8, DEC-129)?** Without versioning, a long-lived access token issued before a role's permissions were tightened keeps the older, looser permissions until expiry. The `rbac_v` claim lets verifiers detect stale tokens and challenge them — the threshold (2 versions behind) tolerates normal refresh lag but catches significantly outdated tokens. Verifiers compare `claims.rbac_v` against `RoleMatrix.version`; > 2 behind → reject with `rbac_version_stale`. This is replay-resistance against tokens issued before a role-tightening ADR landed.

**Why the ADR gate at CI (§1 #17, §1 #25, DEC-121)?** Without the gate, a developer can quietly add a 23rd role via a migration patch and the closed-catalogue design assertion silently dies. The CI test fails the build until ADR-NNN.md exists AND the migration's SQL comment references it. The cost is one ADR file per role change; the benefit is that the catalogue's boundary is enforced by tooling, not by reviewer vigilance.

**Why classify exactly 40 resources and 5 actions (§1 #2, §1 #3)?** Resources and actions are themselves closed enums for the same reason roles are: drift between services on what counts as a "resource" or an "action" produces unauditable matrices. 40 covers every cross-module surface in the planned BACKLOG; 5 actions (Read/Write/Admin/Approve/Sign) cover every business workflow without ABAC-style verb explosion. Adding the 41st resource or 6th action is an ADR — the same governance discipline as adding a role.

**Why does the `roles` claim hold names only and not the matrix (§1 #8, PDPL Art. 6)?** Embedding the full permission matrix in every JWT bloats tokens (typical ~2 KB → ~50 KB) and creates a data-minimisation problem: every service that logs the JWT now logs every permission. Names-only keeps the token small and the verifier-side matrix lookup fast (< 50 µs). The verifier resolves names → permissions via the in-memory `RoleMatrix`; the token never carries the matrix itself.

**Why `auth.has_role()` as a SQL function for RLS (§1 #10)?** RLS policies need to evaluate at the database layer, not the application layer — application-layer checks can be bypassed by a SQL-injection or a misrouted query. The SQL function reads the per-session GUC (set by the JWT middleware at connection acquisition); RLS policies invoke it inline. This makes "tenant-admin or above can read audit rows" express as `USING (tenant_id = current_setting('auth.tenant_id')::uuid AND auth.has_role('tenant-admin'))` — the role check is in the policy, not behind it.

**Why hard-delete role assignments instead of soft-delete (§1 #6)?** The `auth.role_revoked` BRAIN audit row IS the record. Soft-delete would create two records of truth (the row's `revoked_at` field plus the audit row) and invite "is this revoked?" ambiguity in queries. The audit row is unerasable (per AGENTS.md §6.5); soft-delete adds nothing the audit chain doesn't already provide.

**Why a 30-day grace window for stub-era tokens (§1 #18, DEC-125)?** Typical access-token lifetime in the FR-AUTH-004 design is 1 hour; refresh tokens 7 days. A 30-day grace covers every refresh-token cycle plus a safety margin — no production user is surprised by a forced re-auth. The grace-window enforcer (FR-AUTH-109) flips on `30d after this FR ships`; until then, missing-`rbac_v` tokens are accepted. After flip, missing-claim tokens are rejected — and the rejection metric `auth_rbac_check_total{outcome=stub_token_rejected}` tells operations how many tokens are still missing the claim.

---

## §3 — API contract

### 3.1 — Closed role enum

```rust
// services/auth/src/rbac/catalogue.rs
use std::str::FromStr;
use std::fmt;
use serde::{Deserialize, Serialize};

/// The closed 22-role catalogue. Adding a variant requires ADR-NNN + matching migration.
/// String form is kebab-case via Display + FromStr.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "String")]
pub enum Role {
    RootAdmin,
    TenantAdmin,
    TenantMember,
    ServiceAccount,
    AgentPersona,
    Founder,
    Cfo,
    Cto,
    Coo,
    Chro,
    Cmo,
    Cpo,
    Cso,
    Cseco,
    Clo,
    Cdo,
    Dpo,
    Caio,
    ClientPortalUser,
    Auditor,
    Regulator,
    BillingSystem,
}

impl Role {
    /// All 22 variants in catalogue order. Tests assert len() == 22.
    pub const ALL: &'static [Role] = &[
        Role::RootAdmin, Role::TenantAdmin, Role::TenantMember, Role::ServiceAccount,
        Role::AgentPersona, Role::Founder, Role::Cfo, Role::Cto, Role::Coo, Role::Chro,
        Role::Cmo, Role::Cpo, Role::Cso, Role::Cseco, Role::Clo, Role::Cdo, Role::Dpo,
        Role::Caio, Role::ClientPortalUser, Role::Auditor, Role::Regulator, Role::BillingSystem,
    ];

    /// Kebab-case string form (canonical wire/storage representation).
    pub fn as_str(self) -> &'static str {
        match self {
            Role::RootAdmin => "root-admin",
            Role::TenantAdmin => "tenant-admin",
            Role::TenantMember => "tenant-member",
            Role::ServiceAccount => "service-account",
            Role::AgentPersona => "agent-persona",
            Role::Founder => "founder",
            Role::Cfo => "chief-financial-officer",
            Role::Cto => "chief-technology-officer",
            Role::Coo => "chief-operating-officer",
            Role::Chro => "chief-human-resources-officer",
            Role::Cmo => "chief-marketing-officer",
            Role::Cpo => "cpo",
            Role::Cso => "cso",
            Role::Cseco => "cseco",
            Role::Clo => "clo",
            Role::Cdo => "cdo",
            Role::Dpo => "dpo",
            Role::Caio => "chief-ai-officer",
            Role::ClientPortalUser => "client-portal-user",
            Role::Auditor => "auditor",
            Role::Regulator => "regulator",
            Role::BillingSystem => "billing-system",
        }
    }

    /// Reserved roles (cannot be self-assigned via standard REST).
    pub fn is_reserved(self) -> bool {
        matches!(self,
            Role::RootAdmin | Role::ClientPortalUser | Role::Auditor
            | Role::Regulator | Role::BillingSystem
        )
    }

    /// WebAuthn-required roles (founder only at slice 1).
    pub fn requires_webauthn(self) -> bool {
        matches!(self, Role::Founder)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RoleParseError {
    #[error("unknown_role: {0}")]
    UnknownRole(String),
}

impl FromStr for Role {
    type Err = RoleParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for r in Role::ALL { if r.as_str() == s { return Ok(*r); } }
        Err(RoleParseError::UnknownRole(s.to_string()))
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
}

impl TryFrom<&str> for Role {
    type Error = RoleParseError;
    fn try_from(s: &str) -> Result<Self, Self::Error> { Role::from_str(s) }
}

impl From<Role> for String {
    fn from(r: Role) -> String { r.as_str().to_string() }
}
```

### 3.2 — Closed resource + action enums

```rust
// services/auth/src/rbac/permissions.rs
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "&'static str")]
pub enum Resource {
    Subject, Tenant, RoleAssignment, JwtJwks, AuditRow,
    CrmAccount, CrmContact, CrmDeal,
    ProjIssue, ProjEngagement, ProjRateCard, ProjTimeline,
    TimeEntry, TimeExpense,
    InvInvoice, InvPayment, InvHoaDon,
    KbDocument, KbRunbook,
    HrMember, HrContract, HrLeave, HrCccdPhoto,
    RewPayslip, RewBpLedger,
    EsopGrant, EsopValuation,
    LearnSkill, LearnCertification,
    OkrObjective, OkrKr,
    ResAllocation,
    DocDocument, DocSignature,
    EmailThread, ChatChannel, ChatMessage,
    CuoChain, BrainMemory, ObsAlert,
}

impl Resource {
    pub const ALL: &'static [Resource] = &[ /* ...40 variants in order... */ ];
    pub fn as_str(self) -> &'static str { /* kebab-snake match */ todo!() }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "&str", into = "&'static str")]
pub enum Action { Read, Write, Admin, Approve, Sign }

impl Action {
    pub const ALL: &'static [Action] = &[Action::Read, Action::Write, Action::Admin, Action::Approve, Action::Sign];
}
```

### 3.3 — In-memory RoleMatrix + 60s refresher

```rust
// services/auth/src/rbac/matrix.rs
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use arc_swap::ArcSwap;
use sha2::{Digest, Sha256};

use crate::rbac::catalogue::Role;
use crate::rbac::permissions::{Resource, Action};

#[derive(Debug, Clone)]
pub struct RoleMatrix {
    /// Catalogue version (bumped on every ADR-approved schema change).
    pub version: u32,
    /// (role, resource, action) → allowed.
    pub allowed: HashSet<(Role, Resource, Action)>,
    /// hash of the matrix for change detection during refresh.
    pub hash: [u8; 32],
}

impl RoleMatrix {
    pub fn has_permission(&self, role: Role, res: Resource, act: Action) -> bool {
        self.allowed.contains(&(role, res, act))
    }
}

pub struct RoleMatrixHandle {
    inner: Arc<ArcSwap<RoleMatrix>>,
}

impl RoleMatrixHandle {
    pub async fn load_from_db(pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        let m = load_matrix(pool).await?;
        Ok(Self { inner: Arc::new(ArcSwap::from_pointee(m)) })
    }

    pub fn snapshot(&self) -> Arc<RoleMatrix> { self.inner.load_full() }

    /// Spawn 60s refresher. Reloads matrix; only swaps if hash changed.
    pub fn spawn_refresher(self: Arc<Self>, pool: sqlx::PgPool) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                interval.tick().await;
                match load_matrix(&pool).await {
                    Ok(fresh) => {
                        let current = self.inner.load();
                        if current.hash != fresh.hash {
                            self.inner.store(Arc::new(fresh));
                            metrics::counter!("auth_rbac_matrix_refresh_total", "outcome" => "success").increment(1);
                        } else {
                            metrics::counter!("auth_rbac_matrix_refresh_total", "outcome" => "hash_unchanged").increment(1);
                        }
                    }
                    Err(_) => {
                        metrics::counter!("auth_rbac_matrix_refresh_total", "outcome" => "db_unreachable").increment(1);
                        // Keep serving stale matrix; never panic the service.
                    }
                }
            }
        });
    }
}

async fn load_matrix(pool: &sqlx::PgPool) -> anyhow::Result<RoleMatrix> {
    let version: i32 = sqlx::query_scalar("SELECT version FROM role_catalogue_version WHERE id = 1")
        .fetch_one(pool).await?;
    let rows: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT role, resource, action FROM role_permissions ORDER BY role, resource, action"
    ).fetch_all(pool).await?;
    let mut allowed = HashSet::new();
    let mut hasher = Sha256::new();
    for (r, res, act) in &rows {
        hasher.update(r.as_bytes()); hasher.update(b"\0");
        hasher.update(res.as_bytes()); hasher.update(b"\0");
        hasher.update(act.as_bytes()); hasher.update(b"\n");
        let role = Role::from_str(r)?;
        let resource = Resource::from_str(res)?;
        let action = Action::from_str(act)?;
        allowed.insert((role, resource, action));
    }
    Ok(RoleMatrix { version: version as u32, allowed, hash: hasher.finalize().into() })
}
```

### 3.4 — Axum extractors

```rust
// services/auth/src/rbac/check.rs
use axum::{async_trait, extract::FromRequestParts, http::request::Parts, http::StatusCode};
use crate::rbac::catalogue::Role;
use crate::rbac::permissions::{Resource, Action};
use crate::jwt::Claims;

pub struct HasRole(pub Role);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for HasRole {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims: &Claims = parts.extensions.get()
            .ok_or((StatusCode::UNAUTHORIZED, "no_claims"))?;
        // Read required role from request annotation (set by route handler).
        let required: Role = parts.extensions.get::<Role>()
            .copied().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "no_role_annotation"))?;
        if claims.roles().contains(&required) {
            Ok(HasRole(required))
        } else {
            metrics::counter!("auth_rbac_check_total", "outcome" => "deny", "role" => required.as_str()).increment(1);
            Err((StatusCode::FORBIDDEN, "insufficient_role"))
        }
    }
}

pub struct HasPermission(pub Resource, pub Action);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for HasPermission {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let claims: &Claims = parts.extensions.get().ok_or((StatusCode::UNAUTHORIZED, "no_claims"))?;
        let matrix: &Arc<crate::rbac::matrix::RoleMatrix> = parts.extensions.get().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "no_matrix"))?;
        let (res, act) = parts.extensions.get::<(Resource, Action)>().copied().ok_or((StatusCode::INTERNAL_SERVER_ERROR, "no_perm_annotation"))?;
        let allowed = claims.roles().iter().any(|r| matrix.has_permission(*r, res, act));
        if allowed { Ok(HasPermission(res, act)) }
        else {
            metrics::counter!("auth_rbac_check_total", "outcome" => "deny", "role" => "agg").increment(1);
            Err((StatusCode::FORBIDDEN, "insufficient_permission"))
        }
    }
}
```

### 3.5 — Migration 0005

```sql
-- services/auth/migrations/0005_roles_permissions.sql
-- ADR: ADR-101

BEGIN;

CREATE TABLE roles (
    name              TEXT PRIMARY KEY,
    display           TEXT NOT NULL,
    reserved          BOOLEAN NOT NULL,
    requires_webauthn BOOLEAN NOT NULL,
    scope_summary     TEXT NOT NULL,
    lands_in_slice    INT NOT NULL
);

INSERT INTO roles (name, display, reserved, requires_webauthn, scope_summary, lands_in_slice) VALUES
  ('root-admin',         'Root Admin',         TRUE,  FALSE, 'Cross-tenant superuser; CyberSkill operators only',    1),
  ('tenant-admin',       'Tenant Admin',       FALSE, FALSE, 'Full admin within one tenant',                          1),
  ('tenant-member',      'Tenant Member',      FALSE, FALSE, 'Regular member; read shareable+, write own scopes',     1),
  ('service-account',    'Service Account',    FALSE, FALSE, 'Non-human identity; module-to-module mTLS',             1),
  ('agent-persona',      'Agent Persona',      FALSE, FALSE, 'Persona-versioned agent (CUO + sub-skills)',            1),
  ('founder',            'Founder',            FALSE, TRUE,  'Founder-CEO equivalent; WebAuthn required',             3),
  ('cfo',                'CFO',                FALSE, FALSE, 'Financial read + disbursement + ESOP signoff',          4),
  ('cto',                'CTO',                FALSE, FALSE, 'Tech-debt + security advisory + OBS digest target',     4),
  ('coo',                'COO',                FALSE, FALSE, 'Cross-module status + blocker triage + process',        4),
  ('chro',               'CHRO',               FALSE, FALSE, 'HR records + onboarding + perf review + PII-elevated', 4),
  ('cmo',                'CMO',                FALSE, FALSE, 'Campaign briefs + content calendars + comms approval',  4),
  ('cpo',                'CPO',                FALSE, FALSE, 'Product brief + roadmap + feature-request-author canonical',         4),
  ('cso',                'CSO (Strategy)',     FALSE, FALSE, 'OKR cascade + scenarios + competitive intel read',      4),
  ('cseco',              'CSO (Security)',     FALSE, FALSE, 'Security review + key rotation + vuln triage',          4),
  ('clo',                'CLO',                FALSE, FALSE, 'Contract redline + DSAR triage + regulatory signoff',   4),
  ('cdo',                'CDO',                FALSE, FALSE, 'Data quality + lineage + residency + BRAIN owner',      4),
  ('dpo',                'DPO',                FALSE, FALSE, 'DSAR fulfilment + breach notification + purge approval',4),
  ('caio',               'CAIO',               FALSE, FALSE, 'AI Gateway budget + synthesis sub-skill review',        5),
  ('client-portal-user', 'Client Portal User', TRUE,  FALSE, 'External tenant user (PORTAL filter only)',             5),
  ('auditor',            'Auditor',            TRUE,  FALSE, 'External auditor; read-only, time-bounded, scope-pinned',5),
  ('regulator',          'Regulator',          TRUE,  FALSE, 'External regulatory authority; DSAR + breach scopes',   5),
  ('billing-system',     'Billing System',     TRUE,  FALSE, 'Stripe/VietQR/Momo webhook identity; write-restricted', 5);

CREATE TABLE resources (
    name   TEXT PRIMARY KEY,
    module TEXT NOT NULL
);
-- 40 rows inserted here, one per Resource enum variant (omitted for brevity in spec).

CREATE TABLE actions (name TEXT PRIMARY KEY);
INSERT INTO actions (name) VALUES ('read'), ('write'), ('admin'), ('approve'), ('sign');

CREATE TABLE role_permissions (
    role     TEXT NOT NULL REFERENCES roles(name),
    resource TEXT NOT NULL REFERENCES resources(name),
    action   TEXT NOT NULL REFERENCES actions(name),
    PRIMARY KEY (role, resource, action)
);
-- ~280 INSERT rows here, generated from the matrix design in ADR-101 §3.

CREATE TABLE subject_roles (
    tenant_id  UUID NOT NULL,
    subject_id UUID NOT NULL,
    role       TEXT NOT NULL REFERENCES roles(name),
    granted_by UUID NOT NULL,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (subject_id, role)
);
ALTER TABLE subject_roles ENABLE ROW LEVEL SECURITY;
CREATE POLICY subject_roles_tenant ON subject_roles
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE INSERT, UPDATE, DELETE ON role_permissions FROM cyberos_app;  -- runtime cannot mutate matrix
REVOKE INSERT, UPDATE, DELETE ON roles FROM cyberos_app;             -- runtime cannot add roles
REVOKE INSERT, UPDATE, DELETE ON resources FROM cyberos_app;
REVOKE INSERT, UPDATE, DELETE ON actions FROM cyberos_app;

CREATE TABLE scope_grants (
    tenant_id   UUID NOT NULL,
    subject_id  UUID NOT NULL,
    resource    TEXT NOT NULL REFERENCES resources(name),
    resource_id UUID NOT NULL,
    action      TEXT NOT NULL REFERENCES actions(name),
    granted_by  UUID NOT NULL,
    granted_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ,
    PRIMARY KEY (subject_id, resource, resource_id, action)
);
ALTER TABLE scope_grants ENABLE ROW LEVEL SECURITY;
CREATE POLICY scope_grants_tenant ON scope_grants
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE OR REPLACE FUNCTION auth.has_role(role_name TEXT) RETURNS BOOLEAN AS $$
DECLARE
    roles_csv TEXT;
BEGIN
    roles_csv := current_setting('auth.roles', true);
    IF roles_csv IS NULL THEN RETURN FALSE; END IF;
    RETURN role_name = ANY(string_to_array(roles_csv, ','));
END;
$$ LANGUAGE plpgsql STABLE;

COMMIT;
```

### 3.6 — Migration 0006 (catalogue version + trigger)

```sql
-- services/auth/migrations/0006_role_catalogue_version.sql
-- ADR: ADR-101

BEGIN;

CREATE TABLE role_catalogue_version (
    id         INT PRIMARY KEY CHECK (id = 1),
    version    INT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    adr_id     TEXT NOT NULL
);
INSERT INTO role_catalogue_version (id, version, adr_id) VALUES (1, 2, 'ADR-101');
-- v1 was the implicit pre-FR-AUTH-101 catalogue (5 roles); v2 ships here.

CREATE OR REPLACE FUNCTION bump_catalogue_version() RETURNS TRIGGER AS $$
BEGIN
    UPDATE role_catalogue_version SET version = version + 1, updated_at = now() WHERE id = 1;
    -- BRAIN audit row emission via brain_writer bridge (FR-AI-003) is handled at the migration commit hook.
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_bump_on_roles AFTER INSERT OR UPDATE OR DELETE ON roles
    EXECUTE FUNCTION bump_catalogue_version();
CREATE TRIGGER trg_bump_on_perms AFTER INSERT OR UPDATE OR DELETE ON role_permissions
    EXECUTE FUNCTION bump_catalogue_version();

REVOKE INSERT, UPDATE, DELETE ON role_catalogue_version FROM cyberos_app;

COMMIT;
```

### 3.7 — REST handlers

```rust
// services/auth/src/admin/roles_rest.rs
use axum::{Json, extract::{Path, State}, http::StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::rbac::catalogue::Role;
use crate::rbac::matrix::RoleMatrixHandle;
use crate::audit::role_events;

#[derive(Deserialize)]
pub struct AssignRoleRequest { pub role: String }

#[derive(Serialize)]
pub struct AssignRoleResponse {
    pub subject_id: Uuid,
    pub role: String,
    pub granted_by: Uuid,
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

pub async fn assign_role(
    State(state): State<AppState>,
    claims: Claims,                              // injected by FR-AUTH-004 middleware
    Path(subject_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> Result<(StatusCode, Json<AssignRoleResponse>), ApiError> {
    // 1. Permission check.
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::RoleAssignment, Action::Admin)?;
    // 2. Parse role string against closed enum.
    let role: Role = req.role.parse().map_err(|_| ApiError::UnknownRole(req.role.clone()))?;
    // 3. Refuse reserved-role assignment via this endpoint.
    if role.is_reserved() { return Err(ApiError::ReservedRole(role)); }
    // 4. Founder requires WebAuthn enrolment.
    if role.requires_webauthn() {
        let has_passkey = state.webauthn.has_factor(claims.tenant_id(), subject_id).await?;
        if !has_passkey { return Err(ApiError::WebAuthnRequired(role)); }
    }
    // 5. Insert + audit-before-action in one transaction.
    let mut tx = state.db.begin().await?;
    sqlx::query("INSERT INTO subject_roles (tenant_id, subject_id, role, granted_by) VALUES ($1, $2, $3, $4)")
        .bind(claims.tenant_id()).bind(subject_id).bind(role.as_str()).bind(claims.subject_id())
        .execute(&mut *tx).await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.code().as_deref() == Some("23505") { return ApiError::AlreadyGranted; }
            }
            ApiError::Db(e)
        })?;
    role_events::emit_role_assigned(&mut tx, claims.tenant_id(), subject_id, role, claims.subject_id()).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(AssignRoleResponse { subject_id, role: role.as_str().to_string(), granted_by: claims.subject_id(), granted_at: chrono::Utc::now() })))
}

pub async fn revoke_role(
    State(state): State<AppState>,
    claims: Claims,
    Path((subject_id, role_str)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::RoleAssignment, Action::Admin)?;
    let role: Role = role_str.parse().map_err(|_| ApiError::UnknownRole(role_str.clone()))?;
    let mut tx = state.db.begin().await?;
    let rows = sqlx::query("DELETE FROM subject_roles WHERE tenant_id = $1 AND subject_id = $2 AND role = $3")
        .bind(claims.tenant_id()).bind(subject_id).bind(role.as_str())
        .execute(&mut *tx).await?.rows_affected();
    if rows > 0 {
        role_events::emit_role_revoked(&mut tx, claims.tenant_id(), subject_id, role, claims.subject_id()).await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_roles(State(state): State<AppState>) -> Json<RoleCatalogueResponse> {
    let matrix = state.matrix.snapshot();
    let roles: Vec<RoleCatalogueEntry> = Role::ALL.iter().map(|r| RoleCatalogueEntry {
        name: r.as_str().into(),
        display: r.display_name().into(),
        reserved: r.is_reserved(),
        requires_webauthn: r.requires_webauthn(),
        scope_summary: r.scope_summary().into(),
    }).collect();
    Json(RoleCatalogueResponse { version: matrix.version, roles })
}
```

### 3.8 — Updated JWT claims

```rust
// services/auth/src/jwt.rs (delta)
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,               // subject_id
    pub tid: Uuid,               // tenant_id
    pub iss: String,
    pub iat: i64,
    pub exp: i64,
    pub nbf: i64,
    pub roles: Vec<String>,      // NEW — kebab-case role names
    pub rbac_v: u32,             // NEW — catalogue version at issuance
}

impl Claims {
    pub fn roles(&self) -> Vec<Role> {
        // Per §1 #13: refuse tokens with unknown role names.
        self.roles.iter().map(|s| s.parse::<Role>()).collect::<Result<_, _>>()
            .expect("verifier upstream already rejected unknown-role tokens")
    }
    pub fn rbac_v(&self) -> u32 { self.rbac_v }
}
```

### 3.9 — ADR file

```markdown
<!-- services/auth/adr/ADR-101-rbac-22-role-catalogue.md -->
# ADR-101 — 22-role closed RBAC catalogue

**Status:** Accepted
**Date:** 2026-05-16
**Deciders:** Stephen Cheng (CTO), DPO sign-off (pending), CSEC sign-off (pending)

## Context
CyberOS needs a role-based access control model that scales from the initial 1-tenant deployment (CyberSkill itself) to multi-tenant agency operation. Two design options: (a) ABAC (attribute-based), (b) RBAC with a closed catalogue.

## Decision
RBAC with exactly 22 closed roles. Permission matrix is `roles × resources × actions`. Roles are not extensible at runtime — schema-defined only. Adding role 23 requires a new ADR; same for resource 41 or action 6.

## Consequences
**Positive:** every access decision is auditable; permission matrix is enumerable; compliance auditors (ISO 27001 A.5.18) accept by construction.
**Negative:** narrowing per-resource access requires the scope-grants layer (DEC-124); pure RBAC over-grants.
**Mitigation:** scope-grants narrow within a tenant's matrix; never grant what the role doesn't already permit.

## Scope-creep risk assessment
Every new role costs: 1 ADR + DPO review + CSEC review + matrix migration + regression test. The cost is deliberate; it forces the question "does an existing role cover this?" before adding a new one.

## DPO sign-off
PDPL Art. 6 (data minimisation) is satisfied: the JWT carries role names only, not the permission matrix. The matrix never leaves the AUTH service.

## CSEC sign-off
Reserved-role assignment (root-admin, auditor, regulator, billing-system, client-portal-user) requires dedicated elevated endpoints, not the standard REST. Founder requires WebAuthn factor presence.
```

---

## §4 — Acceptance criteria

1. **Closed enum invariants** — `Role::ALL.len() == 22` AND every variant has unique `as_str()` AND `from_str(as_str()) == Ok(variant)` for all 22.
2. **Unknown role rejection** — `"super-admin".parse::<Role>()` returns `Err(RoleParseError::UnknownRole("super-admin"))`.
3. **Reserved role classification** — exactly 5 roles return `true` from `is_reserved()`: root-admin, client-portal-user, auditor, regulator, billing-system.
4. **WebAuthn-required classification** — exactly 1 role (`founder`) returns `true` from `requires_webauthn()`.
5. **Resource closed enum** — `Resource::ALL.len() == 40` AND parse round-trip.
6. **Action closed enum** — `Action::ALL.len() == 5` AND parse round-trip.
7. **Matrix seeded** — `SELECT count(*) FROM role_permissions` returns the expected row count (matches ADR-101 §3 enumeration; assertion in migration test).
8. **Matrix in-memory load** — service start loads matrix; `RoleMatrix::has_permission(TenantAdmin, Subject, Admin)` returns true.
9. **Matrix refresher hash-only-swap** — running the refresher twice on unchanged matrix yields `auth_rbac_matrix_refresh_total{outcome=hash_unchanged}` increment 1, not a fresh allocation.
10. **POST role happy path** — tenant-admin caller, valid target subject, valid role string → 201 with response shape; row appears in `subject_roles`; `auth.role_assigned` audit row emitted.
11. **POST role unknown** — `{"role":"super-admin"}` → 400 with `{"error":"unknown_role"}` body.
12. **POST role reserved** — `{"role":"root-admin"}` → 403 with `{"error":"reserved_role"}`.
13. **POST founder without passkey** — target subject has no WebAuthn factor; `{"role":"founder"}` → 409 with `{"error":"webauthn_required"}`.
14. **POST role idempotent** — re-POST same (subject_id, role) → 409 `{"error":"already_granted"}` (PK enforces).
15. **DELETE role happy** — existing grant → 204; row removed; `auth.role_revoked` audit row emitted.
16. **DELETE role absent** — non-existent grant → 204 (idempotent; matches REST DELETE semantics).
17. **GET /v1/admin/roles** — returns version + 22 entries with stable order; ETag is `W/"rbac-v<n>"`.
18. **JWT carries roles + rbac_v** — newly issued token (after FR-AUTH-101 ships) has both claims populated; older tokens (pre-FR-AUTH-101) lack `rbac_v` and are accepted during grace window.
19. **JWT unknown-role rejection** — verifier given a token with `roles: ["super-admin"]` returns 401 `{"error":"invalid_token","reason":"unknown_role_in_claim"}`.
20. **RLS consults role** — query as a `tenant-member` subject against `audit_row` returns 0 rows; same query as `tenant-admin` returns all tenant-scoped rows. (`auth.has_role` SQL function in policy.)
21. **Stub compatibility** — every (stub_role, resource, action) tuple from the pre-FR-AUTH-101 matrix is present in the new matrix; no removed tuples (additive only).
22. **ADR gate** — migration adding a 23rd role without ADR-NNN comment → `cargo test rbac_adr_gate_test` fails.
23. **Catalogue-changed audit** — INSERT into `roles` (via test migration with ADR) bumps `role_catalogue_version.version`; emits exactly one `auth.role_catalogue_changed` BRAIN audit row with old/new version + adr_id.
24. **Scope-grant narrowing** — `cfo` role + scope-grant `(inv_invoice, invoice-A, Read, expires=future)` → caller may read invoice-A; revoke grant → 403 on next request after 60s cache TTL.
25. **Perf budget** — `rbac_perf_test`: 1k checks against 22-role × 40-resource × 5-action matrix complete in < 50 µs p99 (in-process, no network).
26. **OTel span emission** — every check emits `auth.rbac_check` span with `outcome` attribute; deny + error outcomes always sampled.
27. **OTel metrics emission** — `auth_rbac_check_total`, `auth_rbac_check_latency_us`, `auth_rbac_matrix_refresh_total`, `auth_rbac_subject_role_count`, `auth_rbac_catalogue_version` all observable from `/metrics` endpoint.

---

## §5 — Verification

```rust
// services/auth/tests/rbac_catalogue_test.rs
use cyberos_auth::rbac::catalogue::{Role, RoleParseError};
use std::collections::HashSet;

#[test]
fn all_22_roles_present() {
    assert_eq!(Role::ALL.len(), 22);
    let names: HashSet<&str> = Role::ALL.iter().map(|r| r.as_str()).collect();
    assert_eq!(names.len(), 22, "duplicate role names");
}

#[test]
fn roundtrip_for_every_role() {
    for r in Role::ALL {
        let parsed: Role = r.as_str().parse().unwrap();
        assert_eq!(*r, parsed);
    }
}

#[test]
fn unknown_role_rejected() {
    let err = "super-admin".parse::<Role>().unwrap_err();
    assert!(matches!(err, RoleParseError::UnknownRole(s) if s == "super-admin"));
}

#[test]
fn reserved_roles_exact_set() {
    let reserved: HashSet<Role> = Role::ALL.iter().copied().filter(|r| r.is_reserved()).collect();
    let expected: HashSet<Role> = [Role::RootAdmin, Role::ClientPortalUser, Role::Auditor, Role::Regulator, Role::BillingSystem].into_iter().collect();
    assert_eq!(reserved, expected);
}

#[test]
fn webauthn_required_exactly_founder() {
    let required: Vec<Role> = Role::ALL.iter().copied().filter(|r| r.requires_webauthn()).collect();
    assert_eq!(required, vec![Role::Founder]);
}
```

```rust
// services/auth/tests/rbac_assignment_test.rs (excerpt)
#[tokio::test]
async fn assign_unknown_role_rejected() {
    let ctx = TestCtx::new_with_seed("tenant-admin").await;
    let resp = ctx.post(&format!("/v1/admin/subjects/{}/roles", target_id), json!({"role":"super-admin"})).await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "unknown_role");
    assert_eq!(body["role"], "super-admin");
    assert!(body["allowed"].as_array().unwrap().len() == 22);
}

#[tokio::test]
async fn assign_reserved_role_rejected() {
    let ctx = TestCtx::new_with_seed("tenant-admin").await;
    let resp = ctx.post(&format!("/v1/admin/subjects/{}/roles", target_id), json!({"role":"root-admin"})).await;
    assert_eq!(resp.status(), 403);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "reserved_role");
}

#[tokio::test]
async fn assign_founder_without_passkey_rejected() {
    let ctx = TestCtx::new_with_seed("tenant-admin").await;
    let target_id = ctx.create_subject_without_passkey().await;
    let resp = ctx.post(&format!("/v1/admin/subjects/{}/roles", target_id), json!({"role":"founder"})).await;
    assert_eq!(resp.status(), 409);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "webauthn_required");
}

#[tokio::test]
async fn assign_role_emits_audit_row() {
    let ctx = TestCtx::new_with_seed("tenant-admin").await;
    let target_id = ctx.create_subject().await;
    let resp = ctx.post(&format!("/v1/admin/subjects/{}/roles", target_id), json!({"role":"cfo"})).await;
    assert_eq!(resp.status(), 201);
    let rows = ctx.brain_audit_rows("auth.role_assigned").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["subject_id"], target_id.to_string());
    assert_eq!(rows[0]["role"], "chief-financial-officer");
}
```

```rust
// services/auth/tests/rbac_perf_test.rs
#[test]
fn matrix_check_under_50us_p99() {
    let matrix = synthetic_matrix(22, 40, 5);
    let mut times: Vec<u64> = Vec::with_capacity(10_000);
    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        let t0 = std::time::Instant::now();
        std::hint::black_box(matrix.has_permission(Role::Cfo, Resource::InvInvoice, Action::Read));
        times.push(t0.elapsed().as_nanos() as u64);
    }
    let elapsed = start.elapsed();
    times.sort_unstable();
    let p99_us = (times[(times.len() * 99) / 100] as f64) / 1000.0;
    eprintln!("matrix check p99 = {p99_us:.3} µs (10k iters, total {elapsed:?})");
    assert!(p99_us < 50.0, "p99 = {p99_us}µs exceeds 50µs budget");
}
```

```rust
// services/auth/tests/rbac_adr_gate_test.rs
#[test]
fn every_role_touching_migration_has_adr_reference() {
    let walker = crate::rbac::adr::AdrGate::new("services/auth/migrations", "services/auth/adr");
    let report = walker.scan().expect("scan");
    assert!(report.violations.is_empty(),
        "migrations touch roles/role_permissions without ADR reference: {:#?}", report.violations);
}
```

```rust
// services/auth/tests/rbac_stub_migration_test.rs
#[tokio::test]
async fn stub_token_accepted_during_grace_window() {
    let ctx = TestCtx::new().await;
    let stub_token = ctx.issue_stub_token(/* no rbac_v claim */).await;
    let resp = ctx.get("/v1/admin/roles").header("Authorization", format!("Bearer {stub_token}")).await;
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn stub_token_rejected_after_grace_window() {
    let ctx = TestCtx::new_with_clock_skew_days(31).await;  // simulate post-grace
    let stub_token = ctx.issue_stub_token().await;
    let resp = ctx.get("/v1/admin/roles").header("Authorization", format!("Bearer {stub_token}")).await;
    assert_eq!(resp.status(), 401);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["reason"], "rbac_version_required");
}

#[tokio::test]
async fn stub_role_permissions_strictly_additive() {
    let pre_matrix = load_pre_fr_auth_101_matrix();
    let post_matrix = load_matrix_after_migrations();
    for (role, res, act) in &pre_matrix.allowed {
        assert!(post_matrix.allowed.contains(&(*role, *res, *act)),
            "FR-AUTH-101 removed pre-existing tuple ({role:?}, {res:?}, {act:?})");
    }
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton — Resource enum's full 40-variant body is filled in during implementation; ADR-101 §3 enumerates the matrix rows that the migration seeds.)

---

## §7 — Dependencies

**Upstream (this FR depends on):**
- **FR-AUTH-005** — admin REST router; this FR adds `/v1/admin/subjects/{id}/roles` and `/v1/admin/roles` to it.
- **FR-AUTH-004** — JWT issuance; this FR extends the claim shape.
- **FR-AUTH-003** — RLS enforcement; this FR adds role-aware policies on sensitive tables.
- **FR-AI-003** — brain-audit bridge; receives `auth.role_assigned / .role_revoked / .role_catalogue_changed` rows.

**Downstream (this FR blocks):**
- **FR-AUTH-108** — Lumi tenant-identity JWT shape (needs agent-persona role in production catalogue).
- **FR-AUTH-109** — stub→full migration enforcer (this FR ships the matrix it migrates to).
- **FR-CRM-001, FR-DOC-001, FR-HR-001, FR-KB-001, FR-OKR-001, FR-REW-001** — all need specialist roles (cfo, dpo, chro, founder).

**Cross-module (informational):**
- **FR-AUTH-105 (placeholder)** — passkey enrolment. The founder-WebAuthn gate (§1 #12) calls this FR's `webauthn::has_factor`. Until FR-AUTH-105 ships, the gate returns `false` for all subjects (founder assignment effectively disabled).
- **FR-AUTH-111 (placeholder)** — per-tenant CRL flush for instant revocation. Until FR-AUTH-111 ships, revocations are honoured at 60s latency.

---

## §8 — Example payloads

### 8.1 — POST /v1/admin/subjects/{id}/roles request

```json
{
  "role": "cfo"
}
```

### 8.2 — 201 CREATED response

```json
{
  "subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "role": "chief-financial-officer",
  "granted_by": "8a7c8c80-1234-4567-89ab-cdef01234567",
  "granted_at": "2026-05-16T14:32:11Z"
}
```

### 8.3 — auth.role_assigned BRAIN audit row

```json
{
  "kind": "auth.role_assigned",
  "tenant_id": "5e8f1d2a-...",
  "subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "role": "chief-financial-officer",
  "granted_by": "8a7c8c80-1234-4567-89ab-cdef01234567",
  "rbac_v": 2,
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — GET /v1/admin/roles response (truncated)

```json
{
  "version": 2,
  "roles": [
    {"name":"root-admin","display":"Root Admin","reserved":true,"requires_webauthn":false,"scope_summary":"Cross-tenant superuser; CyberSkill operators only"},
    {"name":"tenant-admin","display":"Tenant Admin","reserved":false,"requires_webauthn":false,"scope_summary":"Full admin within one tenant"},
    {"name":"founder","display":"Founder","reserved":false,"requires_webauthn":true,"scope_summary":"Founder-CEO equivalent; WebAuthn required"}
  ]
}
```

### 8.5 — JWT claims (post-FR-AUTH-101)

```json
{
  "sub": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "tid": "5e8f1d2a-...",
  "iss": "https://auth.cyberskill.world",
  "iat": 1747920731,
  "exp": 1747924331,
  "nbf": 1747920731,
  "roles": ["tenant-member", "chief-financial-officer"],
  "rbac_v": 2
}
```

### 8.6 — auth.role_catalogue_changed BRAIN audit row

```json
{
  "kind": "auth.role_catalogue_changed",
  "old_version": 2,
  "new_version": 3,
  "migration_id": "0007_add_role_caso.sql",
  "adr_id": "ADR-115",
  "changed_at": "2026-08-01T10:00:00Z",
  "ts_ns": 1754049600000000000
}
```

---

## §9 — Open questions

Deferred:
- **Per-tenant CRL flush endpoint** for sub-60s revocation latency — deferred to FR-AUTH-111 (slice 2).
- **Reserved-role assignment endpoints** — root-admin assignment via cyberos-ten bootstrap CLI (FR-TEN-001); auditor/regulator/billing-system via dedicated intake flow; client-portal-user via PORTAL JIT provisioning (FR-PORTAL-003). Each of those FRs documents its own reserved-role grant path.
- **Argon2id migration** — FR-AUTH-114 (slice 4); orthogonal to RBAC.
- **Scope-grant TTL enforcement** — slice 2; current spec allows `expires_at` but does not enforce expiry cleanup (relies on the `has_permission` checker to ignore expired rows).
- **Role-impact diff in PR review** — a tool that surfaces "this migration changes which roles can do X" — slice 3; would be a nice-to-have for ADR reviewers.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| `Role::ALL` length drifts from 22 (developer adds variant without updating ALL) | `rbac_catalogue_test::all_22_roles_present` | CI fails before merge | Add the variant to `ALL` array |
| Two roles share the same `as_str()` | `rbac_catalogue_test::all_22_roles_present` (HashSet len check) | CI fails | Fix duplicate string in `as_str()` match |
| Migration 0005 row count drifts | `rbac_permission_matrix_test` (asserts seeded row count == ADR-101 expectation) | CI fails | Either fix migration OR update ADR + test together |
| Matrix loader hits malformed `role_permissions` row (NULL role) | `load_matrix` returns Err; service stays on prior snapshot; `auth_rbac_matrix_refresh_total{outcome=db_unreachable}` increments | Service serves stale matrix; OBS alarm at 5min sustained | Operator inspects DB; rolls back bad migration |
| RoleMatrix refresher panics | Tokio task supervisor restarts; OBS alarm sev-2 on panic count > 0 | Brief refresh gap (60s); checks still served from stale snapshot | Investigate panic via tracing; ship patch |
| JWT contains `roles: ["super-admin"]` (tampered token) | `Claims::roles()` parse fails at verifier | 401 `unknown_role_in_claim`; OBS sev-3 if pattern repeats | Token is invalid; user re-auths |
| JWT missing `rbac_v` after grace window | Verifier check against clock | 401 `rbac_version_required` | User re-auths and gets a fresh token with `rbac_v` |
| JWT carries `rbac_v: 1` but live version is 4 | Verifier compares; > 2 stale → reject | 401 `rbac_version_stale` | Refresh-token flow issues fresh access token |
| POST role with body `{"role":"super-admin"}` | Handler parse | 400 `unknown_role` with `allowed` array of 22 | Caller picks valid role |
| POST role for reserved role (`root-admin`) | Handler `is_reserved()` check | 403 `reserved_role` | Caller routes to dedicated reserved-role endpoint |
| POST founder for subject without passkey | Handler `webauthn::has_factor` check | 409 `webauthn_required` | Enrol passkey first (FR-AUTH-105) |
| POST role duplicate (race-condition double-click) | Postgres PK violation (23505) | 409 `already_granted` (idempotent on subject_id+role PK) | No action; request is no-op |
| DELETE absent role | Handler returns 204 anyway | 204 NO CONTENT (REST idempotency) | No action |
| Audit row emission fails inside grant tx | Postgres transaction rolls back | 500 `audit_failed`; subject_roles INSERT also reverts | Operator inspects brain_writer health |
| Matrix `auth.has_role` SQL function evaluates wrong (RLS bypass risk) | Negative test `rls_audit_row_member_zero_rows` | Test fails | Fix function or policy |
| Two services see different matrix versions briefly | Expected — 60s eventual consistency | `rbac_v` claim catches stale token verification | Refresher catches up; OTel `auth_rbac_catalogue_version` gauge shows version per service |
| Migration adds 23rd role with no ADR file | `rbac_adr_gate_test` CI gate | Build fails | Either write ADR or revert migration |
| ADR file referenced but doesn't exist | `rbac_adr_gate_test` checks path exists | Build fails | Create ADR or fix path |
| Catalogue version bump trigger fails silently | Test `rbac_catalogue_version_bump` asserts increment | CI fails before merge | Investigate trigger function body |
| Scope-grant points at deleted resource (`resource_id` no longer exists) | Periodic FK-check job (out of scope here) | Stale grant is no-op (resource not found at access time) | Future cleanup job in slice 2 |
| Subject holds 100+ roles (pathological) | Test `rbac_subject_role_count_gauge` warns at > 10 | OBS sev-4; not blocking | Review with tenant-admin; revoke unnecessary |
| RLS policy on `subjects` denies tenant-admin (regression) | `rbac_rls_integration_test` | Test fails | Fix policy USING clause |
| Bcrypt path changed (FR-AUTH-114 lands) and role gate breaks for founder | Test `rbac_founder_webauthn_gate_test` after bcrypt swap | CI fails | Coordinate FR-AUTH-114 + FR-AUTH-101 changes |
| 60s refresher leaks task on shutdown | Service shutdown handler cancels token | Tokio drop drains | None (defensive) |
| In-memory matrix grows unbounded (large tenant) | Matrix size = roles × resources × actions = 4400 rows max | Bounded by closed enums | None — bound by design |
| Subject's role-cache out of sync briefly after grant | Expected 60s window; OTel `auth_rbac_subject_role_count` gauge | Caller waits one refresh cycle | Acceptable per DEC-126 |
| Concurrent grant + revoke on same (subject_id, role) | Postgres serialisation | Either grant-then-revoke (clean) OR revoke-on-nothing (204) | None — serialisable transactions handle it |
| Audit row chain head drift across services | Per-row `prev_chain` validates at brain_writer | Bridge rejects out-of-order rows | Operator runs `cyberos doctor --repair` |
| Test fixtures call `Role::Other(s)` | Compile error (variant doesn't exist) | Build fails | Fix test to use valid Role |
| RBAC service started without role-catalogue table | Migration not applied | Service refuses to start (config validation) | Apply migrations |

---

## §11 — Implementation notes

- **The closed enum is the design**: introducing `Role::Custom(String)` or `Resource::Other(String)` would silently kill the catalogue's value. Any temptation to add it should route through ADR-101 instead — the answer should be "no" until ABAC is unavoidable, which the design assertion says it never is.
- **40 resources / 5 actions / ~280 matrix rows**: the row count is bounded by `roles × resources × actions = 4400`. Most matrix cells are denied (no row). The seeded set covers documented access patterns from website docs §2.6 + ADR-101 §3.
- **`arc_swap` for the matrix snapshot**: lock-free read path; writers swap a fresh `Arc<RoleMatrix>` after refresh. Readers see a consistent snapshot for the duration of their request. The 60-second swap is the only mutation; serialised through the tokio task.
- **`sha2::Sha256` for matrix hash**: deterministic over a sorted query (`ORDER BY role, resource, action`); hash unchanged → skip swap allocation. Avoids 4 KB allocations per refresh when nothing changed (typical case).
- **`auth.has_role()` SQL function reads session GUC**: the JWT middleware sets `auth.roles` GUC on connection acquisition via `SET LOCAL auth.roles = $1`. RLS policies invoke `auth.has_role('tenant-admin')`. The GUC is per-transaction; never leaks across connections.
- **`rbac_v` design rationale**: comparing token-issued version vs live version is a soft-reject (2-version tolerance) rather than a hard match. Strict-match would force re-auth every time any role's matrix tweaked; tolerance preserves UX while catching meaningful drift.
- **Founder-WebAuthn check is intrinsic to the role**: not "configured as required" — preventing operator from accidentally turning the gate off via env var. The cost is one extra match arm in `requires_webauthn()`; the safety is "this is the kind of mistake that won't happen."
- **Reserved-role routing**: standard endpoint refuses; dedicated endpoints (out of scope) handle reserved-role assignment with their own elevated-privilege gates. Until those FRs ship, reserved roles can only be assigned via direct SQL (operator action); the BRAIN audit row carries `granted_by` to attribute the action.
- **30-day grace window**: longer than 7-day refresh-token max (FR-AUTH-004) plus 23-day safety margin. After grace, missing-claim tokens are rejected and the rejection metric tells ops whether to push a re-auth notice.
- **Matrix-version mismatch tolerance** (2 versions): tolerates one missed refresh cycle (60s × 2 minutes) on a single auth-server instance during catalogue change; rejects tokens issued 3+ versions ago (typically > 1 day stale).
- **ADR validator scope**: only checks migrations that *modify* `roles` or `role_permissions` tables; pure DDL on unrelated tables doesn't require ADR. Detection regex: `(INSERT|UPDATE|DELETE).*(roles|role_permissions)`. Bypasses include schema-only changes (column comments, etc.) — see `services/auth/src/rbac/adr.rs` heuristic.
- **`subject_roles` PK on `(subject_id, role)` not `(tenant_id, subject_id, role)`**: subject_id is globally unique (UUIDv4); same subject_id appearing in two tenants would be a deeper invariant violation (subject_id is tenant-scoped in subjects table). The (subject_id, role) PK is sufficient and avoids the redundant tenant_id column in the PK.
- **Scope-grant intersection rule**: role check passes OR scope-grant covers. Never AND. Rationale: scope-grants are *narrowing for restricted subjects* (e.g. external auditor) and *expanding for low-role subjects* (e.g. tenant-member with one-off invoice access). Neither pattern requires AND semantics.
- **The matrix migration is data, not schema**: future role additions are pure INSERT migrations + ADR-NNN.md. Schema changes (adding a resource enum variant) are rare and ADR-gated.
- **`auth_rbac_subject_role_count` gauge**: high cardinality on tenant_id label; aggregate by tenant_id only (no subject_id label). The gauge is for capacity planning, not per-subject debugging.
- **PostgreSQL `current_setting('auth.roles', true)` second arg**: `true` means "return NULL if not set" instead of erroring. The SQL function handles NULL with an early FALSE return — preventing accidental role-check passes from un-initialised connections.
- **JWT claim ordering**: `roles` array order is insertion order (DB returns sorted by `(subject_id, role)` ASC). Verifiers MUST NOT depend on ordering — they should treat `roles` as a set. The `Claims::roles()` helper returns `Vec<Role>` for ergonomics but downstream uses `.contains()`.
- **ADR-101 itself is shipped in this FR**: per AUTHORING.md the spec must be self-contained; the ADR is part of the spec's deliverables, not a separately negotiable artifact.
- **CHANGELOG note**: this FR closes the 17-role gap referenced in `CHANGELOG.md` line 338 ("the remaining 17 land across slices 3–5"); after this FR, all 22 are seeded — the slice number per role only documents intent for SCOPE-CREEP audits, not gating.

---

*End of FR-AUTH-101.*
