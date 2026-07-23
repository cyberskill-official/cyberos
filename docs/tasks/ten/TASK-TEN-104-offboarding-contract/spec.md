---
id: TASK-TEN-104
title: "TEN 90-day offboarding contract — closed 4-state FSM (Active → Terminating-A → Terminating-B → Terminated) + day-pinned transitions + scheduled jobs + read-only freeze + dead-letter recovery + dual-signoff irreversible wipe"
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
module: ten
priority: p0
status: draft
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CLO + CTO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-001, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-TEN-105, TASK-TEN-106, TASK-TEN-202, TASK-OBS-007]
depends_on: [TASK-TEN-001]
blocks: [TASK-TEN-105, TASK-TEN-106, TASK-TEN-202]

source_pages:
  - website/docs/modules/ten.html#offboarding
source_decisions:
  - DEC-500 (closed 4-state offboarding FSM: active → terminating_a (30-day RO + export) → terminating_b (60-day dead-letter recovery) → terminated (irreversible wipe) — adding a 5th state is an ADR)
  - DEC-501 (90-day total grace = 30 days A + 60 days B; configurable per tenant 30-180 total; minimum 30 days; maximum 180 days)
  - DEC-502 (FSM transitions are unidirectional EXCEPT terminating_a → active (cancellation) within first 30 days; terminating_b → active forbidden (data already wiped to dead-letter); terminated → * forbidden (irreversible by Object-Lock))
  - DEC-503 (scheduled job `ten_offboarding_advance` runs hourly — advances state machine based on day-pinned transitions; idempotent via WHERE status + scheduled_advance_at predicate)
  - DEC-504 (terminating_a: tenant data is READ-ONLY at app layer; new writes blocked at handler; existing writes via privileged operator override require ADR; tenant-admin can export signed bundles per TASK-TEN-105)
  - DEC-505 (terminating_b: tenant data wiped to dead-letter S3 bucket with 60-day Object-Lock retention; only operator with CSO+CLO co-sign can restore via dedicated handler; per-tenant policy MAY shorten dead-letter window down to 30 days)
  - DEC-506 (terminated transition requires CSO + CLO dual-signoff per TASK-TEN-106; emits `ten.tenant_terminated` memory row + permanent-delete attestation row; Object-Lock COMPLIANCE mode on the dead-letter S3 retention prevents accidental delete)
  - DEC-507 (memory audit kinds: ten.offboarding_initiated, ten.terminating_a_entered, ten.terminating_b_entered, ten.terminating_cancelled, ten.tenant_terminated, ten.offboarding_extended, ten.dead_letter_restored, ten.read_only_write_attempted)
  - DEC-508 (REVOKE UPDATE, DELETE on tenant_offboarding_log from cyberos_app — append-only at SQL grant)
  - DEC-509 (read-only freeze enforced at TASK-AUTH-004 JWT issuance: tokens for tenants in terminating_a have `scope_grants` filtered to read-only operations; write attempts return 423 `tenant_read_only` + emit `ten.read_only_write_attempted`)
  - DEC-510 (cancellation (terminating_a → active) requires same caller authority as initiation: root-admin OR tenant-admin with explicit confirmation step; emits `ten.terminating_cancelled` row)
  - DEC-511 (TASK-TEN-202 hostile-termination fast-track bypasses 30-day terminating_a window — direct active → terminating_b with CEO+CLO+CSO sign-off; this task ships the FSM that TASK-TEN-202 consumes)
  - DEC-512 (per-tenant offboarding extension via `POST /v1/ten/offboarding/extend` — adds 1-30 days to current state; max 2 extensions per offboarding cycle; CLO role required)
  - DEC-513 (dead-letter restore from terminating_b requires CSO + CLO dual-signoff + restore reason; emits `ten.dead_letter_restored` memory row sev-1)
  - DEC-514 (FSM enforced at trigger AND handler — defense in depth; trigger uses ENUM transition matrix function)
  - DEC-515 (terminating_a + terminating_b durations are PINNED at FSM entry — `scheduled_advance_at` column captures the target transition time; extensions update this; the scheduled job consults this column)
  #13)
  - GDPR Art. 17 (right to erasure — 90-day grace satisfies; legal-hold blocks per TASK-DOC-001 §1
  - PDPL Art. 17 (data subject erasure — equivalent + per-tenant override)
  - SOC 2 CC6.5 (data disposal controls)

language: rust 1.81 + sql
service: cyberos/services/ten/
new_files:
  - services/ten/migrations/0004_tenant_offboarding_state.sql
  - services/ten/migrations/0005_tenant_offboarding_log.sql
  - services/ten/src/offboarding/mod.rs
  # closed transition matrix
  - services/ten/src/offboarding/fsm.rs
  # hourly advance job
  - services/ten/src/offboarding/scheduler.rs
  # JWT issuance hook
  - services/ten/src/offboarding/read_only_gate.rs
  # S3 dead-letter writer + restorer
  - services/ten/src/offboarding/dead_letter.rs
  # CRUD across state + log
  - services/ten/src/offboarding/repo.rs
  # 8 memory row builders
  - services/ten/src/offboarding/audit.rs
  # initiate + cancel + extend + restore
  - services/ten/src/handlers/offboarding.rs
  - services/ten/tests/offboarding_fsm_test.rs
  - services/ten/tests/offboarding_initiate_test.rs
  - services/ten/tests/offboarding_scheduler_advance_test.rs
  - services/ten/tests/offboarding_terminating_a_read_only_test.rs
  - services/ten/tests/offboarding_cancel_within_a_test.rs
  - services/ten/tests/offboarding_cancel_in_b_forbidden_test.rs
  - services/ten/tests/offboarding_extension_test.rs
  - services/ten/tests/offboarding_extension_max_2_test.rs
  - services/ten/tests/offboarding_dead_letter_write_test.rs
  - services/ten/tests/offboarding_dead_letter_restore_dual_signoff_test.rs
  - services/ten/tests/offboarding_terminated_irreversible_test.rs
  - services/ten/tests/offboarding_append_only_log_test.rs
  - services/ten/tests/offboarding_audit_emission_test.rs
modified_files:
  # consult read_only_gate at issuance
  - services/auth/src/jwt/issuer.rs

allowed_tools:
  - file_read: services/ten/**
  - file_write: services/ten/{src,tests,migrations}/**
  - bash: cd services/ten && cargo test offboarding

disallowed_tools:
  - allow terminating_b → active transition (per DEC-502 — data already wiped to dead letter)
  - allow terminated → * transitions (per DEC-502 — irreversible)
  - bypass CSO+CLO dual-signoff on terminated transition (per DEC-506)
  - bypass CSO+CLO dual-signoff on dead-letter restore (per DEC-513)
  - allow > 2 extensions per offboarding cycle (per DEC-512)
  - skip Object-Lock COMPLIANCE mode on dead-letter bucket (per DEC-506)
  - allow tenant-admin to initiate hostile-termination fast-track (per DEC-511 — CEO+CLO+CSO required via TASK-TEN-202)

effort_hours: 12
subtasks:
  - "0.5h: 0004_tenant_offboarding_state.sql + offboarding_state enum + RLS + trigger"
  - "0.4h: 0005_tenant_offboarding_log.sql append-only"
  - "0.5h: fsm.rs — closed transition matrix"
  - "1.2h: scheduler.rs — hourly job + idempotency"
  - "0.6h: read_only_gate.rs — JWT issuance hook"
  - "1.0h: dead_letter.rs — S3 writer + dual-signoff restorer"
  - "0.5h: repo.rs"
  - "0.6h: audit.rs — 8 row builders"
  - "1.2h: handlers/offboarding.rs — initiate + cancel + extend + restore + force-advance (root-admin)"
  - "0.5h: jwt/issuer.rs hook integration"
  - "5.0h: tests — 13 test files (FSM + scheduler + read-only + cancellation rules + extensions + dead-letter + irreversible + append-only + audit + perf)"

risk_if_skipped: "Without TEN-104, every offboarding is operator-mental — tenants disappear from active state with no read-only grace, no dead-letter recovery, no compliance-grade attestation. GDPR Art. 17 satisfaction relies on the 90-day grace being a documented process; absent it, a regulator's request for 'show me the documented offboarding' is unanswerable. TASK-TEN-105 (signed-bundle export) consumes the terminating_a state to know when to allow exports. TASK-TEN-106 (permanent-delete attestation) consumes the terminating_b → terminated transition. Without DEC-509's read-only freeze, terminated tenants continue producing audit-noise writes (defeats the offboarding's intent). Without DEC-506's CSO+CLO dual-signoff + Object-Lock COMPLIANCE, accidental + malicious irreversible-wipe is one tenant-admin away. Without DEC-503's scheduled advance job, offboarding requires manual cron-style operator intervention. The 12h effort lands the legally-defensible tenant-lifecycle exit primitive."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship the 90-day offboarding contract — closed 4-state FSM + scheduled transitions + read-only freeze + dead-letter recovery + dual-signoff irreversible wipe. Each requirement:

1. **MUST** define `tenant_offboarding_state` table: `(tenant_id UUID PRIMARY KEY REFERENCES tenants(id), offboarding_state offboarding_state NOT NULL, initiated_at TIMESTAMPTZ, initiated_by_subject_id UUID, initiated_reason TEXT, scheduled_advance_at TIMESTAMPTZ, extension_count INT NOT NULL DEFAULT 0 CHECK (extension_count BETWEEN 0 AND 2), total_grace_days INT NOT NULL DEFAULT 90 CHECK (total_grace_days BETWEEN 30 AND 180), terminating_a_started_at TIMESTAMPTZ, terminating_b_started_at TIMESTAMPTZ, terminated_at TIMESTAMPTZ)`.

2. **MUST** declare the closed `offboarding_state` Postgres enum with exactly 4 values (per DEC-500): `'active'`, `'terminating_a'`, `'terminating_b'`, `'terminated'`. Adding a 5th is an ADR.

3. **MUST** define `tenant_offboarding_log` table: `(id BIGSERIAL, tenant_id UUID, from_state offboarding_state, to_state offboarding_state NOT NULL, changed_at TIMESTAMPTZ NOT NULL DEFAULT now(), changed_by_subject_id UUID NOT NULL, reason TEXT, signer1_subject_id UUID, signer2_subject_id UUID)`. `REVOKE UPDATE, DELETE FROM cyberos_app` (per DEC-508).

4. **MUST** enforce the closed FSM transition matrix (per DEC-500 + DEC-502):
- `active → terminating_a` (operator-initiated offboarding).
- `terminating_a → active` (cancellation; only within 30-day window).
- `terminating_a → terminating_b` (scheduled advance after 30 days; OR TASK-TEN-202 fast-track).
- `terminating_b → terminated` (scheduled advance after 60 days IN B + CSO+CLO dual-signoff).
- `active → terminating_b` (TASK-TEN-202 hostile-termination fast-track; CEO+CLO+CSO sign-off). All other transitions REJECTED at trigger + handler.

5. **MUST** ship the FSM at `services/ten/src/offboarding/fsm.rs` as a closed Rust function `validate_transition(from, to) -> Result<(), InvalidTransition>`. The trigger `enforce_offboarding_fsm` mirrors the matrix at DB level.

6. **MUST** ship the scheduled advance job at `services/ten/src/offboarding/scheduler.rs` running hourly (per DEC-503). Query: `SELECT tenant_id, offboarding_state FROM tenant_offboarding_state WHERE scheduled_advance_at <= now() AND offboarding_state IN ('terminating_a','terminating_b')`. For each row:
- If `terminating_a` → transition to `terminating_b`; emit memory row; wipe data to dead-letter bucket per §1 #12.
- If `terminating_b` → DO NOT auto-advance; emit `ten.dual_signoff_required` notification to ops; manual transition only. Idempotent — re-running on already-advanced tenant is a no-op (trigger predicate enforces).

7. **MUST** enforce read-only freeze in `terminating_a` (per DEC-509). The TASK-AUTH-004 JWT issuer consults `read_only_gate::is_tenant_read_only(tenant_id)` — true → JWT's `scope_grants` filtered to read-only operations (read | list | view; write | create | update | delete excluded). Existing tokens issued pre-freeze are NOT invalidated; their writes fail at handler with 423 `tenant_read_only` + emit `ten.read_only_write_attempted` memory row.

8. **MUST** ship `POST /v1/ten/offboarding/initiate` handler. Body: `{tenant_slug, reason, total_grace_days?}`. Caller MUST have role `root-admin` per TASK-AUTH-101. Validates:
- Tenant exists and is in `active` state.
- reason 1–500 chars.
- total_grace_days defaults to 90; if specified, must be 30-180. On success: UPDATE state to `terminating_a`; set `terminating_a_started_at = now()`, `scheduled_advance_at = now() + INTERVAL '30 days'` (or proportional); emit `ten.offboarding_initiated` + `ten.terminating_a_entered` memory rows.

9. **MUST** ship `POST /v1/ten/offboarding/cancel` (per DEC-510). Caller MUST have role `root-admin` OR `tenant-admin` for the target tenant. Body: `{tenant_slug, reason, confirmation: "I_UNDERSTAND_CANCELLATION"}`. Validates:
- Tenant is in `terminating_a` state (NEVER allowed in terminating_b or terminated).
- confirmation string matches exactly. On success: UPDATE state to `active`; clear `terminating_a_started_at`, `scheduled_advance_at`; emit `ten.terminating_cancelled` memory row.

10. **MUST** ship `POST /v1/ten/offboarding/extend` (per DEC-512). Caller MUST have role `clo` (Chief Legal Officer) per TASK-AUTH-101. Body: `{tenant_slug, additional_days, reason}`. Validates:
- Tenant in `terminating_a` OR `terminating_b`.
- additional_days in [1, 30].
- reason non-empty.
- extension_count < 2. On success: UPDATE `scheduled_advance_at = scheduled_advance_at + additional_days`; UPDATE `extension_count = extension_count + 1`; UPDATE `total_grace_days += additional_days`; emit `ten.offboarding_extended` memory row.

11. **MUST** ship `POST /v1/ten/offboarding/finalize-termination` (per DEC-506). Body: `{tenant_slug, signer1_subject_id, signer2_subject_id, reason, confirmation: "I_AUTHORISE_IRREVERSIBLE_DELETE"}`. Validates:
- Tenant in `terminating_b`.
- `signer1` has role `cseco` (Chief Security Officer) per TASK-AUTH-101.
- `signer2` has role `clo`.
- `signer1 != signer2`.
- confirmation matches. On success: UPDATE state to `terminated`; set `terminated_at = now()`; emit `ten.tenant_terminated` memory row; trigger TASK-TEN-106 attestation row.

12. **MUST** wipe tenant data to dead-letter S3 bucket on `terminating_a → terminating_b` transition (per DEC-505). The dead-letter writer:
- Streams Postgres tables tagged with the tenant_id to S3 (per-table CSV/JSONL export).
- Encrypts at rest with KMS key separate from production keyspace.
- Applies S3 Object-Lock COMPLIANCE mode with retention = `terminating_b_started_at + 60 days` (or per-tenant policy override; min 30 days).
- Deletes the rows from production Postgres after S3 commit confirmation.
- Emits `ten.dead_letter_written` row carrying `byte_count`, `table_count`, `kms_key_id`, `s3_bucket`, `retention_until`.

13. **MUST** support `POST /v1/ten/offboarding/restore-from-dead-letter` (per DEC-513). Body: `{tenant_slug, signer1_subject_id, signer2_subject_id, reason, confirmation: "I_AUTHORISE_DEAD_LETTER_RESTORE"}`. Validates same shape as §1 #11 (CSO + CLO dual-signoff). Tenant must be in `terminating_b`. On success: restore rows from S3 dead-letter into production Postgres; UPDATE state to `terminating_a` (re-enter the 30-day grace); emit `ten.dead_letter_restored` memory row sev-1.

14. **MUST** emit 8 memory audit row kinds (per DEC-507):
- `ten.offboarding_initiated` — first initiation.
- `ten.terminating_a_entered` — entry into read-only grace.
- `ten.terminating_b_entered` — entry into dead-letter grace.
- `ten.terminating_cancelled` — cancellation from terminating_a.
- `ten.tenant_terminated` — irreversible terminal state.
- `ten.offboarding_extended` — extension applied.
- `ten.dead_letter_restored` — restore from dead-letter; sev-1.
- `ten.read_only_write_attempted` — write rejected during terminating_a.

15. **MUST** PII-scrub `reason` and `initiated_reason` fields via TASK-MEMORY-111 before chain commit.

16. **MUST** ensure the FSM trigger (`enforce_offboarding_fsm`) covers ALL forbidden transitions (per DEC-514). Trigger raises specific error codes per illegal transition; tests assert each.

17. **MUST** pin `terminated_at` immutable post-set — once terminated, the row's terminated_at cannot be changed. A `BEFORE UPDATE` trigger rejects.

18. **MUST** enforce `extension_count <= 2` at DB CHECK constraint (per DEC-512). Third extension attempt fails at INSERT/UPDATE with `extension_count_exceeded`.

19. **MUST** ensure the read-only gate caches per-tenant state with 60-second TTL (matching TASK-AUTH-109's pattern). Cache invalidated on state transition.

20. **MUST** validate `total_grace_days` ∈ [30, 180] at handler initiate + extend. Below 30 (GDPR minimum reasonable grace) → 400; above 180 → 400.

21. **MUST** support per-tenant `dead_letter_retention_days` override in `tenant_offboarding_state` (default 60; min 30). Allows high-trust tenants longer recovery window OR high-risk tenants shorter retention.

22. **MUST** complete state transition handlers in ≤ 200 ms p95 (excluding async dead-letter wipe). `offboarding_perf_test`.

23. **MUST** emit OTel span `ten.offboarding.{initiate,advance,cancel,extend,terminate,restore,write_attempted}` with `outcome` attribute (success | invalid_transition | wrong_state | already_terminated | wrong_role | signer_role_mismatch | self_co_sign | confirmation_mismatch | extension_count_exceeded | dead_letter_unavailable).

24. **MUST** emit OTel metrics:
- `ten_offboarding_state_count{state}` (gauge — count of tenants per state).
- `ten_offboarding_transitions_total{from_state, to_state, outcome}` (counter).
- `ten_offboarding_read_only_writes_blocked_total{tenant_id}` (counter — sev-3 alarm at > 100/h indicates stuck client).
- `ten_dead_letter_bytes_total{tenant_id}` (counter — bytes wiped to dead letter).
- `ten_offboarding_active_extensions{tenant_id}` (gauge — current extension_count).

25. **MUST** ship `GET /v1/ten/offboarding/state/{tenant_slug}` for operator visibility. Returns `{state, initiated_at, scheduled_advance_at, days_remaining_in_state, extension_count, can_cancel: bool, can_extend: bool, can_terminate: bool}`. Caller MUST be root-admin or tenant-admin for the target tenant.

26. **MUST** support `POST /v1/ten/offboarding/force-advance` for emergency operator override — caller MUST be root-admin. Forces FSM transition with explicit reason. Emits memory row at sev-1 (operator override is forensically critical). Used for `terminating_b → terminated` after dual-signoff handler when scheduled advance hasn't fired yet.

---

## §2 — Why this design (rationale for humans)

**Why closed 4-state FSM (DEC-500)?** Each state corresponds to a distinct legal+operational meaning. `active` = customer pays + has full access. `terminating_a` = grace period for export + cancellation. `terminating_b` = data already wiped to dead-letter; only restoration is recovery. `terminated` = irreversible per Object-Lock. A 5th state (e.g. `suspended` for non-payment) is task-TEN-2xx — distinct lifecycle, separate FSM. Closed enum prevents drift.

**Why 30+60 = 90 days default (DEC-501)?** GDPR Art. 17 doesn't specify a fixed grace, but industry practice is 90 days (matches Salesforce, Microsoft 365, Zendesk). 30-day RO + export covers the "I changed my mind" case; 60-day dead-letter covers the "I changed my mind two months later" case. Per-tenant config allows 30-180 for high-trust or high-risk variants.

**Why terminating_b → active forbidden (DEC-502)?** By terminating_b entry, the production data has been wiped to S3 dead-letter. "Active" requires production data; restoring from dead-letter goes via `terminating_a` re-entry (a fresh 30-day grace). Forbidding the direct path prevents partial-restore confusion.

**Why hourly scheduled advance (DEC-503)?** Day-pinned transitions (30 day, 60 day) don't need minute-level precision. Hourly job is cheap to run and gives < 1h transition latency. The scheduler is idempotent — `WHERE scheduled_advance_at <= now() AND state = '<expected>'` prevents double-advance under concurrent runs.

**Why scheduled job DOESN'T auto-terminate (DEC-506, §1 #6)?** `terminating_b → terminated` is irreversible (Object-Lock COMPLIANCE). Auto-firing would mean a buggy job could mass-terminate tenants. Manual dual-signoff CSO+CLO is the deliberate gate; the scheduled job emits notification but never advances.

**Why CSO + CLO dual-signoff (DEC-506, §1 #11)?** Terminated tenant = permanent data loss = legal-grade decision. CSO (Chief Security Officer) confirms operational readiness; CLO (Chief Legal Officer) confirms legal sufficiency. Two distinct roles + non-self-co-sign prevents single-operator catastrophic action.

**Why read-only freeze at JWT issuance (DEC-509, §1 #7)?** Filtering at JWT issuance time is cheap (one lookup); enforcing at every handler is expensive (N × handler-count). The cache (60s TTL) makes the hot path fast. Existing tokens are not invalidated — their writes fail at handler (defense in depth) with clear 423 + emit audit.

**Why 423 status for read-only write (RFC 4918)?** "Locked" is the WebDAV semantic but increasingly used for "resource exists but is read-only". 403 would suggest permission; 410 would suggest gone. 423 communicates "the data is here but you cannot write to it now".

**Why max 2 extensions per cycle (DEC-512, §1 #18)?** Extensions are escape valves for stuck client cleanups; not policy-as-extension. 2 × 30 days max = 60 days additional = total 150 days. Below 30 days minimum is too aggressive; > 180 days defeats the offboarding's intent.

**Why dead-letter restore is sev-1 memory row (DEC-513, §1 #14)?** Restore is a rare + consequential event (operator decided "we shouldn't have offboarded; restore"). Sev-1 ensures every restore is visible in OBS digests for root-cause analysis.

**Why dead-letter Object-Lock COMPLIANCE (DEC-505, DEC-506)?** Compliance mode prevents even AWS root account from deleting before retention expires. Governance mode would allow root override; we want the irreversibility guarantee.

**Why per-tenant dead_letter_retention_days override (§1 #21)?** Some tenants want 30 days (privacy-strict EU); some want 90 days (extra recovery margin). Range 30-90 + per-tenant config preserves the GDPR-floor + tenant-flexibility tradeoff.

**Why explicit confirmation string per handler (§1 #9, #11, #13)?** Destructive operations (cancel, terminate, restore) are rare; operator confirms by typing the exact string. Prevents accidental click-through.

**Why TASK-TEN-202 fast-track bypasses 30-day terminating_a (DEC-511)?** Hostile termination (regulatory order, court injunction) demands faster action than 90-day grace allows. CEO+CLO+CSO triple-sign-off (vs CSO+CLO dual for normal flow) reflects the elevated authority. This task ships the FSM that TASK-TEN-202 consumes; the fast-track handler is TASK-TEN-202.

**Why FSM at trigger AND handler (DEC-514, §1 #16)?** Trigger catches direct SQL access (e.g. operator psql session bypassing handlers). Handler provides clearer error responses (HTTP status, JSON body). Both ensure invariant holds.

**Why pin scheduled_advance_at at entry (DEC-515, §1 #6)?** Captures the target transition time as a column — scheduler queries directly. Extensions UPDATE this; cancellation clears. The pinning makes the day-pinned transition explicit rather than implicit from `terminating_a_started_at + 30 days` (which would re-derive on every job run, susceptible to drift if base column changes).

**Why CLO role for extension, not root-admin (§1 #10)?** Extension is a legal-judgment call (is this delay reasonable + non-prejudicial?). CLO is the natural role; root-admin is more about technical operations. Per RBAC catalogue both have authority, but the principle of least privilege puts extension under CLO.

**Why tenant-admin can also cancel (§1 #9)?** Cancellation is "I changed my mind" — the tenant's own decision. tenant-admin (representing the tenant) has legitimate authority. root-admin (representing CyberSkill ops) can also cancel for operational reasons (e.g. payment received late).

**Why read_only_write_attempted as a memory row + alarm (§1 #14, §1 #24)?** Sustained write attempts during freeze (> 100/h) signal a stuck client that hasn't observed the read-only state — operator notification helps identify integrations needing attention before terminating_b wipe.

**Why force-advance is sev-1 (§1 #26)?** Operator overrides of FSM are forensically critical — "operator advanced to terminated before scheduled time" is exactly the kind of event regulators want to see in audit chains. Sev-1 ensures unmissable.

---

## §3 — API contract

### 3.1 — Migration 0004 — tenant_offboarding_state

```sql
-- services/ten/migrations/0004_tenant_offboarding_state.sql

BEGIN;

CREATE TYPE offboarding_state AS ENUM ('active', 'terminating_a', 'terminating_b', 'terminated');

CREATE TABLE tenant_offboarding_state (
    tenant_id                       UUID         PRIMARY KEY REFERENCES tenants(id),
    offboarding_state               offboarding_state NOT NULL DEFAULT 'active',
    initiated_at                    TIMESTAMPTZ,
    initiated_by_subject_id         UUID,
    initiated_reason                TEXT         CHECK (initiated_reason IS NULL OR length(initiated_reason) BETWEEN 1 AND 500),
    scheduled_advance_at            TIMESTAMPTZ,
    extension_count                 INT          NOT NULL DEFAULT 0 CHECK (extension_count BETWEEN 0 AND 2),
    total_grace_days                INT          NOT NULL DEFAULT 90 CHECK (total_grace_days BETWEEN 30 AND 180),
    dead_letter_retention_days      INT          NOT NULL DEFAULT 60 CHECK (dead_letter_retention_days BETWEEN 30 AND 90),
    terminating_a_started_at        TIMESTAMPTZ,
    terminating_b_started_at        TIMESTAMPTZ,
    terminated_at                   TIMESTAMPTZ
);

ALTER TABLE tenant_offboarding_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY ten_offboarding_state_policy ON tenant_offboarding_state
    USING (current_setting('auth.is_root_admin', true) = 'true'
           OR tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (current_setting('auth.is_root_admin', true) = 'true'
                OR tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON tenant_offboarding_state FROM cyberos_app;
GRANT INSERT, UPDATE (offboarding_state, initiated_at, initiated_by_subject_id, initiated_reason,
                     scheduled_advance_at, extension_count, total_grace_days, dead_letter_retention_days,
                     terminating_a_started_at, terminating_b_started_at, terminated_at)
    ON tenant_offboarding_state TO cyberos_provisioner;

-- FSM enforcement (DEC-514)
CREATE OR REPLACE FUNCTION enforce_offboarding_fsm() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.offboarding_state = OLD.offboarding_state THEN RETURN NEW; END IF;
    IF NOT (
        (OLD.offboarding_state = 'active'         AND NEW.offboarding_state = 'terminating_a')
        OR (OLD.offboarding_state = 'active'      AND NEW.offboarding_state = 'terminating_b')   -- TASK-TEN-202 hostile
        OR (OLD.offboarding_state = 'terminating_a' AND NEW.offboarding_state = 'active')        -- cancel
        OR (OLD.offboarding_state = 'terminating_a' AND NEW.offboarding_state = 'terminating_b') -- scheduled
        OR (OLD.offboarding_state = 'terminating_b' AND NEW.offboarding_state = 'terminating_a') -- dead-letter restore
        OR (OLD.offboarding_state = 'terminating_b' AND NEW.offboarding_state = 'terminated')    -- finalize
    ) THEN
        RAISE EXCEPTION 'invalid_offboarding_transition: % -> %', OLD.offboarding_state, NEW.offboarding_state USING ERRCODE = 'P0090';
    END IF;
    -- Immutable terminated_at post-set
    IF OLD.terminated_at IS NOT NULL AND NEW.terminated_at IS DISTINCT FROM OLD.terminated_at THEN
        RAISE EXCEPTION 'terminated_at_immutable' USING ERRCODE = 'P0091';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_ten_offboarding_fsm BEFORE UPDATE ON tenant_offboarding_state
    FOR EACH ROW EXECUTE FUNCTION enforce_offboarding_fsm();

-- Seed: every tenant starts in 'active'
INSERT INTO tenant_offboarding_state (tenant_id, offboarding_state)
SELECT id, 'active' FROM tenants ON CONFLICT (tenant_id) DO NOTHING;

COMMIT;
```

### 3.2 — Migration 0005 — append-only log

```sql
-- services/ten/migrations/0005_tenant_offboarding_log.sql

BEGIN;

CREATE TABLE tenant_offboarding_log (
    id                       BIGSERIAL    PRIMARY KEY,
    tenant_id                UUID         NOT NULL REFERENCES tenants(id) ON DELETE RESTRICT,
    from_state               offboarding_state,
    to_state                 offboarding_state NOT NULL,
    changed_at               TIMESTAMPTZ  NOT NULL DEFAULT now(),
    changed_by_subject_id    UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    reason                   TEXT,
    signer1_subject_id       UUID,
    signer2_subject_id       UUID
);

CREATE INDEX ten_offboarding_log_tenant_idx ON tenant_offboarding_log (tenant_id, changed_at DESC);

ALTER TABLE tenant_offboarding_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY ten_offboarding_log_policy ON tenant_offboarding_log
    USING (current_setting('auth.is_root_admin', true) = 'true'
           OR tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (current_setting('auth.is_root_admin', true) = 'true');

REVOKE UPDATE, DELETE ON tenant_offboarding_log FROM cyberos_app;

COMMIT;
```

### 3.3 — FSM validator

```rust
// services/ten/src/offboarding/fsm.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type, Serialize, Deserialize)]
#[sqlx(type_name = "offboarding_state", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OffboardingState { Active, TerminatingA, TerminatingB, Terminated }

impl OffboardingState {
    pub const ALL: &'static [OffboardingState] = &[
        OffboardingState::Active, OffboardingState::TerminatingA,
        OffboardingState::TerminatingB, OffboardingState::Terminated,
    ];
}

#[derive(Debug, thiserror::Error)]
#[error("invalid_offboarding_transition: {from:?} -> {to:?}")]
pub struct InvalidTransition { pub from: OffboardingState, pub to: OffboardingState }

pub fn is_valid_transition(from: OffboardingState, to: OffboardingState) -> bool {
    use OffboardingState::*;
    matches!((from, to),
        (Active, TerminatingA)
        | (Active, TerminatingB)       // TASK-TEN-202 hostile
        | (TerminatingA, Active)       // cancel
        | (TerminatingA, TerminatingB) // scheduled
        | (TerminatingB, TerminatingA) // dead-letter restore
        | (TerminatingB, Terminated)   // finalize
    )
}

pub fn validate_transition(from: OffboardingState, to: OffboardingState) -> Result<(), InvalidTransition> {
    if is_valid_transition(from, to) { Ok(()) } else { Err(InvalidTransition { from, to }) }
}
```

### 3.4 — Scheduler

```rust
// services/ten/src/offboarding/scheduler.rs
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::offboarding::fsm::OffboardingState;

pub async fn run_hourly_advance(db: &sqlx::PgPool) -> anyhow::Result<()> {
    // Find tenants ready to advance
    let candidates: Vec<(Uuid, OffboardingState, DateTime<Utc>)> = sqlx::query_as(r#"
        SELECT tenant_id, offboarding_state, scheduled_advance_at
        FROM tenant_offboarding_state
        WHERE scheduled_advance_at <= now()
          AND offboarding_state IN ('terminating_a','terminating_b')
        FOR UPDATE SKIP LOCKED
    "#).fetch_all(db).await?;

    for (tenant_id, state, _scheduled) in candidates {
        match state {
            OffboardingState::TerminatingA => {
                advance_a_to_b(db, tenant_id).await?;
            }
            OffboardingState::TerminatingB => {
                // DO NOT auto-advance to terminated (per DEC-506 — dual signoff required)
                notify_ops_terminating_b_ready(db, tenant_id).await?;
            }
            _ => continue,
        }
    }
    Ok(())
}

async fn advance_a_to_b(db: &sqlx::PgPool, tenant_id: Uuid) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    // Mark state transition (trigger validates)
    let dead_letter_days: i32 = sqlx::query_scalar(
        "SELECT dead_letter_retention_days FROM tenant_offboarding_state WHERE tenant_id = $1 FOR UPDATE"
    ).bind(tenant_id).fetch_one(&mut *tx).await?;

    sqlx::query(r#"
        UPDATE tenant_offboarding_state
        SET offboarding_state = 'terminating_b'::offboarding_state,
            terminating_b_started_at = now(),
            scheduled_advance_at = now() + ($2 || ' days')::interval
        WHERE tenant_id = $1
          AND offboarding_state = 'terminating_a'::offboarding_state
    "#).bind(tenant_id).bind(dead_letter_days as i64).execute(&mut *tx).await?;

    // Log + audit
    sqlx::query(r#"
        INSERT INTO tenant_offboarding_log (tenant_id, from_state, to_state, changed_by_subject_id, reason)
        VALUES ($1, 'terminating_a'::offboarding_state, 'terminating_b'::offboarding_state, $2, 'scheduled_advance')
    "#).bind(tenant_id).bind(system_subject_id()).execute(&mut *tx).await?;

    crate::offboarding::audit::emit_terminating_b_entered(&mut tx, tenant_id).await?;
    tx.commit().await?;

    // Async: wipe to dead-letter (returns soon; actual work in background)
    tokio::spawn(crate::offboarding::dead_letter::wipe_tenant_to_dead_letter(tenant_id, db.clone()));
    Ok(())
}

async fn notify_ops_terminating_b_ready(db: &sqlx::PgPool, tenant_id: Uuid) -> anyhow::Result<()> {
    // Emit OBS sev-3 notification — operator must invoke finalize-termination handler
    metrics::counter!("ten_offboarding_b_ready_count", "tenant_id" => tenant_id.to_string()).increment(1);
    Ok(())
}

fn system_subject_id() -> Uuid {
    Uuid::nil()  // Schedule runs as system; subject_id_hash16 = nil
}
```

### 3.5 — Read-only gate

```rust
// services/ten/src/offboarding/read_only_gate.rs
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;
use crate::offboarding::fsm::OffboardingState;

pub struct ReadOnlyGate {
    cache: Arc<RwLock<HashMap<Uuid, (bool, Instant)>>>,
}

impl ReadOnlyGate {
    pub fn new() -> Self {
        Self { cache: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn is_tenant_read_only(&self, tenant_id: Uuid, db: &sqlx::PgPool) -> bool {
        if let Some((ro, t)) = self.cache.read().await.get(&tenant_id) {
            if t.elapsed() < Duration::from_secs(60) {
                return *ro;
            }
        }
        let state: Option<OffboardingState> = sqlx::query_scalar(
            "SELECT offboarding_state FROM tenant_offboarding_state WHERE tenant_id = $1"
        ).bind(tenant_id).fetch_optional(db).await.ok().flatten();
        let ro = matches!(state, Some(OffboardingState::TerminatingA) | Some(OffboardingState::TerminatingB) | Some(OffboardingState::Terminated));
        self.cache.write().await.insert(tenant_id, (ro, Instant::now()));
        ro
    }

    pub async fn invalidate(&self, tenant_id: Uuid) {
        self.cache.write().await.remove(&tenant_id);
    }
}
```

### 3.6 — Initiate handler

```rust
// services/ten/src/handlers/offboarding.rs (excerpt)
use axum::{Json, extract::State, http::StatusCode};
use cyberos_auth::rbac::Role;
use crate::offboarding::{fsm::OffboardingState, audit};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct InitiateRequest {
    pub tenant_slug: String,
    pub reason: String,
    pub total_grace_days: Option<i32>,
}

pub async fn initiate_offboarding(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<InitiateRequest>,
) -> Result<StatusCode, ApiError> {
    if !claims.roles().contains(&Role::RootAdmin) {
        return Err(ApiError::PermissionDenied);
    }
    if req.reason.is_empty() || req.reason.len() > 500 {
        return Err(ApiError::ReasonInvalid);
    }
    let grace = req.total_grace_days.unwrap_or(90);
    if !(30..=180).contains(&grace) {
        return Err(ApiError::GraceDaysOutOfRange);
    }

    let tenant_id = state.repo.find_tenant_by_slug(&req.tenant_slug).await?
        .ok_or(ApiError::TenantUnknown)?;

    let mut tx = state.db.begin().await?;
    // The 30/60 split holds the proportion: 30/(30+60) = 1/3 → terminating_a = grace/3
    let a_days = (grace * 30) / 90;
    sqlx::query(r#"
        UPDATE tenant_offboarding_state
        SET offboarding_state = 'terminating_a'::offboarding_state,
            initiated_at = now(),
            initiated_by_subject_id = $2,
            initiated_reason = $3,
            terminating_a_started_at = now(),
            scheduled_advance_at = now() + ($4 || ' days')::interval,
            total_grace_days = $5
        WHERE tenant_id = $1 AND offboarding_state = 'active'::offboarding_state
    "#).bind(tenant_id).bind(claims.subject_id()).bind(&req.reason)
       .bind(a_days as i64).bind(grace).execute(&mut *tx).await?;

    sqlx::query(r#"
        INSERT INTO tenant_offboarding_log (tenant_id, from_state, to_state, changed_by_subject_id, reason)
        VALUES ($1, 'active'::offboarding_state, 'terminating_a'::offboarding_state, $2, $3)
    "#).bind(tenant_id).bind(claims.subject_id()).bind(&req.reason).execute(&mut *tx).await?;

    audit::emit_offboarding_initiated(&mut tx, tenant_id, claims.subject_id(), &req.reason).await?;
    audit::emit_terminating_a_entered(&mut tx, tenant_id).await?;
    tx.commit().await?;
    state.read_only_gate.invalidate(tenant_id).await;
    Ok(StatusCode::CREATED)
}
```

### 3.7 — Dual-signoff termination

```rust
// services/ten/src/handlers/offboarding.rs (continued)
#[derive(Deserialize)]
pub struct FinalizeTerminationRequest {
    pub tenant_slug: String,
    pub signer1_subject_id: Uuid,
    pub signer2_subject_id: Uuid,
    pub reason: String,
    pub confirmation: String,
}

const TERMINATE_CONFIRMATION: &str = "I_AUTHORISE_IRREVERSIBLE_DELETE";

pub async fn finalize_termination(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<FinalizeTerminationRequest>,
) -> Result<StatusCode, ApiError> {
    if req.confirmation != TERMINATE_CONFIRMATION {
        return Err(ApiError::ConfirmationMismatch);
    }
    if req.signer1_subject_id == req.signer2_subject_id {
        return Err(ApiError::SelfCoSign);
    }
    // Verify signer roles
    let s1_roles = state.repo.subject_roles(req.signer1_subject_id).await?;
    let s2_roles = state.repo.subject_roles(req.signer2_subject_id).await?;
    if !s1_roles.contains(&Role::Cseco) {
        return Err(ApiError::SignerRoleMismatch { required: Role::Cseco });
    }
    if !s2_roles.contains(&Role::Clo) {
        return Err(ApiError::SignerRoleMismatch { required: Role::Clo });
    }

    let tenant_id = state.repo.find_tenant_by_slug(&req.tenant_slug).await?
        .ok_or(ApiError::TenantUnknown)?;
    let mut tx = state.db.begin().await?;
    sqlx::query(r#"
        UPDATE tenant_offboarding_state
        SET offboarding_state = 'terminated'::offboarding_state,
            terminated_at = now(),
            scheduled_advance_at = NULL
        WHERE tenant_id = $1 AND offboarding_state = 'terminating_b'::offboarding_state
    "#).bind(tenant_id).execute(&mut *tx).await?;

    sqlx::query(r#"
        INSERT INTO tenant_offboarding_log (tenant_id, from_state, to_state, changed_by_subject_id, reason, signer1_subject_id, signer2_subject_id)
        VALUES ($1, 'terminating_b'::offboarding_state, 'terminated'::offboarding_state, $2, $3, $4, $5)
    "#).bind(tenant_id).bind(claims.subject_id()).bind(&req.reason)
       .bind(req.signer1_subject_id).bind(req.signer2_subject_id).execute(&mut *tx).await?;

    audit::emit_tenant_terminated(&mut tx, tenant_id, req.signer1_subject_id, req.signer2_subject_id, &req.reason).await?;
    tx.commit().await?;
    state.read_only_gate.invalidate(tenant_id).await;

    // Trigger TASK-TEN-106 attestation row (out of scope here; placeholder)
    Ok(StatusCode::OK)
}
```

---

## §4 — Acceptance criteria

1. **OffboardingState enum closed at 4** — active, terminating_a, terminating_b, terminated.
2. **Seed at migration** — every existing tenant gets offboarding_state='active' row.
3. **active → terminating_a happy path** — root-admin initiate → 201; row updated; `ten.offboarding_initiated` + `ten.terminating_a_entered` rows emitted.
4. **terminating_a → active cancel** — within 30-day window → 200; row reverts; `ten.terminating_cancelled` row.
5. **terminating_b → active forbidden** — trigger raises invalid_offboarding_transition.
6. **terminated → * forbidden** — trigger rejects.
7. **active → terminated direct forbidden** — must pass through B (only TASK-TEN-202 fast-track allows active → terminating_b).
8. **Scheduled job advances A → B** — tenant with scheduled_advance_at < now() → state flips to terminating_b after hourly run; dead-letter write triggered.
9. **Scheduled job DOES NOT advance B → terminated** — terminating_b with past scheduled_advance_at → notification only.
10. **CSO + CLO dual-signoff terminate** — valid signers + correct confirmation → state → terminated; `ten.tenant_terminated` row.
11. **Single-signer terminate rejected** — same signer for both fields → 400 self_co_sign.
12. **Wrong signer role rejected** — non-CSO at signer1 → 400 signer_role_mismatch.
13. **Confirmation string mismatch** — wrong confirmation → 400 confirmation_mismatch.
14. **Extension by CLO succeeds** — additional_days in [1,30] + extension_count < 2 → 200.
15. **Extension by non-CLO rejected** → 403.
16. **3rd extension rejected** → 409 extension_count_exceeded.
17. **Read-only freeze in terminating_a** — JWT issued for tenant in terminating_a has scope_grants filtered to read-only.
18. **Write attempt in terminating_a** → 423 tenant_read_only + `ten.read_only_write_attempted` row.
19. **Dead-letter write on A→B transition** — S3 object created with Object-Lock COMPLIANCE; `ten.dead_letter_written` row.
20. **Dead-letter restore by CSO+CLO** — restores rows + state → terminating_a + sev-1 audit.
21. **terminated_at immutable** — UPDATE attempt → trigger raises terminated_at_immutable.
22. **append-only tenant_offboarding_log** — UPDATE/DELETE rejected.
23. **append-only via auth_provisioner role only** — cyberos_app blocked.
24. **GET /offboarding/state returns days_remaining + can_cancel + can_extend + can_terminate**.
25. **Force-advance by root-admin** → state advances + sev-1 audit row.
26. **8 memory audit kinds emit correctly** — one per lifecycle event.
27. **OTel span emitted per handler** — outcome populated.
28. **Counter `ten_offboarding_state_count{state}` reflects current counts**.

---

## §5 — Verification

```rust
// services/ten/tests/offboarding_fsm_test.rs
use cyberos_ten::offboarding::fsm::{OffboardingState::*, is_valid_transition};

#[test]
fn valid_transitions() {
    for (from, to) in [(Active, TerminatingA), (Active, TerminatingB),
                       (TerminatingA, Active), (TerminatingA, TerminatingB),
                       (TerminatingB, TerminatingA), (TerminatingB, Terminated)] {
        assert!(is_valid_transition(from, to));
    }
}

#[test]
fn forbidden_transitions() {
    for (from, to) in [(Active, Terminated), (Terminated, Active),
                       (Terminated, TerminatingA), (Terminated, TerminatingB),
                       (TerminatingB, Active)] {
        assert!(!is_valid_transition(from, to));
    }
}
```

```rust
// services/ten/tests/offboarding_terminating_a_read_only_test.rs
#[tokio::test]
async fn write_attempt_returns_423_during_grace(ctx: TestCtx) {
    let tenant = ctx.create_tenant_in_terminating_a().await;
    let token = ctx.issue_token_for_tenant(tenant).await;
    let resp = ctx.post_with_token("/v1/some/write-endpoint", json!({"x":1}), &token).await;
    assert_eq!(resp.status(), 423);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"], "tenant_read_only");
    let rows = ctx.memory_audit_rows("ten.read_only_write_attempted").await;
    assert_eq!(rows.len(), 1);
}

#[tokio::test]
async fn read_attempt_succeeds_during_grace(ctx: TestCtx) {
    let tenant = ctx.create_tenant_in_terminating_a().await;
    let token = ctx.issue_token_for_tenant(tenant).await;
    let resp = ctx.get_with_token("/v1/some/read-endpoint", &token).await;
    assert_eq!(resp.status(), 200);
}
```

```rust
// services/ten/tests/offboarding_dead_letter_restore_dual_signoff_test.rs
#[tokio::test]
async fn restore_requires_both_cso_and_clo(ctx: TestCtx) {
    let tenant = ctx.create_tenant_in_terminating_b_with_dead_letter().await;
    let cso = ctx.subject_with_role(Role::Cseco).await;
    let other = ctx.subject_with_role(Role::Cfo).await;
    let err = ctx.post_as_root_admin("/v1/ten/offboarding/restore-from-dead-letter", json!({
        "tenant_slug": tenant.slug,
        "signer1_subject_id": cso,
        "signer2_subject_id": other,   // not CLO
        "reason": "test restore",
        "confirmation": "I_AUTHORISE_DEAD_LETTER_RESTORE",
    })).await.unwrap_err();
    assert!(format!("{err:?}").contains("signer_role_mismatch"));
}

#[tokio::test]
async fn restore_succeeds_with_proper_signers(ctx: TestCtx) {
    let tenant = ctx.create_tenant_in_terminating_b_with_dead_letter().await;
    let cso = ctx.subject_with_role(Role::Cseco).await;
    let clo = ctx.subject_with_role(Role::Clo).await;
    let resp = ctx.post_as_root_admin("/v1/ten/offboarding/restore-from-dead-letter", json!({
        "tenant_slug": tenant.slug,
        "signer1_subject_id": cso,
        "signer2_subject_id": clo,
        "reason": "Customer disputed termination; ops decided to restore",
        "confirmation": "I_AUTHORISE_DEAD_LETTER_RESTORE",
    })).await.unwrap();
    assert_eq!(resp.status(), 200);
    let updated = ctx.fetch_offboarding_state(tenant.id).await;
    assert_eq!(updated.offboarding_state, OffboardingState::TerminatingA);
    let rows = ctx.memory_audit_rows("ten.dead_letter_restored").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["severity"], "sev-1");
}
```

```rust
// services/ten/tests/offboarding_extension_max_2_test.rs
#[tokio::test]
async fn third_extension_rejected(ctx: TestCtx) {
    let tenant = ctx.create_tenant_in_terminating_a().await;
    let body = json!({"tenant_slug": tenant.slug, "additional_days": 10, "reason": "ext1"});
    ctx.post_as_clo("/v1/ten/offboarding/extend", body.clone()).await.unwrap();
    let body2 = json!({"tenant_slug": tenant.slug, "additional_days": 10, "reason": "ext2"});
    ctx.post_as_clo("/v1/ten/offboarding/extend", body2).await.unwrap();
    let body3 = json!({"tenant_slug": tenant.slug, "additional_days": 10, "reason": "ext3"});
    let err = ctx.post_as_clo("/v1/ten/offboarding/extend", body3).await.unwrap_err();
    assert!(format!("{err:?}").contains("extension_count_exceeded"));
}
```

```rust
// services/ten/tests/offboarding_terminated_irreversible_test.rs
#[sqlx::test]
async fn terminated_to_active_rejected_at_trigger(pool: sqlx::PgPool) {
    let tid = seed_terminated_tenant(&pool).await;
    set_role_provisioner(&pool).await;
    let err = sqlx::query("UPDATE tenant_offboarding_state SET offboarding_state = 'active'::offboarding_state WHERE tenant_id = $1")
        .bind(tid).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("invalid_offboarding_transition"));
}

#[sqlx::test]
async fn terminated_at_mutation_rejected(pool: sqlx::PgPool) {
    let tid = seed_terminated_tenant(&pool).await;
    set_role_provisioner(&pool).await;
    let err = sqlx::query("UPDATE tenant_offboarding_state SET terminated_at = now() + interval '1 day' WHERE tenant_id = $1")
        .bind(tid).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("terminated_at_immutable"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 8 memory row builders follow the canonical pattern; dead_letter wipe uses TASK-DOC-001 S3+KMS pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-TEN-001** — tenant provisioning (this task consumes tenants table).

**Downstream (2 placeholders):**
- **TASK-TEN-105** — signed-bundle export (consumes terminating_a state to gate exports).
- **TASK-TEN-106** — permanent-delete attestation row (consumes terminating_b → terminated transition).

**Cross-module:**
- **TASK-AUTH-101** — RBAC; root-admin (initiate/force), tenant-admin (cancel own), CLO (extend), CSO (terminate-signer1), CLO (terminate-signer2).
- **TASK-AUTH-004** — JWT issuer hooks read_only_gate at issuance.
- **TASK-AI-003** — memory audit bridge.
- **TASK-MEMORY-111** — PII scrub reason fields.
- **TASK-OBS-007** — sev-3 alarm on > 100/h read-only-write attempts; sev-1 always on terminated + restore.
- **TASK-DOC-001** — S3 Object-Lock COMPLIANCE pattern for dead-letter bucket.
- **TASK-TEN-202** — hostile-termination fast-track (consumes active → terminating_b transition).

---

## §8 — Example payloads

### 8.1 — POST /v1/ten/offboarding/initiate

```json
{
  "tenant_slug": "acme-corp",
  "reason": "Customer requested termination per contract clause 12.3; effective 2026-08-15",
  "total_grace_days": 90
}
```

### 8.2 — POST /v1/ten/offboarding/finalize-termination

```json
{
  "tenant_slug": "acme-corp",
  "signer1_subject_id": "<cseco-uuid>",
  "signer2_subject_id": "<clo-uuid>",
  "reason": "60-day dead-letter grace expired; no restoration request received; legal cleared.",
  "confirmation": "I_AUTHORISE_IRREVERSIBLE_DELETE"
}
```

### 8.3 — GET /v1/ten/offboarding/state/acme-corp

```json
{
  "offboarding_state": "terminating_a",
  "initiated_at": "2026-05-15T10:00:00Z",
  "scheduled_advance_at": "2026-06-14T10:00:00Z",
  "days_remaining_in_state": 14,
  "extension_count": 0,
  "can_cancel": true,
  "can_extend": true,
  "can_terminate": false
}
```

### 8.4 — ten.tenant_terminated memory row (sev-1)

```json
{
  "kind": "ten.tenant_terminated",
  "severity": "sev-1",
  "tenant_id": "5e8f1d2a-...",
  "signer1_subject_id_hash16": "abc1234567890def",
  "signer2_subject_id_hash16": "fed0987654321abc",
  "reason_scrubbed": "[REDACTED-DURATION] dead-letter grace expired; no restoration request received; legal cleared.",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — ten.read_only_write_attempted memory row

```json
{
  "kind": "ten.read_only_write_attempted",
  "tenant_id": "5e8f1d2a-...",
  "subject_id_hash16": "9b1deb4d3b7d4bad",
  "attempted_path": "/v1/proj/issues",
  "attempted_method": "POST",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Per-tenant read-only operation allowlist** — slice 4 (e.g. "allow time-tracking close-outs during terminating_a").
- **Automated dead-letter expiry job (terminating_b → terminated)** — operator-manual at slice 1.
- **Bulk offboarding (multiple tenants in one transaction)** — out of scope.
- **Webhook to tenant on state transitions** — slice 4.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid FSM transition | trigger | 400/500 | Designed |
| terminated_at mutation | trigger | rejected | Designed |
| Self-co-sign on terminate | handler | 400 | Designed |
| Wrong signer role | handler | 400 | Designed |
| Confirmation string mismatch | handler | 400 | Designed |
| 3rd extension | CHECK + handler | 409 | Designed |
| Extension out of [1,30] | handler | 400 | Designed |
| total_grace_days out of [30,180] | DB CHECK + handler | 400 | Designed |
| dead_letter_retention_days out of [30,90] | DB CHECK | INSERT/UPDATE fails | Designed |
| Concurrent scheduler runs | FOR UPDATE SKIP LOCKED | One wins per tenant | Designed |
| Dead-letter S3 write fail | error response | 500 + sev-1 | Retry; sev-1 alarm |
| Object-Lock COMPLIANCE not applied | reconciliation check | sev-1 | Verify mode + reapply |
| memory audit fail mid-tx | rollback | 500 | memory_writer health |
| Read-only gate cache stale | 60s TTL + invalidate | Brief delay | Designed |
| Non-CLO extension | role check | 403 | Designed |
| Non-root-admin initiate | role check | 403 | Designed |
| Cancel in terminating_b | state check | 400 wrong_state | Designed |
| Cancel in terminated | state check | 400 wrong_state | Designed |
| Force-advance by non-root-admin | role check | 403 | Designed |
| Scheduler ran but tenant already advanced (concurrent) | trigger 0 rows updated | No-op | Designed |
| append-only log UPDATE | SQL grant | permission denied | Designed |
| Subject deleted while log refs | FK RESTRICT | DELETE auth.subjects fails | Soft-delete |
| Cross-tenant RLS access | USING | 0 rows | Designed |
| Tenant in terminated reads | RLS allows | Read OK | Designed |
| Read-only-write rate > 100/h | counter alarm | sev-3 | Operator investigation |
| Dead-letter restore without S3 backup | dead_letter not found | 500 dead_letter_unavailable | Manual recovery via Object-Lock copy |
| Concurrent finalize-termination + restore | row lock + trigger | Serial | Designed |
| Missing scheduled_advance_at | scheduler skip | None | Designed |
| Scheduler job timing drift | hourly cadence | Sub-1h transition | Designed |
| auth provisioner role not granted | service refuses to write | INSERT fails | Grant role |
| Tenant in terminating_b reads | RLS allows | Read OK | Designed |

---

## §11 — Implementation notes

- **Closed 4-state FSM** — adding states is ADR.
- **30+60 default = 90 days** — matches industry SaaS norm; configurable [30,180].
- **terminating_b → active forbidden** — production data already wiped; restore goes via terminating_a.
- **Hourly scheduled advance + idempotent SKIP LOCKED** — no double-advance under concurrent runs.
- **DOES NOT auto-terminate** — dual-signoff required at every terminating_b → terminated.
- **Read-only freeze at JWT issuance + handler check** — defense in depth.
- **423 status for tenant_read_only** — WebDAV-style semantic.
- **Max 2 extensions** — bounded escape valve.
- **Restore sev-1 audit** — rare + consequential.
- **Object-Lock COMPLIANCE** — even root account can't delete dead letter.
- **Per-tenant dead_letter_retention_days override** — 30-90 days.
- **Explicit confirmation strings** — operator deliberate confirmation.
- **TASK-TEN-202 fast-track via active → terminating_b** — this task ships FSM; TEN-202 ships the triple-signoff handler.
- **FSM at trigger + handler** — direct SQL + handler both blocked.
- **scheduled_advance_at pinned at entry** — extensions UPDATE this.
- **CLO for extension, root-admin for initiate** — role separation.
- **tenant-admin can cancel own** — tenant's own decision.
- **Read-only-write attempts emit + alarm** — stuck-client visibility.
- **Force-advance sev-1** — operator override forensic-critical.
- **8 memory audit kinds** — selective operator queries.
- **PII scrub reason** — chain holds scrubbed.

---

*End of TASK-TEN-104.*
