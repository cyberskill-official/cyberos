---
id: TASK-CRM-001
title: "CRM Account/Contact/Deal Postgres schema — closed entity primitives + custom pipelines + closed stage-template enum + RLS + deal status FSM"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: CRM
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-CRM-002, TASK-CRM-003, TASK-CRM-004, TASK-CRM-005, TASK-PROJ-005, TASK-HR-001]
depends_on: [TASK-AUTH-003, TASK-AUTH-101]
blocks: [TASK-CRM-002, TASK-CRM-003, TASK-CRM-004, TASK-CRM-005, TASK-CRM-006, TASK-CRM-007, TASK-CRM-009, TASK-EMAIL-006, TASK-RES-004]

source_pages:
  - website/docs/modules/crm.html#what
  - website/docs/modules/crm.html#data-model
  - website/docs/modules/crm.html#lifecycle
source_decisions:
  - DEC-340 (3 closed primitives: Account · Contact · Deal — adding a 4th primitive is an ADR)
  - DEC-341 (4 default pipeline shapes: sales · partner · inbound · outbound; tenants can create custom pipelines but must derive from one of the 4 shapes for the TASK-CRM-005 next-action skill to apply)
  - DEC-342 (pipeline stages are tenant-defined per pipeline; ordering is integer position; stages tagged with `is_won|is_lost|is_open` for win/loss/active classification)
  - "DEC-343 (closed `account_type` placeholder enum at slice 1: `unknown` — full VN-specific enum ships in TASK-CRM-003; this task declares the column with single value to keep migration forward-compatible)"
  - DEC-344 (closed `deal_status` enum at 4 values: open · won · lost · cancelled — independent from per-pipeline stage; `won` and `lost` are terminal states gated by a stage with matching `is_won`/`is_lost` flag)
  - DEC-345 (money fields stored as BIGINT minor units per task-audit skill rule 11; `currency CHAR(3)` ISO-4217 alongside; deal.amount_minor + deal.amount_currency)
  - DEC-346 (REVOKE UPDATE, DELETE on deal_status_history from cyberos_app — append-only enforced by SQL grant)
  - DEC-347 (deal status transitions FSM: open → won | open → lost | open → cancelled | won → cancelled (rare; refund) — no transitions out of lost; cancelled is terminal)
  - DEC-348 (Contact is a JOIN entity — many-to-many between contacts table and accounts via contact_account_membership; a contact may belong to multiple accounts; the contact_account_membership row carries the role (decision-maker, technical, billing, etc.))
  - DEC-349 (memory audit kinds: crm.account_created, crm.account_updated, crm.contact_created, crm.deal_created, crm.deal_stage_changed, crm.deal_won, crm.deal_lost, crm.deal_cancelled)
  - DEC-350 (Deal owner is `owner_subject_id` referring to auth.subjects; ownership transfer is handler-supported + emits memory row; orphaned deals (owner suspended) revert to tenant default-owner per per-tenant config out of scope here)
  - DEC-351 (probability is an INT 0-100 stored separately from stage's default — operators may override per deal; stage default is the suggestion)
  - DEC-352 (expected_close_date is required on stage move into `is_open` stages; defaulted to +30 days from today; required field for forecast accuracy)
  - DEC-353 (deal-stage transitions emit memory row before commit per task-audit skill rule 25 audit-before-action)
  - PDPL Art. 13 (data minimisation — contact emails + phone numbers are PII; scrubbed in memory audit chain via TASK-MEMORY-111)
  - ISO 27001:2022 A.5.16 (data classification — CRM data classified as "internal commercial")

language: rust 1.81 + sql
service: cyberos/services/crm/
new_files:
  # accounts table + account_type enum stub + RLS + comp-exclusion
  - services/crm/migrations/0001_accounts.sql
  # contacts table + contact_account_membership join + RLS
  - services/crm/migrations/0002_contacts.sql
  # pipelines + pipeline_stages tables + 4-shape closed enum
  - services/crm/migrations/0003_pipelines_stages.sql
  # deals table + deal_status enum + RLS + status FSM trigger
  - services/crm/migrations/0004_deals.sql
  # append-only deal status transitions
  - services/crm/migrations/0005_deal_status_history.sql
  # seed 4 default pipelines per tenant on creation (run via TASK-TEN-001 hook)
  - services/crm/migrations/0006_seed_pipelines.sql
  # crate root
  - services/crm/src/lib.rs
  # Account, Contact, Deal, Pipeline, PipelineStage, DealStatus, PipelineShape enums
  - services/crm/src/types.rs
  # closed FSM transition matrix
  - services/crm/src/fsm/deal_status.rs
  # CRUD
  - services/crm/src/repo/accounts.rs
  # CRUD + membership management
  - services/crm/src/repo/contacts.rs
  # CRUD + stage transition + status transition
  - services/crm/src/repo/deals.rs
  # pipeline + stage CRUD
  - services/crm/src/repo/pipelines.rs
  # canonical crm.* memory row builders (8 kinds)
  - services/crm/src/audit/crm_events.rs
  # POST/GET/PATCH /v1/crm/accounts
  - services/crm/src/handlers/accounts.rs
  # POST/GET/PATCH /v1/crm/contacts + membership endpoints
  - services/crm/src/handlers/contacts.rs
  # POST/GET/PATCH /v1/crm/deals + transition handlers
  - services/crm/src/handlers/deals.rs
  # POST/GET /v1/crm/pipelines + stage management
  - services/crm/src/handlers/pipelines.rs
  # +sqlx, +uuid, +serde, +chrono, +rust_decimal, +cyberos-cli-exit
  - services/crm/Cargo.toml
  - services/crm/tests/accounts_test.rs
  - services/crm/tests/contacts_membership_test.rs
  - services/crm/tests/deals_create_test.rs
  - services/crm/tests/deal_status_fsm_test.rs
  - services/crm/tests/deal_stage_transitions_test.rs
  - services/crm/tests/pipeline_shapes_test.rs
  - services/crm/tests/pipeline_stages_test.rs
  - services/crm/tests/append_only_history_test.rs
  - services/crm/tests/rls_isolation_test.rs
  - services/crm/tests/money_stored_minor_test.rs
  - services/crm/tests/audit_emission_test.rs
modified_files:
  # add accounts, contacts, deals, pipelines to TENANT_SCOPED_TABLES
  - services/auth/src/rls/templates.rs

allowed_tools:
  - file_read: services/crm/**
  - file_read: services/auth/src/rls/**
  - file_write: services/crm/{src,tests,migrations}/**
  - bash: cd services/crm && cargo test
  - bash: psql -f services/crm/migrations/0001_accounts.sql (local Postgres only)

disallowed_tools:
  - allow UPDATE on deal_status_history (per DEC-346 — append-only enforced at SQL grant)
  - ship full VN-specific account_type enum (TASK-CRM-003 ships)
  - ship MST validation logic (TASK-CRM-003 ships)
  - ship activity-feed handlers (TASK-CRM-002 ships)
  - ship Convert-to-Engagement workflow (TASK-CRM-004 ships)
  - store money as FLOAT/DOUBLE (per task-audit skill rule 11 — BIGINT minor units only)
  - allow Contact to belong to zero accounts (per DEC-348 — must have ≥1 membership)
  - add a 4th primitive entity (per DEC-340)
  - add a 5th deal_status value (per DEC-344)

effort_hours: 6
subtasks:
  - "0.5h: 0001_accounts.sql — accounts + account_type stub enum + RLS"
  - "0.5h: 0002_contacts.sql — contacts + membership join + RLS"
  - "0.5h: 0003_pipelines_stages.sql — pipelines + stages + closed pipeline_shape enum"
  - "0.7h: 0004_deals.sql — deals + deal_status enum + status FSM trigger"
  - "0.3h: 0005_deal_status_history.sql — append-only history + REVOKE writes"
  - "0.3h: 0006_seed_pipelines.sql — 4 default pipelines seed function for TASK-TEN-001 hook"
  - "0.4h: types.rs — 4 entity structs + 3 closed enums"
  - "0.3h: fsm/deal_status.rs — closed transition matrix"
  - "0.6h: repo/*.rs — 4 repository modules"
  - "0.4h: audit/crm_events.rs — 8 row builders"
  - "0.8h: handlers/*.rs — 4 REST handler modules"
  - "1.7h: tests — 11 test files"

risk_if_skipped: "CRM is the sales-pipeline spine upstream of PROJ; without the schema there's nothing to track customer engagement against. Every downstream CRM task (TASK-CRM-002 activity feed, TASK-CRM-003 VN-specific account types, TASK-CRM-004 Convert-to-Engagement, TASK-CRM-005 CUO next-action skill, TASK-CRM-006 AI lead scoring, TASK-CRM-007 win/loss analysis) reads from these tables. Without DEC-344's closed deal_status FSM, status drift across handlers produces inconsistent forecasts. Without DEC-345's BIGINT-minor money storage, FLOAT rounding breaks invoice math downstream. Without DEC-346's append-only history, win/loss attribution becomes ambiguous (operators can re-write 'why did this deal win?'). Without DEC-348's many-to-many contact-account membership, the common case 'this person works at two parent companies' forces operators to create duplicate contacts. The 6h effort lands the foundational schema so every CRM task can trust the invariants."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship the Account / Contact / Deal Postgres schema with custom pipelines and stages as the foundational sales-pipeline primitive. Each requirement:

1. **MUST** define the `accounts` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `legal_name TEXT NOT NULL CHECK (length(legal_name) BETWEEN 1 AND 200)`, `display_name TEXT`, `account_type account_type NOT NULL DEFAULT 'unknown'` (full enum ships in TASK-CRM-003 per DEC-343), `mst TEXT` (VN tax code; validation ships in TASK-CRM-003 + TASK-CRM-008), `website TEXT`, `industry TEXT`, `headquarters_country TEXT`, `owner_subject_id UUID NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `updated_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `created_by_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`.

2. **MUST** define the `contacts` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `full_name TEXT NOT NULL CHECK (length(full_name) BETWEEN 1 AND 200)`, `display_name TEXT`, `email TEXT` (nullable; PII), `phone TEXT` (nullable; PII), `title TEXT`, `language_code language_code` (vi/en per TASK-KB-001's enum reuse), `owner_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`, `created_at TIMESTAMPTZ`, `updated_at TIMESTAMPTZ`, `created_by_subject_id UUID`. Email + phone uniqueness is NOT enforced (same person may appear with the same email twice in different accounts).

3. **MUST** define the `contact_account_membership` JOIN table (per DEC-348) with: `contact_id UUID NOT NULL REFERENCES contacts(id) ON DELETE RESTRICT`, `account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT`, `tenant_id UUID NOT NULL`, `role TEXT NOT NULL DEFAULT 'general'` (closed enum-ish — common values: `decision_maker`, `technical`, `billing`, `legal`, `general`), `is_primary BOOLEAN NOT NULL DEFAULT false`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `PRIMARY KEY (contact_id, account_id)`. A contact MUST have at least one membership (enforced by handler; not by DB constraint to allow transactional bootstrap).

4. **MUST** declare the closed `pipeline_shape` Postgres enum with exactly 4 values (per DEC-341): `'sales'`, `'partner'`, `'inbound'`, `'outbound'`. Adding a 5th shape is an ADR. The shape gates the TASK-CRM-005 next-action skill's applicability.

5. **MUST** define the `pipelines` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100)`, `shape pipeline_shape NOT NULL`, `is_default BOOLEAN NOT NULL DEFAULT false`, `created_at TIMESTAMPTZ`, `created_by_subject_id UUID`. UNIQUE `(tenant_id, name)`; at most one pipeline per tenant per shape may be `is_default = true` (partial unique index).

6. **MUST** define the `pipeline_stages` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `pipeline_id UUID NOT NULL REFERENCES pipelines(id) ON DELETE RESTRICT`, `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 50)`, `position INT NOT NULL CHECK (position BETWEEN 1 AND 50)`, `probability_default INT NOT NULL CHECK (probability_default BETWEEN 0 AND 100) DEFAULT 50` (per DEC-351), `is_open BOOLEAN NOT NULL DEFAULT true`, `is_won BOOLEAN NOT NULL DEFAULT false`, `is_lost BOOLEAN NOT NULL DEFAULT false`, `created_at TIMESTAMPTZ`. Stage tags `is_open/is_won/is_lost` are mutually exclusive (DB CHECK: exactly one true). UNIQUE `(pipeline_id, position)` and UNIQUE `(pipeline_id, name)`.

7. **MUST** define the `deals` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 200)`, `account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT`, `primary_contact_id UUID REFERENCES contacts(id)` (nullable; one specific contact this deal is anchored to), `pipeline_id UUID NOT NULL REFERENCES pipelines(id)`, `current_stage_id UUID NOT NULL REFERENCES pipeline_stages(id)`, `status deal_status NOT NULL DEFAULT 'open'` (per DEC-344), `probability INT NOT NULL CHECK (probability BETWEEN 0 AND 100)`, `amount_minor BIGINT NOT NULL CHECK (amount_minor >= 0)` (per task-audit skill rule 11 + DEC-345), `amount_currency CHAR(3) NOT NULL CHECK (amount_currency ~ '^[A-Z]{3}$')`, `expected_close_date DATE NOT NULL`, `actual_close_date DATE`, `owner_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`, `created_at TIMESTAMPTZ`, `updated_at TIMESTAMPTZ`, `created_by_subject_id UUID NOT NULL`, `won_at TIMESTAMPTZ`, `lost_reason TEXT` (nullable; required when status=lost), `cancelled_reason TEXT` (nullable; required when status=cancelled).

8. **MUST** declare the closed `deal_status` Postgres enum with exactly 4 values (per DEC-344): `'open'`, `'won'`, `'lost'`, `'cancelled'`. Adding a 5th status is an ADR.

9. **MUST** ship the deal-status FSM transition validator at `services/crm/src/fsm/deal_status.rs` as a closed lookup table (per DEC-347). Valid transitions:
- `open → won` (requires current_stage to have `is_won = true`).
- `open → lost` (requires current_stage to have `is_lost = true` AND `lost_reason` non-empty).
- `open → cancelled` (requires `cancelled_reason` non-empty).
- `won → cancelled` (rare; refund/clawback; requires `cancelled_reason`). All other transitions return `INVALID_STATUS_TRANSITION` and are rejected at the API boundary AND by the trigger `enforce_deal_status_fsm`.

10. **MUST** record every deal status transition as an append-only row in `deal_status_history` (per DEC-346): `(id BIGSERIAL, deal_id UUID, tenant_id UUID, from_status deal_status, to_status deal_status, from_stage_id UUID, to_stage_id UUID, changed_at TIMESTAMPTZ, changed_by_subject_id UUID, reason TEXT)`. `REVOKE UPDATE, DELETE ON deal_status_history FROM cyberos_app`.

11. **MUST** record every deal stage transition (within `status=open`) as an append-only row in `deal_stage_history`: `(id BIGSERIAL, deal_id UUID, tenant_id UUID, from_stage_id UUID, to_stage_id UUID, from_probability INT, to_probability INT, changed_at TIMESTAMPTZ, changed_by_subject_id UUID, reason TEXT)`. Append-only via SQL grant.

12. **MUST** enforce RLS with both `USING` AND `WITH CHECK` clauses on `accounts`, `contacts`, `contact_account_membership`, `pipelines`, `pipeline_stages`, `deals`, `deal_status_history`, `deal_stage_history`. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

13. **MUST** store money as BIGINT minor units (per task-audit skill rule 11 + DEC-345). `amount_minor` is the smallest currency unit (e.g. cents for USD, đồng for VND — VND has no minor unit so 1 VND = 1 minor). `amount_currency` is the ISO-4217 code. The conversion to major units for display uses the `Currency::decimals()` helper.

14. **MUST** seed 4 default pipelines per tenant at tenant provisioning (per DEC-341, via TASK-TEN-001's hook). The seed function `seed_default_pipelines(tenant_id, created_by)` creates:
- Sales pipeline: stages `Lead → Qualified → Proposal → Negotiation → Won | Lost`.
- Partner pipeline: stages `Identified → Engaged → Co-pitching → Signed → Active | Disengaged`.
- Inbound pipeline: stages `New → Triaged → Demo → Trial → Converted | Disqualified`.
- Outbound pipeline: stages `Researched → Contacted → Replied → Demo → Won | Lost`. Each pipeline has `is_default = true` for its shape; tenants can add custom stages or rename.

15. **MUST** ship REST handlers:
- `POST /v1/crm/accounts` — create account; idempotent via Idempotency-Key.
- `GET /v1/crm/accounts/{id}` — fetch account + linked contacts.
- `PATCH /v1/crm/accounts/{id}` — update non-immutable fields.
- `POST /v1/crm/contacts` — create contact + initial membership(s).
- `GET /v1/crm/contacts/{id}` — fetch contact + memberships.
- `POST /v1/crm/contacts/{id}/memberships` — add account membership.
- `DELETE /v1/crm/contacts/{id}/memberships/{account_id}` — remove (requires ≥1 remaining).
- `POST /v1/crm/pipelines` — create pipeline (with initial stages).
- `GET /v1/crm/pipelines?shape=<>` — list.
- `POST /v1/crm/pipelines/{id}/stages` — add stage.
- `POST /v1/crm/deals` — create deal.
- `GET /v1/crm/deals/{id}` — fetch.
- `PATCH /v1/crm/deals/{id}` — update non-status non-stage fields.
- `POST /v1/crm/deals/{id}/stage` — transition stage (within open).
- `POST /v1/crm/deals/{id}/status` — transition status (open → won/lost/cancelled).

16. **MUST** validate that **contact has ≥1 membership** at handler boundary. POST `/contacts` with empty `initial_memberships` → 400 `contact_must_have_membership`. DELETE last membership → 409 `cannot_remove_last_membership` (suggest delete contact instead).

17. **MUST** emit memory audit rows for the 8 kinds (per DEC-349):
- `crm.account_created`, `crm.account_updated` on account writes.
- `crm.contact_created` on contact creation.
- `crm.deal_created` on deal creation.
- `crm.deal_stage_changed` on stage transition.
- `crm.deal_won` on status → won.
- `crm.deal_lost` on status → lost (carries `lost_reason`).
- `crm.deal_cancelled` on status → cancelled. All audit-before-action per task-audit skill rule 25 (emitted within the same transaction as the DB write).

18. **MUST** PII-scrub `email`, `phone`, `full_name` via TASK-MEMORY-111 BEFORE chain commit. The PostgreSQL rows retain raw values (tenant-scoped + RLS-protected); memory audit chain holds only `subject_id_hash16` + `email_hash16` + redacted forms.

19. **MUST** require `expected_close_date >= CURRENT_DATE` at deal creation AND on transitions into `is_open` stages (per DEC-352). Past-dated expected_close → 400 `expected_close_in_past`.

20. **MUST** set `won_at = now()` automatically on status → won; set `actual_close_date = now()` on status → won/lost/cancelled. The trigger `set_close_dates` handles this.

21. **MUST** require `lost_reason` (1–500 chars) when status → lost; require `cancelled_reason` (1–500 chars) when status → cancelled. Missing reason → 400 `reason_required`.

22. **MUST** ship probability override semantics (per DEC-351): on stage transition, probability defaults to the new stage's `probability_default`; explicit `probability` in the request overrides. Outside [0, 100] → 400 `probability_out_of_range`.

23. **MUST** complete handlers in ≤ 100 ms p95. `crm_perf_test` asserts.

24. **MUST** emit OTel span `crm.{account,contact,pipeline,deal}.{create,update,stage,status,...}` with attributes: `tenant_id`, `entity_id`, `outcome` (success | invalid_status_transition | invalid_stage_transition | reason_required | expected_close_in_past | permission_denied | not_found).

25. **MUST** emit OTel metrics:
- `crm_account_create_total{outcome}` (counter).
- `crm_contact_create_total{outcome}` (counter).
- `crm_deal_create_total{outcome, pipeline_shape}` (counter).
- `crm_deal_status_transitions_total{from_status, to_status, outcome}` (counter).
- `crm_deal_stage_transitions_total{pipeline_shape, outcome}` (counter).
- `crm_deal_amount_minor{tenant_id, currency, status}` (gauge — periodic sum compute).
- `crm_open_deal_count{tenant_id, pipeline_id}` (gauge).

26. **MUST** support cursor pagination on list handlers (max 200 default 50; cursor on `(updated_at DESC, id)`).

---

## §2 — Why this design (rationale for humans)

**Why closed 3-primitive entity model (DEC-340)?** Account → Contact → Deal is the Salesforce-flavoured industry standard. Adding a 4th primitive (Lead, Opportunity, Quote, Order) duplicates concerns: a Lead is "an unqualified Contact"; an Opportunity is "an early-stage Deal". The closed 3-primitive model forces operators to use the existing entities + stage transitions instead of inventing parallel hierarchies. ADR-required additions force the cross-module impact analysis.

**Why 4 closed pipeline shapes (DEC-341)?** Each shape has different optimal next-action heuristics (TASK-CRM-005's CUO skill consults the shape). Sales = "AM nudge"; Partner = "exec sponsor activation"; Inbound = "speed-to-first-response"; Outbound = "personalisation". A free-form 5th shape would force the skill to fall back to a generic heuristic — worse than picking one of the 4 deliberately. Adding a 5th shape ADR requires updating the skill's logic too.

**Why tenant-defined stages per pipeline (DEC-342, §1 #6)?** Different industries have different sales motions. SaaS sales: `Lead → Qualified → Demo → Trial → Closed`. Consulting: `Lead → Scoping → Proposal → Won`. Recurring services: `Lead → Discovery → Pilot → MSA → Renewal`. Forcing a global stage set would break ~half of tenants. Letting tenants define stages per pipeline + tag with `is_won/is_lost/is_open` keeps the downstream behaviour predictable.

**Why deal_status independent from stage (DEC-344, §1 #8)?** Stage = "where in the pipeline?" (can vary across pipelines). Status = "what's the outcome?" (universal: open/won/lost/cancelled). Treating them as one would duplicate the closed status enum across every tenant's stage list — drift inevitable. Splitting keeps the universal forecast logic clean (filter `status='open'` for pipeline; `status='won'` for revenue).

**Why deal_status FSM at trigger AND handler (DEC-347, §1 #9)?** Defense in depth. Handler validates at API boundary (good UX); trigger catches direct SQL access (e.g. operator psql session). The trigger checks: (a) transition is in the allowed set; (b) `current_stage.is_won/is_lost` matches the transition; (c) required reason fields are present.

**Why won/lost terminal vs cancelled allows won → cancelled (DEC-347, §1 #9)?** `lost` is final — "this deal will not close". `cancelled` is administrative — "even though we won, refund or clawback". The `won → cancelled` transition handles rare cases (customer backs out within return window). `lost → cancelled` makes no sense — a lost deal is already terminal.

**Why money as BIGINT minor (DEC-345, §1 #13)?** task-audit skill rule 11 — financial precision. Storing `1500000` minor units for 1.5M VND is exact; `1.5e6` FLOAT introduces rounding. Downstream TASK-INV-001 invoice math depends on exact addition; FLOAT drift produces invoices that don't sum to the deal amount.

**Why append-only history on status + stage (DEC-346, §1 #10, #11)?** Forecast accuracy + win/loss attribution depend on "what was the stage on 2026-05-15?" — questions only answerable from a chained history. UPDATE in place would lose the prior values. SQL grants make it audit-grade (operator typo can't rewrite); handler discipline alone is insufficient.

**Why many-to-many contact-account membership (DEC-348, §1 #3)?** Common cases: a CFO splits time between two subsidiary entities; an external consultant has billing relationships with multiple parent accounts; a lawyer represents multiple companies. Forcing one-account-per-contact creates duplicate contacts (the same human as 3 rows) — duplicates break dedup, break activity feeds (which appear under each duplicate), break next-action ranking. Many-to-many is the canonical shape.

**Why contact must have ≥1 membership (§1 #16)?** A contact with no membership is orphan data — no account to roll up under, no pipeline to track against. Enforcing at handler (not DB constraint) allows transactional bootstrap (create contact + initial membership in one tx).

**Why probability override (DEC-351, §1 #22)?** Stage defaults capture "typical probability at this stage"; per-deal override captures "this specific deal is unusually strong (or weak)". Salespeople know context the stage doesn't. The default-but-override pattern preserves the forecast-accuracy benefit of stage defaults while allowing per-deal calibration.

**Why expected_close_date required + future (§1 #19, DEC-352)?** Forecast accuracy demands a date. Allowing past-dated values (other than `actual_close_date`) corrupts pipeline forecast queries. Default to +30 days (slice 1 default) is a sane starting point; salesperson refines per deal.

**Why won_at automatic + actual_close_date automatic (§1 #20)?** Operators forget to set these manually; the trigger handles it. The cost is "operator can't backdate" (acceptable — historical correction is a rare ADR-driven path).

**Why language_code on contact (§1 #2)?** Vietnamese-aware salutations + outreach templates need to know the contact's preferred language. Reusing TASK-KB-001's `language_code` enum keeps consistency.

**Why owner_subject_id required on deal + account (§1 #1, #7)?** Ownership drives forecast attribution (CFO dashboard rolls up by owner) and CUO next-action skill (suggests moves for THIS owner's deals). Ownerless deals are forecast noise — make it a required field.

**Why deal close emits 3 distinct audit kinds (deal_won, deal_lost, deal_cancelled) instead of one (DEC-349, §1 #17)?** Different operator queries: "show me wins this quarter" filters `crm.deal_won`; "show me losses to investigate root causes" filters `crm.deal_lost`. One kind with a status field would force every consumer to filter — fewer selectivity benefits.

**Why MST field present but validation deferred (DEC-343, §1 #1)?** The column needs to exist now so deals can be created with the field; TASK-CRM-003 ships the validation (calls `vietnam-mst-validate` skill on write). Slice 1 stores any 10–13 char string; slice 2 enforces format + GDT registry check.

**Why pipeline_stages position 1-50 (§1 #6)?** Reasonable practical bound — pipelines deeper than 50 stages are pathological (operators lose track). 1-indexed for natural display order.

**Why `is_open/is_won/is_lost` mutually exclusive (§1 #6)?** A stage is exactly one of the three classifications. Allowing multiple (e.g. is_open + is_won) makes the deal_status FSM ambiguous. DB CHECK enforces.

**Why no PATCH for `current_stage_id` (§1 #15)?** Stage changes have side effects (probability update, history row, memory audit). PATCH would have to duplicate the orchestration; dedicated handler `POST /deals/{id}/stage` is the contract.

**Why one default pipeline per (tenant, shape) but not strict (§1 #5)?** Most tenants use one pipeline per shape; allowing multiples (partial unique index) supports edge cases (e.g. "Sales-EU" vs "Sales-US" pipelines). The `is_default` flag picks the canonical one for `vn-create-deal` skill auto-pipeline selection.

**Why phone/email nullable on contact (§1 #2)?** Real CRM data is incomplete; forcing both required would block import + bulk add. Nullable lets operators progressively enrich.

**Why account.legal_name vs display_name (§1 #1)?** Legal name is the regulatory entity ("ACME Corporation Joint Stock Company"); display_name is the brand ("ACME"). Reports use legal_name; UI uses display_name. Both nullable-with-fallback simplifies (display_name defaults to legal_name).

---

## §3 — API contract

### 3.1 — Migration 0001 — accounts

```sql
-- services/crm/migrations/0001_accounts.sql

BEGIN;

-- Placeholder enum at slice 1; full VN-specific enum ships in TASK-CRM-003.
CREATE TYPE account_type AS ENUM ('unknown');

CREATE TABLE accounts (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    legal_name             TEXT         NOT NULL CHECK (length(legal_name) BETWEEN 1 AND 200),
    display_name           TEXT         CHECK (display_name IS NULL OR length(display_name) BETWEEN 1 AND 200),
    account_type           account_type NOT NULL DEFAULT 'unknown',
    mst                    TEXT,
    website                TEXT,
    industry               TEXT,
    headquarters_country   TEXT,
    owner_subject_id       UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL REFERENCES auth.subjects(id)
);

CREATE INDEX accounts_tenant_owner_idx ON accounts (tenant_id, owner_subject_id);
CREATE INDEX accounts_tenant_name_idx ON accounts (tenant_id, legal_name);

ALTER TABLE accounts ENABLE ROW LEVEL SECURITY;
CREATE POLICY accounts_tenant_isolation ON accounts
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.2 — Migration 0002 — contacts + membership

```sql
-- services/crm/migrations/0002_contacts.sql

BEGIN;

CREATE TABLE contacts (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    full_name              TEXT         NOT NULL CHECK (length(full_name) BETWEEN 1 AND 200),
    display_name           TEXT,
    email                  TEXT,
    phone                  TEXT,
    title                  TEXT,
    language_code          language_code,
    owner_subject_id       UUID         NOT NULL REFERENCES auth.subjects(id),
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL
);

CREATE TABLE contact_account_membership (
    contact_id   UUID         NOT NULL REFERENCES contacts(id) ON DELETE RESTRICT,
    account_id   UUID         NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    tenant_id    UUID         NOT NULL,
    role         TEXT         NOT NULL DEFAULT 'general',
    is_primary   BOOLEAN      NOT NULL DEFAULT false,
    created_at   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    PRIMARY KEY (contact_id, account_id)
);

CREATE INDEX contacts_tenant_name_idx ON contacts (tenant_id, full_name);
CREATE INDEX contacts_tenant_email_idx ON contacts (tenant_id, email) WHERE email IS NOT NULL;
CREATE INDEX cam_account_idx ON contact_account_membership (tenant_id, account_id);

ALTER TABLE contacts ENABLE ROW LEVEL SECURITY;
ALTER TABLE contact_account_membership ENABLE ROW LEVEL SECURITY;

CREATE POLICY contacts_tenant_isolation ON contacts
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE POLICY cam_tenant_isolation ON contact_account_membership
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.3 — Migration 0003 — pipelines + stages

```sql
-- services/crm/migrations/0003_pipelines_stages.sql

BEGIN;

CREATE TYPE pipeline_shape AS ENUM ('sales', 'partner', 'inbound', 'outbound');

CREATE TABLE pipelines (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    name                   TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    shape                  pipeline_shape NOT NULL,
    is_default             BOOLEAN      NOT NULL DEFAULT false,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL
);

CREATE UNIQUE INDEX uniq_pipeline_tenant_name ON pipelines (tenant_id, name);
CREATE UNIQUE INDEX uniq_pipeline_tenant_shape_default ON pipelines (tenant_id, shape) WHERE is_default = true;

CREATE TABLE pipeline_stages (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    pipeline_id            UUID         NOT NULL REFERENCES pipelines(id) ON DELETE RESTRICT,
    name                   TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 50),
    position               INT          NOT NULL CHECK (position BETWEEN 1 AND 50),
    probability_default    INT          NOT NULL CHECK (probability_default BETWEEN 0 AND 100) DEFAULT 50,
    is_open                BOOLEAN      NOT NULL DEFAULT true,
    is_won                 BOOLEAN      NOT NULL DEFAULT false,
    is_lost                BOOLEAN      NOT NULL DEFAULT false,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK ((is_open::int + is_won::int + is_lost::int) = 1)
);

CREATE UNIQUE INDEX uniq_stage_pipeline_position ON pipeline_stages (pipeline_id, position);
CREATE UNIQUE INDEX uniq_stage_pipeline_name ON pipeline_stages (pipeline_id, name);

ALTER TABLE pipelines ENABLE ROW LEVEL SECURITY;
ALTER TABLE pipeline_stages ENABLE ROW LEVEL SECURITY;

CREATE POLICY pipelines_tenant_isolation ON pipelines
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE POLICY pipeline_stages_tenant_isolation ON pipeline_stages
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.4 — Migration 0004 — deals

```sql
-- services/crm/migrations/0004_deals.sql

BEGIN;

CREATE TYPE deal_status AS ENUM ('open', 'won', 'lost', 'cancelled');

CREATE TABLE deals (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    name                   TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 200),
    account_id             UUID         NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    primary_contact_id     UUID         REFERENCES contacts(id),
    pipeline_id            UUID         NOT NULL REFERENCES pipelines(id),
    current_stage_id       UUID         NOT NULL REFERENCES pipeline_stages(id),
    status                 deal_status  NOT NULL DEFAULT 'open',
    probability            INT          NOT NULL CHECK (probability BETWEEN 0 AND 100),
    amount_minor           BIGINT       NOT NULL CHECK (amount_minor >= 0),
    amount_currency        CHAR(3)      NOT NULL CHECK (amount_currency ~ '^[A-Z]{3}$'),
    expected_close_date    DATE         NOT NULL,
    actual_close_date      DATE,
    won_at                 TIMESTAMPTZ,
    lost_reason            TEXT         CHECK (lost_reason IS NULL OR length(lost_reason) BETWEEN 1 AND 500),
    cancelled_reason       TEXT         CHECK (cancelled_reason IS NULL OR length(cancelled_reason) BETWEEN 1 AND 500),
    owner_subject_id       UUID         NOT NULL REFERENCES auth.subjects(id),
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL
);

CREATE INDEX deals_tenant_owner_idx ON deals (tenant_id, owner_subject_id) WHERE status = 'open';
CREATE INDEX deals_tenant_pipeline_idx ON deals (tenant_id, pipeline_id, current_stage_id) WHERE status = 'open';
CREATE INDEX deals_tenant_account_idx ON deals (tenant_id, account_id);
CREATE INDEX deals_close_date_idx ON deals (tenant_id, expected_close_date) WHERE status = 'open';

ALTER TABLE deals ENABLE ROW LEVEL SECURITY;
CREATE POLICY deals_tenant_isolation ON deals
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- FSM transition trigger (DEC-347)
CREATE OR REPLACE FUNCTION enforce_deal_status_fsm() RETURNS TRIGGER AS $$
DECLARE
    stage_is_won BOOLEAN; stage_is_lost BOOLEAN;
BEGIN
    IF NEW.status = OLD.status THEN RETURN NEW; END IF;

    -- Allowed: open → won/lost/cancelled, won → cancelled
    IF NOT (
        (OLD.status = 'open' AND NEW.status IN ('won', 'lost', 'cancelled'))
        OR (OLD.status = 'won' AND NEW.status = 'cancelled')
    ) THEN
        RAISE EXCEPTION 'invalid_deal_status_transition: % -> %', OLD.status, NEW.status USING ERRCODE = 'P0050';
    END IF;

    -- Stage gate: won requires is_won; lost requires is_lost
    IF NEW.status = 'won' THEN
        SELECT is_won INTO stage_is_won FROM pipeline_stages WHERE id = NEW.current_stage_id;
        IF NOT stage_is_won THEN
            RAISE EXCEPTION 'cannot_mark_won_from_non_won_stage' USING ERRCODE = 'P0051';
        END IF;
        NEW.won_at := now();
        NEW.actual_close_date := CURRENT_DATE;
    ELSIF NEW.status = 'lost' THEN
        SELECT is_lost INTO stage_is_lost FROM pipeline_stages WHERE id = NEW.current_stage_id;
        IF NOT stage_is_lost THEN
            RAISE EXCEPTION 'cannot_mark_lost_from_non_lost_stage' USING ERRCODE = 'P0052';
        END IF;
        IF NEW.lost_reason IS NULL OR length(NEW.lost_reason) = 0 THEN
            RAISE EXCEPTION 'lost_reason_required' USING ERRCODE = 'P0053';
        END IF;
        NEW.actual_close_date := CURRENT_DATE;
    ELSIF NEW.status = 'cancelled' THEN
        IF NEW.cancelled_reason IS NULL OR length(NEW.cancelled_reason) = 0 THEN
            RAISE EXCEPTION 'cancelled_reason_required' USING ERRCODE = 'P0054';
        END IF;
        NEW.actual_close_date := CURRENT_DATE;
    END IF;

    NEW.updated_at := now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_deals_status_fsm BEFORE UPDATE ON deals
    FOR EACH ROW EXECUTE FUNCTION enforce_deal_status_fsm();

-- Expected-close future check on insert + stage-change-to-open
CREATE OR REPLACE FUNCTION enforce_expected_close_future() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'open' AND NEW.expected_close_date < CURRENT_DATE THEN
        RAISE EXCEPTION 'expected_close_in_past' USING ERRCODE = 'P0055';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_deals_expected_close BEFORE INSERT ON deals
    FOR EACH ROW EXECUTE FUNCTION enforce_expected_close_future();

COMMIT;
```

### 3.5 — Migration 0005 — status + stage history

```sql
-- services/crm/migrations/0005_deal_status_history.sql

BEGIN;

CREATE TABLE deal_status_history (
    id                     BIGSERIAL    PRIMARY KEY,
    deal_id                UUID         NOT NULL REFERENCES deals(id),
    tenant_id              UUID         NOT NULL,
    from_status            deal_status,
    to_status              deal_status  NOT NULL,
    from_stage_id          UUID         REFERENCES pipeline_stages(id),
    to_stage_id            UUID         REFERENCES pipeline_stages(id),
    changed_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id  UUID         NOT NULL,
    reason                 TEXT
);

CREATE TABLE deal_stage_history (
    id                     BIGSERIAL    PRIMARY KEY,
    deal_id                UUID         NOT NULL REFERENCES deals(id),
    tenant_id              UUID         NOT NULL,
    from_stage_id          UUID         REFERENCES pipeline_stages(id),
    to_stage_id            UUID         NOT NULL REFERENCES pipeline_stages(id),
    from_probability       INT,
    to_probability         INT          NOT NULL,
    changed_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id  UUID         NOT NULL,
    reason                 TEXT
);

CREATE INDEX dsh_deal_idx ON deal_status_history (deal_id, changed_at DESC);
CREATE INDEX dgh_deal_idx ON deal_stage_history (deal_id, changed_at DESC);

ALTER TABLE deal_status_history ENABLE ROW LEVEL SECURITY;
ALTER TABLE deal_stage_history ENABLE ROW LEVEL SECURITY;

CREATE POLICY dsh_tenant_isolation ON deal_status_history
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

CREATE POLICY dgh_tenant_isolation ON deal_stage_history
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON deal_status_history FROM cyberos_app;
REVOKE UPDATE, DELETE ON deal_stage_history FROM cyberos_app;

COMMIT;
```

### 3.6 — Default-pipelines seed function

```sql
-- services/crm/migrations/0006_seed_pipelines.sql

BEGIN;

CREATE OR REPLACE FUNCTION seed_default_pipelines(p_tenant_id UUID, p_created_by UUID) RETURNS VOID AS $$
DECLARE
    pid UUID;
BEGIN
    -- Sales pipeline
    INSERT INTO pipelines (id, tenant_id, name, shape, is_default, created_by_subject_id)
    VALUES (gen_random_uuid(), p_tenant_id, 'Sales', 'sales'::pipeline_shape, true, p_created_by)
    RETURNING id INTO pid;
    INSERT INTO pipeline_stages (id, tenant_id, pipeline_id, name, position, probability_default, is_open, is_won, is_lost) VALUES
        (gen_random_uuid(), p_tenant_id, pid, 'Lead',         1, 10, true,  false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Qualified',    2, 25, true,  false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Proposal',     3, 50, true,  false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Negotiation',  4, 75, true,  false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Won',          5, 100, false, true,  false),
        (gen_random_uuid(), p_tenant_id, pid, 'Lost',         6, 0,   false, false, true);

    -- Partner pipeline
    INSERT INTO pipelines (id, tenant_id, name, shape, is_default, created_by_subject_id)
    VALUES (gen_random_uuid(), p_tenant_id, 'Partner', 'partner'::pipeline_shape, true, p_created_by)
    RETURNING id INTO pid;
    INSERT INTO pipeline_stages (id, tenant_id, pipeline_id, name, position, probability_default, is_open, is_won, is_lost) VALUES
        (gen_random_uuid(), p_tenant_id, pid, 'Identified',  1, 10,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Engaged',     2, 30,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Co-pitching', 3, 60,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Signed',      4, 90,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Active',      5, 100, false, true,  false),
        (gen_random_uuid(), p_tenant_id, pid, 'Disengaged',  6, 0,   false, false, true);

    -- Inbound pipeline
    INSERT INTO pipelines (id, tenant_id, name, shape, is_default, created_by_subject_id)
    VALUES (gen_random_uuid(), p_tenant_id, 'Inbound', 'inbound'::pipeline_shape, true, p_created_by)
    RETURNING id INTO pid;
    INSERT INTO pipeline_stages (id, tenant_id, pipeline_id, name, position, probability_default, is_open, is_won, is_lost) VALUES
        (gen_random_uuid(), p_tenant_id, pid, 'New',           1, 15,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Triaged',       2, 30,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Demo',          3, 55,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Trial',         4, 80,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Converted',     5, 100, false, true, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Disqualified',  6, 0,   false, false, true);

    -- Outbound pipeline
    INSERT INTO pipelines (id, tenant_id, name, shape, is_default, created_by_subject_id)
    VALUES (gen_random_uuid(), p_tenant_id, 'Outbound', 'outbound'::pipeline_shape, true, p_created_by)
    RETURNING id INTO pid;
    INSERT INTO pipeline_stages (id, tenant_id, pipeline_id, name, position, probability_default, is_open, is_won, is_lost) VALUES
        (gen_random_uuid(), p_tenant_id, pid, 'Researched', 1, 5,   true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Contacted',  2, 15,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Replied',    3, 35,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Demo',       4, 60,  true, false, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Won',        5, 100, false, true, false),
        (gen_random_uuid(), p_tenant_id, pid, 'Lost',       6, 0,   false, false, true);
END;
$$ LANGUAGE plpgsql;

COMMIT;
```

### 3.7 — Rust types

```rust
// services/crm/src/types.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "deal_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DealStatus { Open, Won, Lost, Cancelled }

impl DealStatus {
    pub const ALL: &'static [DealStatus] = &[DealStatus::Open, DealStatus::Won, DealStatus::Lost, DealStatus::Cancelled];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "pipeline_shape", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PipelineShape { Sales, Partner, Inbound, Outbound }

impl PipelineShape {
    pub const ALL: &'static [PipelineShape] = &[PipelineShape::Sales, PipelineShape::Partner, PipelineShape::Inbound, PipelineShape::Outbound];
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Account {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub legal_name: String,
    pub display_name: Option<String>,
    pub account_type: String,   // closed enum expansion in TASK-CRM-003
    pub mst: Option<String>,
    pub website: Option<String>,
    pub industry: Option<String>,
    pub headquarters_country: Option<String>,
    pub owner_subject_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Deal {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub account_id: Uuid,
    pub primary_contact_id: Option<Uuid>,
    pub pipeline_id: Uuid,
    pub current_stage_id: Uuid,
    pub status: DealStatus,
    pub probability: i32,
    pub amount_minor: i64,
    pub amount_currency: String,
    pub expected_close_date: NaiveDate,
    pub actual_close_date: Option<NaiveDate>,
    pub won_at: Option<DateTime<Utc>>,
    pub lost_reason: Option<String>,
    pub cancelled_reason: Option<String>,
    pub owner_subject_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}
```

### 3.8 — Status FSM validator

```rust
// services/crm/src/fsm/deal_status.rs
use crate::types::DealStatus;

pub fn is_valid_transition(from: DealStatus, to: DealStatus) -> bool {
    use DealStatus::*;
    matches!((from, to),
        (Open, Won) | (Open, Lost) | (Open, Cancelled) | (Won, Cancelled)
    )
}

#[derive(Debug, thiserror::Error)]
#[error("invalid_deal_status_transition: {from:?} -> {to:?}")]
pub struct InvalidStatusTransition { pub from: DealStatus, pub to: DealStatus }

pub fn validate_transition(from: DealStatus, to: DealStatus) -> Result<(), InvalidStatusTransition> {
    if from == to { return Ok(()); }
    if is_valid_transition(from, to) { Ok(()) } else { Err(InvalidStatusTransition { from, to }) }
}
```

---

## §4 — Acceptance criteria

1. **DealStatus enum closed at 4** — `DealStatus::ALL.len() == 4`; Postgres enum has exactly 4 labels.
2. **PipelineShape enum closed at 4** — same.
3. **RLS isolates by tenant** — query as tenant-A returns 0 rows of tenant-B.
4. **POST account happy path** — 201 + `crm.account_created` memory row.
5. **POST contact with ≥1 membership** — 201 + `crm.contact_created` row.
6. **POST contact with 0 memberships** → 400 `contact_must_have_membership`.
7. **DELETE last membership** → 409 `cannot_remove_last_membership`.
8. **POST deal happy path** — 201 + `crm.deal_created` row; default probability matches stage.
9. **POST deal past expected_close_date** → 400 `expected_close_in_past`.
10. **Stage transition emits deal_stage_history row** + `crm.deal_stage_changed` memory row.
11. **Status open → won at non-won stage** → 400 `cannot_mark_won_from_non_won_stage`.
12. **Status open → won at won-stage** → 200; `won_at` set; `actual_close_date` set; `crm.deal_won` memory row.
13. **Status open → lost without reason** → 400 `lost_reason_required`.
14. **Status open → lost with reason** → 200; `crm.deal_lost` memory row carries reason.
15. **Status open → cancelled with reason** → 200; `crm.deal_cancelled` memory row.
16. **Status lost → won rejected** — invalid_deal_status_transition.
17. **Status won → cancelled allowed** (clawback scenario).
18. **Money stored as BIGINT minor** — 1500000 stored; rendered as 1.5M VND via Currency helper.
19. **deal_status_history append-only** — UPDATE/DELETE rejected by SQL grant.
20. **deal_stage_history append-only** — same.
21. **Pipeline stage mutual exclusion** — `is_open=true, is_won=true` insert → DB CHECK fails.
22. **One default pipeline per (tenant, shape)** — second pipeline with `is_default=true` for same shape → UNIQUE violation.
23. **seed_default_pipelines creates 4 pipelines** — called per TASK-TEN-001 hook; 4 pipelines × 6 stages each = 24 stages seeded.
24. **Probability 0-100 enforced** — outside range → 400.
25. **OTel span emitted** — `crm.deal.create` with `outcome=success`.
26. **Counters increment** per audit row kind.
27. **Perf < 100ms p95** — `crm_perf_test`.

---

## §5 — Verification

```rust
// services/crm/tests/deal_status_fsm_test.rs
use cyberos_crm::fsm::deal_status::{is_valid_transition, validate_transition};
use cyberos_crm::types::DealStatus::*;

#[test]
fn valid_transitions() {
    for (from, to) in [(Open, Won), (Open, Lost), (Open, Cancelled), (Won, Cancelled)] {
        assert!(is_valid_transition(from, to));
    }
}

#[test]
fn invalid_transitions_rejected() {
    for (from, to) in [(Lost, Won), (Won, Open), (Cancelled, Open), (Cancelled, Won)] {
        assert!(!is_valid_transition(from, to));
        assert!(validate_transition(from, to).is_err());
    }
}
```

```rust
// services/crm/tests/pipeline_stages_test.rs
#[sqlx::test]
async fn stage_mutual_exclusion(pool: sqlx::PgPool) {
    let (tenant, pipeline) = seed_pipeline(&pool, PipelineShape::Sales).await;
    let err = sqlx::query("INSERT INTO pipeline_stages (id, tenant_id, pipeline_id, name, position, probability_default, is_open, is_won, is_lost) VALUES ($1, $2, $3, 'BadStage', 99, 50, true, true, false)")
        .bind(Uuid::new_v4()).bind(tenant).bind(pipeline).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("violates check constraint"));
}
```

```rust
// services/crm/tests/contacts_membership_test.rs
#[tokio::test]
async fn contact_without_membership_rejected(ctx: TestCtx) {
    let resp = ctx.post_contact_with_zero_memberships().await;
    assert_eq!(resp.status(), 400);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "contact_must_have_membership");
}

#[tokio::test]
async fn cannot_remove_last_membership(ctx: TestCtx) {
    let contact = ctx.create_contact_with_one_membership().await;
    let resp = ctx.delete_membership(contact.id, contact.account_id).await;
    assert_eq!(resp.status(), 409);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "cannot_remove_last_membership");
}
```

```rust
// services/crm/tests/deals_create_test.rs
#[tokio::test]
async fn deal_won_at_lost_stage_rejected(ctx: TestCtx) {
    let stage = ctx.create_stage_with_lost_flag().await;
    let deal = ctx.create_deal_at_stage(stage.id).await;
    let resp = ctx.transition_status(deal.id, json!({"to": "won"})).await;
    assert_eq!(resp.status(), 400);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "cannot_mark_won_from_non_won_stage");
}

#[tokio::test]
async fn deal_won_emits_memory_row(ctx: TestCtx) {
    let stage = ctx.create_won_stage().await;
    let deal = ctx.create_deal_at_stage(stage.id).await;
    ctx.transition_status(deal.id, json!({"to": "won"})).await;
    let rows = ctx.memory_audit_rows("crm.deal_won").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["deal_id"], deal.id.to_string());
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 8 memory row builders follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-003** — RLS enforcement.
- **TASK-AUTH-101** — RBAC catalogue (`Resource::CrmAccount`, `CrmContact`, `CrmDeal`).

**Downstream (3 placeholders):**
- **TASK-CRM-002** — activity feed auto-log from EMAIL/CHAT/Calendar.
- **TASK-CRM-003** — VN-specific account_type enum + MST validation.
- **TASK-CRM-004** — Convert-to-Engagement workflow (consumes deals + emits to PROJ-005).

**Cross-module:**
- **TASK-AI-003** — memory audit bridge.
- **TASK-MEMORY-111** — PII scrubbing for email, phone, full_name.
- **TASK-TEN-001** — seeds default pipelines via the seed_default_pipelines function during tenant provisioning.

---

## §8 — Example payloads

### 8.1 — POST /v1/crm/deals request

```json
{
  "name": "ACME — CyberOS Annual Subscription",
  "account_id": "9b1deb4d-...",
  "primary_contact_id": "0a1deb4d-...",
  "pipeline_id": "<default sales pipeline>",
  "current_stage_id": "<Qualified stage>",
  "amount_minor": 12000000000,
  "amount_currency": "VND",
  "expected_close_date": "2026-07-15"
}
```

### 8.2 — crm.deal_created memory row

```json
{
  "kind": "crm.deal_created",
  "tenant_id": "5e8f1d2a-...",
  "deal_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "account_id": "9b1deb4d-...",
  "pipeline_id": "<uuid>",
  "stage_id": "<uuid>",
  "amount_minor": 12000000000,
  "amount_currency": "VND",
  "expected_close_date": "2026-07-15",
  "owner_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

### 8.3 — POST /v1/crm/deals/{id}/status (won)

```json
{ "to": "won", "reason": "Customer signed MSA; kickoff Monday" }
```

### 8.4 — crm.deal_won memory row

```json
{
  "kind": "crm.deal_won",
  "tenant_id": "5e8f1d2a-...",
  "deal_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "amount_minor": 12000000000,
  "amount_currency": "VND",
  "won_at": "2026-07-15T14:00:00Z",
  "changed_by_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **VN-specific account_type enum (Sole / LLC / JSC / FDI)** — TASK-CRM-003.
- **MST format + GDT registry validation** — TASK-CRM-003 + TASK-CRM-008.
- **Activity feed auto-log** — TASK-CRM-002.
- **Convert-to-Engagement workflow** — TASK-CRM-004.
- **Next-action CUO skill** — TASK-CRM-005.
- **AI lead scoring** — TASK-CRM-006.
- **Win/loss analysis CUO draft** — TASK-CRM-007.
- **VietQR invoice collection** — TASK-CRM-009.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| RLS bypass attempt | `USING` predicate | 0 rows | None — designed |
| Cross-tenant FK | RLS | permission_denied | None — designed |
| Contact with 0 memberships | handler | 400 contact_must_have_membership | Add membership in same tx |
| Delete last membership | handler | 409 cannot_remove_last_membership | Delete contact instead |
| Stage `is_open + is_won` both true | DB CHECK | INSERT fails | Fix stage definition |
| Two defaults per (tenant, shape) | UNIQUE partial index | INSERT fails | Unset existing default first |
| Status transition lost → won | trigger FSM | 400 invalid_deal_status_transition | Designed |
| Status open → won at non-won stage | trigger | 400 cannot_mark_won_from_non_won_stage | Move to won stage first |
| Status open → lost at non-lost stage | trigger | 400 cannot_mark_lost_from_non_lost_stage | Move to lost stage first |
| Status open → lost without reason | trigger | 400 lost_reason_required | Provide reason |
| Status open → cancelled without reason | trigger | 400 cancelled_reason_required | Provide reason |
| Past expected_close_date | trigger | 400 expected_close_in_past | Use ≥ today |
| Probability out of [0, 100] | DB CHECK | INSERT/UPDATE fails | Use valid range |
| Pipeline stage position > 50 | DB CHECK | INSERT fails | Use ≤ 50 |
| Amount_minor negative | DB CHECK | INSERT/UPDATE fails | Use ≥ 0 |
| Currency not ISO-4217 | DB CHECK | INSERT/UPDATE fails | Use valid 3-letter code |
| memory row commit fails mid-tx | rollback | 500 audit_failed | memory_writer health |
| Audit row contains unscrubbed PII | TASK-MEMORY-111 + pre-commit test | CI fails | Add PII rule |
| Duplicate stage position in same pipeline | UNIQUE | INSERT fails | Adjust positions |
| Duplicate stage name in same pipeline | UNIQUE | INSERT fails | Use different name |
| Deal account deleted while deal exists | FK RESTRICT | DELETE fails | Cancel deal first |
| Stage transition skips multiple positions | allowed (deals can jump stages) | None — designed | None |
| Owner subject deleted while deals exist | FK RESTRICT | DELETE fails | Transfer ownership first |
| Pipeline deleted while deals reference it | FK RESTRICT | DELETE fails | Migrate deals first |
| Idempotency-Key collision different body | 409 | Caller fixes | None |
| won_at retroactive set | not exposed via API | None — automatic | None |
| Status transition concurrent (two operators) | Postgres serialisable | One wins | Caller refetches |
| seed_default_pipelines called twice | UNIQUE constraint on name | Second call fails | Already seeded; OK |
| Stage rename mid-deal | UPDATE allowed; deal still points by id | None — designed | None |
| Negative probability_default | DB CHECK | INSERT fails | Use 0-100 |
| Deal close date in the past at creation | trigger | 400 expected_close_in_past | Use future date |

---

## §11 — Implementation notes

- **Account → Contact → Deal is the canonical 3-primitive shape** — Salesforce-flavoured; adding 4th primitive is ADR.
- **Closed pipeline_shape enum drives TASK-CRM-005 next-action skill heuristics** — each shape has different optimal moves.
- **Tenant-defined stages per pipeline + `is_open/is_won/is_lost` classification** — universal forecast logic + per-tenant flexibility.
- **deal_status independent from stage** — universal 4-value enum; stage gates the won/lost transitions.
- **FSM at trigger AND handler** — defense in depth; trigger catches direct SQL.
- **Money as BIGINT minor + CHAR(3) currency** — task-audit skill rule 11; FLOAT forbidden.
- **Append-only history at SQL grant** — `REVOKE UPDATE, DELETE FROM cyberos_app` on deal_status_history + deal_stage_history.
- **Many-to-many contact-account membership** — supports cross-account contacts (CFO of subsidiary parent).
- **Contact must have ≥1 membership** — enforced at handler (DB allows zero for transactional bootstrap).
- **Probability override semantics** — stage default suggested; per-deal override allowed.
- **expected_close_date required + future on `is_open` stages** — forecast hygiene.
- **won_at + actual_close_date automatic via trigger** — operator doesn't need to remember.
- **language_code reused from TASK-KB-001** — consistency across modules for VN/EN.
- **PII scrubbing applies to email + phone + full_name** — memory audit chain holds hashed forms only.
- **seed_default_pipelines invoked by TASK-TEN-001's hook** — every tenant gets 4 default pipelines on provisioning.
- **3 separate audit kinds for close (won/lost/cancelled)** — selectivity benefits at query time.
- **owner_subject_id required on deal + account** — forecast attribution depends on it.
- **No PATCH on current_stage_id** — dedicated stage handler ensures side-effects fire.
- **Pipeline + stage UNIQUE constraints** — `(pipeline_id, position)` + `(pipeline_id, name)`.
- **deals indexes** — owner + pipeline + close_date partial-WHERE-open indexes optimise the hot forecast queries.
- **`account_type` placeholder enum at slice 1** — TASK-CRM-003 ships full VN-specific expansion.
- **MST stored but unvalidated at slice 1** — TASK-CRM-003 + TASK-CRM-008 ship validation against GDT.
- **Bcc-like privacy on contact email** — no UNIQUE constraint; same email may appear in two contacts (e.g. CFO same email different accounts).

---

*End of TASK-CRM-001.*
