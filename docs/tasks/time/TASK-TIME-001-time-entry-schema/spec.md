---
id: TASK-TIME-001
title: "TIME TimeEntry append-only schema — correction_to link semantics + tenant-scoped RLS + invoice-grade integrity"
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
module: TIME
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-PROJ-001, TASK-TIME-002, TASK-TIME-003, TASK-TIME-005, TASK-TIME-006, TASK-TIME-007, TASK-TIME-009, TASK-HR-001, TASK-HR-008]
depends_on: [TASK-AUTH-003, TASK-AUTH-101]
# 9 downstream consumers
blocks: [TASK-TIME-002, TASK-TIME-003, TASK-TIME-005, TASK-TIME-006, TASK-TIME-007, TASK-TIME-009, TASK-HR-008, TASK-LEARN-003, TASK-RES-001]

source_pages:
  - website/docs/modules/time.html#what
  - website/docs/modules/time.html#data-model
  - website/docs/modules/time.html#audit-model
source_decisions:
  - DEC-220 (append-only at audit layer; mutations write a fresh row with correction_to pointing at the prior row id)
  - DEC-221 (timestamps stored as TIMESTAMPTZ + duration as INT minutes — never as separate start/end on the wire; UI may render as start+end)
  - DEC-222 (entry is bound to (engagement_id, issue_id) — issue is REQUIRED at slice 1; standalone-engagement entries deferred to slice 2)
  - DEC-223 (billable flag is materialised on the row but COMPUTED by TASK-TIME-005 — this task declares the column as `billable BOOLEAN NOT NULL DEFAULT false` with no constraint until TASK-TIME-005 ships the cascade)
  - DEC-224 (rate_card snapshot is JSONB on the row — never a foreign key; rate-card mutations don't shift past entries per the snapshot pattern)
  - DEC-225 (correction_to is a self-FK; the chain is acyclic — enforced by trigger + CI test)
  - DEC-226 (one row supersedes at most one row; multiple corrections to the same original create a chain, not a tree)
  - DEC-227 (entries < 1 minute are forbidden — minimum_minutes = 1; capacity-planning floor)
  - DEC-228 (entries > 24 hours are forbidden — a single entry cannot span a calendar day; the daily cap is enforced by TASK-TIME-007 across rows)
  - DEC-229 (entry currency = engagement.invoice_currency at row creation time; snapshotted on the row; multi-currency invoice math happens at TASK-INV-001)
  - DEC-230 (REVOKE UPDATE, DELETE on time_entries from cyberos_app; corrections are INSERTs not UPDATEs — append-only enforced by SQL grant)
  - DEC-231 (memory audit kinds: time.entry_recorded, time.entry_corrected, time.entry_submitted_for_approval — submission is TASK-TIME-006's responsibility but the kind is reserved here)
  - VN Labour Code Art. 107 (300h/yr OT cap — enforced by TASK-TIME-007, not this task)
  - Decree 145/2020 Art. 105 (40h regular week — enforced by TASK-TIME-007)
  - PDPL Art. 13 (data minimisation — entry rows store description as PII-scrubbable but not categorically banned; TASK-TIME-006 audit row carries scrubbed form)
  - ISO 27001:2022 A.12.4 (audit logging — append-only chain satisfies)

language: rust 1.81 + sql
service: cyberos/services/time/
new_files:
  # time_entries table + ENUM kind + RLS + correction_to FK + REVOKE writes + duration CHECK
  - services/time/migrations/0001_time_entries.sql
  # current_time_entries_view (effective rows = no superseder) + entry_chain_walker function
  - services/time/migrations/0002_time_entries_view.sql
  # crate root
  - services/time/src/lib.rs
  # TimeEntry struct + EntryKind enum (regular | overtime | weekend | holiday) + EntryStatus (draft | submitted | approved | reverted)
  - services/time/src/types.rs
  # repository — create + get + list + correct_via_new_row
  - services/time/src/repo/entries.rs
  # correction-chain walker; detect cycles; return head + tail
  - services/time/src/chain.rs
  # duration bounds (1 ≤ minutes ≤ 1440); ts_end > ts_start; correction_to references same engagement
  - services/time/src/validation.rs
  # canonical time.entry_recorded + time.entry_corrected memory row builders
  - services/time/src/audit/entry_events.rs
  # POST/GET /v1/time/entries + POST /v1/time/entries/{id}/correct
  - services/time/src/handlers/entries.rs
  # +sqlx, +uuid, +serde, +chrono, +rust_decimal, +async-trait, +cyberos-cli-exit
  - services/time/Cargo.toml
  # happy + invalid duration + invalid kind + cross-tenant + idempotent
  - services/time/tests/entries_create_test.rs
  # correction creates new row pointing at prior; original kept
  - services/time/tests/entries_correct_test.rs
  # chain walks; tree-form (two correctors of same parent) rejected
  - services/time/tests/correction_chain_test.rs
  # A → B → A cycle rejected at trigger
  - services/time/tests/correction_acyclic_test.rs
  # UPDATE/DELETE rejected by SQL grant
  - services/time/tests/append_only_test.rs
  # tenant-A cannot see tenant-B entries
  - services/time/tests/rls_isolation_test.rs
  # < 1 minute and > 1440 minutes rejected
  - services/time/tests/duration_bounds_test.rs
  # superseded rows omitted; corrections visible
  - services/time/tests/current_view_test.rs
  # rate-card change does not retroactively alter past entries
  - services/time/tests/rate_card_snapshot_test.rs
  # every create/correct emits exactly one memory row
  - services/time/tests/audit_row_test.rs
  # slice-1 default is false; TASK-TIME-005 will set via cascade
  - services/time/tests/billable_default_test.rs
modified_files:
  # add `(engagement_id, issue_id)` GIN index to support TIME entry FK joins (read-only join target)
  - services/proj/migrations/0010_issues_addendum.sql

allowed_tools:
  - file_read: services/time/**
  - file_read: services/proj/**
  - file_write: services/time/{src,tests,migrations}/**
  - bash: cd services/time && cargo test
  - bash: psql -f services/time/migrations/0001_time_entries.sql (local Postgres only)

disallowed_tools:
  - allow UPDATE or DELETE on time_entries (per DEC-230 — append-only is SQL-grant-enforced)
  #11)
  - allow correction_to pointing at an entry in a different tenant or different engagement (per §1
  - allow correction creating a tree (two rows correcting the same parent) — chain only (per DEC-226)
  - allow durations < 1 minute or > 1440 minutes per row (per DEC-227 + DEC-228)
  - hard-code billable cascade logic in this task — that is TASK-TIME-005's responsibility (per DEC-223)
  - enforce VN Labour Code OT caps at this task — that is TASK-TIME-007's responsibility (per spec scope)

effort_hours: 5
subtasks:
  - "0.5h: 0001_time_entries.sql — time_entries table + EntryKind enum + correction_to self-FK + RLS + REVOKE UPDATE/DELETE + duration CHECK"
  - "0.4h: 0002_time_entries_view.sql — current_time_entries_view + entry_chain_walker SQL function"
  - "0.3h: types.rs — TimeEntry struct + 2 closed enums (EntryKind 4, EntryStatus 4)"
  - "0.4h: repo/entries.rs — create + get + list + correct_via_new_row (transaction-wrapped)"
  - "0.4h: chain.rs — walker function; cycle detector; head/tail computation"
  - "0.3h: validation.rs — duration bounds + ts_end > ts_start + same-engagement correction_to"
  - "0.3h: audit/entry_events.rs — 2 row builders (recorded, corrected)"
  - "0.5h: handlers/entries.rs — 3 REST endpoints"
  - "1.9h: tests — 11 test files covering append-only enforcement, RLS, correction chain semantics, cycle rejection, duration bounds, view filtering"

risk_if_skipped: "TIME is invoice-grade infrastructure — one missed entry breaks a client invoice. Every downstream task (TASK-TIME-002 timer UI, TASK-TIME-003 manual entry form, TASK-TIME-005 billable cascade, TASK-TIME-006 weekly approval, TASK-TIME-007 OT cap enforcement, TASK-TIME-009 per-cycle rollup, TASK-INV-001 invoice draft from rollup) reads from this schema. Without DEC-220's append-only audit, mutations could silently rewrite past hours — every audit chain claim breaks. Without DEC-225's acyclic correction enforcement, the chain becomes a graph and 'what is the current value?' becomes ambiguous. Without DEC-224's rate-card snapshot, a CFO bumping the rate card retroactively would silently shift every historic billable hour's amount — invoices on the books would diverge from the system. Without DEC-228's per-row duration cap, a single buggy entry could log a year of hours; daily caps don't help if one row spans a year. The 5h effort defends the integrity at the row level; TASK-TIME-005 onwards build on the guarantees."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship the TimeEntry schema as the canonical append-only record of "Member X spent N minutes on issue Y at time Z". Each requirement:

1. **MUST** define the `time_entries` table with the following columns (full DDL in §3.1):
    - `id UUID PRIMARY KEY` — row identity.
    - `tenant_id UUID NOT NULL` — RLS partitioning key.
    - `member_subject_id UUID NOT NULL REFERENCES auth.subjects(id)` — who performed the work.
    - `engagement_id UUID NOT NULL` — the engagement billable target (per TASK-PROJ-005; placeholder FK at slice 1).
    - `issue_id UUID NOT NULL` — the specific issue (per TASK-PROJ-001).
    - `ts_start TIMESTAMPTZ NOT NULL` — entry start in UTC.
    - `duration_minutes INT NOT NULL CHECK (duration_minutes BETWEEN 1 AND 1440)` — minimum 1 minute, maximum 24 hours (per DEC-227 + DEC-228).
    - `entry_kind entry_kind NOT NULL` — closed enum: `regular | overtime | weekend | holiday`.
    - `entry_status entry_status NOT NULL DEFAULT 'draft'` — closed enum: `draft | submitted | approved | reverted` (TASK-TIME-006 transitions).
    - `billable BOOLEAN NOT NULL DEFAULT false` — set by TASK-TIME-005's cascade; this task declares the column only (per DEC-223).
    - `rate_card_snapshot JSONB` — populated at entry creation by TASK-TIME-005; snapshot of the engagement rate-card at the row's instant (per DEC-224); empty `{}` at slice 1.
    - `entry_currency CHAR(3) NOT NULL` — ISO-4217; defaulted from engagement.invoice_currency at row creation (per DEC-229).
    - `description TEXT` — nullable; 0–1000 chars; PII-scrubbable.
    - `correction_to UUID REFERENCES time_entries(id) DEFERRABLE INITIALLY IMMEDIATE` — nullable; non-null on correction rows (per DEC-220 + DEC-225).
    - `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`.
    - `created_by_subject_id UUID NOT NULL REFERENCES auth.subjects(id)`.

2. **MUST** enforce RLS with both `USING` and `WITH CHECK` (task-audit skill rule 13). Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`. Cross-tenant reads return 0 rows; cross-tenant writes fail `permission_denied`.

3. **MUST** declare the closed `entry_kind` Postgres enum with exactly 4 values (per VN-1 working-time classification): `'regular'`, `'overtime'`, `'weekend'`, `'holiday'`. Adding a 5th is an ADR. Holiday-list driving the kind defaulting is TASK-TIME-007's responsibility; this task just stores the column.

4. **MUST** declare the closed `entry_status` Postgres enum with exactly 4 values: `'draft'`, `'submitted'`, `'approved'`, `'reverted'`. State transitions are TASK-TIME-006's responsibility; this task ships the column at default `'draft'`.

5. **MUST** be **append-only** at the SQL grant layer (per DEC-230 + task-audit skill rule 12). Migration applies `REVOKE UPDATE, DELETE ON time_entries FROM cyberos_app;`. Mutations write a fresh row with `correction_to` pointing at the prior row (per §1 #6 below).

6. **MUST** support **correction via new row** (per DEC-220). The handler `POST /v1/time/entries/{id}/correct` creates a new row with:
    - `correction_to = <prior_id>`.
    - All other fields from the body OR copied from prior if unspecified.
    - `tenant_id`, `engagement_id`, `issue_id`, `member_subject_id` MUST be identical to the prior row (per §1 #11). Cross-engagement correction is forbidden.
    - The same audit emission contract as `create` but with kind `time.entry_corrected`.
   Corrections are themselves correctable (chains, not trees — per DEC-226).

7. **MUST** enforce **acyclic correction chains** (per DEC-225). A `BEFORE INSERT` trigger walks `correction_to` upward; if the walk visits the new row's `id` (via a future re-correction back), the insert is rejected with `correction_cycle_detected`. A CI test (`correction_acyclic_test`) seeds a 5-row chain + an attempted cycle and asserts the reject.

8. **MUST** enforce **chain (not tree) topology** (per DEC-226). At most one row may have `correction_to = <prior_id>` for any given `prior_id`. A `BEFORE INSERT` trigger checks `EXISTS (SELECT 1 FROM time_entries WHERE correction_to = $new.correction_to)`; if so → reject with `prior_row_already_corrected`. Reasoning: "what is the current value of entry X?" must be unambiguous; trees create N parallel current values.

9. **MUST** ship the `current_time_entries_view` SQL view filtering to "effective rows only" — rows that no other row supersedes via `correction_to`. Definition: `SELECT * FROM time_entries WHERE id NOT IN (SELECT correction_to FROM time_entries WHERE correction_to IS NOT NULL)`. Downstream tasks (TASK-TIME-005 billable cascade, TASK-TIME-009 rollup) MUST read from the view, not the raw table.

10. **MUST** validate at API layer:
    - `duration_minutes` between 1 and 1440 (per DEC-227 + DEC-228; the DB CHECK constraint duplicates).
    - `ts_start` not in the future (clock-skew tolerance: +5 minutes; entries beyond that → 400 `ts_start_in_future`).
    - `entry_kind` parses to closed enum (`unknown_entry_kind` otherwise).
    - On correction: `correction_to` row exists, belongs to the same tenant + engagement + issue + member.
    - `description` length 0–1000 chars.

11. **MUST** reject corrections where any of `tenant_id`, `engagement_id`, `issue_id`, `member_subject_id` differ from the prior row. A trigger `enforce_correction_inheritance` raises `correction_cross_scope` on violation. Reasoning: a correction is "I logged this entry wrong" — never "I logged this entry under the wrong engagement"; the latter is a new entry + a `time.entry_recorded` row.

12. **MUST** emit memory audit row `time.entry_recorded` on every create (non-correction row) and `time.entry_corrected` on every correction. Both rows carry `{entry_id, tenant_id, member_subject_id_hash16, engagement_id, issue_id, duration_minutes, entry_kind, entry_status, billable, ts_start, ts_ns_recorded}`. The correction row additionally carries `correction_to`.

13. **MUST** PII-scrub the `description` field via TASK-MEMORY-111 BEFORE chain commit. The PostgreSQL row retains the raw text (tenant-scoped + RLS-protected); the memory audit chain holds only the scrubbed form (task-audit skill rule 18).

14. **MUST** complete create/correct/get/list handlers in ≤ 50 ms p95. `entries_perf_test` asserts on 1000 iterations.

15. **MUST** expose REST handlers:
    - `POST /v1/time/entries` — create new entry; caller `Resource::TimeEntry + Action::Write`.
    - `POST /v1/time/entries/{id}/correct` — create correction row; same permission.
    - `GET /v1/time/entries/{id}` — fetch (effective via `current_time_entries_view` by default; `?include=history` walks the chain).
    - `GET /v1/time/entries?member_subject_id=<>&engagement_id=<>&from=<>&to=<>` — list with cursor pagination; defaults to `current_time_entries_view`.

16. **MUST** support idempotent creation via `Idempotency-Key` header (same semantics as TASK-AUTH-002 §1 #6).

17. **MUST** emit OTel span `time.entry.{create,correct,get,list}` per handler with attributes: `tenant_id`, `member_subject_id_hash16`, `engagement_id`, `entry_id`, `outcome` (success | invalid_duration | invalid_kind | cross_scope_correction | cycle_detected | prior_already_corrected | permission_denied).

18. **MUST** emit OTel metrics:
    - `time_entry_create_total{outcome, entry_kind}` (counter).
    - `time_entry_correct_total{outcome}` (counter).
    - `time_entry_correction_chain_depth` (histogram; alarm at p99 > 5 — a deep chain suggests a workflow issue).
    - `time_entry_duration_minutes` (histogram per `entry_kind`).
    - `time_entry_count{tenant_id, entry_status}` (gauge).

19. **MUST** ship `entry_chain_walker(entry_id UUID) RETURNS SETOF UUID` SQL function that returns the chain from oldest (original) to newest (effective). Used by `GET ?include=history` and by the cycle-detection trigger. Maximum walk depth 100 (anti-infinite-loop safety floor).

20. **MUST** ensure `correction_to` FK is `DEFERRABLE INITIALLY IMMEDIATE` so the trigger sees the new row exists when validating self-references during the same transaction (Postgres-specific; needed for the cycle walker to see its own new row).

21. **MUST** maintain the **rate_card_snapshot pattern** (per DEC-224). When TASK-TIME-005 ships, it populates `rate_card_snapshot` at row creation with the engagement's then-current rate card. Mutations to the engagement's rate card NEVER alter past `rate_card_snapshot` values — the snapshot is frozen at the row's instant. Slice 1 ships the column with `{}` default; TASK-TIME-005 fills.

22. **MUST** support **list filters**: `?member_subject_id`, `?engagement_id`, `?issue_id`, `?from=<ts>`, `?to=<ts>`, `?entry_status`, `?billable`. Default page size 50, max 500. Cursor pagination on `(ts_start DESC, id)`.

23. **MUST** support `GET /v1/time/entries/{id}?include=history` returning the full chain `[<original>, <correction_1>, <correction_2>, ...]` ordered oldest to newest. Caller `Action::Read` on `TimeEntry`.

24. **MUST** ensure the `current_time_entries_view` performance is acceptable (< 100 ms for 10K-entry tenant). Index: `CREATE INDEX time_entries_correction_to_idx ON time_entries (correction_to) WHERE correction_to IS NOT NULL;` — bounded by correction count (typically < 5% of total).

25. **MUST** treat `created_by_subject_id` as the **actor** (who created the row), which MAY differ from `member_subject_id` (whose work is being recorded). For self-entry both are equal; for AM-on-behalf-of entry (TASK-TIME-003 manual form) they differ. Both fields are immutable per row.

---

## §2 — Why this design (rationale for humans)

**Why append-only with correction_to and not UPDATE in place (DEC-220, DEC-230)?** Time entries are invoice-grade financial records. An UPDATE in place loses the prior value; the audit trail of "what was originally claimed?" disappears. Append-only via correction_to means every prior value is preserved; the chain is the legal record. SOC 2 + ISO 27001 audit-logging requirements (A.12.4) are satisfied by construction. The cost is the slight complexity of "the current row is the one not pointed to by any other row's correction_to" — but the `current_time_entries_view` collapses that to a single query.

**Why a chain (not tree) topology for corrections (DEC-226, §1 #8)?** If two rows could both correct the same prior, "what is the current value?" becomes ambiguous (which correction is the latest?). Allowing only one row per correction_to enforces a deterministic linearisation — the chain head is unambiguously "the current value." The trigger rejects the second attempt with `prior_row_already_corrected`. Operators who need to correct an already-corrected row simply correct the current head (the chain extends).

**Why acyclic enforcement via trigger (DEC-225, §1 #7)?** A correction cycle (`A → B → A`) is a logic error that breaks every consumer (the chain walker would loop forever; the "current value" predicate has no answer). The trigger walks the chain at INSERT time and rejects the cycle. The CI test (`correction_acyclic_test`) seeds a deliberate cycle attempt to assert protection.

**Why `correction_to` cross-scope rejection (§1 #11)?** A correction is "I logged the same thing wrong" — same engagement, same issue, same member. Allowing the operator to "correct" entry-1 (engagement-A, issue-X) to a new value in (engagement-B, issue-Y) would let them effectively rewrite the engagement bill silently. Cross-scope corrections are blocked at trigger; the right action is `time.entry_reverted` (slice 2) plus a new `time.entry_recorded` under the new scope.

**Why durations bounded 1 ≤ minutes ≤ 1440 (DEC-227, DEC-228)?** The lower bound (1 minute) is operational — entries below 1 minute are typically test data or accidental clicks. The upper bound (1440 = 24 hours) prevents a single buggy entry from claiming a year of work; daily-cap enforcement (TASK-TIME-007) operates across rows, but per-row cap catches the most extreme typos before they reach the daily aggregator.

**Why `rate_card_snapshot` JSONB on the row (DEC-224)?** The billable amount of an entry is rate × hours. If `rate` is fetched at invoice generation by joining to the engagement's current rate card, a CFO bumping the rate card retroactively shifts every historic invoice line — silently. Snapshotting the rate card AT the row's instant freezes the billable basis. JSONB (not FK) is the right shape because rate cards have nested structure (per-role rates, member overrides, time-of-day adjustments), and we want the snapshot to be a pure value copy that's immune to FK cascades.

**Why entry_kind closed at 4 values (§1 #3)?** Working-time classification under VN Labour Code is `regular | overtime | weekend | holiday`. Each kind has different statutory rate multipliers (TASK-REW-004 ships). Allowing a 5th (e.g. `night_shift`) is an ADR — the rate-multiplier table would need to extend. Closed enum prevents drift.

**Why entry_status separate from billable (§1 #1)?** `entry_status` is workflow ("has this been approved?"); `billable` is financial classification ("is this hour invoiced?"). An entry can be `status=approved, billable=false` (approved internal work, not billed to client). They are orthogonal axes.

**Why slice-1 ships billable=false default (DEC-223)?** The billable cascade (TASK-TIME-005) is a non-trivial 4-step decision involving the engagement's non-billable categories, the rate card's role default, and member overrides. Splitting it to its own task keeps this schema task focused on integrity. Default `false` is conservative — if the cascade fails to set the flag, the entry is treated as non-billable (no invoice line) rather than mis-billed. TASK-TIME-005's tests assert the default is replaced on every entry.

**Why entry_currency on the row (DEC-229)?** Multi-currency tenants run engagements in VND, USD, SGD, etc. The entry's currency is the engagement's invoice currency at the row's instant. Snapshotting prevents the engagement's currency switch (rare but possible during contract renegotiation) from retroactively converting past entries. Invoice math is TASK-INV-001's concern; this task just preserves the source-of-truth currency.

**Why created_by_subject_id distinct from member_subject_id (§1 #25)?** Most entries are self-logged (member = creator). Some entries are AM-on-behalf-of (manual entry on someone else's behalf with their confirmation — TASK-TIME-003 ships that flow). Both fields capture different facts: "whose work this records" vs "who pressed the button." Audit trails need both.

**Why per-row immutability of all fields (implicit in DEC-230)?** Combined with append-only at SQL grant, this means a row's content is its forever-record. Workflows that need "edit" semantics use the correction handler (which creates a new row pointing at the prior); UI affordances may present this as "edit" but the underlying mechanism is always insert-new.

**Why `current_time_entries_view` instead of always-current column on the row (§1 #9)?** Adding `is_current BOOLEAN` to the row creates a writer dependency — every correction would have to update the prior row's `is_current = false`, breaking append-only. The view filters at read time; the index on `correction_to WHERE correction_to IS NOT NULL` is small (~5% of rows).

**Why DEFERRABLE INITIALLY IMMEDIATE on the self-FK (§1 #20)?** The cycle-detection trigger walks `correction_to` to check whether the chain loops back to the new row's id. Standard (non-deferrable) FK constraints would reject the row before the trigger sees it. DEFERRABLE INITIALLY IMMEDIATE means the FK is checked at the end of the statement, after the trigger; the trigger can use the new row's id during validation.

**Why list defaults to current view (§1 #22)?** The 99% query pattern is "what hours did this Member log this week?" — and the answer is current-effective, not raw history. The 1% query "show me the correction history of entry-1" uses `?include=history`. Defaulting to history-aware listing would surface every superseded row, confusing operators.

**Why chain max-depth 100 (§1 #19)?** Practical chains are 1–3 rows (entry + one correction is typical; pathological cases reach 5–10). 100 is a safety floor — any chain that deep is either an integration bug or a stress test; the walker bails to prevent infinite-loop-like CPU consumption. Production chains hit 100 are alarmable.

**Why `description` 0–1000 chars (§1 #10)?** Short enough to discourage prose-narrative entries (which belong in PROJ issue comments, not TIME); long enough to let "fixed merge conflict in chat/auth bridge — see PR-451" fit. PII-scrubbed via TASK-MEMORY-111 before chain commit; operators warned that descriptions are visible to AM + CFO via TASK-TIME-006.

**Why two memory audit row kinds (recorded + corrected) and not one (DEC-231)?** Different operator queries: "show me the original-creation activity for this engagement" filters on `time.entry_recorded`; "show me the correction activity" filters on `time.entry_corrected`. A single kind would require an extra `is_correction` flag and degrade query selectivity.

---

## §3 — API contract

### 3.1 — Migration 0001 — time_entries

```sql
-- services/time/migrations/0001_time_entries.sql

BEGIN;

CREATE TYPE entry_kind AS ENUM ('regular', 'overtime', 'weekend', 'holiday');
CREATE TYPE entry_status AS ENUM ('draft', 'submitted', 'approved', 'reverted');

CREATE TABLE time_entries (
    id                     UUID         PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    member_subject_id      UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    engagement_id          UUID         NOT NULL,
    issue_id               UUID         NOT NULL,
    ts_start               TIMESTAMPTZ  NOT NULL,
    duration_minutes       INT          NOT NULL CHECK (duration_minutes BETWEEN 1 AND 1440),
    entry_kind             entry_kind   NOT NULL,
    entry_status           entry_status NOT NULL DEFAULT 'draft',
    billable               BOOLEAN      NOT NULL DEFAULT false,
    rate_card_snapshot     JSONB        NOT NULL DEFAULT '{}'::jsonb,
    entry_currency         CHAR(3)      NOT NULL CHECK (entry_currency ~ '^[A-Z]{3}$'),
    description            TEXT         CHECK (description IS NULL OR length(description) BETWEEN 0 AND 1000),
    correction_to          UUID         REFERENCES time_entries(id) DEFERRABLE INITIALLY IMMEDIATE,
    created_at             TIMESTAMPTZ  NOT NULL DEFAULT now(),
    created_by_subject_id  UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT
);

CREATE INDEX time_entries_tenant_member_ts_idx ON time_entries (tenant_id, member_subject_id, ts_start DESC);
CREATE INDEX time_entries_tenant_engagement_ts_idx ON time_entries (tenant_id, engagement_id, ts_start DESC);
CREATE INDEX time_entries_correction_to_idx ON time_entries (correction_to) WHERE correction_to IS NOT NULL;

ALTER TABLE time_entries ENABLE ROW LEVEL SECURITY;
CREATE POLICY time_entries_tenant_isolation ON time_entries
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only enforcement (DEC-230)
REVOKE UPDATE, DELETE ON time_entries FROM cyberos_app;

-- Cycle detection on correction_to (DEC-225)
CREATE OR REPLACE FUNCTION detect_correction_cycle() RETURNS TRIGGER AS $$
DECLARE
    walker UUID;
    depth INT := 0;
BEGIN
    IF NEW.correction_to IS NULL THEN RETURN NEW; END IF;
    walker := NEW.correction_to;
    WHILE walker IS NOT NULL AND depth < 100 LOOP
        IF walker = NEW.id THEN
            RAISE EXCEPTION 'correction_cycle_detected' USING ERRCODE = 'P0010';
        END IF;
        SELECT correction_to INTO walker FROM time_entries WHERE id = walker;
        depth := depth + 1;
    END LOOP;
    IF depth >= 100 THEN
        RAISE EXCEPTION 'correction_chain_too_deep' USING ERRCODE = 'P0011';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_time_entries_no_cycle BEFORE INSERT ON time_entries
    FOR EACH ROW EXECUTE FUNCTION detect_correction_cycle();

-- Chain (not tree) enforcement (DEC-226)
CREATE OR REPLACE FUNCTION enforce_chain_topology() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.correction_to IS NULL THEN RETURN NEW; END IF;
    IF EXISTS (SELECT 1 FROM time_entries WHERE correction_to = NEW.correction_to AND id != NEW.id) THEN
        RAISE EXCEPTION 'prior_row_already_corrected' USING ERRCODE = 'P0012';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_time_entries_chain_topology BEFORE INSERT ON time_entries
    FOR EACH ROW EXECUTE FUNCTION enforce_chain_topology();

-- Cross-scope correction rejection (§1 #11)
CREATE OR REPLACE FUNCTION enforce_correction_inheritance() RETURNS TRIGGER AS $$
DECLARE prior RECORD;
BEGIN
    IF NEW.correction_to IS NULL THEN RETURN NEW; END IF;
    SELECT tenant_id, engagement_id, issue_id, member_subject_id
        INTO prior FROM time_entries WHERE id = NEW.correction_to;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'correction_target_missing' USING ERRCODE = 'P0013';
    END IF;
    IF prior.tenant_id != NEW.tenant_id
       OR prior.engagement_id != NEW.engagement_id
       OR prior.issue_id != NEW.issue_id
       OR prior.member_subject_id != NEW.member_subject_id THEN
        RAISE EXCEPTION 'correction_cross_scope' USING ERRCODE = 'P0014';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_time_entries_correction_inheritance BEFORE INSERT ON time_entries
    FOR EACH ROW EXECUTE FUNCTION enforce_correction_inheritance();

COMMIT;
```

### 3.2 — Migration 0002 — current view + chain walker

```sql
-- services/time/migrations/0002_time_entries_view.sql

BEGIN;

CREATE VIEW current_time_entries_view AS
    SELECT * FROM time_entries
    WHERE id NOT IN (
        SELECT correction_to FROM time_entries WHERE correction_to IS NOT NULL
    );

-- Walk a chain from any node to its head (effective row).
CREATE OR REPLACE FUNCTION entry_chain_walker(p_entry_id UUID) RETURNS SETOF UUID AS $$
DECLARE
    head UUID;
    walker UUID;
    depth INT := 0;
BEGIN
    -- Walk to the original.
    walker := p_entry_id;
    WHILE EXISTS (SELECT 1 FROM time_entries WHERE id = walker AND correction_to IS NOT NULL) AND depth < 100 LOOP
        SELECT correction_to INTO walker FROM time_entries WHERE id = walker;
        depth := depth + 1;
    END LOOP;
    head := walker;
    -- Walk down to the effective row, returning each id.
    walker := head;
    depth := 0;
    LOOP
        RETURN NEXT walker;
        SELECT id INTO walker FROM time_entries WHERE correction_to = walker;
        EXIT WHEN NOT FOUND OR depth >= 100;
        depth := depth + 1;
    END LOOP;
END;
$$ LANGUAGE plpgsql STABLE;

COMMIT;
```

### 3.3 — Rust types

```rust
// services/time/src/types.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "entry_kind", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntryKind { Regular, Overtime, Weekend, Holiday }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Type, Serialize, Deserialize)]
#[sqlx(type_name = "entry_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntryStatus { Draft, Submitted, Approved, Reverted }

impl EntryKind {
    pub const ALL: &'static [EntryKind] = &[EntryKind::Regular, EntryKind::Overtime, EntryKind::Weekend, EntryKind::Holiday];
}

impl EntryStatus {
    pub const ALL: &'static [EntryStatus] = &[EntryStatus::Draft, EntryStatus::Submitted, EntryStatus::Approved, EntryStatus::Reverted];
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub member_subject_id: Uuid,
    pub engagement_id: Uuid,
    pub issue_id: Uuid,
    pub ts_start: DateTime<Utc>,
    pub duration_minutes: i32,
    pub entry_kind: EntryKind,
    pub entry_status: EntryStatus,
    pub billable: bool,
    pub rate_card_snapshot: serde_json::Value,
    pub entry_currency: String,
    pub description: Option<String>,
    pub correction_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub created_by_subject_id: Uuid,
}
```

### 3.4 — Validation

```rust
// services/time/src/validation.rs
use chrono::{DateTime, Duration, Utc};

pub const MIN_DURATION_MINUTES: i32 = 1;
pub const MAX_DURATION_MINUTES: i32 = 1440;
pub const FUTURE_TOLERANCE: Duration = Duration::minutes(5);

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("duration_out_of_range: {0}")]
    DurationOutOfRange(i32),
    #[error("ts_start_in_future: {0}")]
    TsStartInFuture(DateTime<Utc>),
    #[error("description_too_long: {0}")]
    DescriptionTooLong(usize),
}

pub fn validate_duration(minutes: i32) -> Result<(), ValidationError> {
    if minutes < MIN_DURATION_MINUTES || minutes > MAX_DURATION_MINUTES {
        return Err(ValidationError::DurationOutOfRange(minutes));
    }
    Ok(())
}

pub fn validate_ts_start(ts: DateTime<Utc>, now: DateTime<Utc>) -> Result<(), ValidationError> {
    if ts > now + FUTURE_TOLERANCE { return Err(ValidationError::TsStartInFuture(ts)); }
    Ok(())
}

pub fn validate_description(desc: &Option<String>) -> Result<(), ValidationError> {
    if let Some(d) = desc {
        if d.len() > 1000 { return Err(ValidationError::DescriptionTooLong(d.len())); }
    }
    Ok(())
}
```

### 3.5 — REST handlers (excerpt)

```rust
// services/time/src/handlers/entries.rs
use axum::{Json, extract::{Path, State, Query}, http::StatusCode};
use crate::types::*;
use crate::validation::*;
use crate::audit::entry_events;

#[derive(Deserialize)]
pub struct CreateEntryRequest {
    pub member_subject_id: Uuid,
    pub engagement_id: Uuid,
    pub issue_id: Uuid,
    pub ts_start: DateTime<Utc>,
    pub duration_minutes: i32,
    pub entry_kind: EntryKind,
    pub entry_currency: String,
    pub description: Option<String>,
}

pub async fn create_entry(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<CreateEntryRequest>,
) -> Result<(StatusCode, Json<TimeEntry>), ApiError> {
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::TimeEntry, Action::Write)?;
    validate_duration(req.duration_minutes)?;
    validate_ts_start(req.ts_start, Utc::now())?;
    validate_description(&req.description)?;

    let id = Uuid::new_v4();
    let mut tx = state.db.begin().await?;
    let entry = sqlx::query_as!(TimeEntry, r#"
        INSERT INTO time_entries (id, tenant_id, member_subject_id, engagement_id, issue_id,
            ts_start, duration_minutes, entry_kind, entry_status, billable,
            rate_card_snapshot, entry_currency, description, correction_to, created_by_subject_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::entry_kind, 'draft'::entry_status, false,
                '{}'::jsonb, $9, $10, NULL, $11)
        RETURNING *
    "#,
        id, claims.tenant_id(), req.member_subject_id, req.engagement_id, req.issue_id,
        req.ts_start, req.duration_minutes, req.entry_kind as EntryKind,
        req.entry_currency, req.description, claims.subject_id(),
    ).fetch_one(&mut *tx).await?;

    entry_events::emit_entry_recorded(&mut tx, &entry).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(entry)))
}

#[derive(Deserialize)]
pub struct CorrectEntryRequest {
    pub duration_minutes: Option<i32>,
    pub entry_kind: Option<EntryKind>,
    pub description: Option<String>,
    pub ts_start: Option<DateTime<Utc>>,
}

pub async fn correct_entry(
    State(state): State<AppState>,
    claims: Claims,
    Path(prior_id): Path<Uuid>,
    Json(req): Json<CorrectEntryRequest>,
) -> Result<(StatusCode, Json<TimeEntry>), ApiError> {
    state.matrix.snapshot().require_permission(&claims.roles(), Resource::TimeEntry, Action::Write)?;
    let mut tx = state.db.begin().await?;
    let prior: TimeEntry = sqlx::query_as!(TimeEntry, "SELECT * FROM time_entries WHERE id = $1", prior_id)
        .fetch_one(&mut *tx).await?;

    let new_id = Uuid::new_v4();
    let duration = req.duration_minutes.unwrap_or(prior.duration_minutes);
    let kind = req.entry_kind.unwrap_or(prior.entry_kind);
    let desc = req.description.or(prior.description.clone());
    let ts_start = req.ts_start.unwrap_or(prior.ts_start);

    validate_duration(duration)?;
    validate_ts_start(ts_start, Utc::now())?;
    validate_description(&desc)?;

    let entry = sqlx::query_as!(TimeEntry, r#"
        INSERT INTO time_entries (id, tenant_id, member_subject_id, engagement_id, issue_id,
            ts_start, duration_minutes, entry_kind, entry_status, billable,
            rate_card_snapshot, entry_currency, description, correction_to, created_by_subject_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8::entry_kind, $9::entry_status, $10,
                $11, $12, $13, $14, $15)
        RETURNING *
    "#,
        new_id, prior.tenant_id, prior.member_subject_id, prior.engagement_id, prior.issue_id,
        ts_start, duration, kind as EntryKind, prior.entry_status as EntryStatus, prior.billable,
        prior.rate_card_snapshot, prior.entry_currency, desc, prior.id, claims.subject_id(),
    ).fetch_one(&mut *tx).await?;

    entry_events::emit_entry_corrected(&mut tx, &entry, &prior).await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(entry)))
}
```

---

## §4 — Acceptance criteria

1. **EntryKind closed at 4 values** — `EntryKind::ALL.len() == 4`; Postgres enum has exactly 4 labels.
2. **EntryStatus closed at 4 values** — same shape.
3. **RLS isolates by tenant** — query as tenant-A returns 0 entries of tenant-B.
4. **Create entry happy path** — valid body → 201 with `TimeEntry` JSON; one `time.entry_recorded` memory row.
5. **Create entry < 1 minute** — 400 `duration_out_of_range`.
6. **Create entry > 1440 minutes** — 400 `duration_out_of_range`.
7. **Create entry ts_start in future** — 400 `ts_start_in_future`.
8. **UPDATE on time_entries blocked** — `UPDATE time_entries SET duration_minutes = 100 WHERE id = $1` as `cyberos_app` user → permission denied.
9. **DELETE on time_entries blocked** — same as #8 for DELETE.
10. **Correct entry happy** — valid prior id + new duration → 201 with new row; original kept; new row's `correction_to = prior_id`.
11. **Correct entry cross-tenant rejected** — correction targeting a prior in a different tenant → 400 `correction_cross_scope`.
12. **Correct entry cross-engagement rejected** — modifying engagement_id in correction body → 400 `correction_cross_scope`.
13. **Tree topology rejected** — two correctors of same prior → second insert raises `prior_row_already_corrected`.
14. **Cycle topology rejected** — A → B → A attempted → `correction_cycle_detected`.
15. **Chain depth > 100 rejected** — synthetic 101-row chain → 102nd insert raises `correction_chain_too_deep`.
16. **current_time_entries_view filters correctly** — query on view never returns rows that are pointed to by `correction_to`.
17. **GET ?include=history returns full chain** — chain of 3 → 3 rows returned in oldest-first order.
18. **Idempotent create** — same Idempotency-Key + same body → same entry.
19. **OTel span `time.entry.create` emitted** — with `outcome=success`.
20. **OTel counter `time_entry_create_total{outcome=success, entry_kind=regular}` increments** — per create.
21. **OTel counter `time_entry_correct_total{outcome=success}` increments** — per correction.
22. **OTel histogram `time_entry_correction_chain_depth` observes** — chain of 4 → observation of 4.
23. **Perf budget < 50 ms p95** — `entries_perf_test` 1000 iterations.
24. **Subject FK ON DELETE RESTRICT** — `DELETE FROM auth.subjects WHERE id = <member_subject_id>` raises FK violation if entries exist.
25. **rate_card_snapshot defaults to `{}`** — slice 1 default until TASK-TIME-005 fills.
26. **`created_by` distinct from `member_subject_id`** — AM-on-behalf-of entry creates row with `created_by = AM_id, member_subject_id = staff_id`.

---

## §5 — Verification

```rust
// services/time/tests/append_only_test.rs
#[sqlx::test]
async fn update_blocked_at_grant(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_entry(&pool).await;
    let err = sqlx::query("UPDATE time_entries SET duration_minutes = 999 WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}

#[sqlx::test]
async fn delete_blocked_at_grant(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_entry(&pool).await;
    let err = sqlx::query("DELETE FROM time_entries WHERE id = $1").bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}
```

```rust
// services/time/tests/correction_acyclic_test.rs
#[sqlx::test]
async fn direct_cycle_rejected(pool: sqlx::PgPool) {
    let a = seed_entry(&pool).await;
    let b = correct(&pool, a, /*..*/).await;
    // Now attempt to correct b with correction_to = a → would form A→B→A cycle.
    let err = insert_with_correction_to(&pool, /*new_id=*/Uuid::new_v4(), /*correction_to=*/a).await.unwrap_err();
    assert!(format!("{err}").contains("prior_row_already_corrected"));
    // (chain topology check fires first; cycle check fires for self-targeting case)
}

#[sqlx::test]
async fn self_reference_cycle_rejected(pool: sqlx::PgPool) {
    let id = Uuid::new_v4();
    let err = insert_self_referencing(&pool, id).await.unwrap_err();
    assert!(format!("{err}").contains("correction_cycle_detected"));
}
```

```rust
// services/time/tests/current_view_test.rs
#[sqlx::test]
async fn corrected_rows_omitted_from_current_view(pool: sqlx::PgPool) {
    let a = seed_entry(&pool).await;
    let b = correct(&pool, a, /* new duration */).await;
    let rows: Vec<Uuid> = sqlx::query_scalar("SELECT id FROM current_time_entries_view WHERE id IN ($1, $2)")
        .bind(a).bind(b).fetch_all(&pool).await.unwrap();
    assert_eq!(rows, vec![b]);
}
```

```rust
// services/time/tests/correction_chain_test.rs
#[sqlx::test]
async fn entry_chain_walker_returns_oldest_first(pool: sqlx::PgPool) {
    let a = seed_entry(&pool).await;
    let b = correct(&pool, a, ()).await;
    let c = correct(&pool, b, ()).await;
    let chain: Vec<Uuid> = sqlx::query_scalar("SELECT entry_chain_walker($1)").bind(c).fetch_all(&pool).await.unwrap();
    assert_eq!(chain, vec![a, b, c]);
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; cycle/chain triggers are in §3.1; chain walker in §3.2.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-003** — RLS enforcement; same `current_setting('auth.tenant_id')` pattern.
- **TASK-AUTH-101** — RBAC catalogue; `Resource::TimeEntry + Action::Write/Read` matrix entry.

**Downstream (8 placeholders):**
- **TASK-TIME-002** — timer start/stop UI (creates entries via this task's API).
- **TASK-TIME-003** — manual entry form + VN Labour Code cap validation.
- **TASK-TIME-005** — billable cascade (populates `billable` + `rate_card_snapshot`).
- **TASK-TIME-006** — weekly approval flow (transitions `entry_status`).
- **TASK-TIME-007** — VN Labour Code Art. 107 OT cap hard-block at write.
- **TASK-TIME-009** — per-cycle billable rollup → INV.
- **TASK-HR-008** — performance signal aggregator (read-only consumer).
- **TASK-RES-001** — capacity-vs-demand matrix (joins on member × time).

**Cross-module:**
- **TASK-AI-003** — memory audit bridge; receives `time.entry_recorded`, `time.entry_corrected`.
- **TASK-PROJ-001** — issue schema; `issue_id` FK target.
- **TASK-PROJ-005** — rate card schema; `rate_card_snapshot` source.
- **TASK-MEMORY-111** — PII detection for `description` scrubbing.

---

## §8 — Example payloads

### 8.1 — POST /v1/time/entries request

```json
{
  "member_subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "engagement_id": "e1234567-1234-1234-1234-123456789012",
  "issue_id": "i1234567-1234-1234-1234-123456789012",
  "ts_start": "2026-05-16T09:00:00Z",
  "duration_minutes": 90,
  "entry_kind": "regular",
  "entry_currency": "VND",
  "description": "Worked on TASK-AUTH-101 spec review"
}
```

### 8.2 — 201 CREATED response

```json
{
  "id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "tenant_id": "5e8f1d2a-...",
  "member_subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "engagement_id": "e1234567-1234-1234-1234-123456789012",
  "issue_id": "i1234567-1234-1234-1234-123456789012",
  "ts_start": "2026-05-16T09:00:00Z",
  "duration_minutes": 90,
  "entry_kind": "regular",
  "entry_status": "draft",
  "billable": false,
  "rate_card_snapshot": {},
  "entry_currency": "VND",
  "description": "Worked on TASK-AUTH-101 spec review",
  "correction_to": null,
  "created_at": "2026-05-16T09:01:00Z",
  "created_by_subject_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d"
}
```

### 8.3 — POST correct request

```json
{ "duration_minutes": 105, "description": "Worked on TASK-AUTH-101 spec review — corrected: forgot to add 15min lunch overlap" }
```

### 8.4 — time.entry_recorded memory row

```json
{
  "kind": "time.entry_recorded",
  "tenant_id": "5e8f1d2a-...",
  "entry_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "member_subject_id_hash16": "9b1deb4d3b7d4bad",
  "engagement_id": "e1234567-1234-1234-1234-123456789012",
  "issue_id": "i1234567-1234-1234-1234-123456789012",
  "duration_minutes": 90,
  "entry_kind": "regular",
  "entry_status": "draft",
  "billable": false,
  "entry_currency": "VND",
  "description_scrubbed": "Worked on TASK-AUTH-101 spec review",
  "ts_start": "2026-05-16T09:00:00Z",
  "ts_ns_recorded": 1747920731000000000
}
```

### 8.5 — time.entry_corrected memory row

```json
{
  "kind": "time.entry_corrected",
  "tenant_id": "5e8f1d2a-...",
  "entry_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8N",
  "correction_to": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "member_subject_id_hash16": "9b1deb4d3b7d4bad",
  "engagement_id": "e1234567-1234-1234-1234-123456789012",
  "issue_id": "i1234567-1234-1234-1234-123456789012",
  "duration_minutes_old": 90,
  "duration_minutes_new": 105,
  "ts_ns_recorded": 1747921000000000000
}
```

---

## §9 — Open questions

Deferred:
- **Standalone-engagement entries (no issue_id)** — slice 2; some non-billable internal time has no specific issue.
- **Holiday-list driving entry_kind defaulting** — TASK-TIME-007 ships the holiday table.
- **VN Labour Code OT cap enforcement** — TASK-TIME-007 (this task allows the kind = overtime; the cap is TASK-TIME-007's gate).
- **Billable cascade computation** — TASK-TIME-005.
- **Approval flow transitions** — TASK-TIME-006.
- **Per-cycle rollup emit to INV** — TASK-TIME-009.
- **Time-entry edit UI** — TASK-TIME-002 + TASK-TIME-003.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| UPDATE on time_entries | SQL grant `REVOKE UPDATE` | Permission denied at DB | None — designed |
| DELETE on time_entries | SQL grant `REVOKE DELETE` | Permission denied at DB | None — designed |
| Duration < 1 minute | DB CHECK + handler validation | 400 + `duration_out_of_range` | Caller fixes |
| Duration > 1440 minutes | DB CHECK + handler validation | 400 + `duration_out_of_range` | Split into multiple entries |
| ts_start in future > 5 min | Handler validation | 400 + `ts_start_in_future` | Use current time |
| Two correctors of same prior | `prior_row_already_corrected` trigger | 400 + error | Correct the chain head instead |
| A → B → A cycle | `correction_cycle_detected` trigger | 400 + error | Re-do correction without cycle |
| Chain > 100 rows deep | `correction_chain_too_deep` trigger | 400 + error | Investigate workflow producing deep chains |
| Cross-tenant correction | `correction_cross_scope` trigger | 400 + error | New entry under correct tenant |
| Cross-engagement correction | `correction_cross_scope` trigger | 400 + error | Revert + new entry under correct engagement (slice 2 reverts) |
| Cross-issue correction | same | same | same |
| Cross-member correction | same | same | same |
| `correction_to` references non-existent | `correction_target_missing` trigger | 400 + error | Verify prior id |
| Description > 1000 chars | Handler validation | 400 + `description_too_long` | Shorten or move detail to PROJ comment |
| `entry_currency` not 3 uppercase | DB CHECK | INSERT fails | Use valid ISO-4217 |
| RLS bypass attempt | RLS `USING` predicate | 0 rows returned | None — designed |
| `member_subject_id` deleted while entries exist | FK ON DELETE RESTRICT | DELETE auth.subjects fails | Use HR termination flow |
| memory row emit fails mid-transaction | Outer rollback | 500 `audit_failed`; entry not persisted | memory_writer diagnosis |
| Idempotency-Key reused with different body | Idempotency layer | 409 `idempotency_key_reuse` | New key |
| `current_time_entries_view` slow on 10K-row tenant | Perf test | Sev-3 | Verify `correction_to` index health |
| `entry_chain_walker` exceeds depth 100 | Function returns up to 100 rows | Truncated chain returned | Alarmable |
| OTel span attribute missing | `otel_attrs_test` | CI fails | Fix span builder |
| `entry_kind` enum drift (someone adds `night_shift`) | Closed-enum test | CI fails | ADR + migration + code together |
| `entry_status` enum drift | same | CI fails | same |
| `rate_card_snapshot` mutated after row write | Append-only enforcement | Permission denied | None — designed |
| Race: concurrent corrections to same prior | First INSERT wins; second fails `prior_row_already_corrected` | Second caller sees 400 | Caller re-fetches and retries |
| `description` contains PII not scrubbed | memory PII test | Pre-commit failure | Add PII rule |
| Chain walker called on entry id from different tenant | RLS filters out | Returns 0 rows | None — designed |
| Subject deleted but cleanup ordering wrong | FK ON DELETE RESTRICT | Migration fails | Restore subject first |
| Daily aggregation exceeds 24h via many sub-row entries | This task's per-row cap is 24h; aggregate cap is TASK-TIME-007 | Out-of-scope here | TASK-TIME-007 |
| Time-zone confusion: ts_start interpreted in local time | DB stores TIMESTAMPTZ — always UTC | None | Document for UI implementers |
| Description-edit on existing entry attempted | Handler omits from PATCH — there is no PATCH | Use correct endpoint | None — designed |
| `billable` field set in create request | Handler ignores (slice 1; cascade sets) | Default false applied | TASK-TIME-005 takes over |

---

## §11 — Implementation notes

- **Append-only is the design assertion** — every other invariant rests on it. SQL grant enforcement (not handler discipline) makes accidental UPDATE impossible.
- **Correction chains, not trees** — operators occasionally want "two parallel corrections" (e.g. "what if duration was 90, what if it was 105"). The chain rule forces them to pick one. The 1% of cases needing branching go through reverted-status + new entry.
- **Cycle detection at trigger, not application** — the trigger sees the actual DB state including the new row; application-layer checks would race with concurrent inserts.
- **`current_time_entries_view` is the default query target** — downstream code should `SELECT * FROM current_time_entries_view WHERE engagement_id = $1` rather than the raw table. The index on `correction_to WHERE correction_to IS NOT NULL` keeps the NOT IN cheap.
- **`rate_card_snapshot JSONB` not FK** — the rate card has nested shape (per-role rates, member overrides, time-of-day adjustments); FK to a rate-card-version table would cascade unwantedly. JSONB snapshot is the audit-grade pattern.
- **`entry_currency CHAR(3)` not VARCHAR** — fixed-width for ISO-4217 codes; the CHECK constraint enforces uppercase 3-letter shape.
- **Chain walker max-depth 100** — safety floor; production chains > 5 are rare. Alarmable.
- **DEFERRABLE INITIALLY IMMEDIATE on self-FK** — Postgres-specific quirk. Without it, the cycle-detection trigger fires before the new row's FK validates; with it, FK validation defers to statement end (which is fine because trigger handles cycle).
- **`created_by_subject_id` ≠ `member_subject_id` for AM-on-behalf-of entries** — both fields are immutable; the memory row carries both hash16.
- **PII scrubbing applies to `description`** — operators may inadvertently log "called Person A re: their salary inquiry"; TASK-MEMORY-111 rules strip.
- **`entry_status` default `draft`** — entries start in draft until submitted; TASK-TIME-006 ships the transition handlers. Reading drafts is allowed (caller can review own draft).
- **`billable` default false** — conservative. Cascade (TASK-TIME-005) updates to true via correction (creating a new row with billable=true) at row-creation time, before the row is committed. Slice-1 entries are non-billable by default.
- **Per-row 24h cap protects daily aggregator** — TASK-TIME-007's daily-cap logic operates on rows; if one row could be 100h, the daily cap math underestimates. Per-row cap closes that hole.
- **Idempotency-Key applies to create only** — corrections are not idempotent in the same sense (the prior row determines the new row's contents); retrying a correction creates a new chain entry. Operators should be aware.
- **PROJ issue FK is a logical foreign key** — declared in §1 #1 but not enforced as SQL FK at slice 1 (cross-service FK is operationally complex). TASK-PROJ-001's data integrity is trusted; if an issue is deleted, time entries pointing at it become orphaned (still queryable, just decorative).
- **`engagement_id` similarly soft-FK** — same reason; TASK-PROJ-005 owns the engagements table.
- **No `updated_at`** — append-only means there's no update; `created_at` is the only timestamp.
- **The cycle-detection trigger walks depth 100 max** — beyond that, raises `correction_chain_too_deep`. Production chains hitting this are bugs.
- **`entry_chain_walker` returns oldest-first** — that's the natural temporal order ("here's how this entry evolved"). UIs may render newest-first if appropriate.
- **The view's `NOT IN` query** — Postgres optimises with the `correction_to_idx` partial index; performance is bounded by correction count (~5% of rows).
- **REVOKE applies to `cyberos_app` role only** — superuser + migration role can mutate (for backups, manual repair, etc.). Production app code uses cyberos_app.
- **`entry_kind` and `entry_status` are orthogonal axes** — workflow status (draft → submitted → approved) is independent of working-time classification (regular vs overtime).
- **`description` is NOT searchable via full-text** at slice 1 — that's a task-TIME-2xx ambition. Slice 1 stores as TEXT.

---

*End of TASK-TIME-001.*
