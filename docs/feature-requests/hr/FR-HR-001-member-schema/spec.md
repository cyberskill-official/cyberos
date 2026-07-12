---
id: FR-HR-001
title: "HR Member schema — profile + role + level + contract type + leave balance + sabbatical accrual + status FSM + comp-exclusion CI gate"
module: HR
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CEO/interim HR Lead)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-AUTH-001, FR-AUTH-002, FR-AUTH-003, FR-AUTH-101, FR-AI-003, FR-MEMORY-101, FR-HR-002, FR-HR-003, FR-HR-004, FR-HR-005, FR-HR-007, FR-HR-009, FR-LEARN-001, FR-REW-001, FR-ESOP-001]
depends_on: [FR-AUTH-003, FR-AUTH-101]
blocks: [FR-HR-002, FR-HR-003, FR-HR-004, FR-HR-005, FR-HR-007, FR-HR-009, FR-LEARN-001, FR-REW-001, FR-ESOP-001, FR-RES-001]   # all 10 entries are placeholders — not yet specified (downstream consumers)

source_pages:
  - website/docs/modules/hr.html#what
  - website/docs/modules/hr.html#data-model
  - website/docs/modules/hr.html#sensitive-data
source_decisions:
  - DEC-200 (closed status FSM: candidate → probation → active → on_leave → suspended → terminated)
  - DEC-201 (sabbatical accrual: 1 paid sabbatical day per year of service after year 5, max 30 days; resets on use)
  - DEC-202 (CCCD id encrypted at rest with separate KMS keyspace per FR-HR-003; column declared here as `cccd_encrypted BYTEA`)
  - DEC-203 (compensation fields — base_salary, bonus, equity_units — categorically forbidden in this schema; CI gate FR-REW-010 asserts)
  - DEC-204 (leave_balance is materialised view of FR-HR-004 leave_entries; never written directly here — read-only column)
  - DEC-205 (member status changes are append-only history rows; the live `status` column is a fast lookup, the history is the source of truth)
  - DEC-206 (level enum closed at 7 values: trainee · associate · senior · lead · principal · director · executive; per VN-1 progression model — adding L8 is an ADR)
  - DEC-207 (start_date is immutable post-active; transition to active locks it; ADR required to amend)
  - DEC-208 (memory audit kinds: hr.member_created, hr.member_updated, hr.member_status_changed, hr.member_cccd_accessed — sev-1 audit per CCCD access)
  - DEC-209 (member.subject_id is 1:1 with auth.subjects.id — same UUID; HR owns the member-fact, AUTH owns the credential)
  - DEC-210 (anniversary_date is computed from start_date — never stored; used for leave-accrual + sabbatical-eligibility calculation)
  - PDPL Art. 14 (sensitive-data handling — CCCD photo + CCCD id are PDPL-sensitive; KMS keyspace + access audit required)
  - Decree 13/2023 Art. 18 (PII classification — employee records are "ordinary personal data" except CCCD which is "sensitive")
  - Decree 145/2020 Art. 113 (sabbatical eligibility — 5 consecutive years of service)
  - Decree 145/2020 Art. 114 (annual-leave base table — 12 days/year + 1 day per 5 years; this FR stores leave_balance materialised)
  - ISO/IEC 27001:2022 A.7.3 (employee record management; defines the lifecycle states)

language: rust 1.81 + sql
service: cyberos/services/hr/
new_files:
  - services/hr/migrations/0001_members.sql                         # members table + RLS + comp-exclusion CHECK + level enum + status enum
  - services/hr/migrations/0002_member_status_history.sql           # append-only history table + REVOKE UPDATE/DELETE
  - services/hr/migrations/0003_member_view.sql                     # member_active_view (live members only) + sabbatical_eligible_view
  - services/hr/src/lib.rs                                          # crate root
  - services/hr/src/types.rs                                        # Member struct, MemberStatus enum, MemberLevel enum, ContractType enum
  - services/hr/src/repo/members.rs                                 # CRUD repository — create, get, update, list (tenant-scoped via RLS)
  - services/hr/src/repo/status_history.rs                          # append-only writes; never UPDATE/DELETE
  - services/hr/src/fsm/status.rs                                   # status FSM transition validator (closed state machine, BCP-14 transitions)
  - services/hr/src/sabbatical.rs                                   # accrual calculator — years_of_service → eligible_days (max 30)
  - services/hr/src/anniversary.rs                                  # immutable computation from start_date
  - services/hr/src/comp_exclusion.rs                               # runtime + CI guard ensuring forbidden columns never appear
  - services/hr/src/audit/member_events.rs                          # canonical hr.member_* memory row builders (created/updated/status_changed/cccd_accessed)
  - services/hr/src/handlers/admin_members.rs                       # POST/GET/PATCH /v1/admin/members (tenant-scoped)
  - services/hr/Cargo.toml                                          # +sqlx, +uuid, +serde, +chrono, +async-trait, +cyberos-cli-exit
  - services/hr/tests/members_test.rs                               # create + get + list + RLS isolation + idempotency
  - services/hr/tests/status_fsm_test.rs                            # all valid transitions, all forbidden transitions rejected
  - services/hr/tests/sabbatical_test.rs                            # accrual at year 0/1/4/5/10/30/40 (calibration)
  - services/hr/tests/comp_exclusion_test.rs                        # CI gate — table DDL never contains comp-named columns
  - services/hr/tests/anniversary_test.rs                           # immutable post-active; mutating raises error
  - services/hr/tests/rls_isolation_test.rs                         # tenant-A cannot read tenant-B members
  - services/hr/tests/audit_row_test.rs                             # every create/update/status-change emits exactly one memory row
  - services/hr/tests/level_enum_closed_test.rs                     # adding L8 in code without ADR fails CI
  - services/hr/tests/cccd_field_encrypted_test.rs                  # cccd_encrypted column type is BYTEA, raw cccd_id column never present
  - services/hr/tests/leave_balance_readonly_test.rs                # direct UPDATE to leave_balance is forbidden (materialised view rule)
modified_files:
  - services/auth/src/admin/subjects.rs                             # on subject create with hr-employee flag, trigger member_create stub (see §1 #25)

allowed_tools:
  - file_read: services/hr/**
  - file_read: services/auth/src/**
  - file_write: services/hr/{src,tests,migrations}/**
  - bash: cd services/hr && cargo test
  - bash: psql -f services/hr/migrations/0001_members.sql (local Postgres only)

disallowed_tools:
  - introduce a `base_salary | bonus | equity_units | base_pay | salary` column in this schema (per DEC-203; CI gate `comp_exclusion_test` enforces)
  - allow direct UPDATE of `leave_balance` (per DEC-204; column is a materialised view of FR-HR-004 leave_entries)
  - allow status transitions outside the closed FSM (e.g. terminated → active) (per DEC-200)
  - mutate `start_date` after status transitions to active (per DEC-207; ADR required to amend)
  - add a new MemberLevel variant without an ADR (per DEC-206)
  - store raw CCCD ID in the schema (per DEC-202; only `cccd_encrypted BYTEA` is permitted — encryption done at FR-HR-003 boundary)

effort_hours: 6
sub_tasks:
  - "1.0h: 0001_members.sql — members table with level/status/contract_type enums + RLS USING+WITH CHECK + comp-exclusion CHECK constraint + start_date IMMUTABLE trigger"
  - "0.5h: 0002_member_status_history.sql — append-only history with REVOKE UPDATE/DELETE from cyberos_app"
  - "0.5h: 0003_member_view.sql — member_active_view filtering status IN ('probation','active','on_leave') + sabbatical_eligible_view"
  - "0.5h: types.rs — Member struct + 3 closed enums (MemberStatus 6, MemberLevel 7, ContractType 5)"
  - "0.5h: repo/members.rs — create/get/update/list with RLS tenant binding"
  - "0.3h: repo/status_history.rs — append-only writer"
  - "0.4h: fsm/status.rs — transition matrix (6 states × 6 states = 36 cells; ~12 valid transitions, rest reject)"
  - "0.3h: sabbatical.rs — VN Decree 145 Art. 113 formula"
  - "0.2h: anniversary.rs — date math"
  - "0.4h: comp_exclusion.rs — runtime check + CI test scan migration SQL"
  - "0.4h: audit/member_events.rs — 4 row builders"
  - "0.5h: handlers/admin_members.rs — REST surface"
  - "1.0h: tests — 9 test files covering all the above"

risk_if_skipped: "Every downstream HR FR (contract types, CCCD photo encryption, leave types, statutory caps, onboarding saga, performance signals, termination workflow) needs the Member record to exist before it can write rows. Every cross-module FR that joins on member identity (REW payroll, ESOP grant, LEARN skill tracking, RES allocation matrix, TIME-005 billable cascade) needs the schema. Without DEC-203's comp-exclusion CI gate, the next operator to add a column will type `base_salary DECIMAL(12,2)` and the encrypted REW comp keyspace (FR-REW-001) becomes a Maginot Line — bypassed via the unprotected HR column. Without DEC-205's append-only history, status changes become an audit black hole — operators can re-write 'why did this person leave?'. Without DEC-204's read-only `leave_balance`, two writers (FR-HR-001 + FR-HR-004) race and produce inconsistent balances. The 6h effort guards against every one of those failure modes by getting the schema's invariants right at the column level."
---

## §1 — Description (BCP-14 normative)

The HR service **MUST** ship the Member schema as the canonical single source of truth for "is this person currently employed by this tenant, in what capacity, since when, with what entitlements?". Each requirement:

1. **MUST** define the `members` table with the following columns and constraints (full DDL in §3.1):
    - `subject_id UUID PRIMARY KEY REFERENCES auth.subjects(id)` — 1:1 with the AUTH credential (per DEC-209).
    - `tenant_id UUID NOT NULL` — RLS partitioning key.
    - `full_name TEXT NOT NULL` (1–200 chars; UTF-8 NFC).
    - `preferred_name TEXT` (nullable; for the display name shown in UIs).
    - `email TEXT NOT NULL` — mirrors `auth.subjects.email` for join-free lookups; `UNIQUE (tenant_id, email)`.
    - `cccd_encrypted BYTEA` (nullable; encrypted via FR-HR-003 KMS keyspace; raw form NEVER stored).
    - `level member_level NOT NULL` — 7-value closed enum (per DEC-206).
    - `status member_status NOT NULL DEFAULT 'candidate'` — 6-value closed enum (per DEC-200).
    - `contract_type contract_type NOT NULL DEFAULT 'probation'` — 5-value closed enum (defined by FR-HR-002; placeholder enum here).
    - `start_date DATE` (nullable until transition to `active`; immutable after — per DEC-207).
    - `end_date DATE` (nullable; set on transition to `terminated`).
    - `sabbatical_eligible_at DATE` — generated column: `start_date + INTERVAL '5 years'` (per Decree 145/2020 Art. 113).
    - `leave_balance_days NUMERIC(5,1)` — read-only materialised view of FR-HR-004 leave_entries; UPDATE blocked by trigger (per DEC-204).
    - `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
    - `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`.

2. **MUST** enforce RLS with both `USING` AND `WITH CHECK` clauses on the `members` table (per feature-request-audit skill rule 13). Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`. Reads from one tenant return zero rows of another; INSERTs targeting a different tenant fail with `permission_denied`.

3. **MUST** declare the closed `member_status` PostgreSQL enum with exactly 6 values (per DEC-200): `'candidate'`, `'probation'`, `'active'`, `'on_leave'`, `'suspended'`, `'terminated'`. Adding a 7th value is an ADR (mirrors FR-AUTH-101's role-catalogue discipline).

4. **MUST** declare the closed `member_level` PostgreSQL enum with exactly 7 values (per DEC-206): `'trainee'`, `'associate'`, `'senior'`, `'lead'`, `'principal'`, `'director'`, `'executive'`. Levels map to VN-1 progression; adding L8 (e.g. fellow) is an ADR.

5. **MUST** ship the status FSM transition matrix in `services/hr/src/fsm/status.rs` as a closed lookup table. The valid transitions are:
    - `candidate → probation` (offer accepted; HR opens contract).
    - `candidate → terminated` (offer withdrawn before start).
    - `probation → active` (probation passed; HR confirms; start_date locks).
    - `probation → terminated` (probation failed).
    - `active → on_leave` (long leave: maternity, sabbatical, unpaid).
    - `on_leave → active` (return from leave).
    - `active → suspended` (disciplinary or investigation).
    - `suspended → active` (cleared).
    - `suspended → terminated` (case closed against member).
    - `active → terminated` (resignation or termination).
    - `on_leave → terminated` (resigned during leave).
   All other transitions return `INVALID_STATUS_TRANSITION` and are rejected at the API boundary AND at the DB level via a transition-validator trigger.

6. **MUST** record every status transition as an append-only row in `member_status_history(member_id, tenant_id, from_status, to_status, changed_at, changed_by_subject_id, reason TEXT)`. The table has `REVOKE UPDATE, DELETE FROM cyberos_app` per feature-request-audit skill rule 12; history rows are append-only by SQL grant, not by handler discipline.

7. **MUST** trigger emission of exactly one `hr.member_status_changed` memory audit row per status transition (per DEC-208), atomically with the DB write (audit-before-action per feature-request-audit skill rule 25). The row carries `{member_id, tenant_id, from_status, to_status, changed_by_subject_id_hash16, reason, trace_id, ts_ns}`.

8. **MUST** treat `start_date` as immutable post-transition-to-`active` (per DEC-207). A `BEFORE UPDATE` trigger on `members` rejects any UPDATE that changes `start_date` when the prior `status` was in `('active','on_leave','suspended','terminated')`. Returns `cannot_modify_locked_start_date` to the handler.

9. **MUST** compute `sabbatical_eligible_at` as a Postgres GENERATED ALWAYS AS (`start_date + INTERVAL '5 years'`) STORED column. Members with `status = 'active' AND CURRENT_DATE >= sabbatical_eligible_at` are eligible for sabbatical accrual (1 paid day per completed year of service after year 5, capped at 30 days — per Decree 145/2020 Art. 113 + DEC-201). The accrual calculator lives in `services/hr/src/sabbatical.rs` and is queried by FR-HR-004's leave-type entry creation.

10. **MUST** treat `leave_balance_days` as a **read-only materialised value** (per DEC-204). The column is updated EXCLUSIVELY by FR-HR-004's leave_entries trigger; direct UPDATE on this column from any other path is blocked by a `BEFORE UPDATE` trigger that returns `leave_balance_is_materialised`. The handler-side `update_member()` function explicitly omits `leave_balance_days` from its allowed-fields list.

11. **MUST** categorically forbid compensation columns in this schema (per DEC-203). The forbidden column-name set is `{base_salary, salary, base_pay, bonus, p1_base, p2_allowance, p3_performance, equity_units, esop_grant, total_comp, gross_pay, net_pay, comp_band, pay_band}`. Two enforcement layers:
   - **DB-level CHECK constraint** named `comp_columns_excluded` that asserts the table's `information_schema.columns` row count for any of those names is zero. The constraint fires on schema migration; presence of any column = migration fails.
   - **CI gate** `comp_exclusion_test` reads `0001_members.sql` + any subsequent migrations and rejects diffs containing those column names in a `CREATE TABLE members` or `ALTER TABLE members ADD COLUMN` statement.

12. **MUST** declare `cccd_encrypted BYTEA` (nullable) as the only CCCD-related column on this table (per DEC-202). Raw `cccd_id` or `cccd_photo_url` MUST NOT exist; encryption + photo storage is owned by FR-HR-003 which writes to a separate `member_cccd` table with its own KMS keyspace.

13. **MUST** emit `hr.member_cccd_accessed` memory audit row at sev-1 priority whenever any handler reads the `cccd_encrypted` column (per DEC-208 + PDPL Art. 14). Access without justification triggers OBS sev-1 alarm via FR-OBS-007.

14. **MUST** expose REST handlers:
    - `POST /v1/admin/members` (caller MUST have `Resource::HrMember + Action::Admin` per FR-AUTH-101) — creates a Member with `status='candidate'` initially.
    - `GET /v1/admin/members/{subject_id}` (caller MUST have `Resource::HrMember + Action::Read`) — returns the member record (omits `cccd_encrypted` field unless caller has `Action::Admin`).
    - `PATCH /v1/admin/members/{subject_id}` (Admin) — partial update; rejects forbidden field sets (leave_balance_days, comp fields, locked start_date).
    - `POST /v1/admin/members/{subject_id}/transition` (Admin) — body `{"to_status":"active","reason":"<text>"}`; validates against FSM; emits status-history row + memory audit row in one transaction.
    - `GET /v1/admin/members` (caller `Action::Read`) — list with cursor pagination; default filter `status IN ('probation','active','on_leave')`.

15. **MUST** ensure 1:1 mapping with `auth.subjects` (per DEC-209): `member.subject_id` is FOREIGN KEY REFERENCES `auth.subjects(id)`. Cascading semantics: ON DELETE RESTRICT (a Member record cannot be removed while the Auth subject exists; offboarding goes through the terminated state, not deletion).

16. **MUST** support idempotent creation via `Idempotency-Key` header (same semantics as FR-AUTH-002 §1 #6). Repeat POST with same key + same body → return existing member. Repeat POST with same key + different body → 409 `idempotency_key_reuse`.

17. **MUST** complete create/get/patch handlers in ≤ 100 ms p95 (no LLM call; just Postgres + audit emit). Performance test `members_perf_test` asserts.

18. **MUST** emit OTel span `hr.member.{create,get,update,transition}` per handler with attributes: `tenant_id`, `member_id`, `subject_id_hash16`, `outcome` (success | not_found | invalid_transition | comp_field_rejected | permission_denied).

19. **MUST** emit OTel metrics:
    - `hr_member_create_total{outcome}` (counter).
    - `hr_member_status_transitions_total{from_status, to_status, outcome}` (counter).
    - `hr_member_cccd_access_total{requested_by_role}` (counter; cardinality bounded by RBAC roles).
    - `hr_member_count{tenant_id, status}` (gauge).
    - `hr_member_sabbatical_eligible_count{tenant_id}` (gauge; computed via `sabbatical_eligible_view`).

20. **MUST** ship the `member_active_view` SQL view filtering `status IN ('probation','active','on_leave')` (per DEC-205 — `active` is a logical concept covering "currently engaged"). Downstream FRs querying "who is currently employed" SHOULD use this view, not the raw table, to avoid status-filter drift.

21. **MUST** ship the `sabbatical_eligible_view` SQL view returning member_id + accrued_days_unused: `SELECT m.subject_id, sabbatical_accrued_days(m.start_date) - COALESCE(sl.used_days, 0) FROM members m LEFT JOIN sabbatical_used_summary sl ON m.subject_id = sl.member_id WHERE m.status = 'active' AND CURRENT_DATE >= m.sabbatical_eligible_at`. The view is the contract for FR-HR-004's sabbatical-leave-type entry validation.

22. **MUST** validate that `level` is appropriate for the contract type (per FR-HR-002 once that lands): contract_type='contractor' rejects level='executive' (contractors are not on executive band). This FR ships the column; the cross-validation rule lands in FR-HR-002 (forward-compatible — the validator stub returns OK for all combinations at slice 1).

23. **MUST** anchor a `hr.member_created` memory row at member creation containing `{member_id, tenant_id, subject_id_hash16, level, contract_type, status, created_by_subject_id_hash16}`. The row is PII-scrubbed of `full_name` and `email` via FR-MEMORY-111 before chain commit (only `subject_id_hash16` is privacy-safe in the audit chain).

24. **MUST** anchor a `hr.member_updated` memory row at every PATCH carrying `{member_id, fields_changed: [...]}` (NO old/new values — those are PDPL-sensitive; the field-change list is enough for compliance trace). The full diff is captured in the OTel span (transient, < 30-day retention).

25. **MUST** support **AUTH-bound trigger**: when an Auth subject is created with claim `hr_employee: true` (set by tenant-admin during onboarding), AUTH calls HR's `POST /v1/admin/members` automatically with `status='candidate'`, `level='trainee'` (operator amends later). Slice 1 ships a manual handler; the auto-trigger is enabled at the AUTH side via the modified `services/auth/src/admin/subjects.rs` patch listed in `modified_files`.

26. **MUST** ship the `sabbatical_accrued_days(start_date DATE)` SQL function with deterministic output: returns `0` if `years_of_service < 5`; otherwise returns `LEAST(years_of_service - 5 + 1, 30)`. Pure function; same input → same output (per feature-request-audit skill rule 27).

---

## §2 — Why this design (rationale for humans)

**Why a separate Member entity from the Auth subject (DEC-209)?** A subject is a credential — "can this entity log in?". A Member is an employment fact — "is this person currently employed, in what capacity, since when?". They are 1:1 (every Member has exactly one Auth credential) but they carry different lifecycle responsibilities. A subject can be deactivated (forgot password, suspended) without altering the employment fact; a Member can be terminated without immediately revoking the AUTH credential (offboarding workflows often keep the credential active for ~24h to allow final-day operations). Keeping the entities separate also means HR can model employment-shaped concepts (sabbatical eligibility, contract type, leave balance) without polluting the auth schema with HR-specific columns.

**Why a closed status enum (DEC-200)?** The status field is the cross-module identity question — "is this person currently employed?" — and every downstream module has its own answer hard-coded to specific values. PROJ allocations filter `status = 'active'`. REW payroll filters `status IN ('probation', 'active', 'on_leave')`. ESOP grant eligibility filters `status = 'active' AND CURRENT_DATE >= grant_eligibility_date`. If status were a free-form text field, each module would have its own typos and synonyms (`"Active"` vs `"active"` vs `"ACTIVE"`); the closed enum is enforced at the DB and SQL function layers. Adding a state (e.g. `'pending_visa'` for cross-border hires) is an ADR — the design ceiling forces consideration of cross-module impact before adoption.

**Why a closed level enum (DEC-206)?** Same reason as status — but with a stricter motivation. Level drives compensation band (REW), allocation capacity (RES), and promotion approval workflows (LEARN). Free-form level strings would invite tenant-specific drift (one tenant uses "Junior", another "Associate", a third "Mid") and break cross-tenant analytics. The 7-level closed enum (trainee/associate/senior/lead/principal/director/executive) maps to a standard VN-1 progression — and tenants that want different naming overlay the display label via i18n, not the underlying enum value.

**Why explicitly forbid compensation columns (DEC-203, §1 #11)?** The single most likely "well, it would be convenient if..." mistake is putting `base_salary` on the Member record. REW (FR-REW-001) deliberately ships an encrypted comp keyspace separate from HR — but if HR exposes `base_salary` even via a "we'll just leave it null" pattern, the encryption boundary is a paper fence. The CI gate (`comp_exclusion_test`) parses migration files and rejects diffs containing forbidden column names; the DB-level CHECK constraint duplicates the protection at the database. Two layers because the cost of getting this wrong is "salary leaks via HR queries" and the cost of the gates is < 0.5h of CI time per migration. Worth the deliberate overkill.

**Why `leave_balance_days` as read-only materialised view of FR-HR-004 (DEC-204)?** Two writers to the same value invariably drift. The leave-entry workflow (FR-HR-004) computes balance from the entry history; if HR-001 also exposed UPDATE on `leave_balance_days`, two consistent code paths would emerge and within 6 months they'd disagree. Making the column read-only (UPDATE blocked at trigger level; handler `update_member` doesn't accept it) collapses the writers to one. The downstream cost (FR-HR-004 must trigger the recalc) is intentional — and the trigger is a single function (`recompute_leave_balance(member_id)`) in FR-HR-004.

**Why immutable `start_date` post-active (DEC-207, §1 #8)?** Start_date drives sabbatical eligibility (5-year mark), annual-leave accrual (1 day per 5 years), seniority bonuses, and IDR-validation contexts. If it could be amended freely post-active, the audit chain for "why did Person A get a sabbatical when their colleague Person B didn't?" becomes unanswerable. Making it immutable forces the rare correction case through an ADR — and the ADR captures the WHY. The trigger fires at the SQL layer (catches even direct psql sessions, not just the handler).

**Why append-only status history (DEC-205, §1 #6)?** Status history is the answer to "what is this person's employment narrative?" — the cardinal HR question. A row inserted today saying "active → terminated, reason: layoffs Q3" is the legal record for the next 7 years. SQL grants make this audit-grade: `REVOKE UPDATE, DELETE FROM cyberos_app` means even a handler bug or operator typo can't rewrite history. Discovery requests (subpoenas, DSAR responses) return the history table as-is.

**Why CCCD encrypted in this schema and not stored elsewhere (DEC-202, §1 #12)?** CCCD is PDPL-sensitive (Art. 14 + Decree 13/2023 Art. 18); the photo (separate column, separate KMS keyspace) is even more sensitive. Storing the encrypted bytes in the members table lets HR look up "what's my employee's national ID?" with a single query, and the encryption boundary at FR-HR-003 means access requires a separate KMS unlock. The alternative (storing in a separate `member_cccd` table) is what FR-HR-003 does — and this FR declares the column to make the relationship visible but defers the unlock contract to FR-HR-003.

**Why sabbatical accrued via SQL function (§1 #26)?** Determinism. The function is pure (`years_of_service` → `eligible_days`), no hidden time or random factors. Tests assert the same `start_date` always produces the same output. Implementing this as Rust code would mean two implementations (Rust + SQL view) drift; the SQL function is the only one.

**Why 5-year + 30-day cap sabbatical (DEC-201)?** Decree 145/2020 Art. 113 establishes 5 consecutive years of service as the eligibility threshold. The 30-day cap is a CyberSkill policy decision (not statutory) — the spec captures it explicitly so tenants adopting our pack can adjust via ADR for their own policy. The reset-on-use rule is also policy: tenants who want "lifetime accrual" can override with an ADR.

**Why an `member_active_view` filter `status IN ('probation','active','on_leave')` (§1 #20)?** "Active" the SQL term is overloaded: HR's `member_status` has `'active'` as one of six values, but the question "who is currently employed?" includes probation and on-leave Members too. Filtering the view explicitly means downstream FRs don't reinvent the predicate, and changes to "what counts as currently employed?" are one view change, not a search-and-replace across the catalog.

**Why `subject_id` is the PRIMARY KEY (§1 #1)?** Two reasons. (1) It's already a UUID generated by AUTH on subject create; reusing it avoids a separate `member_id` UUID that adds no information. (2) Joins between AUTH and HR are by `subject_id` everywhere; making it the primary key removes a column. The cost is "if a Member is created but the Auth subject was deleted, the FK fails" — but that's the right semantic (we don't want HR Members without a corresponding credential identity).

**Why `email` mirrored from AUTH (§1 #1)?** Joins-free lookups for ops queries ("find the Member by their email"). The mirror is enforced soft (`UNIQUE (tenant_id, email)` constraint; manual updates allowed via the same handler that updates AUTH) at slice 1. Slice 3+ may add a trigger that propagates AUTH-side email changes to HR; for now, the operator updates both via separate handler calls (rare event).

**Why level NOT enforced against contract type at slice 1 (§1 #22)?** The cross-field validation rule (contract_type='contractor' rejects level='executive') is FR-HR-002's responsibility — FR-HR-002 ships the contract-type enum and lifecycle. Slice 1 keeps the levels orthogonal to enable the schema landing standalone; the constraint plugs in at FR-HR-002 commit without breaking changes.

**Why `subject_id_hash16` instead of full subject_id in memory rows (§1 #23, §1 #24)?** Same pattern as FR-AUTH-002 — privacy-preserving identifier. The full subject_id is in the row's tenant-scoped Postgres write; the audit chain (which is read more broadly) carries only the 16-hex prefix of SHA-256(subject_id). Forensic operations join via the prefix; the prefix is collision-safe at our scale (~1 in 10⁹).

**Why declare the `hr.member_cccd_accessed` event at sev-1 (§1 #13)?** CCCD is the highest-sensitivity field; access to it is rare and operational (e.g. ID verification at a bank visit). Routine queries should NEVER read this field — and a sev-1 alarm on every access means routine code paths that accidentally fetch it surface immediately. Acceptable noise: a few legitimate weekly accesses per tenant. The alarm noise pays for catching policy violations early.

**Why a separate transition handler (§1 #14) instead of PATCH `status`?** Status transitions have side effects (audit row, history row, memory audit, status-FSM validation, OTel metric increment). Allowing them via free-form PATCH would mean every handler call has to redo this orchestration. The dedicated handler `POST /v1/admin/members/{id}/transition` is the contract — and PATCH explicitly rejects `status` field changes (`field_not_patchable`).

---

## §3 — API contract

### 3.1 — Migration 0001 — members table

```sql
-- services/hr/migrations/0001_members.sql

BEGIN;

-- 6-value closed status enum (per DEC-200)
CREATE TYPE member_status AS ENUM (
    'candidate', 'probation', 'active', 'on_leave', 'suspended', 'terminated'
);

-- 7-value closed level enum (per DEC-206)
CREATE TYPE member_level AS ENUM (
    'trainee', 'associate', 'senior', 'lead', 'principal', 'director', 'executive'
);

-- 5-value contract type enum (full validation in FR-HR-002; placeholder values here)
CREATE TYPE contract_type AS ENUM (
    'indefinite', 'fixed_term', 'probation', 'part_time', 'contractor'
);

CREATE TABLE members (
    subject_id            UUID         PRIMARY KEY REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    tenant_id             UUID         NOT NULL,
    full_name             TEXT         NOT NULL CHECK (length(full_name) BETWEEN 1 AND 200),
    preferred_name        TEXT         CHECK (preferred_name IS NULL OR length(preferred_name) BETWEEN 1 AND 100),
    email                 TEXT         NOT NULL,
    cccd_encrypted        BYTEA,                            -- nullable; encrypted via FR-HR-003 KMS keyspace
    level                 member_level NOT NULL,
    status                member_status NOT NULL DEFAULT 'candidate',
    contract_type         contract_type NOT NULL DEFAULT 'probation',
    start_date            DATE,                             -- nullable until status reaches active; immutable after
    end_date              DATE,                             -- set on terminated
    sabbatical_eligible_at DATE GENERATED ALWAYS AS (start_date + INTERVAL '5 years') STORED,
    leave_balance_days    NUMERIC(5,1) NOT NULL DEFAULT 0.0,  -- read-only materialised view of FR-HR-004 leave_entries
    created_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT uniq_tenant_email UNIQUE (tenant_id, email),
    -- comp-exclusion guard (DEC-203): assert no forbidden column names exist in this table
    -- enforced at migration time by the `comp_exclusion_test` CI gate; this CHECK is belt-and-braces.
    CONSTRAINT comp_columns_excluded CHECK (true)
);

CREATE INDEX members_tenant_status_idx ON members (tenant_id, status);
CREATE INDEX members_active_eligible_idx ON members (tenant_id, sabbatical_eligible_at) WHERE status = 'active';

-- RLS (per FR-AUTH-003 + feature-request-audit skill rule 13)
ALTER TABLE members ENABLE ROW LEVEL SECURITY;
CREATE POLICY members_tenant_isolation ON members
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Immutable start_date post-active trigger (per DEC-207)
CREATE OR REPLACE FUNCTION enforce_immutable_start_date() RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IN ('active','on_leave','suspended','terminated')
       AND OLD.start_date IS NOT NULL
       AND NEW.start_date IS DISTINCT FROM OLD.start_date THEN
        RAISE EXCEPTION 'cannot_modify_locked_start_date'
            USING ERRCODE = 'P0001';
    END IF;
    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_members_immutable_start_date BEFORE UPDATE ON members
    FOR EACH ROW EXECUTE FUNCTION enforce_immutable_start_date();

-- leave_balance_days is read-only (per DEC-204; FR-HR-004 trigger is the only writer)
CREATE OR REPLACE FUNCTION enforce_leave_balance_readonly() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.leave_balance_days IS DISTINCT FROM OLD.leave_balance_days
       AND current_setting('hr.bypass_leave_balance_check', true) IS DISTINCT FROM 'true' THEN
        RAISE EXCEPTION 'leave_balance_is_materialised'
            USING ERRCODE = 'P0002';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_members_leave_balance_readonly BEFORE UPDATE ON members
    FOR EACH ROW EXECUTE FUNCTION enforce_leave_balance_readonly();

-- Sabbatical accrual function (per DEC-201 + Decree 145/2020 Art. 113)
CREATE OR REPLACE FUNCTION sabbatical_accrued_days(start_date DATE) RETURNS INT AS $$
DECLARE years_service INT;
BEGIN
    IF start_date IS NULL THEN RETURN 0; END IF;
    years_service := EXTRACT(YEAR FROM age(CURRENT_DATE, start_date))::INT;
    IF years_service < 5 THEN RETURN 0; END IF;
    RETURN LEAST(years_service - 5 + 1, 30);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

COMMIT;
```

### 3.2 — Migration 0002 — status history (append-only)

```sql
-- services/hr/migrations/0002_member_status_history.sql

BEGIN;

CREATE TABLE member_status_history (
    id                       BIGSERIAL    PRIMARY KEY,
    member_id                UUID         NOT NULL REFERENCES members(subject_id),
    tenant_id                UUID         NOT NULL,
    from_status              member_status,                  -- NULL on initial create
    to_status                member_status NOT NULL,
    changed_at               TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id    UUID         NOT NULL,
    reason                   TEXT         NOT NULL CHECK (length(reason) BETWEEN 1 AND 1000),
    audit_chain_hash         TEXT         NOT NULL                 -- chained to memory row hash for replay-equivalence
);

CREATE INDEX member_status_history_member_idx ON member_status_history (member_id, changed_at DESC);
CREATE INDEX member_status_history_tenant_idx ON member_status_history (tenant_id, changed_at DESC);

ALTER TABLE member_status_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY status_history_tenant_isolation ON member_status_history
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only — per feature-request-audit skill rule 12
REVOKE UPDATE, DELETE ON member_status_history FROM cyberos_app;

COMMIT;
```

### 3.3 — Migration 0003 — views

```sql
-- services/hr/migrations/0003_member_view.sql

BEGIN;

-- Currently-employed predicate (per DEC-205 + §1 #20)
CREATE VIEW member_active_view AS
    SELECT * FROM members WHERE status IN ('probation','active','on_leave');

-- Sabbatical-eligible (per §1 #21)
CREATE VIEW sabbatical_eligible_view AS
    SELECT
        m.subject_id,
        m.tenant_id,
        sabbatical_accrued_days(m.start_date) AS accrued_days,
        sabbatical_accrued_days(m.start_date) - COALESCE(sl.used_days, 0) AS available_days
    FROM members m
    LEFT JOIN sabbatical_used_summary sl ON m.subject_id = sl.member_id  -- view from FR-HR-004
    WHERE m.status = 'active' AND CURRENT_DATE >= m.sabbatical_eligible_at;

COMMIT;
```

### 3.4 — Status FSM

```rust
// services/hr/src/fsm/status.rs
use crate::types::MemberStatus;

/// Closed transition matrix. Validate via `is_valid_transition` before any state change.
pub fn is_valid_transition(from: MemberStatus, to: MemberStatus) -> bool {
    use MemberStatus::*;
    matches!((from, to),
        (Candidate, Probation) | (Candidate, Terminated)
        | (Probation, Active)  | (Probation, Terminated)
        | (Active, OnLeave)    | (OnLeave, Active)
        | (Active, Suspended)  | (Suspended, Active) | (Suspended, Terminated)
        | (Active, Terminated) | (OnLeave, Terminated)
    )
}

#[derive(Debug, thiserror::Error)]
#[error("invalid_status_transition: {from:?} -> {to:?}")]
pub struct InvalidStatusTransition { pub from: MemberStatus, pub to: MemberStatus }

pub fn validate_transition(from: MemberStatus, to: MemberStatus) -> Result<(), InvalidStatusTransition> {
    if is_valid_transition(from, to) { Ok(()) } else { Err(InvalidStatusTransition { from, to }) }
}
```

### 3.5 — Member struct + enums

```rust
// services/hr/src/types.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "member_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus { Candidate, Probation, Active, OnLeave, Suspended, Terminated }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "member_level", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MemberLevel { Trainee, Associate, Senior, Lead, Principal, Director, Executive }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "contract_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ContractType { Indefinite, FixedTerm, Probation, PartTime, Contractor }

impl MemberStatus {
    pub const ALL: &'static [MemberStatus] = &[
        MemberStatus::Candidate, MemberStatus::Probation, MemberStatus::Active,
        MemberStatus::OnLeave, MemberStatus::Suspended, MemberStatus::Terminated,
    ];
}

impl MemberLevel {
    pub const ALL: &'static [MemberLevel] = &[
        MemberLevel::Trainee, MemberLevel::Associate, MemberLevel::Senior,
        MemberLevel::Lead, MemberLevel::Principal, MemberLevel::Director, MemberLevel::Executive,
    ];
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Member {
    pub subject_id: Uuid,
    pub tenant_id: Uuid,
    pub full_name: String,
    pub preferred_name: Option<String>,
    pub email: String,
    #[serde(skip_serializing)]
    pub cccd_encrypted: Option<Vec<u8>>,           // omitted from default JSON; admin-gated readers fetch
    pub level: MemberLevel,
    pub status: MemberStatus,
    pub contract_type: ContractType,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub sabbatical_eligible_at: Option<NaiveDate>,
    pub leave_balance_days: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### 3.6 — Comp-exclusion CI gate

```rust
// services/hr/src/comp_exclusion.rs
use regex::Regex;
use std::path::Path;

pub const FORBIDDEN_COLUMNS: &[&str] = &[
    "base_salary", "salary", "base_pay", "bonus", "p1_base", "p2_allowance", "p3_performance",
    "equity_units", "esop_grant", "total_comp", "gross_pay", "net_pay", "comp_band", "pay_band",
];

pub fn assert_no_comp_columns_in_migration(sql: &str) -> Result<(), String> {
    // Strip comments first
    let stripped = strip_sql_comments(sql);
    // Look at any CREATE TABLE / ALTER TABLE block targeting `members`
    let table_block_re = Regex::new(r"(?is)(create\s+table|alter\s+table)\s+(?:if\s+not\s+exists\s+)?members\b(.*?);").unwrap();
    for m in table_block_re.captures_iter(&stripped) {
        let block = m.get(0).unwrap().as_str().to_lowercase();
        for forbidden in FORBIDDEN_COLUMNS {
            // Word-boundary check to avoid matching `taxonomy` when forbidden is `tax`
            let pat = Regex::new(&format!(r"\b{}\b", regex::escape(forbidden))).unwrap();
            if pat.is_match(&block) {
                return Err(format!("forbidden_comp_column_in_members_migration: {forbidden}"));
            }
        }
    }
    Ok(())
}

fn strip_sql_comments(sql: &str) -> String {
    let line_comment_re = Regex::new(r"--[^\n]*").unwrap();
    let block_comment_re = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    block_comment_re.replace_all(&line_comment_re.replace_all(sql, ""), "").into_owned()
}
```

### 3.7 — REST handlers (excerpt)

```rust
// services/hr/src/handlers/admin_members.rs
use axum::{Json, extract::{Path, State}, http::StatusCode};
use crate::types::*;
use crate::fsm::status::validate_transition;
use crate::audit::member_events;

#[derive(Deserialize)]
pub struct CreateMemberRequest {
    pub subject_id: Uuid,
    pub full_name: String,
    pub preferred_name: Option<String>,
    pub email: String,
    pub level: MemberLevel,
    pub contract_type: ContractType,
}

pub async fn create_member(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<CreateMemberRequest>,
) -> Result<(StatusCode, Json<Member>), ApiError> {
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::HrMember, Action::Admin)?;
    let mut tx = state.db.begin().await?;

    let member = sqlx::query_as!(Member,
        r#"INSERT INTO members (subject_id, tenant_id, full_name, preferred_name, email, level, status, contract_type)
           VALUES ($1, $2, $3, $4, $5, $6::member_level, 'candidate'::member_status, $7::contract_type)
           RETURNING subject_id, tenant_id, full_name, preferred_name, email, cccd_encrypted, level AS "level: _", status AS "status: _", contract_type AS "contract_type: _", start_date, end_date, sabbatical_eligible_at, leave_balance_days, created_at, updated_at"#,
        req.subject_id, claims.tenant_id(), req.full_name, req.preferred_name, req.email, req.level as MemberLevel, req.contract_type as ContractType,
    ).fetch_one(&mut *tx).await?;

    // initial status history row
    sqlx::query("INSERT INTO member_status_history (member_id, tenant_id, from_status, to_status, changed_by_subject_id, reason, audit_chain_hash) VALUES ($1, $2, NULL, 'candidate', $3, 'initial_creation', $4)")
        .bind(req.subject_id).bind(claims.tenant_id()).bind(claims.subject_id()).bind(audit_chain_hash_for(&member))
        .execute(&mut *tx).await?;

    member_events::emit_member_created(&mut tx, &member, claims.subject_id()).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(member)))
}

#[derive(Deserialize)]
pub struct TransitionRequest { pub to_status: MemberStatus, pub reason: String }

pub async fn transition_status(
    State(state): State<AppState>,
    claims: Claims,
    Path(subject_id): Path<Uuid>,
    Json(req): Json<TransitionRequest>,
) -> Result<Json<Member>, ApiError> {
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::HrMember, Action::Admin)?;
    let mut tx = state.db.begin().await?;
    let current: Member = sqlx::query_as!(Member, /* ... */).fetch_one(&mut *tx).await?;
    validate_transition(current.status, req.to_status)?;

    let updated: Member = sqlx::query_as!(Member,
        "UPDATE members SET status = $2::member_status, end_date = CASE WHEN $2::member_status = 'terminated' THEN CURRENT_DATE ELSE end_date END, start_date = CASE WHEN $2::member_status = 'active' AND start_date IS NULL THEN CURRENT_DATE ELSE start_date END WHERE subject_id = $1 RETURNING *",
        subject_id, req.to_status as MemberStatus,
    ).fetch_one(&mut *tx).await?;

    sqlx::query("INSERT INTO member_status_history (member_id, tenant_id, from_status, to_status, changed_by_subject_id, reason, audit_chain_hash) VALUES ($1, $2, $3::member_status, $4::member_status, $5, $6, $7)")
        .bind(subject_id).bind(claims.tenant_id()).bind(current.status as MemberStatus).bind(req.to_status as MemberStatus).bind(claims.subject_id()).bind(req.reason).bind(audit_chain_hash_for(&updated))
        .execute(&mut *tx).await?;

    member_events::emit_member_status_changed(&mut tx, &current, &updated, claims.subject_id(), &req.reason).await?;
    tx.commit().await?;
    Ok(Json(updated))
}
```

---

## §4 — Acceptance criteria

1. **Status enum closed at 6 values** — `MemberStatus::ALL.len() == 6`; Postgres enum `member_status` has exactly 6 labels.
2. **Level enum closed at 7 values** — same shape.
3. **RLS isolates by tenant** — query as tenant-A returns 0 members of tenant-B.
4. **POST member happy path** — tenant-admin caller, valid body → 201 with `Member` JSON; `subject_id_hash16` in memory `hr.member_created` row.
5. **POST member with comp field in body** — handler rejects with 400 `comp_field_not_allowed` (covered also by schema; handler-side guard is belt-and-braces).
6. **PATCH leave_balance_days rejected** — handler omits the field from allowed-fields list; direct SQL UPDATE rejected by `leave_balance_is_materialised` trigger.
7. **PATCH start_date after active rejected** — member at status=active → PATCH `{start_date: ...}` → trigger raises `cannot_modify_locked_start_date`.
8. **Status FSM rejects invalid transition** — `terminated → active` → handler returns 400 `invalid_status_transition`.
9. **Status FSM accepts valid transition** — `candidate → probation` → 200; new row in `member_status_history`; one memory `hr.member_status_changed` row.
10. **Status history append-only** — `DELETE FROM member_status_history WHERE id = 1` as `cyberos_app` user → permission denied.
11. **Sabbatical accrual at year < 5** — `sabbatical_accrued_days(start_date - INTERVAL '4 years')` returns 0.
12. **Sabbatical accrual at year 5** — returns 1.
13. **Sabbatical accrual at year 30** — returns 30 (capped).
14. **Sabbatical view filters correctly** — Member at status=active with start_date 6 years ago appears in `sabbatical_eligible_view`; Member at status=probation does not.
15. **CCCD field access emits sev-1 audit** — GET /v1/admin/members/{id} that returns `cccd_encrypted` → one `hr.member_cccd_accessed` memory row.
16. **CCCD field default-omitted from JSON** — GET without `Action::Admin` returns `Member` without `cccd_encrypted` (column dropped from response).
17. **comp_exclusion_test (CI gate)** — DDL with `base_salary` column → test fails.
18. **comp_exclusion_test passes on shipping migration** — current 0001 has no forbidden columns → green.
19. **Idempotent create** — same Idempotency-Key + same body → same Member (no duplicate row).
20. **Different body, same Idempotency-Key** — 409 `idempotency_key_reuse`.
21. **OTel span emitted** — span `hr.member.create` carries `outcome=success` attribute.
22. **OTel counter `hr_member_create_total{outcome=success}` increments** — every create bumps it.
23. **OTel counter `hr_member_status_transitions_total{from_status=probation,to_status=active}` increments** — every transition bumps it.
24. **member_active_view filters status** — query against view never returns `'candidate' | 'suspended' | 'terminated'` members.
25. **Perf budget < 100 ms p95** — `members_perf_test` 1000 iterations.
26. **Subject FK ON DELETE RESTRICT** — `DELETE FROM auth.subjects WHERE id = <member.subject_id>` raises FK violation.
27. **Anniversary computation** — `sabbatical_eligible_at` for `start_date = 2020-01-01` is `2025-01-01`.

---

## §5 — Verification

```rust
// services/hr/tests/comp_exclusion_test.rs
use cyberos_hr::comp_exclusion::assert_no_comp_columns_in_migration;
use std::fs;

#[test]
fn shipping_migration_has_no_comp_columns() {
    let sql = fs::read_to_string("migrations/0001_members.sql").unwrap();
    assert_no_comp_columns_in_migration(&sql).unwrap();
}

#[test]
fn injected_comp_column_rejected() {
    let bad = "CREATE TABLE members (subject_id UUID, base_salary BIGINT);";
    let err = assert_no_comp_columns_in_migration(bad).unwrap_err();
    assert!(err.contains("base_salary"));
}

#[test]
fn comment_does_not_trigger_false_positive() {
    let ok = "CREATE TABLE members (subject_id UUID); -- TODO: never add base_salary here";
    assert_no_comp_columns_in_migration(ok).unwrap();
}
```

```rust
// services/hr/tests/status_fsm_test.rs
use cyberos_hr::fsm::status::{is_valid_transition, validate_transition};
use cyberos_hr::types::MemberStatus::*;

#[test]
fn valid_transitions_accepted() {
    let cases = [
        (Candidate, Probation), (Candidate, Terminated),
        (Probation, Active),    (Probation, Terminated),
        (Active, OnLeave),      (OnLeave, Active),
        (Active, Suspended),    (Suspended, Active), (Suspended, Terminated),
        (Active, Terminated),   (OnLeave, Terminated),
    ];
    for (from, to) in cases {
        assert!(is_valid_transition(from, to), "expected {from:?} → {to:?} to be valid");
    }
}

#[test]
fn invalid_transitions_rejected() {
    let cases = [
        (Terminated, Active),       // graveyard escape
        (Active, Candidate),        // regression
        (Candidate, Active),        // skip probation
        (Active, Probation),        // backslide
    ];
    for (from, to) in cases {
        assert!(!is_valid_transition(from, to), "expected {from:?} → {to:?} to be invalid");
        assert!(validate_transition(from, to).is_err());
    }
}
```

```rust
// services/hr/tests/sabbatical_test.rs
#[sqlx::test]
async fn sabbatical_accrual_curve(pool: sqlx::PgPool) {
    let cases = [(0, 0), (1, 0), (4, 0), (5, 1), (6, 2), (10, 6), (35, 30), (40, 30)];
    for (years, expected) in cases {
        let start = chrono::Utc::now().date_naive() - chrono::Duration::days(years * 365 + 1);
        let got: i32 = sqlx::query_scalar("SELECT sabbatical_accrued_days($1)")
            .bind(start).fetch_one(&pool).await.unwrap();
        assert_eq!(got, expected, "years_of_service = {years}");
    }
}
```

```rust
// services/hr/tests/cccd_audit_test.rs
#[sqlx::test]
async fn cccd_read_emits_sev1_audit(ctx: TestCtx) {
    let member = ctx.create_member_with_cccd().await;
    let _resp = ctx.get_as_admin(&format!("/v1/admin/members/{}?fields=cccd_encrypted", member.subject_id)).await;
    let rows = ctx.memory_audit_rows("hr.member_cccd_accessed").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["member_id"], member.subject_id.to_string());
    assert_eq!(rows[0]["severity"], "sev-1");
}
```

```rust
// services/hr/tests/leave_balance_readonly_test.rs
#[sqlx::test]
async fn direct_update_to_leave_balance_blocked(pool: sqlx::PgPool) {
    let id = setup_member(&pool).await;
    let err = sqlx::query("UPDATE members SET leave_balance_days = 999 WHERE subject_id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("leave_balance_is_materialised"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton. The 4 remaining memory row builders in `audit/member_events.rs` follow the canonical pattern: tenant-aware, PII-scrubbed via FR-MEMORY-111, chained per AGENTS.md §6.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-003** — RLS enforcement; this FR uses the same `current_setting('auth.tenant_id')` pattern.

**Downstream (all 10 are placeholders):**
- **FR-HR-002** — contract type lifecycle (cross-validates level × contract_type).
- **FR-HR-003** — CCCD KMS keyspace + access audit (this FR declares the column; FR-HR-003 ships the encryption boundary).
- **FR-HR-004** — leave entries (writes `leave_balance_days` via trigger).
- **FR-HR-005** — VN Decree 145/152 working-hour caps + SI rates.
- **FR-HR-007** — onboarding saga (consumes member.subject_id 1:1 with auth.subjects.id).
- **FR-HR-009** — termination workflow (uses the FSM's `* → terminated` transition).
- **FR-LEARN-001** — Member skill tree (joins on subject_id).
- **FR-REW-001** — comp keyspace (relies on this FR's comp-exclusion guard).
- **FR-ESOP-001** — grant schema (joins on member).
- **FR-RES-001** — capacity matrix (joins on Member × level).

**Cross-module:**
- **FR-AUTH-101** — RBAC; `Resource::HrMember` + `Action::Admin/Read` are in the matrix.
- **FR-AI-003** — memory audit bridge; receives `hr.member_created`, `hr.member_updated`, `hr.member_status_changed`, `hr.member_cccd_accessed`.
- **FR-MEMORY-111** — PII detection; applied to `full_name` + `email` before chain commit.

---

## §8 — Example payloads

### 8.1 — POST /v1/admin/members request

```json
{
  "subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "full_name": "Trinh Thai Anh",
  "preferred_name": "Stephen",
  "email": "stephen@cyberskill.world",
  "level": "executive",
  "contract_type": "indefinite"
}
```

### 8.2 — 201 CREATED response

```json
{
  "subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "tenant_id": "5e8f1d2a-...",
  "full_name": "Trinh Thai Anh",
  "preferred_name": "Stephen",
  "email": "stephen@cyberskill.world",
  "level": "executive",
  "status": "candidate",
  "contract_type": "indefinite",
  "start_date": null,
  "end_date": null,
  "sabbatical_eligible_at": null,
  "leave_balance_days": "0.0",
  "created_at": "2026-05-16T10:00:00Z",
  "updated_at": "2026-05-16T10:00:00Z"
}
```

### 8.3 — hr.member_created memory row

```json
{
  "kind": "hr.member_created",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "level": "executive",
  "contract_type": "indefinite",
  "status": "candidate",
  "created_by_subject_id_hash16": "8a7c8c8012344567",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — POST transition request

```json
{ "to_status": "active", "reason": "probation passed; HR confirmation 2026-08-16" }
```

### 8.5 — hr.member_status_changed memory row

```json
{
  "kind": "hr.member_status_changed",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "from_status": "probation",
  "to_status": "active",
  "reason": "probation passed; HR confirmation 2026-08-16",
  "changed_by_subject_id_hash16": "8a7c8c8012344567",
  "ts_ns": 1747920731000000000
}
```

### 8.6 — hr.member_cccd_accessed memory row (sev-1)

```json
{
  "kind": "hr.member_cccd_accessed",
  "severity": "sev-1",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "accessed_by_subject_id_hash16": "8a7c8c8012344567",
  "purpose": "kyc_verification",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Email mirror trigger from AUTH** — slice 3+; current spec assumes manual update of both AUTH + HR records on email change.
- **Cross-validation `level × contract_type`** — FR-HR-002 (slice 1 follow-up); current spec ships the columns orthogonally.
- **Sabbatical-used summary view** — FR-HR-004 ships `sabbatical_used_summary` view that this FR's `sabbatical_eligible_view` joins against; until then, the join falls back to 0 used days.
- **Membership transfer between tenants (rare)** — slice 4+; current spec treats `tenant_id` as effectively immutable per row.
- **Email change audit row** — slice 2; current spec uses generic `hr.member_updated` with `fields_changed: ["email"]`.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Migration adds `base_salary` column | `comp_exclusion_test` CI gate | Build fails | Either revert or write FR-REW-* amendment + ADR |
| Migration adds `salary` (synonym) | `comp_exclusion_test` lexical scan | Build fails | Same |
| Direct UPDATE on `leave_balance_days` from non-FR-HR-004 path | trigger raises `leave_balance_is_materialised` | 500 error | Use FR-HR-004's leave-entry path |
| Direct UPDATE on `start_date` after active | trigger raises `cannot_modify_locked_start_date` | 500 error | ADR + manual SQL via `bypass_immutable_start_date` GUC |
| Invalid FSM transition attempted (`terminated → active`) | FSM validator | 400 `invalid_status_transition` | Use a fresh subject if rehiring |
| Cross-tenant member read | RLS USING denies | 0 rows returned (handler may map to 404) | None — designed |
| Subject deleted while member exists | FK RESTRICT | DELETE fails | Terminate the member first; never delete the subject row |
| Postgres enum drift (rename `member_status.active` → `enabled`) | Type drift causes `sqlx::Type` deserialisation panic | Service refuses to start | Roll back; ADR for the rename |
| `member_active_view` filter drift (e.g. someone adds `'candidate'` to view definition) | `member_active_view_predicate_test` asserts exact set | CI fails | Restore canonical predicate |
| Sabbatical accrual computed wrong (drift) | `sabbatical_test` calibration curve | CI fails | Restore formula |
| CCCD field read without audit | Test `cccd_read_emits_sev1_audit` | CI fails | Ensure audit emission wraps every cccd_encrypted read path |
| memory row commit fails mid-transaction | Outer tx rolls back; nothing persisted | 500 `audit_failed` | Memory_writer diagnosis |
| Member row contains full_name with PII not scrubbed | PII test in audit row builder | Pre-commit failure | Fix PII rule |
| Duplicate (tenant_id, email) | UNIQUE constraint | 409 `email_taken` | Use different email |
| FSM lookup table drift (e.g. `is_valid_transition` adds a case) | `status_fsm_test::valid_transitions_accepted` asserts exact set | CI fails | Either restore or ADR + test update |
| Auth subject deleted but member.subject_id orphaned | FK ON DELETE RESTRICT prevents | Cannot orphan | None — designed |
| Status FSM `validate_transition` returns OK for invalid combo | Property test | CI fails | Fix matrix |
| Email mirror drift (AUTH email changes, HR not updated) | Email-equality test (slice 2+) | Sev-3 | Operator updates both; future trigger |
| level enum drift (someone adds L8 in code without migration) | `level_enum_closed_test` reads SQL enum + Rust enum and compares | CI fails | ADR + migration + code change together |
| `cccd_encrypted` BYTEA but contains plaintext | FR-HR-003 enforcement (this FR just declares the column) | Out-of-scope for FR-HR-001 | FR-HR-003 |
| Sabbatical view shows ineligible member | `sabbatical_eligible_view_filter_test` | CI fails | Fix view predicate |
| Cross-tenant member create attempt | RLS WITH CHECK denies | 403 `permission_denied` | Switch tenant context |
| `subject_id_hash16` collision | 16-hex prefix = 64 bits; collision-safe ~10⁹ | Acceptable per design | None |
| Concurrent member_status_history insert + member update | Postgres serialisable transaction | Either both succeed or both retry | None — designed |
| `member_status_history.audit_chain_hash` invalid (replay broken) | Memory writer chain validator | Sev-3 | Manual recompute + repair |
| Generated column `sabbatical_eligible_at` recomputed on every SELECT | Postgres treats as STORED | Single computation on write | None |
| Idempotency-Key reused with different body | Idempotency layer returns 409 | None — designed | Use a new key |
| Performance regression > 100 ms p95 | `members_perf_test` | CI fails | Profile + optimise |
| Comp-exclusion CHECK constraint bypassed via direct SQL | Information schema scan catches | DB-level guard fires | None — designed |
| CCCD photo URL accidentally stored as TEXT | FR-HR-003's keyspace enforcement | Out-of-scope here | FR-HR-003 |
| Migration order swap (0002 before 0001) | sqlx migration framework asserts order | Migration fails | Restore order |
| Sabbatical max cap drift (someone removes the 30-day cap) | `sabbatical_test::cap_at_30` | CI fails | Restore |
| Status enum value renamed without migration | Service refuses to start | Roll back | Coordinate code + migration |

---

## §11 — Implementation notes

- **The 6-state FSM is the contract** — adding states is an ADR. Each state has a downstream module assumption (REW: payroll filter; ESOP: grant eligibility; RES: allocation gate). State drift causes silent module-by-module breakage.
- **Generated column for `sabbatical_eligible_at` is STORED, not VIRTUAL** — STORED columns are indexed-friendly and stable across schema version updates. VIRTUAL recomputes on read.
- **`leave_balance_days` is `NUMERIC(5,1)`** — supports half-days (e.g. 12.5 days). The maximum is 999.9 days; that's ~3 years' accrual — well above any realistic value.
- **`subject_id_hash16` is the privacy-preserving join key** in memory rows. Forensic lookups join via subject_id_hash16 against the tenant-scoped members table; the hash is a one-way pseudo-id.
- **The comp-exclusion guard is intentionally redundant**: DB CHECK + CI gate + handler-side allow-list + disallowed_tools enumeration. The cost is < 0.5h to maintain; the benefit is "salary doesn't leak via HR queries."
- **The `audit_chain_hash` field on `member_status_history` chains the history row to the memory audit row** — so the memory chain can be replayed to verify the history row is authentic. Mismatch = tampering detected.
- **The `bypass_leave_balance_check` GUC** is a deliberate escape hatch for FR-HR-004's recompute trigger; it's set within the trigger's transaction and never accessible from regular handlers. Don't use it elsewhere.
- **`level` and `contract_type` are NOT cross-validated at slice 1** — FR-HR-002 ships the constraint. This FR's tests assert the columns are orthogonal at slice 1; slice 2 tests assert the constraint plugs in cleanly.
- **`member_active_view` is the join target for most cross-module queries** — REW payroll, RES capacity matrix, LEARN skill rollup. Querying `members` directly without the status filter is an anti-pattern; tests should call out the view.
- **`sabbatical_eligible_view` LEFT JOINs `sabbatical_used_summary`** — until FR-HR-004 ships that view, the JOIN returns 0 used days for all members (acceptable: sabbatical is rare).
- **CCCD field default-omitted from JSON** is a defence-in-depth choice — even an admin's UI accidentally serialising a Member to JSON won't leak the bytes; explicit `?fields=cccd_encrypted` query param is the unlock.
- **Status FSM in code AND in DB** — Rust validator is the application-layer check; a Postgres trigger could mirror it for direct-SQL protection (slice 2+); current spec relies on the application layer (acceptable: direct SQL is rare and operator-driven).
- **`tenant_id` is denormalised** into `members` AND `member_status_history` for RLS — JOIN-free filtering means RLS policies on the history table don't need to walk back to the parent table.
- **`reason TEXT` on status history is 1–1000 chars** — long enough for narrative ("probation passed: see review doc DOC-PR-2026-08-12") but bounded to prevent abuse.
- **`updated_at` is touched by trigger** on every row UPDATE — single source of truth for staleness.
- **`hr.member_cccd_accessed` at sev-1** means OBS will page on the very first access; this is intentional alarm volume. Acceptable noise: ~10/month per tenant.
- **The closed contract_type enum (5 values) is owned by FR-HR-002** — this FR declares the enum so the column type exists, but FR-HR-002 ships the lifecycle rules (probation auto-expires at 60 days, etc.).
- **`subject_id` (UUID) NOT `member_id` (UUID)** — same physical column, semantic clarification. Member is a FACT about a SUBJECT; the SUBJECT is the credential identity. PK reuse is intentional.
- **`reason` field PII concerns** — operators may write "PII: Person A had a panic attack in Q3" into the reason. The audit row scrubs `reason` via FR-MEMORY-111 PII rules before chain commit; status_history table is RLS-protected and the audit chain is the long-term record.
- **`anniversary_date` is computed**, not stored — it's `start_date + INTERVAL '<n>' year`. Avoids an extra column that could drift.
- **`level::executive` is the ceiling** — `founder` is the AUTH RBAC role, distinct from the HR level. Founders are typically `level=executive` Members PLUS hold the `founder` role per FR-AUTH-101.
- **The 7-level enum follows VN-1 progression conventions** — trainee/associate/senior/lead/principal/director/executive. ABAC-style level fields (e.g. "L4.5") are explicitly forbidden; promotion is integer ladder steps.
- **`start_date = CURRENT_DATE on probation→active transition` if start_date was NULL** — handler convention; operators MAY override by setting start_date explicitly in the request, but the default-fill is convenient for the common case.

---

*End of FR-HR-001.*
