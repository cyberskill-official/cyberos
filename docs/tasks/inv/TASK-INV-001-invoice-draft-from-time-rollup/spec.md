---
id: TASK-INV-001
title: "INV invoice substrate — draft invoices from TIME per-cycle rollup with rate-card snapshot preservation + closed enums + lifecycle FSM + per-line traceability"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: INV
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-INV-002, TASK-INV-003, TASK-INV-005, TASK-INV-006, TASK-INV-007, TASK-TIME-009, TASK-TEN-003, TASK-TEN-102, TASK-PORTAL-001, TASK-PORTAL-006, TASK-CRM-001, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007]
depends_on: [TASK-TIME-009]   # TASK-TIME-009 placeholder — not yet specified
blocks: [TASK-INV-002, TASK-INV-007, TASK-INV-009, TASK-INV-011]   # INV-009/011 use INV-001 invoice substrate; INV-003/005/006 already shipped + depend on AUTH-101 instead of this task

source_pages:
  - website/docs/modules/inv.html
  - https://thuvienphapluat.vn/van-ban/Doanh-nghiep/Nghi-dinh-123-2020-ND-CP-quy-dinh-ve-hoa-don-chung-tu-454681.aspx

source_decisions:
  - DEC-1360 2026-05-17 — Invoice = the canonical billable artefact tying TIME entries (TASK-TIME-009 cycle rollup) to a client engagement; produces a draft auto-generated at cycle boundary; manually reviewed + adjusted; sent for payment
  - DEC-1361 2026-05-17 — Closed enum `invoice_status` = {draft, ready_for_review, approved, sent, partially_paid, paid, void, written_off}; CI cardinality test asserts 8
  - DEC-1362 2026-05-17 — Closed enum `invoice_line_kind` = {time_entry, fixed_fee, expense_reimbursement, discount, tax, late_fee}; CI cardinality test asserts 6
  - DEC-1363 2026-05-17 — Rate-card snapshot: at invoice creation, the prevailing rate-card (per-person-per-hour or fixed-fee schedule) is COPIED into the invoice — never referenced live; ensures invoice immutability even if rate-card changes
  - DEC-1364 2026-05-17 — Per-line traceability: each `invoice_line` row carries `source_kind` + `source_ref` (e.g., time_entry_id, expense_id) for forensic backtracking to the entry that generated the charge
  - DEC-1365 2026-05-17 — Amounts stored as BIGINT minor (task-audit skill rule 11) + closed currency enum (billing_currency_enum from TASK-TEN-003 reused)
  - DEC-1366 2026-05-17 — Append-only at line + header level: corrections via NEW lines (`source_kind='correction'`) NOT mutations; status transitions via append-only `invoice_status_history` row
  - "DEC-1367 2026-05-17 — Invoice number format: `INV-{tenant_slug}-{YY}-{6-digit-sequential}`; per-tenant annual gap-free sequence (same pattern as TASK-TEN-102 hóa đơn DEC-983)"
  - DEC-1368 2026-05-17 — Per-engagement billing: invoice belongs to ONE engagement (multi-engagement invoices = anti-pattern; client confusion + audit-attribution unclear)
  - DEC-1369 2026-05-17 — Draft auto-generation: scheduled job at end of each billing cycle (per-engagement cycle config — monthly default) creates draft from unbilled TIME entries; manual override path for off-cycle invoices
  - DEC-1370 2026-05-17 — Approval gate: only `cfo` or `engagement_admin` can transition `ready_for_review → approved`; sent transition requires approved status
  - DEC-1371 2026-05-17 — Write-off requires CFO + reason; preserves audit trail; surfaces in financial reports per task-INV-2xx
  - DEC-1372 2026-05-17 — VAT handling at slice 1: per-line VAT rate (decimal 0-100%), defaults to engagement-configured rate; line.amount_minor is pre-tax; totals computed at render time
  - DEC-1373 2026-05-17 — Multi-currency support deferred to TASK-INV-002; slice 1 = engagement.billing_currency only (single-currency per invoice per DEC-1368 derivative)
  - DEC-1374 2026-05-17 — memory audit kinds: inv.draft_created, inv.lines_added, inv.status_transitioned, inv.approved, inv.sent, inv.paid, inv.void, inv.written_off, inv.correction_added

build_envelope:
  language: rust 1.81
  service: cyberos/services/inv/
  new_files:
    - services/inv/migrations/0001_invoices.sql
    - services/inv/migrations/0002_invoice_lines.sql
    - services/inv/migrations/0003_invoice_status_history.sql
    - services/inv/migrations/0004_invoice_number_sequence.sql
    - services/inv/migrations/0005_rate_card_snapshot.sql
    - services/inv/src/lib.rs
    - services/inv/src/types.rs
    - services/inv/src/draft/mod.rs
    - services/inv/src/draft/builder.rs
    - services/inv/src/draft/scheduler.rs
    - services/inv/src/lines/mod.rs
    - services/inv/src/lines/time_rollup.rs
    - services/inv/src/lines/corrections.rs
    - services/inv/src/status/mod.rs
    - services/inv/src/status/state_machine.rs
    - services/inv/src/numbering/mod.rs
    - services/inv/src/snapshot/rate_card.rs
    - services/inv/src/handlers/invoice_routes.rs
    - services/inv/src/audit/invoice_events.rs
    - services/inv/tests/invoice_draft_from_time_test.rs
    - services/inv/tests/invoice_status_fsm_test.rs
    - services/inv/tests/invoice_rate_card_snapshot_test.rs
    - services/inv/tests/invoice_correction_appends_test.rs
    - services/inv/tests/invoice_numbering_gap_free_test.rs
    - services/inv/tests/invoice_status_enum_cardinality_test.rs
    - services/inv/tests/invoice_line_kind_enum_cardinality_test.rs
    - services/inv/tests/invoice_per_engagement_scope_test.rs
    - services/inv/tests/invoice_approval_gate_test.rs
    - services/inv/tests/invoice_write_off_cfo_only_test.rs
    - services/inv/tests/invoice_amounts_bigint_minor_test.rs
    - services/inv/tests/invoice_audit_emission_test.rs

  modified_files:
    - services/time/src/                                              # add invoiced_at marker on TIME entries

  allowed_tools:
    - file_read: services/{inv,time}/**
    - file_write: services/inv/{src,tests,migrations}/**
    - bash: cd services/inv && cargo test

  disallowed_tools:
    - mutate invoice lines after approval (per DEC-1366 — append corrections only)
    - reference rate-card live (per DEC-1363 — snapshot at creation)
    - multi-engagement per invoice (per DEC-1368)
    - allow gap in invoice numbering (per DEC-1367)
    - write-off without CFO + reason (per DEC-1371)

effort_hours: 8
subtasks:
  - "0.6h: 0001..0005 migrations (5 tables + RLS + REVOKE)"
  - "0.4h: types.rs (3 closed enums)"
  - "0.6h: draft/builder.rs (TIME rollup → draft invoice)"
  - "0.4h: draft/scheduler.rs (per-engagement cycle job)"
  - "0.4h: lines/time_rollup.rs (per-entry → line)"
  - "0.4h: lines/corrections.rs (append-only)"
  - "0.4h: status/state_machine.rs (8-state FSM + transitions)"
  - "0.4h: numbering/mod.rs (per-tenant gap-free sequence)"
  - "0.5h: snapshot/rate_card.rs (immutable snapshot)"
  - "0.4h: handlers/invoice_routes.rs (CRUD + approve + send + write-off)"
  - "0.3h: audit/invoice_events.rs (9 builders)"
  - "2.0h: tests — 12 test files"
  - "0.3h: TIME-side modifications (invoiced_at marker)"
  - "0.3h: integration smoke"

risk_if_skipped: "Without invoice substrate, the entire billing → revenue → cash-application pipeline (INV-002 through INV-011, plus PORTAL-001 invoice view, PORTAL-006 billing inquiry workflows, TEN-003 Stripe ref, TEN-102 VND hóa đơn ref) has nothing to anchor on. TIME entries accumulate forever with no path to revenue. Without DEC-1363 rate-card snapshot, retroactive rate changes corrupt historical invoices = legally indefensible. Without DEC-1367 gap-free numbering, VN tax audit fails (Decree 123 §10). Without DEC-1366 append-only corrections, post-approval edits silently revise the audit record. Without DEC-1370 approval gate, any user can send invoices to clients = brand + revenue control breakdown. The 8h effort lands the financial-system foundation."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship invoice substrate at `services/inv/src/` with 5 migrations, 8-state status FSM, append-only line corrections, rate-card snapshot, per-engagement scoping, gap-free per-tenant numbering, CFO-gated write-off, and 9 memory audit kinds. Anchors all downstream INV tasks (002-011) and cross-module invoice references.

1. **MUST** define closed `invoice_status` enum: `('draft','ready_for_review','approved','sent','partially_paid','paid','void','written_off')` per DEC-1361. Cardinality 8.

2. **MUST** define closed `invoice_line_kind` enum: `('time_entry','fixed_fee','expense_reimbursement','discount','tax','late_fee')` per DEC-1362. Cardinality 6.

3. **MUST** define `invoices` at migration `0001`: `(invoice_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, invoice_number TEXT UNIQUE NOT NULL, status invoice_status NOT NULL DEFAULT 'draft', billing_currency billing_currency_enum NOT NULL, issued_at TIMESTAMPTZ, due_at TIMESTAMPTZ, sent_at TIMESTAMPTZ, paid_at TIMESTAMPTZ, voided_at TIMESTAMPTZ, written_off_at TIMESTAMPTZ, billing_period_start TIMESTAMPTZ, billing_period_end TIMESTAMPTZ, client_subject_id UUID, rate_card_snapshot JSONB NOT NULL, total_pre_tax_minor BIGINT NOT NULL DEFAULT 0, total_tax_minor BIGINT NOT NULL DEFAULT 0, total_minor BIGINT NOT NULL DEFAULT 0, paid_minor BIGINT NOT NULL DEFAULT 0, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by_subject_id UUID NOT NULL, trace_id CHAR(32))`. Append-only on mutation of lines (status, amount fields updated through state-machine handlers only).

4. **MUST** define `invoice_lines` at migration `0002`: `(line_id UUID PRIMARY KEY, invoice_id UUID NOT NULL REFERENCES invoices(invoice_id), line_kind invoice_line_kind NOT NULL, description TEXT NOT NULL, quantity NUMERIC(18,4) NOT NULL, unit_price_minor BIGINT NOT NULL, amount_minor BIGINT NOT NULL, vat_rate_pct NUMERIC(5,2) NOT NULL DEFAULT 0, source_kind TEXT NOT NULL CHECK (source_kind IN ('time_entry','expense','manual','correction','discount_policy')), source_ref UUID, sort_order INT NOT NULL, correction_of_line_id UUID REFERENCES invoice_lines(line_id), created_at TIMESTAMPTZ NOT NULL DEFAULT now(), trace_id CHAR(32))`. Append-only via REVOKE UPDATE/DELETE per task-audit skill rule 12 + DEC-1366; correction = new row referencing original.

5. **MUST** define `invoice_status_history` at migration `0003`: `(id BIGSERIAL PRIMARY KEY, invoice_id UUID NOT NULL REFERENCES invoices(invoice_id), from_status invoice_status, to_status invoice_status NOT NULL, transitioned_at TIMESTAMPTZ NOT NULL DEFAULT now(), transitioned_by_subject_id UUID NOT NULL, reason TEXT, trace_id CHAR(32))`. Append-only.

6. **MUST** define `invoice_number_sequence` at migration `0004`: `(tenant_id UUID NOT NULL, year INT NOT NULL, last_sequence INT NOT NULL DEFAULT 0, notes JSONB NOT NULL DEFAULT '[]'::jsonb, PRIMARY KEY (tenant_id, year))`. Gap-free per-tenant annual; skipped sequences logged in `notes` for Decree-123 conformance (slice-2 hóa đơn integration).

7. **MUST** define `rate_card_snapshot` table at migration `0005` storing per-engagement rate-card versions: `(snapshot_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, engagement_id UUID NOT NULL, version INT NOT NULL, rates_jsonb JSONB NOT NULL, effective_from TIMESTAMPTZ NOT NULL DEFAULT now(), created_by_subject_id UUID NOT NULL, UNIQUE(engagement_id, version))`. Versioned rate cards; invoices snapshot the latest at creation time.

8. **MUST** enforce RLS with USING and WITH CHECK on all 5 tables: `tenant_id = current_setting('auth.tenant_id')::uuid`.

9. **MUST** expose draft creation `POST /v1/inv/invoices/draft` body `{ engagement_id, billing_period_start, billing_period_end }`. Handler:
   - Validates engagement_id in tenant scope.
   - Resolves billing currency from engagement.
   - Allocates invoice number per §1 #10.
   - Snapshots current rate card per §1 #11.
   - Aggregates unbilled TIME entries via TASK-TIME-009 rollup into `invoice_lines` rows.
   - Computes totals.
   - Marks TIME entries `invoiced_at = now()` to prevent double-billing.
   - INSERTs invoices + invoice_lines + invoice_status_history (status='draft').
   - Emits `inv.draft_created` sev-2.

10. **MUST** allocate invoice number per DEC-1367 via `numbering/mod.rs`:
    - `SELECT last_sequence FROM invoice_number_sequence WHERE tenant_id=$1 AND year=$2 FOR UPDATE`.
    - `last_sequence + 1`.
    - `INVOICE_NUMBER = "INV-" + tenant_slug + "-" + (year%100) + "-" + ZeroPad(seq, 6)`.
    - UPDATE sequence.
    - On rollback after allocation: record skipped number in `notes` JSONB for Decree-123 audit clarity (matches TASK-TEN-102 §1 #17 pattern).

11. **MUST** snapshot rate card per DEC-1363. The `snapshot/rate_card.rs::snapshot_for_engagement(engagement_id)`:
    - Resolves latest active rate-card version.
    - Deep-copies into `invoices.rate_card_snapshot` JSONB.
    - Records snapshot version reference for traceability.

12. **MUST** validate state-machine transitions per DEC-1361 via `status/state_machine.rs`. Allowed transitions:
    - `draft → ready_for_review` (any user can flip own draft).
    - `ready_for_review → approved` (cfo OR engagement_admin only per DEC-1370).
    - `approved → sent` (auto on send action OR manual).
    - `sent → partially_paid` (auto on partial payment receipt via TASK-INV-003/005).
    - `sent | partially_paid → paid` (auto on full payment).
    - `draft | ready_for_review | approved → void` (cfo only; with reason).
    - `sent | partially_paid → written_off` (cfo only; with reason per DEC-1371).
    Invalid transition → 400 `invalid_status_transition`.

13. **MUST** support append-only corrections per DEC-1366. `POST /v1/inv/invoices/{id}/lines/correction` body `{ correction_of_line_id, line_kind, description, quantity, unit_price_minor, amount_minor, vat_rate_pct, reason }`. Handler:
    - Validates `correction_of_line_id` exists + same invoice.
    - INSERTs new line with `source_kind='correction'` + `correction_of_line_id` populated.
    - Recomputes invoice totals.
    - Emits `inv.correction_added` sev-2.

14. **MUST** support approve action `POST /v1/inv/invoices/{id}/approve`. Caller has `cfo` OR `engagement_admin`. Validates transition; INSERTs status history; UPDATEs status; emits `inv.approved` sev-1 (material commercial event).

15. **MUST** support send action `POST /v1/inv/invoices/{id}/send` body `{ delivery_method, delivery_target }`. Caller has `cfo` OR `engagement_admin`. Validates status='approved'; transitions to 'sent'; sets `sent_at`; emits `inv.sent` sev-1 (triggers TASK-INV-007 VN hóa đơn for VN tenants).

16. **MUST** support write-off `POST /v1/inv/invoices/{id}/write-off` body `{ reason }`. Caller has `cfo` only. Records reason; transitions to 'written_off'; emits `inv.written_off` sev-1.

17. **MUST** support void `POST /v1/inv/invoices/{id}/void` body `{ reason }`. Caller has `cfo` only. Validates status NOT IN ('sent','partially_paid','paid','written_off'); transitions to 'void'; emits `inv.void` sev-1.

18. **MUST** auto-generate drafts at billing-cycle boundaries per DEC-1369 via `draft/scheduler.rs::run_for_engagement(engagement_id, cycle_end)`. Scheduled job runs hourly; selects engagements whose cycle ends in past hour; creates draft if unbilled TIME entries exist + engagement has `auto_draft_enabled=true`.

19. **MUST** scope invoice to ONE engagement per DEC-1368. Cross-engagement consolidation = anti-pattern; deferred.

20. **MUST** store amounts as BIGINT minor per task-audit skill rule 11. NEVER FLOAT.

21. **MUST** mark TIME entries `invoiced_at` to prevent double-billing per DEC-1369 derivative. TASK-TIME-009 rollup filter excludes entries with non-NULL `invoiced_at`.

22. **MUST** emit 9 memory audit kinds per DEC-1374:
    - `inv.draft_created` (sev-2)
    - `inv.lines_added` (sev-3)
    - `inv.status_transitioned` (sev-2)
    - `inv.approved` (sev-1)
    - `inv.sent` (sev-1)
    - `inv.paid` (sev-1)
    - `inv.void` (sev-1)
    - `inv.written_off` (sev-1)
    - `inv.correction_added` (sev-2)

23. **MUST** PII-scrub description + reason via TASK-MEMORY-111 — SHA256 in chain.

24. **MUST** thread trace_id end-to-end.

25. **MUST NOT** mutate invoice_lines after approval per DEC-1366 — correction-only path.

26. **MUST NOT** allow gap in invoice numbering per DEC-1367 — skipped numbers documented in notes JSONB.

27. **MUST NOT** allow non-cfo write-off per DEC-1371.

---

## §2 — Why this design (rationale)

**Why rate-card snapshot (§1 #11, DEC-1363)?** Rates change. An invoice generated last quarter at $X/hr stays at $X/hr regardless of subsequent rate hikes. Live reference = re-invoicing legally indefensible (client paid based on a number we can no longer reproduce).

**Why append-only corrections (§1 #13, DEC-1366)?** Post-approval edits silently changing past invoices = fraud signal in any audit. Correction lines (with reason) maintain the original AND the fix AND the auditor's ability to see the delta.

**Why gap-free numbering (§1 #10, DEC-1367)?** Decree 123 + general accounting principle — gaps suggest hidden transactions. Skip-with-reason logged in notes satisfies tax-authority audit while permitting transaction rollback.

**Why ONE engagement per invoice (§1 #19, DEC-1368)?** Multi-engagement invoices conflate client billing relationships; "Acme paid invoice X — for which engagement?" becomes ambiguous. Engagement-scoped invoices = clean audit + clean reporting.

**Why CFO-only write-off (§1 #16, DEC-1371)?** Write-off = revenue erasure with tax implications. Requires CFO sign-off both for accounting control (prevent fraud) and for tax-treatment correctness.

**Why 8-state FSM vs simpler 4-state (§1 #12)?** Financial workflows need draft/review/approved/sent/partially_paid/paid as distinct states for status reporting + automation triggers. void + written_off cover the two distinct "no payment received" terminal cases (cancelled-by-us vs uncollectable).

---

## §3 — API contract

```sql
-- 0001_invoices.sql
CREATE TYPE invoice_status AS ENUM ('draft','ready_for_review','approved','sent','partially_paid','paid','void','written_off');

CREATE TABLE invoices (
  invoice_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  invoice_number TEXT UNIQUE NOT NULL,
  status invoice_status NOT NULL DEFAULT 'draft',
  billing_currency billing_currency_enum NOT NULL,
  issued_at TIMESTAMPTZ,
  due_at TIMESTAMPTZ,
  sent_at TIMESTAMPTZ,
  paid_at TIMESTAMPTZ,
  voided_at TIMESTAMPTZ,
  written_off_at TIMESTAMPTZ,
  billing_period_start TIMESTAMPTZ,
  billing_period_end TIMESTAMPTZ,
  client_subject_id UUID,
  rate_card_snapshot JSONB NOT NULL,
  total_pre_tax_minor BIGINT NOT NULL DEFAULT 0,
  total_tax_minor BIGINT NOT NULL DEFAULT 0,
  total_minor BIGINT NOT NULL DEFAULT 0,
  paid_minor BIGINT NOT NULL DEFAULT 0,
  notes TEXT,
  internal_notes TEXT,
  sync_class TEXT NOT NULL DEFAULT 'client-visible' CHECK (sync_class IN ('private','team-internal','client-visible','client-visible-redacted')),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_subject_id UUID NOT NULL,
  trace_id CHAR(32)
);
CREATE INDEX idx_inv_engagement ON invoices(engagement_id, status);
CREATE INDEX idx_inv_status_due ON invoices(status, due_at) WHERE status NOT IN ('paid','void','written_off');
ALTER TABLE invoices ENABLE ROW LEVEL SECURITY;
CREATE POLICY invoices_rls ON invoices
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON invoices FROM cyberos_app;
GRANT UPDATE (status, issued_at, due_at, sent_at, paid_at, voided_at, written_off_at,
              total_pre_tax_minor, total_tax_minor, total_minor, paid_minor, notes) ON invoices TO cyberos_app;

-- 0002_invoice_lines.sql
CREATE TYPE invoice_line_kind AS ENUM ('time_entry','fixed_fee','expense_reimbursement','discount','tax','late_fee');

CREATE TABLE invoice_lines (
  line_id UUID PRIMARY KEY,
  invoice_id UUID NOT NULL REFERENCES invoices(invoice_id),
  line_kind invoice_line_kind NOT NULL,
  description TEXT NOT NULL,
  quantity NUMERIC(18,4) NOT NULL,
  unit_price_minor BIGINT NOT NULL,
  amount_minor BIGINT NOT NULL,
  vat_rate_pct NUMERIC(5,2) NOT NULL DEFAULT 0,
  source_kind TEXT NOT NULL CHECK (source_kind IN ('time_entry','expense','manual','correction','discount_policy')),
  source_ref UUID,
  sort_order INT NOT NULL,
  correction_of_line_id UUID REFERENCES invoice_lines(line_id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
CREATE INDEX idx_inv_lines_invoice ON invoice_lines(invoice_id, sort_order);
ALTER TABLE invoice_lines ENABLE ROW LEVEL SECURITY;
CREATE POLICY invoice_lines_rls ON invoice_lines
  USING (invoice_id IN (SELECT invoice_id FROM invoices WHERE tenant_id = current_setting('auth.tenant_id')::uuid))
  WITH CHECK (invoice_id IN (SELECT invoice_id FROM invoices WHERE tenant_id = current_setting('auth.tenant_id')::uuid));
REVOKE UPDATE, DELETE ON invoice_lines FROM cyberos_app;

-- 0003_invoice_status_history.sql
CREATE TABLE invoice_status_history (
  id BIGSERIAL PRIMARY KEY,
  invoice_id UUID NOT NULL REFERENCES invoices(invoice_id),
  from_status invoice_status,
  to_status invoice_status NOT NULL,
  transitioned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  transitioned_by_subject_id UUID NOT NULL,
  reason TEXT,
  trace_id CHAR(32)
);
CREATE INDEX idx_inv_history_invoice ON invoice_status_history(invoice_id, transitioned_at);
ALTER TABLE invoice_status_history ENABLE ROW LEVEL SECURITY;
CREATE POLICY invoice_status_history_rls ON invoice_status_history
  USING (invoice_id IN (SELECT invoice_id FROM invoices WHERE tenant_id = current_setting('auth.tenant_id')::uuid))
  WITH CHECK (invoice_id IN (SELECT invoice_id FROM invoices WHERE tenant_id = current_setting('auth.tenant_id')::uuid));
REVOKE UPDATE, DELETE ON invoice_status_history FROM cyberos_app;

-- 0004_invoice_number_sequence.sql
CREATE TABLE invoice_number_sequence (
  tenant_id UUID NOT NULL,
  year INT NOT NULL,
  last_sequence INT NOT NULL DEFAULT 0,
  notes JSONB NOT NULL DEFAULT '[]'::jsonb,
  PRIMARY KEY (tenant_id, year)
);
ALTER TABLE invoice_number_sequence ENABLE ROW LEVEL SECURITY;
CREATE POLICY invoice_number_sequence_rls ON invoice_number_sequence
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON invoice_number_sequence FROM cyberos_app;
GRANT UPDATE (last_sequence, notes) ON invoice_number_sequence TO cyberos_app;

-- 0005_rate_card_snapshot.sql
CREATE TABLE rate_card_snapshot (
  snapshot_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  engagement_id UUID NOT NULL,
  version INT NOT NULL,
  rates_jsonb JSONB NOT NULL,
  effective_from TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_subject_id UUID NOT NULL,
  UNIQUE (engagement_id, version)
);
ALTER TABLE rate_card_snapshot ENABLE ROW LEVEL SECURITY;
CREATE POLICY rate_card_snapshot_rls ON rate_card_snapshot
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON rate_card_snapshot FROM cyberos_app;
```

Endpoints:
```text
POST   /v1/inv/invoices/draft                                     (engagement_admin or cfo)
POST   /v1/inv/invoices/{id}/lines/correction                     (cfo or engagement_admin)
POST   /v1/inv/invoices/{id}/approve                              (cfo or engagement_admin)
POST   /v1/inv/invoices/{id}/send                                 (cfo or engagement_admin)
POST   /v1/inv/invoices/{id}/void                                 (cfo)
POST   /v1/inv/invoices/{id}/write-off                            (cfo)
GET    /v1/inv/invoices/{id}                                      (engagement_admin or cfo or client per sync_class)
GET    /v1/inv/invoices?engagement_id=...&status=...              (engagement_admin or cfo)
```

---

## §4 — Acceptance criteria

1. **invoice_status cardinality 8**.
2. **invoice_line_kind cardinality 6**.
3. **Draft from TIME rollup** — unbilled TIME entries aggregated into invoice_lines; entries marked `invoiced_at`.
4. **Rate-card snapshot immutable** — invoice created; rate card subsequently changed; invoice retains original rates in snapshot.
5. **Append-only lines** — UPDATE on `invoice_lines` raises permission-denied.
6. **Correction line** — POST correction creates new line with `correction_of_line_id` populated; totals recomputed.
7. **Status FSM transitions** — `draft → ready_for_review → approved → sent → paid` legal; reverse illegal.
8. **Approval requires cfo or engagement_admin** — tenant_admin → 403.
9. **Write-off requires cfo** — engagement_admin write-off → 403.
10. **Write-off requires reason** — empty reason → 400.
11. **Invoice number format** — `INV-acme-26-000001` for first tenant=acme invoice in 2026.
12. **Gap-free numbering** — 1000 concurrent draft creations → 1000 unique numbers no duplicates.
13. **Skipped numbers logged** — rollback after allocation → skip recorded in notes.
14. **Per-engagement scope** — invoice has exactly one engagement_id.
15. **TIME entries not double-billed** — second draft generation on same period → 0 lines (all entries invoiced_at).
16. **Status history append-only** — every transition recorded; UPDATE on history → permission-denied.
17. **9 memory audit kinds emitted** — full lifecycle.
18. **Trace_id end-to-end**.
19. **BIGINT minor amounts** — schema validation; never FLOAT.
20. **RLS cross-tenant denied** — tenant A invoice invisible to tenant B.

---

## §5 — Verification

```rust
#[tokio::test]
async fn draft_from_time_rollup() {
    let ctx = TestContext::with_engagement_and_time_entries().await;
    let r = ctx.post_draft(ctx.eng_id, ctx.period_start, ctx.period_end).await;
    assert_eq!(r.status(), 201);
    let body: serde_json::Value = r.json().await.unwrap();
    let inv_id: Uuid = body["invoice_id"].as_str().unwrap().parse().unwrap();
    let lines: Vec<(String,)> = sqlx::query_as("SELECT description FROM invoice_lines WHERE invoice_id=$1")
        .bind(inv_id).fetch_all(&ctx.pool).await.unwrap();
    assert!(lines.len() >= 1);

    let invoiced_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM time_entries WHERE engagement_id=$1 AND invoiced_at IS NOT NULL"
    ).bind(ctx.eng_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(invoiced_count > 0);
}

#[tokio::test]
async fn rate_card_snapshot_immutable() {
    let ctx = TestContext::with_engagement_and_time_entries().await;
    let inv_id = ctx.create_draft().await;
    let snapshot1: serde_json::Value = sqlx::query_scalar(
        "SELECT rate_card_snapshot FROM invoices WHERE invoice_id=$1"
    ).bind(inv_id).fetch_one(&ctx.pool).await.unwrap();

    ctx.bump_rate_card(ctx.eng_id, /*new rate*/ 200_00).await;
    let snapshot2: serde_json::Value = sqlx::query_scalar(
        "SELECT rate_card_snapshot FROM invoices WHERE invoice_id=$1"
    ).bind(inv_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(snapshot1, snapshot2);
}

#[tokio::test]
async fn append_only_corrections() {
    let ctx = TestContext::with_invoice().await;
    let r = sqlx::query("UPDATE invoice_lines SET amount_minor=999 WHERE invoice_id=$1")
        .bind(ctx.inv_id).execute(&ctx.pool).await;
    assert!(r.is_err());

    let r = ctx.post_correction(ctx.inv_id, ctx.line_id, /*amount*/ 500_00, "rate correction").await;
    assert_eq!(r.status(), 201);
    let line_count: i64 = sqlx::query_scalar("SELECT count(*) FROM invoice_lines WHERE invoice_id=$1")
        .bind(ctx.inv_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(line_count, 2);  // original + correction
}

#[tokio::test]
async fn status_fsm_enforces_transitions() {
    let ctx = TestContext::with_invoice().await;
    let r = ctx.transition_invoice(ctx.inv_id, "paid").await;  // draft → paid skip
    assert_eq!(r.status(), 400);

    ctx.transition_invoice(ctx.inv_id, "ready_for_review").await;
    ctx.as_cfo().transition_invoice(ctx.inv_id, "approved").await;
    ctx.transition_invoice(ctx.inv_id, "sent").await;
    let r = ctx.transition_invoice(ctx.inv_id, "paid").await;
    assert_eq!(r.status(), 200);
}

#[tokio::test]
async fn write_off_cfo_only_with_reason() {
    let ctx = TestContext::with_sent_invoice().await;
    let r = ctx.as_engagement_admin().post_write_off(ctx.inv_id, "uncollectable").await;
    assert_eq!(r.status(), 403);
    let r = ctx.as_cfo().post_write_off(ctx.inv_id, "").await;
    assert_eq!(r.status(), 400);
    let r = ctx.as_cfo().post_write_off(ctx.inv_id, "uncollectable").await;
    assert_eq!(r.status(), 200);
}

#[tokio::test]
async fn invoice_number_gap_free() {
    let ctx = TestContext::new().await;
    let nums: Vec<String> = futures::stream::iter(0..100).then(|_| async {
        let r = ctx.post_draft_minimal().await;
        r.json::<serde_json::Value>().await.unwrap()["invoice_number"].as_str().unwrap().to_owned()
    }).collect::<Vec<_>>().await;
    let unique: std::collections::HashSet<_> = nums.iter().cloned().collect();
    assert_eq!(unique.len(), nums.len());
}

// 5.7..5.12: enum cardinality, per-engagement scope, TIME no-double-bill, status history, audit emission, RLS
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-009 (per-cycle rollup source).
**Cross-module:** TASK-INV-002 (multi-currency), TASK-INV-003 (Stripe payment), TASK-INV-005 (VietQR), TASK-INV-006 (cash app), TASK-INV-007 (VN hóa đơn), TASK-TEN-003 (Stripe ref), TASK-TEN-102 (VND ref), TASK-PORTAL-001 (invoices view), TASK-PORTAL-006 (billing_inquiry workflows), TASK-CRM-001 (client subject), TASK-AUTH-101 (cfo + engagement_admin roles), TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007.
**Downstream:** TASK-INV-002 through TASK-INV-011.

---

## §8 — Example payload

`inv.draft_created`:
```json
{
  "kind": "inv.draft_created",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.engagement_admin.456",
  "trace_id": "...",
  "occurred_at": "2026-05-17T...",
  "payload": {
    "invoice_id": "0190...",
    "invoice_number": "INV-acme-26-000042",
    "engagement_id": "0190...",
    "billing_currency": "USD",
    "billing_period_start": "2026-04-17T00:00:00Z",
    "billing_period_end": "2026-05-17T00:00:00Z",
    "line_count": 12,
    "total_minor": 875_000,
    "rate_card_version": 3
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Multi-currency per invoice (slice 2 / TASK-INV-002).
- **Deferred:** Subscription-style recurring invoice templates (slice 3).
- **Deferred:** Customer-facing payment link in sent email (slice 2).
- **Deferred:** Late-fee auto-calc + late_fee line generation (slice 3).
- **Deferred:** Multi-engagement consolidated invoicing (anti-pattern; not planned).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| TIME entry already invoiced | filter excludes invoiced_at | Skipped from draft | Inherent |
| Rate card missing for engagement | snapshot fails | 412 + `no_rate_card_configured` | Engagement_admin configures |
| Invalid status transition | FSM check | 400 + `invalid_status_transition` | Caller fixes flow |
| Approve without chief-financial-officer/engagement_admin role | check | 403 | Inherent |
| Write-off without CFO | check | 403 | Inherent |
| Write-off without reason | validation | 400 | Inherent |
| Numbering rollback | exception in INSERT after sequence allocated | Skip logged in notes | Inherent forensic |
| Duplicate invoice_number (race) | UNIQUE constraint | Re-attempt with next sequence | Inherent retry |
| Correction reference invalid | FK check | 400 | Caller fixes line_id |
| Cross-tenant invoice access | RLS | 0 rows | Inherent |
| Concurrent line addition pre-approval | tx isolation | Last writer wins | Inherent |
| Multi-currency on single invoice | engagement.billing_currency immutable | Inherent prevention | New engagement required |
| Send before approval | FSM check | 400 + invalid transition | Approve first |
| Void after send | FSM check | 400 | Use write-off instead |
| Negative total computed | balance check | sev-2 alert | Operator review |
| Status history insert fails post-transition | tx isolation | Rollback; status unchanged | Inherent |
| TIME entry deleted after invoiced | source_ref orphan | Line retained with description; TASK-TIME-009 prevents delete after invoiced_at | Inherent guard |
| Rate card snapshot JSONB > 100 KB | size limit | sev-2; investigate rate-card explosion | Slim rate card |
| Approval after period close | period_close hooks | Allowed; backfills supported | Inherent |
| Invoice viewed by client with internal_notes leaked | sync_class filter via PORTAL-001 | Internal hidden | PORTAL-001 enforces |

---

## §11 — Implementation notes

**§11.1** Rate-card snapshot deep-copy via `serde_json::to_value(&rate_card)` ensures complete capture.

**§11.2** TIME-side `invoiced_at` column added via migration in this task's `modified_files`.

**§11.3** Numbering sequence uses Postgres `FOR UPDATE` row lock to prevent race; skipped logged via post-rollback hook.

**§11.4** State machine encoded as Rust `match` table; CI test asserts every (from, to) pair documented.

**§11.5** Per-engagement billing currency consumed from `engagements.billing_currency`; immutable once set (TASK-TEN-003 derivative).

**§11.6** Cross-line totals computed at write-time (not view-time) for query simplicity.

**§11.7** Append-only at SQL grant level prevents handler bypass.

**§11.8** Status_history `from_status` is NULL on initial create row.

**§11.9** PORTAL-001 view of invoices applies `sync_class` filter to hide internal fields.

**§11.10** Draft auto-scheduler runs hourly; engagement.auto_draft_enabled gates participation.

---

*End of TASK-INV-001 spec.*
