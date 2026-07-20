---
id: TASK-OKR-001
title: "OKR Objective × Key Result schema — Company → Team → Member cascade + quarterly Cycle + closed alignment FSM + RLS + face-saving status enum"
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
module: OKR
priority: p0
status: draft
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng (CEO + CSO seat)
created: 2026-05-16
shipped: now()
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-OKR-002, TASK-OKR-003, TASK-OKR-004, TASK-OKR-005, TASK-OKR-006, TASK-OKR-007, TASK-HR-001]
depends_on: [TASK-AUTH-003, TASK-AUTH-101]
# 4 downstream consumers
blocks: [TASK-OKR-002, TASK-OKR-003, TASK-OKR-005, TASK-OKR-007]

source_pages:
  - website/docs/modules/okr.html#what
  - website/docs/modules/okr.html#data-model
source_decisions:
  - DEC-360 (cycles are quarterly by default; configurable per tenant via cycle_kind enum: quarterly | monthly | trimester)
  - DEC-361 (closed 3-tier scope enum: company · team · member — adding a 4th tier (department/division) is an ADR)
  - DEC-362 (Objective requires 3-5 KRs at slice 1; enforced at handler not DB CHECK so transactional bootstrap is possible)
  - DEC-363 (KR type closed enum at 3 values per Doerr/Grove: hit_target · improvement · milestone — full type validation ships in TASK-OKR-002)
  - DEC-364 (KR status closed enum at 5 face-saving values: on_track · at_risk · learned · achieved · cycled_forward — per Vietnamese cultural adaptation; "missed" never appears)
  - DEC-365 (alignment FSM: Company OKRs have no parent; Team OKRs have exactly 1 Company parent; Member OKRs have exactly 1 Team parent — strict tree, no cross-cascade)
  - DEC-366 (closed cycle_status enum at 4 values: planning · active · closing · closed — transitions are unidirectional)
  - DEC-367 (REVOKE UPDATE, DELETE on kr_progress_log, objective_status_history from cyberos_app — append-only at SQL grant)
  - DEC-368 (memory audit kinds: okr.cycle_opened, okr.cycle_closed, okr.objective_created, okr.objective_updated, okr.kr_progress_recorded, okr.kr_status_changed, okr.alignment_created, okr.cycle_retro_recorded)
  - DEC-369 (EU AI Act Art. 14 — OKR-driven employment decisions REQUIRE explicit human approval; this task ships the data model that downstream HR/REW/LEARN consumes; the human-in-loop gate is in those tasks)
  - DEC-370 (KR `progress_value_numeric` is BIGINT for hit_target + improvement; for milestone, the value is a boolean (achieved = 1 | not = 0); TASK-OKR-002 ships full per-type validation)
  - DEC-371 (cascading delete: Cycle delete CASCADES to objectives; Objective delete CASCADES to KRs; KR delete RESTRICTs if progress_log exists — preserves audit history)
  - DEC-372 (Member OKRs reference HR Member by subject_id 1:1 with auth.subjects; team OKRs reference a Team entity declared in this task (tenant-local) since HR doesn't ship a Team primitive)
  - PDPL Art. 13 (data minimisation — KR rationale + progress comments PII-scrubbed in memory chain)
  - ISO 27001:2022 A.5.16 (information classification — OKR data classified as "internal strategy")
  - EU AI Act Art. 14 Annex III §4 (high-risk-adjacent: OKR-driven employment decisions)

language: rust 1.81 + sql
service: cyberos/services/okr/
new_files:
  # cycles table + cycle_kind + cycle_status enums + RLS
  - services/okr/migrations/0001_cycles.sql
  # tenant-local teams (task-HR ships members; OKR ships team primitive)
  - services/okr/migrations/0002_teams.sql
  # objectives + scope enum + alignment FK + RLS
  - services/okr/migrations/0003_objectives.sql
  # key_results + kr_type + kr_status enums + RLS
  - services/okr/migrations/0004_key_results.sql
  # append-only KR progress recordings (consumed by TASK-OKR-005 check-ins)
  - services/okr/migrations/0005_progress_log.sql
  # append-only status transitions
  - services/okr/migrations/0006_objective_status_history.sql
  # crate root
  - services/okr/src/lib.rs
  # Cycle, Team, Objective, KeyResult + 5 enums
  - services/okr/src/types.rs
  # closed transition matrix
  - services/okr/src/fsm/kr_status.rs
  # planning → active → closing → closed (unidirectional)
  - services/okr/src/fsm/cycle_status.rs
  # tree-invariant validator (Company/Team/Member parent rules)
  - services/okr/src/alignment/validator.rs
  - services/okr/src/repo/cycles.rs
  - services/okr/src/repo/teams.rs
  - services/okr/src/repo/objectives.rs
  - services/okr/src/repo/key_results.rs
  - services/okr/src/repo/progress_log.rs
  # 8 memory row builders
  - services/okr/src/audit/okr_events.rs
  - services/okr/src/handlers/cycles.rs
  - services/okr/src/handlers/teams.rs
  - services/okr/src/handlers/objectives.rs
  - services/okr/src/handlers/key_results.rs
  - services/okr/src/handlers/progress.rs
  # +sqlx, +uuid, +serde, +chrono, +cyberos-cli-exit
  - services/okr/Cargo.toml
  - services/okr/tests/cycles_test.rs
  - services/okr/tests/teams_test.rs
  - services/okr/tests/objectives_create_test.rs
  # 3-5 KRs per Objective enforced
  - services/okr/tests/kr_count_bounds_test.rs
  # Company/Team/Member parent rules
  - services/okr/tests/alignment_tree_test.rs
  # face-saving status transitions
  - services/okr/tests/kr_status_fsm_test.rs
  # unidirectional cycle flow
  - services/okr/tests/cycle_status_fsm_test.rs
  # progress_log immutable
  - services/okr/tests/append_only_progress_test.rs
  # tenant isolation
  - services/okr/tests/rls_isolation_test.rs
  # no "miss" / "fail" in enum values; CI lint
  - services/okr/tests/face_saving_terminology_test.rs
  - services/okr/tests/audit_emission_test.rs
modified_files:
  # add OKR tables to TENANT_SCOPED_TABLES
  - services/auth/src/rls/templates.rs

allowed_tools:
  - file_read: services/okr/**
  - file_write: services/okr/{src,tests,migrations}/**
  - bash: cd services/okr && cargo test

disallowed_tools:
  - allow UPDATE on kr_progress_log or objective_status_history (per DEC-367)
  - introduce "missed" / "failed" / "behind" status values (per DEC-364 — face-saving terminology only)
  - allow cross-cascade alignment (Member OKR aligned to Company directly) (per DEC-365)
  - ship KR progress_source DSL here (TASK-OKR-003 ships)
  - ship weekly check-in handler here (TASK-OKR-005 ships)
  - ship Monday digest here (TASK-OKR-006 ships)
  - allow Cycle delete without explicit operator confirmation (cascade is destructive)

effort_hours: 6
subtasks:
  - "0.5h: 0001_cycles.sql — cycles table + 2 enums + RLS"
  - "0.4h: 0002_teams.sql — tenant-local teams"
  - "0.7h: 0003_objectives.sql — objectives + scope enum + alignment FK + RLS"
  - "0.7h: 0004_key_results.sql — KRs + 2 enums + face-saving status + RLS"
  - "0.4h: 0005_progress_log.sql — append-only + REVOKE writes"
  - "0.3h: 0006_objective_status_history.sql — append-only + REVOKE writes"
  - "0.4h: types.rs — 4 entity structs + 5 enums"
  - "0.3h: fsm/*.rs — 2 closed transition matrices"
  - "0.3h: alignment/validator.rs — tree-invariant validator"
  - "0.5h: repo/*.rs — 5 repository modules"
  - "0.4h: audit/okr_events.rs — 8 row builders"
  - "0.8h: handlers/*.rs — 5 REST handler modules"
  - "0.3h: face-saving-terminology CI lint test"
  - "1.0h: tests — 11 test files"

risk_if_skipped: "OKR is the quarterly strategy operating loop; without the schema, the cascade is operator-mental rather than data-canonical. Every downstream OKR task (TASK-OKR-002 KR types, TASK-OKR-003 progress source DSL, TASK-OKR-004 auto-progress batch, TASK-OKR-005 weekly check-ins, TASK-OKR-006 Monday digest, TASK-OKR-007 retros) reads from these tables. Without DEC-365's strict alignment tree, member OKRs drift away from company strategy. Without DEC-364's face-saving status enum, the Vietnamese cultural adaptation is lost — operators see 'failed' and the retro becomes blame-finding. Without DEC-367's append-only progress log, KR progress can be retroactively edited — forecast integrity breaks. Without DEC-369's EU AI Act Art. 14 acknowledgement, OKR-driven employment decisions skip the human-in-loop requirement. The 6h effort lands the foundational primitives + the cultural adaptations baked into the schema."
---

## §1 — Description (BCP-14 normative)

The OKR service **MUST** ship the Cycle + Team + Objective + KeyResult schema as the quarterly strategy operating loop primitive. Each requirement:

1. **MUST** define the `cycles` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100)` (e.g. "Q3 2026"), `cycle_kind cycle_kind NOT NULL DEFAULT 'quarterly'`, `start_date DATE NOT NULL`, `end_date DATE NOT NULL CHECK (end_date > start_date)`, `status cycle_status NOT NULL DEFAULT 'planning'`, `theme TEXT` (nullable; optional theme for the cycle), `created_at TIMESTAMPTZ`, `created_by_subject_id UUID NOT NULL`. UNIQUE `(tenant_id, name)`.

2. **MUST** declare the closed `cycle_kind` Postgres enum with exactly 3 values (per DEC-360): `'quarterly'`, `'monthly'`, `'trimester'`. Adding a 4th is an ADR. Default is quarterly.

3. **MUST** declare the closed `cycle_status` Postgres enum with exactly 4 values (per DEC-366): `'planning'`, `'active'`, `'closing'`, `'closed'`. Transitions are unidirectional: `planning → active → closing → closed`. Backward transitions are forbidden.

4. **MUST** define the `teams` table (per DEC-372) with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 100)`, `parent_team_id UUID REFERENCES teams(id) ON DELETE RESTRICT`, `lead_subject_id UUID REFERENCES auth.subjects(id)`, `created_at TIMESTAMPTZ`. UNIQUE `(tenant_id, name)`. Teams form a hierarchy (Company → Eng → Backend → Auth); used by Team-scope objectives.

5. **MUST** define the `objectives` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `cycle_id UUID NOT NULL REFERENCES cycles(id) ON DELETE CASCADE`, `scope okr_scope NOT NULL` (closed 3-value enum), `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 300)`, `description TEXT`, `parent_objective_id UUID REFERENCES objectives(id) ON DELETE RESTRICT` (alignment FK — nullable for Company scope; required for Team + Member per FSM), `team_id UUID REFERENCES teams(id)` (required when scope='team'), `owner_subject_id UUID REFERENCES auth.subjects(id)` (required when scope='member'), `status objective_status NOT NULL DEFAULT 'planning'`, `created_at TIMESTAMPTZ`, `updated_at TIMESTAMPTZ`, `created_by_subject_id UUID NOT NULL`.

6. **MUST** declare the closed `okr_scope` Postgres enum with exactly 3 values (per DEC-361): `'company'`, `'team'`, `'member'`. Adding a 4th tier (e.g. `department`) is an ADR.

7. **MUST** declare the closed `objective_status` Postgres enum with 4 face-saving values: `'planning'`, `'active'`, `'closed_achieved'`, `'closed_learned'`. The terminal states `closed_achieved` (KRs met) and `closed_learned` (KRs not met — face-saving framing per DEC-364) replace conventional "completed/failed".

8. **MUST** ship the alignment-tree FSM validator (per DEC-365) at `services/okr/src/alignment/validator.rs`. Rules:
- `scope='company'` → `parent_objective_id MUST BE NULL`.
- `scope='team'` → `parent_objective_id MUST reference an objective with scope='company'` AND `team_id MUST be set`.
- `scope='member'` → `parent_objective_id MUST reference an objective with scope='team'` AND `owner_subject_id MUST be set`.
- Cross-cascade (e.g. member → company) is forbidden.
- The parent objective MUST be in the same cycle (`cycle_id`). Validated at handler boundary AND by trigger `enforce_alignment_tree`.

9. **MUST** define the `key_results` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `objective_id UUID NOT NULL REFERENCES objectives(id) ON DELETE CASCADE`, `name TEXT NOT NULL CHECK (length(name) BETWEEN 1 AND 300)`, `kr_type kr_type NOT NULL` (closed 3-value placeholder; full type validation ships in TASK-OKR-002), `start_value_numeric BIGINT` (nullable for milestone), `target_value_numeric BIGINT NOT NULL`, `current_value_numeric BIGINT NOT NULL DEFAULT 0`, `unit TEXT NOT NULL DEFAULT ''` (e.g. "VND", "%", "count", ""), `status kr_status NOT NULL DEFAULT 'on_track'` (closed 5-value face-saving enum per DEC-364), `progress_source_query TEXT` (nullable; TASK-OKR-003 ships the DSL), `last_progress_at TIMESTAMPTZ`, `created_at TIMESTAMPTZ`, `created_by_subject_id UUID NOT NULL`.

10. **MUST** declare the closed `kr_type` Postgres enum with exactly 3 values (per DEC-363): `'hit_target'`, `'improvement'`, `'milestone'`. Adding a 4th is an ADR; full per-type validation ships in TASK-OKR-002.

11. **MUST** declare the closed `kr_status` Postgres enum with exactly 5 **face-saving** values (per DEC-364): `'on_track'`, `'at_risk'`, `'learned'`, `'achieved'`, `'cycled_forward'`. The terms "missed", "failed", "behind", "delayed" MUST NOT appear in any enum value, error message, or UI string — enforced by the `face_saving_terminology_test` CI lint.

12. **MUST** enforce **3-5 KRs per Objective** at handler boundary (per DEC-362). POST `/objectives` with `initial_key_results` length < 3 or > 5 → 400 `kr_count_out_of_range`. Adding a KR that would bring count above 5 → 409 `objective_at_kr_limit`. Removing a KR that would drop count below 3 → 409 `objective_below_kr_min`.

13. **MUST** ship the `kr_progress_log` append-only table (per DEC-367) with: `id BIGSERIAL PRIMARY KEY`, `kr_id UUID NOT NULL REFERENCES key_results(id) ON DELETE RESTRICT`, `tenant_id UUID NOT NULL`, `value_numeric BIGINT NOT NULL`, `source TEXT NOT NULL CHECK (source IN ('manual','auto','check_in'))`, `rationale TEXT CHECK (rationale IS NULL OR length(rationale) BETWEEN 1 AND 1000)`, `recorded_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `recorded_by_subject_id UUID NOT NULL`. `REVOKE UPDATE, DELETE FROM cyberos_app`.

14. **MUST** ship `objective_status_history` append-only table with: `(id BIGSERIAL, objective_id UUID, tenant_id UUID, from_status objective_status, to_status objective_status, changed_at TIMESTAMPTZ, changed_by_subject_id UUID, reason TEXT)`. `REVOKE UPDATE, DELETE FROM cyberos_app`.

15. **MUST** enforce **face-saving terminology** via CI test (per DEC-364). `face_saving_terminology_test` scans `types.rs`, all migration SQL, all error messages, and the OpenAPI spec for the forbidden word list `["missed", "failed", "behind", "delayed", "fail", "miss", "behind schedule"]`. Any occurrence → CI fails. Vietnamese equivalents `["thất bại", "trễ", "không đạt"]` also forbidden.

16. **MUST** enforce RLS with both `USING` and `WITH CHECK` on cycles, teams, objectives, key_results, kr_progress_log, objective_status_history. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

17. **MUST** ship REST handlers:
- `POST /v1/okr/cycles` — create cycle (status=planning).
- `POST /v1/okr/cycles/{id}/transition` — transition status (planning → active → closing → closed).
- `GET /v1/okr/cycles?status=<>` — list with filter.
- `POST /v1/okr/teams` — create team.
- `GET /v1/okr/teams` — list.
- `POST /v1/okr/objectives` — create objective + initial KRs (3-5).
- `PATCH /v1/okr/objectives/{id}` — update non-cascade fields.
- `POST /v1/okr/objectives/{id}/transition` — status transition.
- `POST /v1/okr/key_results/{id}/progress` — record progress (manual or check_in).
- `POST /v1/okr/objectives/{id}/key_results` — add KR (must stay ≤ 5).
- `DELETE /v1/okr/objectives/{id}/key_results/{kr_id}` — remove KR (must stay ≥ 3).
- `GET /v1/okr/objectives?cycle_id=<>&scope=<>` — list with filters.

18. **MUST** emit memory audit rows for the 8 kinds (per DEC-368):
- `okr.cycle_opened` (cycle status → active).
- `okr.cycle_closed` (cycle status → closed).
- `okr.objective_created` (POST /objectives).
- `okr.objective_updated` (PATCH /objectives).
- `okr.kr_progress_recorded` (POST /key_results/{id}/progress).
- `okr.kr_status_changed` (KR status transition).
- `okr.alignment_created` (Team or Member objective with parent set).
- `okr.cycle_retro_recorded` (TASK-OKR-007 retro entries; placeholder kind at slice 1).

19. **MUST** PII-scrub `description`, `rationale`, and `name` fields via TASK-MEMORY-111 before chain commit. Tenant-scoped Postgres rows retain raw; memory chain holds scrubbed.

20. **MUST** complete handlers in ≤ 100 ms p95. `okr_perf_test` asserts.

21. **MUST** emit OTel span `okr.{cycle,team,objective,kr,progress}.{create,update,transition,...}` with `outcome` attribute (success | invalid_alignment | kr_count_out_of_range | invalid_status_transition | not_found | permission_denied | forbidden_terminology).

22. **MUST** emit OTel metrics:
- `okr_cycle_count{status, tenant_id}` (gauge).
- `okr_objective_count{scope, status, tenant_id}` (gauge).
- `okr_kr_count{status, kr_type, tenant_id}` (gauge).
- `okr_kr_progress_records_total{source}` (counter).
- `okr_objective_status_transitions_total{from_status, to_status, scope}` (counter).
- `okr_alignment_violations_total{kind}` (counter — should remain 0).

23. **MUST** ship cascading delete: deleting a `cycles` row CASCADES to objectives + key_results + history rows (per DEC-371). Deleting a `key_results` row with progress_log entries is RESTRICTED — preserve audit history. Cycles older than 5 years MAY be archived (not deleted) via a separate handler (out of scope for slice 1).

24. **MUST** record an `okr.alignment_created` memory row on every Team/Member objective creation, carrying `{objective_id, parent_objective_id, scope, alignment_depth}` where depth is 1 (team→company) or 2 (member→team→company).

25. **MUST** include an EU AI Act Art. 14 acknowledgement in the OpenAPI spec for endpoints that downstream HR/REW/LEARN modules will consume for employment decisions: every response includes a `_compliance_note` field stating "OKR data is informational only; employment decisions require human approval per EU AI Act Art. 14 + Annex III §4".

26. **MUST** validate parent objective is in same cycle on Team/Member objective creation. Cross-cycle parent → 400 `cross_cycle_alignment_forbidden`.

---

## §2 — Why this design (rationale for humans)

**Why 3-tier scope (company/team/member) closed (DEC-361)?** Doerr/Grove canonical. A 4th tier (department/division) creates organizational ambiguity — is "Backend Engineering" a team or a department? Forcing the 3-tier model collapses sub-divisions into the team hierarchy via `parent_team_id`. The 3-tier scope drives the alignment FSM cleanly; adding a 4th would require complex parent-rules.

**Why quarterly default + alternatives (DEC-360)?** Quarterly is the standard Doerr/Grove cadence; allows enough time for meaningful KRs while staying tight. Monthly cadence suits high-velocity teams in early-stage; trimester (4-month) suits enterprises with slower release cycles. The closed enum prevents drift to weird cadences ("hexamonthly" etc.).

**Why face-saving status enum (DEC-364, §1 #11, §1 #15)?** Vietnamese cultural norm prefers "what did we learn?" over "what did you miss?". The schema bakes this in: status `'learned'` replaces "failed"; `'cycled_forward'` replaces "deferred to next quarter". The CI lint enforces no English forbidden terms anywhere — preventing well-meaning developers from re-introducing "missed" via error messages. This is the "Vietnamese-cultural-fit" design assertion documented in `(Vietnamese-cultural fit).md` (TODO in source_pages).

**Why strict alignment tree (DEC-365, §1 #8)?** Cross-cascade alignments (e.g. a Member OKR directly under a Company OKR, skipping the Team) defeat the cascade's purpose. Forcing the strict Company → Team → Member tree maintains the "every member OKR rolls up to a team OKR rolls up to a company OKR" guarantee — and the rollup is the foundation for TASK-OKR-006's Monday digest.

**Why 3-5 KRs per Objective (DEC-362, §1 #12)?** Doerr/Grove industry rule of thumb — fewer than 3 means the Objective isn't operationally meaningful (too few measures); more than 5 means the team can't focus. Enforcing at handler keeps the rule explicit + visible; using a DB CHECK constraint would block transactional bootstrap (create objective + 3 KRs in one tx — Postgres deferred constraints could work but add complexity).

**Why parent_objective_id NULLABLE on Company scope (§1 #5)?** Company OKRs have no parent — they're the top of the cascade. Allowing NULL with the FSM rule (`scope='company' → parent IS NULL`) is cleaner than synthetic root entity. Trigger enforces.

**Why team_id on Team objectives + owner_subject_id on Member objectives (§1 #5, §1 #8)?** Scope alone is insufficient — we need to know WHICH team / WHICH member. Per-scope conditional required field at handler validation. The team_id references the tenant-local `teams` table; owner_subject_id references AUTH.

**Why teams as a tenant-local primitive (DEC-372, §1 #4)?** HR (TASK-HR-001) ships Member records but doesn't ship a Team primitive (deferred to task-HR-2xx). OKR needs teams now for the Team-scope objectives. Shipping the teams table here (in OKR's schema) is the pragmatic answer; future HR Team primitive can either supersede or join.

**Why append-only kr_progress_log (DEC-367, §1 #13)?** Quarterly retros depend on "what was the KR's progression over time?" — answerable only with a chained history. UPDATE in place loses prior recordings; the log preserves them. The `source` field (manual | auto | check_in) lets TASK-OKR-003's auto-progress batch distinguish its own writes from operator manual recordings.

**Why cycle_status unidirectional (DEC-366, §1 #3)?** Cycles flow `planning → active → closing → closed`. Backward transitions ("reopen a closed cycle") would corrupt historical OKR analyses. The 1% case requiring reopen routes through ADR + manual SQL with audit trail.

**Why face-saving terminology CI lint (§1 #15, DEC-364)?** Well-meaning developers writing error messages like "missed deadline" reintroduce the cultural anti-pattern. The CI lint catches at build time before merge. Enforces the design assertion mechanically rather than via review vigilance.

**Why 8 memory audit kinds split by lifecycle event (DEC-368, §1 #18)?** Operators query specific events: "show me all cycle closes this year" vs "show me all KR progress recordings this week". Split kinds give selectivity benefits at query time.

**Why EU AI Act Art. 14 acknowledgement in OpenAPI (§1 #25, DEC-369)?** OKR data drives employment decisions (promotion, performance review) downstream in HR/REW. The Act requires human-in-loop for high-risk decisions. Embedding the compliance note in every response ensures consumers can't claim they didn't know — the gate is explicit at the API contract level.

**Why parent objective MUST be in same cycle (§1 #26)?** Cross-cycle alignment (a Q3 Member OKR aligned to a Q2 Team OKR) is semantically meaningless. The alignment is "this Member OKR rolls up to this Team OKR THIS QUARTER". Enforcing at trigger prevents accidental cross-cycle setups.

**Why milestone KR uses boolean target_value_numeric (DEC-370)?** Milestones are binary (delivered or not). Storing as `target=1, current=0|1` uses the same BIGINT column for all KR types — simplifies the schema. TASK-OKR-002 ships the per-type validation that enforces this convention.

**Why cascading delete Cycle → Objectives → KRs but RESTRICT on KR if progress_log exists (DEC-371, §1 #23)?** Cycle deletion is an operator-explicit destructive action; cascading to dependent rows is expected. But KR progress_log is forensic — if you've recorded KR progress, the KR row must persist for the log to make sense. RESTRICT forces operators to either keep the KR or explicitly clear the log first (which itself requires elevated permission).

**Why `description`, `rationale`, `name` PII-scrubbed (§1 #19)?** Objective descriptions may contain employee names ("Improve Alice's onboarding time"); KR rationale during weekly check-ins may carry personal context. TASK-MEMORY-111 scrubs before memory chain commit; Postgres retains raw for in-tenant queries.

**Why slice 1 ships only the schema + handlers, not the progress DSL or check-in flow?** Split: TASK-OKR-001 = data model; TASK-OKR-003 = progress source DSL (substantial — queries against PROJ/INV/HR/LEARN); TASK-OKR-005 = weekly check-in handler. Splitting keeps this task focused on the foundational schema.

---

## §3 — API contract

### 3.1 — Migration 0001 — cycles

```sql
-- services/okr/migrations/0001_cycles.sql

BEGIN;

CREATE TYPE cycle_kind   AS ENUM ('quarterly', 'monthly', 'trimester');
CREATE TYPE cycle_status AS ENUM ('planning', 'active', 'closing', 'closed');

CREATE TABLE cycles (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    name                   TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    cycle_kind             cycle_kind   NOT NULL DEFAULT 'quarterly',
    start_date             DATE         NOT NULL,
    end_date               DATE         NOT NULL CHECK (end_date > start_date),
    status                 cycle_status NOT NULL DEFAULT 'planning',
    theme                  TEXT,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL
);

CREATE UNIQUE INDEX uniq_cycle_name ON cycles (tenant_id, name);
CREATE INDEX cycles_status_idx ON cycles (tenant_id, status);

ALTER TABLE cycles ENABLE ROW LEVEL SECURITY;
CREATE POLICY cycles_tenant_isolation ON cycles
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Unidirectional status FSM
CREATE OR REPLACE FUNCTION enforce_cycle_status_fsm() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = OLD.status THEN RETURN NEW; END IF;
    IF NOT (
        (OLD.status = 'planning' AND NEW.status = 'active')
        OR (OLD.status = 'active'   AND NEW.status = 'closing')
        OR (OLD.status = 'closing'  AND NEW.status = 'closed')
    ) THEN
        RAISE EXCEPTION 'invalid_cycle_status_transition: % -> %', OLD.status, NEW.status USING ERRCODE = 'P0060';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_cycle_status_fsm BEFORE UPDATE ON cycles
    FOR EACH ROW EXECUTE FUNCTION enforce_cycle_status_fsm();

COMMIT;
```

### 3.2 — Migration 0002 — teams

```sql
-- services/okr/migrations/0002_teams.sql

BEGIN;

CREATE TABLE teams (
    id                UUID         PRIMARY KEY,
    tenant_id         UUID         NOT NULL,
    name              TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 100),
    parent_team_id    UUID         REFERENCES teams(id) ON DELETE RESTRICT,
    lead_subject_id   UUID         REFERENCES auth.subjects(id),
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uniq_team_name ON teams (tenant_id, name);
CREATE INDEX teams_parent_idx ON teams (parent_team_id) WHERE parent_team_id IS NOT NULL;

ALTER TABLE teams ENABLE ROW LEVEL SECURITY;
CREATE POLICY teams_tenant_isolation ON teams
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.3 — Migration 0003 — objectives

```sql
-- services/okr/migrations/0003_objectives.sql

BEGIN;

CREATE TYPE okr_scope        AS ENUM ('company', 'team', 'member');
CREATE TYPE objective_status AS ENUM ('planning', 'active', 'closed_achieved', 'closed_learned');

CREATE TABLE objectives (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    cycle_id               UUID         NOT NULL REFERENCES cycles(id) ON DELETE CASCADE,
    scope                  okr_scope    NOT NULL,
    name                   TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 300),
    description            TEXT,
    parent_objective_id    UUID         REFERENCES objectives(id) ON DELETE RESTRICT,
    team_id                UUID         REFERENCES teams(id),
    owner_subject_id       UUID         REFERENCES auth.subjects(id),
    status                 objective_status NOT NULL DEFAULT 'planning',
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL
);

CREATE INDEX objectives_cycle_scope_idx ON objectives (tenant_id, cycle_id, scope);
CREATE INDEX objectives_parent_idx ON objectives (parent_objective_id) WHERE parent_objective_id IS NOT NULL;
CREATE INDEX objectives_team_idx ON objectives (team_id) WHERE team_id IS NOT NULL;
CREATE INDEX objectives_owner_idx ON objectives (owner_subject_id) WHERE owner_subject_id IS NOT NULL;

ALTER TABLE objectives ENABLE ROW LEVEL SECURITY;
CREATE POLICY objectives_tenant_isolation ON objectives
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Alignment tree FSM (DEC-365)
CREATE OR REPLACE FUNCTION enforce_alignment_tree() RETURNS TRIGGER AS $$
DECLARE parent RECORD;
BEGIN
    -- Company scope: no parent + no team + no owner
    IF NEW.scope = 'company' THEN
        IF NEW.parent_objective_id IS NOT NULL THEN
            RAISE EXCEPTION 'company_objective_has_no_parent' USING ERRCODE = 'P0070';
        END IF;
        RETURN NEW;
    END IF;
    -- Team scope: must have parent (must be company scope) + must have team_id
    IF NEW.scope = 'team' THEN
        IF NEW.parent_objective_id IS NULL THEN
            RAISE EXCEPTION 'team_objective_requires_parent' USING ERRCODE = 'P0071';
        END IF;
        IF NEW.team_id IS NULL THEN
            RAISE EXCEPTION 'team_objective_requires_team_id' USING ERRCODE = 'P0072';
        END IF;
        SELECT scope, cycle_id INTO parent FROM objectives WHERE id = NEW.parent_objective_id;
        IF NOT FOUND OR parent.scope != 'company' THEN
            RAISE EXCEPTION 'team_objective_parent_must_be_company' USING ERRCODE = 'P0073';
        END IF;
        IF parent.cycle_id != NEW.cycle_id THEN
            RAISE EXCEPTION 'cross_cycle_alignment_forbidden' USING ERRCODE = 'P0074';
        END IF;
        RETURN NEW;
    END IF;
    -- Member scope: must have parent (must be team scope) + must have owner_subject_id
    IF NEW.scope = 'member' THEN
        IF NEW.parent_objective_id IS NULL THEN
            RAISE EXCEPTION 'member_objective_requires_parent' USING ERRCODE = 'P0075';
        END IF;
        IF NEW.owner_subject_id IS NULL THEN
            RAISE EXCEPTION 'member_objective_requires_owner' USING ERRCODE = 'P0076';
        END IF;
        SELECT scope, cycle_id INTO parent FROM objectives WHERE id = NEW.parent_objective_id;
        IF NOT FOUND OR parent.scope != 'team' THEN
            RAISE EXCEPTION 'member_objective_parent_must_be_team' USING ERRCODE = 'P0077';
        END IF;
        IF parent.cycle_id != NEW.cycle_id THEN
            RAISE EXCEPTION 'cross_cycle_alignment_forbidden' USING ERRCODE = 'P0074';
        END IF;
        RETURN NEW;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_objectives_alignment BEFORE INSERT OR UPDATE ON objectives
    FOR EACH ROW EXECUTE FUNCTION enforce_alignment_tree();

COMMIT;
```

### 3.4 — Migration 0004 — key_results

```sql
-- services/okr/migrations/0004_key_results.sql

BEGIN;

CREATE TYPE kr_type   AS ENUM ('hit_target', 'improvement', 'milestone');
CREATE TYPE kr_status AS ENUM ('on_track', 'at_risk', 'learned', 'achieved', 'cycled_forward');

CREATE TABLE key_results (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    objective_id           UUID         NOT NULL REFERENCES objectives(id) ON DELETE CASCADE,
    name                   TEXT         NOT NULL CHECK (length(name) BETWEEN 1 AND 300),
    kr_type                kr_type      NOT NULL,
    start_value_numeric    BIGINT,
    target_value_numeric   BIGINT       NOT NULL,
    current_value_numeric  BIGINT       NOT NULL DEFAULT 0,
    unit                   TEXT         NOT NULL DEFAULT '',
    status                 kr_status    NOT NULL DEFAULT 'on_track',
    progress_source_query  TEXT,
    last_progress_at       TIMESTAMPTZ,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL
);

CREATE INDEX key_results_obj_idx ON key_results (objective_id);
CREATE INDEX key_results_status_idx ON key_results (tenant_id, status);

ALTER TABLE key_results ENABLE ROW LEVEL SECURITY;
CREATE POLICY key_results_tenant_isolation ON key_results
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

COMMIT;
```

### 3.5 — Migration 0005 — append-only progress log

```sql
-- services/okr/migrations/0005_progress_log.sql

BEGIN;

CREATE TABLE kr_progress_log (
    id                     BIGSERIAL    PRIMARY KEY,
    kr_id                  UUID         NOT NULL REFERENCES key_results(id) ON DELETE RESTRICT,
    tenant_id              UUID         NOT NULL,
    value_numeric          BIGINT       NOT NULL,
    source                 TEXT         NOT NULL CHECK (source IN ('manual','auto','check_in')),
    rationale              TEXT         CHECK (rationale IS NULL OR length(rationale) BETWEEN 1 AND 1000),
    recorded_at            TIMESTAMPTZ  NOT NULL DEFAULT now(),
    recorded_by_subject_id UUID         NOT NULL
);

CREATE INDEX progress_log_kr_idx ON kr_progress_log (kr_id, recorded_at DESC);
CREATE INDEX progress_log_tenant_recorded_idx ON kr_progress_log (tenant_id, recorded_at DESC);

ALTER TABLE kr_progress_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY progress_log_tenant_isolation ON kr_progress_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON kr_progress_log FROM cyberos_app;

COMMIT;
```

### 3.6 — Rust types

```rust
// services/okr/src/types.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "okr_scope", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OkrScope { Company, Team, Member }

impl OkrScope { pub const ALL: &'static [OkrScope] = &[OkrScope::Company, OkrScope::Team, OkrScope::Member]; }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "cycle_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CycleStatus { Planning, Active, Closing, Closed }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "objective_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveStatus { Planning, Active, ClosedAchieved, ClosedLearned }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "kr_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum KrType { HitTarget, Improvement, Milestone }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "kr_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum KrStatus { OnTrack, AtRisk, Learned, Achieved, CycledForward }

impl KrStatus {
    pub const ALL: &'static [KrStatus] = &[
        KrStatus::OnTrack, KrStatus::AtRisk, KrStatus::Learned, KrStatus::Achieved, KrStatus::CycledForward,
    ];
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Cycle {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub cycle_kind: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: CycleStatus,
    pub theme: Option<String>,
    pub created_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct Objective {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub cycle_id: Uuid,
    pub scope: OkrScope,
    pub name: String,
    pub description: Option<String>,
    pub parent_objective_id: Option<Uuid>,
    pub team_id: Option<Uuid>,
    pub owner_subject_id: Option<Uuid>,
    pub status: ObjectiveStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct KeyResult {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub objective_id: Uuid,
    pub name: String,
    pub kr_type: KrType,
    pub start_value_numeric: Option<i64>,
    pub target_value_numeric: i64,
    pub current_value_numeric: i64,
    pub unit: String,
    pub status: KrStatus,
    pub progress_source_query: Option<String>,
    pub last_progress_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}
```

### 3.7 — Face-saving terminology lint

```rust
// services/okr/tests/face_saving_terminology_test.rs
use std::path::Path;
use regex::Regex;

const FORBIDDEN_TERMS: &[&str] = &[
    "missed", "failed", "behind", "delayed", " miss ", " fail ",
    "thất bại", "trễ", "không đạt",
];

#[test]
fn no_forbidden_terminology_in_module() {
    let scan_dirs = [
        "src", "migrations", "tests/fixtures",
    ];
    let mut violations: Vec<(String, String)> = Vec::new();
    for dir in scan_dirs {
        let root = Path::new(dir);
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                let content = std::fs::read_to_string(entry.path()).unwrap_or_default();
                let lower = content.to_lowercase();
                for term in FORBIDDEN_TERMS {
                    if lower.contains(&term.to_lowercase()) {
                        violations.push((entry.path().display().to_string(), term.to_string()));
                    }
                }
            }
        }
    }
    assert!(violations.is_empty(),
        "Face-saving terminology violated; reintroduces blame language:\n{:#?}", violations);
}
```

---

## §4 — Acceptance criteria

1. **All 5 OKR enums closed at stated cardinalities** — OkrScope (3), CycleStatus (4), ObjectiveStatus (4), KrType (3), KrStatus (5).
2. **RLS isolates by tenant** — cross-tenant queries return 0 rows.
3. **POST cycle happy path** — status=planning created; UNIQUE on `(tenant_id, name)`.
4. **Cycle backward transition rejected** — `active → planning` raises invalid_cycle_status_transition.
5. **Cycle status forward transitions allowed** — planning → active → closing → closed.
6. **Company objective without parent** — accepted.
7. **Company objective with parent** → 400 company_objective_has_no_parent.
8. **Team objective without parent** → 400 team_objective_requires_parent.
9. **Team objective without team_id** → 400 team_objective_requires_team_id.
10. **Team objective with member-scope parent** → 400 team_objective_parent_must_be_company.
11. **Member objective without owner_subject_id** → 400 member_objective_requires_owner.
12. **Cross-cycle alignment** → 400 cross_cycle_alignment_forbidden.
13. **Objective with 2 KRs** → 400 kr_count_out_of_range.
14. **Objective with 6 KRs** → 400 kr_count_out_of_range.
15. **Adding 6th KR to objective at limit** → 409 objective_at_kr_limit.
16. **Removing 3rd KR (down to 2)** → 409 objective_below_kr_min.
17. **KR status closed at exactly 5 face-saving values** — `KrStatus::ALL.len() == 5`; no "missed/failed/behind".
18. **Face-saving terminology CI lint** — adding "missed" anywhere in src/migrations/tests → CI fails.
19. **kr_progress_log append-only** — UPDATE/DELETE blocked from cyberos_app.
20. **objective_status_history append-only** — same.
21. **POST progress emits `okr.kr_progress_recorded` memory row**.
22. **Cycle delete cascades** — deleting cycle deletes all child objectives + KRs.
23. **KR delete with progress_log entries** → RESTRICT.
24. **OTel span `okr.objective.create` emitted** — outcome=success.
25. **OTel counter `okr_objective_count{scope=team}` increments**.
26. **OTel counter `okr_alignment_violations_total` is 0 on clean tenant** — should never increment in normal use.
27. **OpenAPI compliance note** — every endpoint response includes `_compliance_note` per EU AI Act Art. 14.

---

## §5 — Verification

```rust
// services/okr/tests/alignment_tree_test.rs
#[sqlx::test]
async fn company_with_parent_rejected(pool: sqlx::PgPool) {
    let cycle = seed_cycle(&pool).await;
    let company = seed_company_objective(&pool, cycle).await;
    // Attempt: create another "company" objective pointing at the first
    let err = sqlx::query("INSERT INTO objectives (id, tenant_id, cycle_id, scope, name, parent_objective_id, created_by_subject_id) VALUES ($1, $2, $3, 'company'::okr_scope, 'Bad', $4, $5)")
        .bind(Uuid::new_v4()).bind(test_tenant()).bind(cycle).bind(company).bind(test_subject())
        .execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("company_objective_has_no_parent"));
}

#[sqlx::test]
async fn team_parent_must_be_company(pool: sqlx::PgPool) {
    let cycle = seed_cycle(&pool).await;
    let company = seed_company_objective(&pool, cycle).await;
    let team_team = seed_team(&pool).await;
    let team_obj = seed_team_objective(&pool, cycle, team_team, company).await;
    // Attempt: another team objective pointing at the team (not company) parent
    let err = sqlx::query("INSERT INTO objectives (id, tenant_id, cycle_id, scope, name, parent_objective_id, team_id, created_by_subject_id) VALUES ($1, $2, $3, 'team'::okr_scope, 'Bad', $4, $5, $6)")
        .bind(Uuid::new_v4()).bind(test_tenant()).bind(cycle).bind(team_obj).bind(team_team).bind(test_subject())
        .execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("team_objective_parent_must_be_company"));
}
```

```rust
// services/okr/tests/kr_count_bounds_test.rs
#[tokio::test]
async fn objective_with_2_krs_rejected(ctx: TestCtx) {
    let resp = ctx.post_objective_with_n_krs(2).await;
    assert_eq!(resp.status(), 400);
    assert_eq!(resp.json::<serde_json::Value>().await.unwrap()["error"], "kr_count_out_of_range");
}

#[tokio::test]
async fn objective_with_6_krs_rejected(ctx: TestCtx) {
    let resp = ctx.post_objective_with_n_krs(6).await;
    assert_eq!(resp.status(), 400);
}
```

```rust
// services/okr/tests/cycle_status_fsm_test.rs
#[sqlx::test]
async fn backward_transition_rejected(pool: sqlx::PgPool) {
    let cycle = seed_cycle_at_status(&pool, "active").await;
    let err = sqlx::query("UPDATE cycles SET status = 'planning'::cycle_status WHERE id = $1")
        .bind(cycle).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("invalid_cycle_status_transition"));
}

#[sqlx::test]
async fn forward_transition_accepted(pool: sqlx::PgPool) {
    let cycle = seed_cycle_at_status(&pool, "planning").await;
    sqlx::query("UPDATE cycles SET status = 'active'::cycle_status WHERE id = $1").bind(cycle).execute(&pool).await.unwrap();
    sqlx::query("UPDATE cycles SET status = 'closing'::cycle_status WHERE id = $1").bind(cycle).execute(&pool).await.unwrap();
    sqlx::query("UPDATE cycles SET status = 'closed'::cycle_status WHERE id = $1").bind(cycle).execute(&pool).await.unwrap();
}
```

```rust
// services/okr/tests/append_only_progress_test.rs
#[sqlx::test]
async fn progress_log_immutable_from_app(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let row_id = seed_progress(&pool).await;
    let err = sqlx::query("UPDATE kr_progress_log SET value_numeric = 999 WHERE id = $1")
        .bind(row_id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 8 memory row builders follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-003** — RLS enforcement.
- **TASK-AUTH-101** — RBAC (`Resource::OkrObjective`, `OkrKr`).

**Downstream (3 placeholders):**
- **TASK-OKR-002** — full KR type validation (per-type rules for hit_target / improvement / milestone).
- **TASK-OKR-003** — progress_source DSL.
- **TASK-OKR-005** — weekly check-in handler (writes to kr_progress_log with source='check_in').

**Cross-module:**
- **TASK-AI-003** — memory audit bridge.
- **TASK-MEMORY-111** — PII scrubbing.
- **TASK-HR-001** — Member subject_id referenced by Member-scope objectives.

---

## §8 — Example payloads

### 8.1 — POST /v1/okr/cycles

```json
{
  "name": "Q3 2026",
  "cycle_kind": "quarterly",
  "start_date": "2026-07-01",
  "end_date": "2026-09-30",
  "theme": "International expansion"
}
```

### 8.2 — POST /v1/okr/objectives (Team scope with 3 KRs)

```json
{
  "cycle_id": "<cycle uuid>",
  "scope": "team",
  "name": "Backend Engineering — ship multi-region by end of Q3",
  "parent_objective_id": "<company-objective uuid>",
  "team_id": "<backend-team uuid>",
  "initial_key_results": [
    {"name": "Deploy to SG-1 region", "kr_type": "milestone", "target_value_numeric": 1, "unit": ""},
    {"name": "Region failover RTO ≤ 60s", "kr_type": "hit_target", "target_value_numeric": 60, "unit": "seconds"},
    {"name": "Move p95 cross-region latency from 200ms to 50ms", "kr_type": "improvement", "start_value_numeric": 200, "target_value_numeric": 50, "unit": "ms"}
  ]
}
```

### 8.3 — okr.objective_created memory row

```json
{
  "kind": "okr.objective_created",
  "tenant_id": "5e8f1d2a-...",
  "objective_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "cycle_id": "<uuid>",
  "scope": "team",
  "name_scrubbed": "Backend Engineering — [REDACTED-PROJECT]",
  "parent_objective_id": "<uuid>",
  "team_id": "<uuid>",
  "kr_count": 3,
  "ts_ns": 1747920731000000000
}
```

### 8.4 — POST /v1/okr/key_results/{id}/progress

```json
{
  "value_numeric": 75,
  "source": "manual",
  "rationale": "Closed 25 of 30 issues in proposal; remaining 5 in review"
}
```

---

## §9 — Open questions

Deferred:
- **Full KR per-type validation** — TASK-OKR-002 (hit_target ≥ baseline; improvement target ≠ start; milestone target_value = 1).
- **Progress source DSL** — TASK-OKR-003 (queries against PROJ/INV/HR/LEARN).
- **Auto-progress nightly batch** — TASK-OKR-004.
- **Weekly check-in handler** — TASK-OKR-005.
- **Monday digest** — TASK-OKR-006.
- **Quarterly retro draft** — TASK-OKR-007.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| RLS bypass | `USING` predicate | 0 rows | None — designed |
| Cycle backward transition | trigger | 400 invalid_cycle_status_transition | Designed |
| Company with parent | trigger | 400 company_objective_has_no_parent | Designed |
| Team without parent | trigger | 400 team_objective_requires_parent | Designed |
| Team without team_id | trigger | 400 team_objective_requires_team_id | Designed |
| Team with non-company parent | trigger | 400 team_objective_parent_must_be_company | Designed |
| Member without owner | trigger | 400 member_objective_requires_owner | Designed |
| Member with non-team parent | trigger | 400 member_objective_parent_must_be_team | Designed |
| Cross-cycle alignment | trigger | 400 cross_cycle_alignment_forbidden | Designed |
| KR count < 3 | handler | 400 kr_count_out_of_range | Add KRs |
| KR count > 5 | handler | 400 kr_count_out_of_range | Remove KRs |
| Add 6th KR | handler | 409 objective_at_kr_limit | Designed |
| Remove last 3rd KR | handler | 409 objective_below_kr_min | Designed |
| Face-saving violation introduced | CI lint | Build fails | Use approved terminology |
| kr_progress_log UPDATE/DELETE from app | SQL grant | permission denied | Designed |
| objective_status_history UPDATE/DELETE | SQL grant | permission denied | Designed |
| Cycle delete cascades to objectives + KRs | ON DELETE CASCADE | Designed | None |
| KR delete with progress_log entries | FK RESTRICT | DELETE fails | Clear progress_log first (elevated perm) |
| Duplicate cycle name | UNIQUE | INSERT fails | Use different name |
| Duplicate team name | UNIQUE | INSERT fails | Use different name |
| end_date <= start_date | DB CHECK | INSERT fails | Use correct dates |
| memory audit fail mid-tx | rollback | 500 audit_failed | memory_writer health |
| PII not scrubbed | TASK-MEMORY-111 + CI test | Pre-commit failure | Add rule |
| Parent objective deleted while children exist | FK RESTRICT | DELETE fails | Reassign children first |
| Team deleted while objectives reference | FK RESTRICT | DELETE fails | Reassign |
| Owner subject deleted while objectives reference | FK RESTRICT | DELETE fails | Reassign owner |
| Concurrent KR add/remove on same objective | Postgres serialisable | One wins; second sees 409 | Caller refetches |
| Progress recorded with bad source value | DB CHECK | INSERT fails | Use manual/auto/check_in |
| Rationale > 1000 chars | DB CHECK | INSERT fails | Shorten |
| Cycle name > 100 chars | DB CHECK | INSERT fails | Shorten |
| OpenAPI compliance note missing | spec lint | CI fails | Add note |
| Trigger error code drift | tests assert specific codes | CI fails | Restore codes |

---

## §11 — Implementation notes

- **Doerr/Grove canonical** — 3-tier cascade + 3-5 KRs + quarterly cycles. Closed enums prevent organisational drift.
- **Face-saving terminology baked into schema + CI lint** — Vietnamese cultural adaptation enforced mechanically.
- **Alignment tree at trigger** — defense in depth; handler validates too.
- **Tenant-local teams primitive** — task-HR ships Members; OKR ships Teams (no HR Team primitive yet).
- **Append-only progress_log via SQL grant** — quarterly retros depend on the full history.
- **EU AI Act Art. 14 acknowledgement in OpenAPI** — high-risk-adjacent module; human-in-loop is the contract.
- **Cascading delete Cycle → Objectives → KRs** — operator-explicit destructive action.
- **KR RESTRICT on progress_log existence** — preserve audit history.
- **Milestone as boolean (target=1)** — same column for all KR types; per-type rules in TASK-OKR-002.
- **Cross-cycle alignment forbidden at trigger** — semantic correctness.
- **`source` enum {manual, auto, check_in}** — distinguishes auto-progress batch writes (TASK-OKR-004) from operator + check-in writes.
- **PII scrubbing on description + rationale + name** — quarterly retros may carry personal context.
- **3 closed kr_type enum** — hit_target / improvement / milestone (Doerr canonical).
- **5 closed kr_status face-saving values** — on_track / at_risk / learned / achieved / cycled_forward.
- **Cycle status unidirectional** — forward only.
- **8 memory audit kinds split by lifecycle event** — selectivity at query time.
- **Forbidden terminology list bilingual** — Vietnamese equivalents covered.
- **OpenAPI `_compliance_note` mandatory** — embedded in every response for AI Act discoverability.

---

*End of TASK-OKR-001.*
