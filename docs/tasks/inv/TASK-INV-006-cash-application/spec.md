---
id: TASK-INV-006
title: "INV cash application — closed 4-step matching cascade (exact-ref → amount+date → fuzzy-fraction → manual) + atomic ledger reconciliation + partial allocation + over-allocation block + memory audit per match"
module: INV
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-INV-001, TASK-INV-003, TASK-INV-005, TASK-OBS-007]
depends_on: [TASK-INV-003, TASK-INV-005]
blocks: []

source_pages:
  - website/docs/modules/inv.html#cash-application
source_decisions:
  - DEC-560 (closed 4-step matching cascade: step1 exact_invoice_ref_from_memo · step2 amount_plus_close_invoice_date_within_7d · step3 fuzzy_partial_amount_within_5pct · step4 manual_operator_match — operator can skip ahead; auto-cascade stops at first match)
  - DEC-561 (matching is async via `inv_cash_app_job` running every 5 minutes — receipts arrive via webhook + sit in `payment_receipts` with `invoice_id` possibly NULL; this job populates invoice_id retroactively where memo parser missed)
  - DEC-562 (partial allocation supported — one receipt MAY allocate across multiple invoices via `payment_allocations` table; one invoice MAY receive multiple receipts; sum constraint: SUM(allocations) ≤ receipt.amount_minor AND SUM(allocations) ≤ invoice.amount_minor_outstanding)
  - DEC-563 (over-allocation blocked at DB trigger — `SUM(allocations_for_invoice) > invoice.amount_minor_outstanding` raises P0100 over_allocation_blocked; mirrors TASK-INV-001's outstanding-amount invariant)
  - DEC-564 (REVOKE UPDATE, DELETE on payment_allocations from cyberos_app — append-only at SQL grant; corrections via reversal row pattern same as TASK-TIME-001 correction_to)
  - DEC-565 (memory audit kinds: inv.cash_app_match_attempted, inv.cash_app_matched_step1, inv.cash_app_matched_step2, inv.cash_app_matched_step3, inv.cash_app_matched_manual, inv.cash_app_no_match, inv.cash_app_over_allocation_blocked, inv.cash_app_partial_allocated, inv.cash_app_allocation_reversed)
  - DEC-566 (closed 5-value allocation_source enum: memo_parser · amount_date · fuzzy · manual · reversal — tied to which cascade step performed the match; downstream analytics depends on this)
  - DEC-567 (fuzzy step3 tolerance = 5% — most underpayments fall within this window; > 5% mismatch routes to manual review; ADR to raise/lower threshold)
  - DEC-568 (manual handler requires CFO role per TASK-AUTH-101 — accountant can SUGGEST but CFO commits; emits sev-2 audit row since manual ledger touches are forensically critical)
  - DEC-569 (reversal supported — operator can reverse a prior allocation by inserting an `allocation_source=reversal` row with negative amount + reference to original allocation_id; sum-constraint trigger handles)
  - DEC-570 (per-tenant fuzzy threshold override via tenant_policy YAML — `cash_app_fuzzy_threshold_pct` default 5; min 1; max 20)
  - DEC-571 (job scheduler `inv_cash_app_job` runs every 5 minutes; idempotent via WHERE invoice_id IS NULL predicate; advisory lock per receipt prevents double-processing under concurrent runs)
  - DEC-572 (invoice's `amount_minor_outstanding` is a generated column = amount_minor - SUM(allocations) — kept consistent via the over-allocation trigger; invoice auto-marked `paid` when outstanding = 0)
  - DEC-573 (`inv.cash_app_no_match` after all 4 steps emit at sev-3 — accumulates an actionable backlog for CFO review; > 10 unmatched at hour 24 → sev-2 escalation)
  - DEC-574 (cash app uses the privileged `inv_cash_applier` SQL role from TASK-INV-005 — column-level UPDATE on payment_receipts.invoice_id; mirrors that pattern for payment_allocations)
  - PDPL Art. 13 (data minimisation — invoice memo + payment reference scrubbed in memory chain)
  - SOC 2 CC6.1 (data integrity — ledger reconciliation must be deterministic + auditable)
  - ISO 27001 A.12.4 (audit logging — all matching decisions logged)

language: rust 1.81 + sql
service: cyberos/services/inv/
new_files:
  - services/inv/migrations/0014_payment_allocations.sql
  - services/inv/migrations/0015_invoice_outstanding_view.sql
  - services/inv/src/cash_app/mod.rs
  - services/inv/src/cash_app/cascade.rs                       # 4-step closed cascade dispatcher
  - services/inv/src/cash_app/step1_exact_ref.rs               # HD/INV prefix match (extends TASK-INV-005 memo parser)
  - services/inv/src/cash_app/step2_amount_date.rs             # amount + close-date window match
  - services/inv/src/cash_app/step3_fuzzy.rs                   # 5% fuzzy partial-amount match
  - services/inv/src/cash_app/step4_manual.rs                  # CFO-driven manual handler
  - services/inv/src/cash_app/allocator.rs                     # atomic allocation writer + over-allocation check
  - services/inv/src/cash_app/scheduler.rs                     # 5-min job
  - services/inv/src/cash_app/repo.rs
  - services/inv/src/cash_app/audit.rs                         # 9 memory row builders
  - services/inv/src/handlers/cash_app.rs                      # POST /allocate-manual + POST /reverse + GET /unmatched
  - services/inv/tests/cash_app_cascade_test.rs
  - services/inv/tests/cash_app_step1_exact_ref_test.rs
  - services/inv/tests/cash_app_step2_amount_date_test.rs
  - services/inv/tests/cash_app_step3_fuzzy_5pct_test.rs
  - services/inv/tests/cash_app_step4_manual_cfo_test.rs
  - services/inv/tests/cash_app_partial_allocation_test.rs
  - services/inv/tests/cash_app_over_allocation_blocked_test.rs
  - services/inv/tests/cash_app_reversal_test.rs
  - services/inv/tests/cash_app_scheduler_idempotent_test.rs
  - services/inv/tests/cash_app_no_match_escalation_test.rs
  - services/inv/tests/cash_app_append_only_test.rs
  - services/inv/tests/cash_app_audit_emission_test.rs
modified_files:
  - services/inv/src/types.rs                                   # +AllocationSource enum
  - services/inv/src/lib.rs                                     # pub mod cash_app

allowed_tools:
  - file_read: services/inv/**
  - file_write: services/inv/{src,tests,migrations}/**
  - bash: cd services/inv && cargo test cash_app

disallowed_tools:
  - allow UPDATE/DELETE on payment_allocations (per DEC-564 — reversal-row pattern only)
  - allow over-allocation > invoice.amount_minor_outstanding (per DEC-563 — DB trigger blocks)
  - allow non-CFO to commit manual matches (per DEC-568)
  - skip cascade steps via env override (operator can skip via API but not silently)
  - allow fuzzy threshold > 20% (per DEC-570 — ADR required)

effort_hours: 8
subtasks:
  - "0.6h: 0014_payment_allocations.sql + AllocationSource enum + RLS + REVOKE + sum-constraint trigger"
  - "0.4h: 0015_invoice_outstanding_view.sql — generated outstanding column + paid auto-marker"
  - "0.5h: cascade.rs — closed 4-step dispatcher"
  - "0.4h: step1_exact_ref.rs — reuse TASK-INV-005 memo parser"
  - "0.6h: step2_amount_date.rs — amount+date window query"
  - "0.7h: step3_fuzzy.rs — 5% fuzzy match with per-tenant override"
  - "0.4h: step4_manual.rs — CFO-driven handler"
  - "0.8h: allocator.rs — atomic write + over-allocation check + reversal"
  - "0.5h: scheduler.rs — 5-min job + advisory lock"
  - "0.4h: repo.rs"
  - "0.5h: audit.rs — 9 memory row builders"
  - "0.6h: handlers/cash_app.rs"
  - "1.6h: tests — 12 test files"

risk_if_skipped: "Without cash application, every webhook receipt that arrives with an unparseable memo OR partial amount becomes a manual operator task — at scale, this is intractable. The 80% case (HD/INV prefix in memo) handles via TASK-INV-005's parser; the remaining 20% requires this FR's 4-step cascade. Without DEC-562's partial allocation, customers paying in installments (common in VN B2B) can't be reconciled — invoices stay open with mismatched amounts. Without DEC-563's over-allocation block, accountant mistakes silently overpay invoices + corrupt AR aging. Without DEC-568's CFO-only manual gate, every operator can touch the ledger (segregation-of-duties violation). Without DEC-569's reversal pattern, allocation mistakes require DB surgery. The 8h effort lands the deterministic 4-step cascade + atomic ledger primitive that AR/AP reconciliation depends on."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship cash application as a closed 4-step matching cascade with atomic allocation + over-allocation block + reversal pattern. Each requirement:

1. **MUST** define `payment_allocations` table: `(id UUID PRIMARY KEY, tenant_id UUID NOT NULL, receipt_id UUID NOT NULL REFERENCES payment_receipts(id), invoice_id UUID NOT NULL REFERENCES invoices(id), amount_minor BIGINT NOT NULL, allocation_source allocation_source NOT NULL, allocated_by_subject_id UUID NOT NULL, reverses_allocation_id UUID REFERENCES payment_allocations(id), allocated_at TIMESTAMPTZ NOT NULL DEFAULT now(), notes TEXT)`. `amount_minor` MAY be negative iff `reverses_allocation_id` is set.

2. **MUST** declare the closed `allocation_source` Postgres enum with exactly 5 values (per DEC-566): `'memo_parser'`, `'amount_date'`, `'fuzzy'`, `'manual'`, `'reversal'`. Adding a 6th is an ADR.

3. **MUST** enforce RLS with `USING + WITH CHECK` on `payment_allocations`. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

4. **MUST** be **append-only** on `payment_allocations` (per DEC-564). `REVOKE UPDATE, DELETE FROM cyberos_app`. Reversals create new rows with negative `amount_minor` + `reverses_allocation_id` link.

5. **MUST** enforce **over-allocation block** via DB trigger (per DEC-563). On INSERT or UPDATE on `payment_allocations`:
    - Compute `SUM(amount_minor) WHERE invoice_id = NEW.invoice_id` (positive + negative reversals net).
    - If `SUM > invoices.amount_minor` for that invoice → RAISE `over_allocation_blocked` ERRCODE P0100.
    - Compute `SUM(amount_minor) WHERE receipt_id = NEW.receipt_id`.
    - If `SUM > payment_receipts.amount_minor` for that receipt → RAISE `receipt_over_allocated` ERRCODE P0101.

6. **MUST** ship the `inv_cash_app_job` scheduled job running every 5 minutes (per DEC-561 + DEC-571):
    - Query: `SELECT id, amount_minor, currency, transfer_memo FROM payment_receipts WHERE invoice_id IS NULL AND received_at > now() - interval '90 days' FOR UPDATE SKIP LOCKED`.
    - For each receipt: invoke `cascade::try_match(receipt)` running steps 1 → 2 → 3 sequentially; step 4 is operator-driven (never auto-run).
    - Each step returns `Match { invoice_id, amount_minor_allocated, source }` or `NoMatch`.
    - On Match: INSERT `payment_allocations` row + UPDATE `payment_receipts.invoice_id` + emit step-specific memory row.
    - On NoMatch after step 3: emit `inv.cash_app_no_match` memory row sev-3.

7. **MUST** implement **step 1 — exact memo reference match** (per DEC-560). Reuses TASK-INV-005's `extract_invoice_id` parser on `transfer_memo`. Match → INSERT allocation with full receipt amount; `allocation_source='memo_parser'`. Already handled at webhook time for ~80% of cases; this step catches receipts where memo parser deferred (e.g. when `payment_intent.metadata.invoice_id` was absent on Stripe path).

8. **MUST** implement **step 2 — amount + close-date match** (per DEC-560). Query: invoices with `tenant_id = receipt.tenant_id AND amount_minor = receipt.amount_minor AND currency = receipt.currency AND status = 'open' AND due_date BETWEEN receipt.received_at - interval '7 days' AND receipt.received_at + interval '7 days'`. Match exactly one → INSERT allocation; `allocation_source='amount_date'`. Multiple candidates → skip to step 3 (ambiguous).

9. **MUST** implement **step 3 — fuzzy partial match** (per DEC-560 + DEC-567 + DEC-570). Per-tenant `cash_app_fuzzy_threshold_pct` lookup (default 5%, range 1-20%). Query: invoices where `ABS(invoice.amount_minor - receipt.amount_minor) / invoice.amount_minor <= threshold/100 AND status = 'open'`. Match exactly one → INSERT allocation; allocation amount = MIN(receipt.amount_minor, invoice.amount_minor_outstanding); `allocation_source='fuzzy'`. Multiple candidates → skip to manual step.

10. **MUST** implement **step 4 — manual operator match** (per DEC-568). NOT auto-invoked. `POST /v1/inv/cash-app/allocate-manual` handler:
    - Caller MUST have role `cfo` per TASK-AUTH-101.
    - Body: `{receipt_id, invoice_id, amount_minor, notes}`.
    - Validates `amount_minor > 0 AND <= receipt.amount_minor_remaining AND <= invoice.amount_minor_outstanding`.
    - INSERT allocation with `allocation_source='manual'`.
    - Emit `inv.cash_app_matched_manual` memory row sev-2 (every manual ledger touch is forensically critical).

11. **MUST** support **partial allocation** (per DEC-562). One receipt MAY split across N invoices; one invoice MAY receive M receipts. Constraint enforced at trigger:
    - SUM(allocations.amount_minor WHERE receipt_id = R) ≤ receipt.amount_minor.
    - SUM(allocations.amount_minor WHERE invoice_id = I) ≤ invoice.amount_minor.
    - Within-receipt overflow → 409 receipt_over_allocated; within-invoice overflow → 409 over_allocation_blocked.

12. **MUST** support **reversal** (per DEC-569). `POST /v1/inv/cash-app/reverse` handler:
    - Caller MUST have role `cfo`.
    - Body: `{allocation_id, reason}`.
    - INSERT new row with `amount_minor = -original.amount_minor`, `reverses_allocation_id = original.id`, `allocation_source='reversal'`, `notes = reason`.
    - Constraint trigger handles — net sums adjust.
    - Emit `inv.cash_app_allocation_reversed` memory row sev-2.

13. **MUST** ship `invoice_outstanding_view` SQL view (per DEC-572):
    ```sql
    SELECT i.id AS invoice_id, i.tenant_id, i.amount_minor, i.currency,
           COALESCE(SUM(a.amount_minor), 0) AS amount_allocated_minor,
           i.amount_minor - COALESCE(SUM(a.amount_minor), 0) AS amount_outstanding_minor,
           CASE WHEN i.amount_minor - COALESCE(SUM(a.amount_minor), 0) <= 0 THEN 'paid' ELSE i.status END AS effective_status
      FROM invoices i
      LEFT JOIN payment_allocations a ON a.invoice_id = i.id
     GROUP BY i.id;
    ```
    Downstream consumers (TASK-INV-009 AR aging, TASK-INV-010 dunning) use this view, NOT raw `invoices.status`.

14. **MUST** auto-mark invoice `paid` via trigger when `SUM(allocations.amount_minor) >= invoice.amount_minor` (per DEC-572). The trigger fires on `payment_allocations` AFTER INSERT and runs `UPDATE invoices SET status='paid', paid_at=now() WHERE id = NEW.invoice_id AND ... <= SUM(allocations)`. Idempotent — already-paid stays paid.

15. **MUST** ship `GET /v1/inv/cash-app/unmatched?from=<ts>&to=<ts>` handler (per DEC-573). Returns unmatched receipts (`invoice_id IS NULL` after step 3 ran). Caller MUST have role `cfo`. Body: `[{receipt_id, amount_minor, currency, transfer_memo, received_at, suggested_matches: [...top 3 fuzzy candidates...]}]`.

16. **MUST** emit 9 memory audit row kinds (per DEC-565):
    - `inv.cash_app_match_attempted` — every job-run on a receipt (sampled 10%).
    - `inv.cash_app_matched_step1` — memo_parser hit during scheduled job.
    - `inv.cash_app_matched_step2` — amount_date hit.
    - `inv.cash_app_matched_step3` — fuzzy hit.
    - `inv.cash_app_matched_manual` — CFO manual commit (sev-2).
    - `inv.cash_app_no_match` — all auto-steps failed (sev-3; sev-2 if unmatched > 24h).
    - `inv.cash_app_over_allocation_blocked` — trigger fired (sev-1; alarm).
    - `inv.cash_app_partial_allocated` — receipt split across multiple invoices.
    - `inv.cash_app_allocation_reversed` — reversal recorded (sev-2).

17. **MUST** PII-scrub `notes` + `transfer_memo` via TASK-MEMORY-111 before chain commit.

18. **MUST** sev-2 escalate when receipts remain unmatched > 24h (per DEC-573). OBS rule: `inv.cash_app_no_match` count over last hour > 10 → sev-2 alarm to CFO Slack channel.

19. **MUST** complete cascade per-receipt in ≤ 200 ms p95 (3 DB queries + 1 INSERT). `cash_app_perf_test`.

20. **MUST** emit OTel span `inv.cash_app.{cascade,allocate_manual,reverse,unmatched_query}` with attributes: `tenant_id`, `receipt_id`, `invoice_id`, `allocation_source`, `outcome` (matched_step1 | matched_step2 | matched_step3 | matched_manual | no_match | over_allocation | receipt_over_allocated | invalid_amount | not_found | permission_denied).

21. **MUST** emit OTel metrics:
    - `inv_cash_app_match_total{tenant_id, source, outcome}` (counter).
    - `inv_cash_app_unmatched_count{tenant_id}` (gauge — receipts with invoice_id NULL after step 3).
    - `inv_cash_app_partial_allocations_total{tenant_id}` (counter — receipts allocated across > 1 invoice).
    - `inv_cash_app_reversals_total{tenant_id}` (counter — sev-2 alarm at > 3/h).
    - `inv_cash_app_over_allocation_blocked_total{tenant_id}` (counter — sev-1 alarm always).
    - `inv_cash_app_cascade_latency_ms{step}` (histogram).

22. **MUST** ship `GET /v1/inv/cash-app/allocations?invoice_id=<>` for operator visibility. Returns chronological list including reversals. Caller MUST have role `cfo` or `cdo` (financial visibility roles).

23. **MUST** use the `inv_cash_applier` SQL role (defined in TASK-INV-005) to INSERT into `payment_allocations` + UPDATE `payment_receipts.invoice_id` (per DEC-574). cyberos_app role gets SELECT only.

24. **MUST** advisory-lock each receipt during cascade (per DEC-571). `SELECT pg_advisory_xact_lock(hashtext(receipt_id::text))` at top of cascade — prevents concurrent job runs from double-processing the same receipt.

25. **MUST** detect + skip currency-mismatched candidates at every step. Receipt in USD MUST NOT match invoice in VND even if amount-minor happens to match. Currency mismatch is an explicit skip condition in step 2 + step 3 queries.

26. **MUST** ship `POST /v1/inv/cash-app/dry-run` (caller role `cfo` or `cdo`). Body `{receipt_id}`. Runs cascade WITHOUT writing allocations + returns the would-be match (or no_match). Used by operators to preview before committing manual matches.

---

## §2 — Why this design (rationale for humans)

**Why closed 4-step cascade (DEC-560)?** Each step is a distinct matching strategy with distinct false-positive risk. Step 1 (exact ref) = near-zero false positives. Step 2 (amount+date) = low false positives. Step 3 (fuzzy) = some false positives — acceptable for low-value partial payments. Step 4 (manual) = operator judgment, always correct. The cascade order is by descending confidence + ascending operator effort.

**Why async via scheduled job (DEC-561)?** Webhook handlers must ack within 5s (Stripe + Napas247 SLAs). Matching takes 50-200ms per receipt × thousands of receipts = unworkable synchronously. Async job decouples: webhook persists receipt fast; job retroactively matches when capacity allows.

**Why 5-min cadence (DEC-561)?** Operator-acceptable latency for "I received a payment; when does my invoice show paid?" is < 10 minutes. 5-min cadence + < 200ms per receipt cascade = < 6 minutes worst case from receipt to allocation.

**Why partial allocation (DEC-562)?** Real-world: VN B2B customers often pay invoices in 2-3 installments (30/30/40 or 50/50). Forcing 1:1 receipt-to-invoice = these installments stay unallocated indefinitely. M:N allocation matches the financial reality.

**Why over-allocation blocked at trigger (DEC-563)?** Defense in depth: handler validates; trigger catches direct SQL + concurrent inserts. Sev-1 audit row + alarm = operator immediately notified (over-allocation = ledger corruption; must be investigated).

**Why append-only via SQL grant (DEC-564)?** Ledger integrity is the cardinal AR/AP property. UPDATE/DELETE on allocations would let operators silently rewrite history. Reversal-row pattern preserves the audit trail (original allocation + reversal both present in chain).

**Why CFO-only manual match (DEC-568)?** Manual matching = direct ledger touch = segregation-of-duties concern. Accountant prepares (via dry-run); CFO commits. Tenants without CFO role need ADR + alternate role assignment in attribute_mapping_yaml. Sev-2 audit on every manual match means every CFO touch is forensically visible.

**Why 5% fuzzy threshold default (DEC-567 + DEC-570)?** Industry standard for partial-payment tolerance. Below 1% = nearly exact; above 20% = false-positive risk too high. Tenant override [1, 20]% gives flexibility; > 20% requires ADR.

**Why reversal as new row not UPDATE (DEC-569)?** Same as TIME-001 correction_to pattern. Preserves the original record + makes "what was the original allocation?" trivially queryable. Net sums via aggregation respect both rows.

**Why `inv.cash_app_no_match` sev-3 initially + sev-2 after 24h (DEC-573)?** Some receipts genuinely defer to manual review (e.g. wrong customer, refund pending). Sev-3 = inform; sev-2 after 24h = action required. Two-tier escalation matches operator workflow.

**Why generated outstanding view (DEC-572)?** Computing outstanding inline in every consumer query would proliferate. View centralises; consumers query view + trust the math. Trigger auto-marks paid status — invoices stay in sync without batch jobs.

**Why advisory lock per receipt (DEC-571)?** SKIP LOCKED on the receipt query handles row-level concurrency. Advisory lock additionally guards the multi-statement cascade (read invoice + INSERT allocation must be atomic per receipt). Tx-scope ensures lock releases at commit/rollback.

**Why dry-run handler (§1 #26)?** Operator wants to preview "if I assigned this receipt to invoice X, would the ledger accept it?" before committing. Dry-run runs cascade + returns hypothetical outcome — no writes. Reduces error rate on manual matches.

**Why currency-mismatch hard skip (§1 #25)?** Receipt in USD matching invoice in VND would be cents-vs-đồng confusion — financially catastrophic. Currency equality is non-negotiable.

**Why 90-day receipt-age window in scheduler (§1 #6)?** Receipts older than 90 days without auto-match = manual review backlog. Scheduler stops auto-trying after 90d to avoid pointless retries; manual handler still works for older receipts.

**Why allocation_source enum closed at 5 (DEC-566)?** Downstream analytics ("what % of receipts matched at step 1?") need a fixed dimension. Adding a 6th (e.g. `ml_predicted`) is an ADR — needs accuracy validation before contributing to ledger.

**Why operator role check on dry-run (§1 #26)?** Even read-only operations expose customer + invoice data. CFO/CDO are financial visibility roles; tenant-admin doesn't need ledger detail.

**Why partial-allocation counter as adoption metric (§1 #21)?** Tracks "how often do we split receipts?" — useful for understanding customer payment behaviour + sizing operator workflows.

**Why over-allocation sev-1 alarm (§1 #21)?** Over-allocation = ledger corruption attempt. Either bug or attack; either way, page-on-call equivalent. Routine matching never hits this — every trigger is a real problem.

**Why memo_parser as a cascade step (step 1) when TASK-INV-005 already does it?** TASK-INV-005's memo parser runs at webhook time, but the Stripe path may have empty metadata.invoice_id (operator didn't set it during invoice creation). Step 1 re-runs the parser on transfer_memo + Stripe description fields — catches the deferred 5-10%.

---

## §3 — API contract

### 3.1 — Migration 0014 — payment_allocations

```sql
-- services/inv/migrations/0014_payment_allocations.sql

BEGIN;

CREATE TYPE allocation_source AS ENUM ('memo_parser', 'amount_date', 'fuzzy', 'manual', 'reversal');

CREATE TABLE payment_allocations (
    id                          UUID         PRIMARY KEY,
    tenant_id                   UUID         NOT NULL,
    receipt_id                  UUID         NOT NULL REFERENCES payment_receipts(id) ON DELETE RESTRICT,
    invoice_id                  UUID         NOT NULL REFERENCES invoices(id) ON DELETE RESTRICT,
    amount_minor                BIGINT       NOT NULL,
    allocation_source           allocation_source NOT NULL,
    allocated_by_subject_id     UUID         NOT NULL REFERENCES auth.subjects(id) ON DELETE RESTRICT,
    reverses_allocation_id      UUID         REFERENCES payment_allocations(id),
    allocated_at                TIMESTAMPTZ  NOT NULL DEFAULT now(),
    notes                       TEXT         CHECK (notes IS NULL OR length(notes) BETWEEN 1 AND 1000),
    CHECK (amount_minor != 0),
    CHECK ((amount_minor < 0) = (reverses_allocation_id IS NOT NULL))
);

CREATE INDEX payment_allocations_receipt_idx ON payment_allocations (receipt_id);
CREATE INDEX payment_allocations_invoice_idx ON payment_allocations (invoice_id);
CREATE INDEX payment_allocations_tenant_allocated_idx ON payment_allocations (tenant_id, allocated_at DESC);

ALTER TABLE payment_allocations ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_allocations_tenant_iso ON payment_allocations
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only (DEC-564)
REVOKE UPDATE, DELETE ON payment_allocations FROM cyberos_app;
GRANT INSERT ON payment_allocations TO inv_cash_applier;
GRANT SELECT ON payment_allocations TO cyberos_app, inv_cash_applier;

-- Over-allocation block (DEC-563)
CREATE OR REPLACE FUNCTION enforce_allocation_sums() RETURNS TRIGGER AS $$
DECLARE
    receipt_amount      BIGINT;
    invoice_amount      BIGINT;
    sum_for_receipt     BIGINT;
    sum_for_invoice     BIGINT;
BEGIN
    SELECT amount_minor INTO receipt_amount FROM payment_receipts WHERE id = NEW.receipt_id;
    SELECT amount_minor INTO invoice_amount FROM invoices WHERE id = NEW.invoice_id;

    SELECT COALESCE(SUM(amount_minor), 0) INTO sum_for_receipt
      FROM payment_allocations WHERE receipt_id = NEW.receipt_id;
    sum_for_receipt := sum_for_receipt + NEW.amount_minor;
    IF sum_for_receipt > receipt_amount THEN
        RAISE EXCEPTION 'receipt_over_allocated: % > %', sum_for_receipt, receipt_amount USING ERRCODE = 'P0101';
    END IF;

    SELECT COALESCE(SUM(amount_minor), 0) INTO sum_for_invoice
      FROM payment_allocations WHERE invoice_id = NEW.invoice_id;
    sum_for_invoice := sum_for_invoice + NEW.amount_minor;
    IF sum_for_invoice > invoice_amount THEN
        RAISE EXCEPTION 'over_allocation_blocked: % > %', sum_for_invoice, invoice_amount USING ERRCODE = 'P0100';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_payment_allocations_sum_check BEFORE INSERT ON payment_allocations
    FOR EACH ROW EXECUTE FUNCTION enforce_allocation_sums();

-- Auto-mark invoice paid (DEC-572)
CREATE OR REPLACE FUNCTION mark_invoice_paid_if_settled() RETURNS TRIGGER AS $$
DECLARE
    invoice_amount  BIGINT;
    total_allocated BIGINT;
BEGIN
    SELECT amount_minor INTO invoice_amount FROM invoices WHERE id = NEW.invoice_id;
    SELECT COALESCE(SUM(amount_minor), 0) INTO total_allocated
      FROM payment_allocations WHERE invoice_id = NEW.invoice_id;
    IF total_allocated >= invoice_amount THEN
        UPDATE invoices SET status = 'paid', paid_at = COALESCE(paid_at, now())
         WHERE id = NEW.invoice_id AND status != 'paid';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_payment_allocations_mark_paid AFTER INSERT ON payment_allocations
    FOR EACH ROW EXECUTE FUNCTION mark_invoice_paid_if_settled();

COMMIT;
```

### 3.2 — Migration 0015 — outstanding view

```sql
-- services/inv/migrations/0015_invoice_outstanding_view.sql

BEGIN;

CREATE VIEW invoice_outstanding_view AS
SELECT
    i.id                                                                 AS invoice_id,
    i.tenant_id,
    i.amount_minor,
    i.currency,
    i.status                                                             AS recorded_status,
    COALESCE(SUM(a.amount_minor), 0)                                     AS amount_allocated_minor,
    i.amount_minor - COALESCE(SUM(a.amount_minor), 0)                    AS amount_outstanding_minor,
    CASE
        WHEN i.amount_minor - COALESCE(SUM(a.amount_minor), 0) <= 0     THEN 'paid'
        WHEN i.due_date < CURRENT_DATE                                   THEN 'overdue'
        ELSE i.status
    END                                                                  AS effective_status
FROM invoices i
LEFT JOIN payment_allocations a ON a.invoice_id = i.id
GROUP BY i.id;

COMMIT;
```

### 3.3 — Cascade dispatcher

```rust
// services/inv/src/cash_app/cascade.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::cash_app::{step1_exact_ref, step2_amount_date, step3_fuzzy};
use crate::types::AllocationSource;

#[derive(Debug, Clone)]
pub enum CascadeResult {
    Matched { invoice_id: Uuid, amount_minor: i64, source: AllocationSource },
    NoMatch,
}

pub async fn try_match(pool: &PgPool, receipt_id: Uuid) -> anyhow::Result<CascadeResult> {
    // Advisory lock per receipt for atomic cascade
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1::text))")
        .bind(receipt_id.to_string()).execute(pool).await?;

    if let Some(r) = step1_exact_ref::try_match(pool, receipt_id).await? {
        return Ok(CascadeResult::Matched { invoice_id: r, amount_minor: 0 /* full receipt */, source: AllocationSource::MemoParser });
    }
    if let Some(r) = step2_amount_date::try_match(pool, receipt_id).await? {
        return Ok(CascadeResult::Matched { invoice_id: r.invoice_id, amount_minor: r.amount, source: AllocationSource::AmountDate });
    }
    if let Some(r) = step3_fuzzy::try_match(pool, receipt_id).await? {
        return Ok(CascadeResult::Matched { invoice_id: r.invoice_id, amount_minor: r.amount, source: AllocationSource::Fuzzy });
    }
    Ok(CascadeResult::NoMatch)
}
```

### 3.4 — Step 2 — amount + date

```rust
// services/inv/src/cash_app/step2_amount_date.rs
use sqlx::PgPool;
use uuid::Uuid;

pub struct Step2Match { pub invoice_id: Uuid, pub amount: i64 }

pub async fn try_match(pool: &PgPool, receipt_id: Uuid) -> anyhow::Result<Option<Step2Match>> {
    // Load receipt
    let (receipt_amount, receipt_currency, received_at): (i64, String, chrono::DateTime<chrono::Utc>) =
        sqlx::query_as("SELECT amount_minor, currency, received_at FROM payment_receipts WHERE id = $1")
            .bind(receipt_id).fetch_one(pool).await?;

    // Find candidate invoices: exact amount + matching currency + within ±7-day due-date window + status='open'
    let candidates: Vec<(Uuid,)> = sqlx::query_as(r#"
        SELECT i.id FROM invoices i
        WHERE i.amount_minor = $1
          AND i.currency = $2
          AND i.status = 'open'
          AND i.due_date BETWEEN $3 - interval '7 days' AND $3 + interval '7 days'
          AND i.id NOT IN (SELECT invoice_id FROM payment_allocations WHERE invoice_id IS NOT NULL)
        LIMIT 2
    "#)
    .bind(receipt_amount).bind(&receipt_currency).bind(received_at.date_naive())
    .fetch_all(pool).await?;

    // Match only if exactly one candidate (ambiguous → skip to step 3)
    if candidates.len() == 1 {
        Ok(Some(Step2Match { invoice_id: candidates[0].0, amount: receipt_amount }))
    } else {
        Ok(None)
    }
}
```

### 3.5 — Step 3 — fuzzy match

```rust
// services/inv/src/cash_app/step3_fuzzy.rs
use sqlx::PgPool;
use uuid::Uuid;

pub struct Step3Match { pub invoice_id: Uuid, pub amount: i64 }

pub const DEFAULT_FUZZY_THRESHOLD_PCT: i32 = 5;
pub const FUZZY_MIN_PCT: i32 = 1;
pub const FUZZY_MAX_PCT: i32 = 20;

pub async fn try_match(pool: &PgPool, receipt_id: Uuid) -> anyhow::Result<Option<Step3Match>> {
    let (receipt_amount, receipt_currency, tenant_id): (i64, String, Uuid) =
        sqlx::query_as("SELECT amount_minor, currency, tenant_id FROM payment_receipts WHERE id = $1")
            .bind(receipt_id).fetch_one(pool).await?;

    // Per-tenant fuzzy threshold (with bounds check)
    let raw_pct: Option<i32> = sqlx::query_scalar(
        "SELECT cash_app_fuzzy_threshold_pct FROM tenant_policy WHERE tenant_id = $1"
    ).bind(tenant_id).fetch_optional(pool).await?;
    let pct = raw_pct.unwrap_or(DEFAULT_FUZZY_THRESHOLD_PCT)
        .clamp(FUZZY_MIN_PCT, FUZZY_MAX_PCT);

    // Find candidates within ±pct%
    let candidates: Vec<(Uuid, i64)> = sqlx::query_as(r#"
        SELECT i.id, i.amount_minor FROM invoices i
        LEFT JOIN payment_allocations a ON a.invoice_id = i.id
        WHERE i.currency = $1
          AND i.status = 'open'
          AND i.tenant_id = $2
          AND ABS(i.amount_minor - $3) * 100 <= i.amount_minor * $4
        GROUP BY i.id
        HAVING COALESCE(SUM(a.amount_minor), 0) < i.amount_minor
        LIMIT 2
    "#)
    .bind(&receipt_currency).bind(tenant_id).bind(receipt_amount).bind(pct as i64)
    .fetch_all(pool).await?;

    if candidates.len() == 1 {
        let (invoice_id, invoice_amount) = candidates[0];
        let allocation_amount = receipt_amount.min(invoice_amount);
        Ok(Some(Step3Match { invoice_id, amount: allocation_amount }))
    } else {
        Ok(None)
    }
}
```

### 3.6 — Scheduler

```rust
// services/inv/src/cash_app/scheduler.rs
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;
use crate::cash_app::{cascade, audit, allocator};

pub async fn run_loop(pool: PgPool) {
    let mut tick = tokio::time::interval(Duration::from_secs(5 * 60));
    loop {
        tick.tick().await;
        if let Err(e) = run_once(&pool).await {
            tracing::error!("cash_app scheduler iter error: {e:?}");
        }
    }
}

async fn run_once(pool: &PgPool) -> anyhow::Result<()> {
    // Find unmatched receipts within 90-day window
    let receipts: Vec<(Uuid, Uuid)> = sqlx::query_as(r#"
        SELECT id, tenant_id FROM payment_receipts
        WHERE invoice_id IS NULL
          AND received_at > now() - interval '90 days'
        FOR UPDATE SKIP LOCKED
        LIMIT 100
    "#).fetch_all(pool).await?;

    for (receipt_id, tenant_id) in receipts {
        match cascade::try_match(pool, receipt_id).await {
            Ok(cascade::CascadeResult::Matched { invoice_id, amount_minor, source }) => {
                allocator::allocate(pool, tenant_id, receipt_id, invoice_id, amount_minor, source, system_subject_id()).await?;
                audit::emit_matched(pool, tenant_id, receipt_id, invoice_id, source).await?;
            }
            Ok(cascade::CascadeResult::NoMatch) => {
                audit::emit_no_match(pool, tenant_id, receipt_id).await?;
            }
            Err(e) => tracing::warn!("cascade error on {receipt_id}: {e:?}"),
        }
    }
    Ok(())
}

fn system_subject_id() -> Uuid { Uuid::nil() }
```

### 3.7 — Allocator

```rust
// services/inv/src/cash_app/allocator.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::types::AllocationSource;

pub async fn allocate(
    pool: &PgPool,
    tenant_id: Uuid,
    receipt_id: Uuid,
    invoice_id: Uuid,
    amount_minor: i64,
    source: AllocationSource,
    allocated_by: Uuid,
) -> anyhow::Result<Uuid> {
    let allocation_id = Uuid::new_v4();
    let mut tx = pool.begin().await?;
    sqlx::query(r#"
        INSERT INTO payment_allocations (id, tenant_id, receipt_id, invoice_id, amount_minor, allocation_source, allocated_by_subject_id)
        VALUES ($1, $2, $3, $4, $5, $6::allocation_source, $7)
    "#)
    .bind(allocation_id).bind(tenant_id).bind(receipt_id).bind(invoice_id)
    .bind(amount_minor).bind(format!("{source:?}").to_lowercase()).bind(allocated_by)
    .execute(&mut *tx).await?;

    // If full receipt allocated, set invoice_id on payment_receipts for fast lookup
    let receipt_sum: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_minor), 0) FROM payment_allocations WHERE receipt_id = $1"
    ).bind(receipt_id).fetch_one(&mut *tx).await?;
    let receipt_total: i64 = sqlx::query_scalar(
        "SELECT amount_minor FROM payment_receipts WHERE id = $1"
    ).bind(receipt_id).fetch_one(&mut *tx).await?;
    if receipt_sum >= receipt_total {
        sqlx::query("UPDATE payment_receipts SET invoice_id = $2 WHERE id = $1")
            .bind(receipt_id).bind(invoice_id).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(allocation_id)
}

pub async fn reverse(
    pool: &PgPool,
    original_id: Uuid,
    reversed_by: Uuid,
    reason: &str,
) -> anyhow::Result<Uuid> {
    let original: (Uuid, Uuid, Uuid, i64, String) = sqlx::query_as(
        "SELECT tenant_id, receipt_id, invoice_id, amount_minor, allocation_source::text FROM payment_allocations WHERE id = $1"
    ).bind(original_id).fetch_one(pool).await?;

    let reversal_id = Uuid::new_v4();
    sqlx::query(r#"
        INSERT INTO payment_allocations (id, tenant_id, receipt_id, invoice_id, amount_minor, allocation_source, allocated_by_subject_id, reverses_allocation_id, notes)
        VALUES ($1, $2, $3, $4, $5, 'reversal'::allocation_source, $6, $7, $8)
    "#)
    .bind(reversal_id).bind(original.0).bind(original.1).bind(original.2)
    .bind(-original.3).bind(reversed_by).bind(original_id).bind(reason)
    .execute(pool).await?;
    Ok(reversal_id)
}
```

### 3.8 — Manual handler

```rust
// services/inv/src/handlers/cash_app.rs (excerpt)
use axum::{Json, extract::State, http::StatusCode};
use cyberos_auth::rbac::Role;
use crate::cash_app::{allocator, audit};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AllocateManualRequest {
    pub receipt_id: Uuid,
    pub invoice_id: Uuid,
    pub amount_minor: i64,
    pub notes: String,
}

pub async fn allocate_manual(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<AllocateManualRequest>,
) -> Result<(StatusCode, Json<AllocateResponse>), ApiError> {
    if !claims.roles().contains(&Role::Cfo) {
        return Err(ApiError::PermissionDenied);
    }
    if req.amount_minor <= 0 || req.notes.is_empty() || req.notes.len() > 1000 {
        return Err(ApiError::InvalidAmountOrNotes);
    }
    // Currency match
    let receipt_currency: String = sqlx::query_scalar(
        "SELECT currency FROM payment_receipts WHERE id = $1 AND tenant_id = $2"
    ).bind(req.receipt_id).bind(claims.tenant_id()).fetch_one(&state.db).await?;
    let invoice_currency: String = sqlx::query_scalar(
        "SELECT currency FROM invoices WHERE id = $1 AND tenant_id = $2"
    ).bind(req.invoice_id).bind(claims.tenant_id()).fetch_one(&state.db).await?;
    if receipt_currency != invoice_currency {
        return Err(ApiError::CurrencyMismatch);
    }

    let allocation_id = allocator::allocate(
        &state.db, claims.tenant_id(), req.receipt_id, req.invoice_id, req.amount_minor,
        crate::types::AllocationSource::Manual, claims.subject_id(),
    ).await?;

    audit::emit_matched_manual(
        &state.db, claims.tenant_id(), req.receipt_id, req.invoice_id, allocation_id,
        claims.subject_id(), &req.notes,
    ).await?;

    Ok((StatusCode::CREATED, Json(AllocateResponse { allocation_id })))
}
```

---

## §4 — Acceptance criteria

1. **AllocationSource enum closed at 5** — memo_parser, amount_date, fuzzy, manual, reversal.
2. **RLS isolates by tenant** — cross-tenant queries return 0 rows.
3. **Step 1 memo_parser match** — receipt with HD123456 memo → invoice with number HD123456 matched.
4. **Step 2 amount+date match** — receipt amount + date within ±7d of due_date + currency match → matched.
5. **Step 2 ambiguous (>1 candidate) skips to step 3** — multiple matching invoices → no step 2 match.
6. **Step 3 fuzzy 5% match** — receipt amount within ±5% of invoice → matched at allocation = min(receipt, outstanding).
7. **Step 3 per-tenant override** — `cash_app_fuzzy_threshold_pct=10` → 10% tolerance.
8. **Step 3 outside threshold rejected** — receipt 6% off invoice (default 5%) → no match.
9. **Currency mismatch hard skip** — receipt USD vs invoice VND → no match at any step.
10. **No match emits sev-3 audit** — 4-step cascade fails → `inv.cash_app_no_match` row.
11. **Manual match by CFO** → 201 + `inv.cash_app_matched_manual` sev-2 row.
12. **Manual match by non-CFO** → 403.
13. **Manual match with invalid amount** → 400.
14. **Currency mismatch on manual** → 400.
15. **Over-allocation blocked at trigger** — INSERT exceeding invoice outstanding → P0100 + sev-1 audit.
16. **Receipt over-allocation blocked** — INSERT exceeding receipt amount → P0101.
17. **Partial allocation supported** — one receipt → 2 invoices with split amounts.
18. **Reversal via `POST /reverse`** — original allocation + negative-amount reversal coexist.
19. **Reversal by non-CFO** → 403.
20. **Net sums respect reversals** — original +500 + reversal -500 → invoice outstanding restored.
21. **`UPDATE/DELETE` blocked from cyberos_app** — only `inv_cash_applier` may INSERT.
22. **Invoice auto-marked `paid`** — full allocation triggers status=paid + paid_at set.
23. **Outstanding view reflects allocations** — invoice with $500 + allocation $200 → outstanding $300.
24. **Scheduler advisory-locks per receipt** — concurrent runs don't double-allocate.
25. **Scheduler skips receipts > 90 days old** — only manual path remaining.
26. **`GET /unmatched` returns receipts** — caller MUST have role cfo or cdo.
27. **`POST /dry-run` returns hypothetical match** — no allocation written.
28. **Cascade p95 < 200 ms** — perf test.
29. **OTel span `inv.cash_app.cascade` emitted** — outcome attribute.
30. **Counter `inv_cash_app_match_total{source=memo_parser, outcome=matched}` increments**.
31. **Counter `inv_cash_app_over_allocation_blocked_total` sev-1 alarm always**.
32. **9 memory audit kinds emit correctly** — one per scenario.

---

## §5 — Verification

```rust
// services/inv/tests/cash_app_step3_fuzzy_5pct_test.rs
#[sqlx::test]
async fn fuzzy_within_5pct_matches(pool: sqlx::PgPool) {
    let tenant = ctx_seed_tenant(&pool).await;
    let invoice = seed_invoice(&pool, tenant, /* amount_minor */ 1_000_000, /* currency */ "VND").await;
    let receipt = seed_receipt(&pool, tenant, 970_000, "VND", "").await;  // 3% short
    let r = cyberos_inv::cash_app::cascade::try_match(&pool, receipt).await.unwrap();
    assert!(matches!(r, cyberos_inv::cash_app::cascade::CascadeResult::Matched { ref source, .. }
        if matches!(source, cyberos_inv::types::AllocationSource::Fuzzy)));
}

#[sqlx::test]
async fn fuzzy_outside_5pct_no_match(pool: sqlx::PgPool) {
    let tenant = ctx_seed_tenant(&pool).await;
    let _invoice = seed_invoice(&pool, tenant, 1_000_000, "VND").await;
    let receipt = seed_receipt(&pool, tenant, 930_000, "VND", "").await;  // 7% short
    let r = cyberos_inv::cash_app::cascade::try_match(&pool, receipt).await.unwrap();
    assert!(matches!(r, cyberos_inv::cash_app::cascade::CascadeResult::NoMatch));
}
```

```rust
// services/inv/tests/cash_app_over_allocation_blocked_test.rs
#[sqlx::test]
async fn over_allocation_trigger_blocks(pool: sqlx::PgPool) {
    set_role_cash_applier(&pool).await;
    let invoice = seed_invoice(&pool, 1_000_000).await;
    let receipt = seed_receipt(&pool, 1_500_000).await;
    // First allocation 800,000 OK
    let _ = sqlx::query("INSERT INTO payment_allocations (id, tenant_id, receipt_id, invoice_id, amount_minor, allocation_source, allocated_by_subject_id) VALUES ($1, $2, $3, $4, 800000, 'amount_date'::allocation_source, $5)")
        .bind(uuid::Uuid::new_v4()).bind(tenant()).bind(receipt).bind(invoice).bind(subject()).execute(&pool).await.unwrap();
    // Second allocation 300,000 → total 1,100,000 > 1,000,000 → must fail
    let err = sqlx::query("INSERT INTO payment_allocations (id, tenant_id, receipt_id, invoice_id, amount_minor, allocation_source, allocated_by_subject_id) VALUES ($1, $2, $3, $4, 300000, 'amount_date'::allocation_source, $5)")
        .bind(uuid::Uuid::new_v4()).bind(tenant()).bind(receipt).bind(invoice).bind(subject()).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("over_allocation_blocked"));
}
```

```rust
// services/inv/tests/cash_app_reversal_test.rs
#[sqlx::test]
async fn reversal_creates_negative_row_and_nets_to_zero(pool: sqlx::PgPool) {
    let invoice = seed_invoice(&pool, 1_000_000).await;
    let receipt = seed_receipt(&pool, 1_000_000).await;
    let original = cyberos_inv::cash_app::allocator::allocate(
        &pool, tenant(), receipt, invoice, 1_000_000,
        cyberos_inv::types::AllocationSource::Manual, subject(),
    ).await.unwrap();
    let _reversal = cyberos_inv::cash_app::allocator::reverse(&pool, original, subject(), "operator error").await.unwrap();

    let total: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(amount_minor), 0) FROM payment_allocations WHERE invoice_id = $1"
    ).bind(invoice).fetch_one(&pool).await.unwrap();
    assert_eq!(total, 0, "reversal must net to zero");
    let outstanding: i64 = sqlx::query_scalar(
        "SELECT amount_outstanding_minor FROM invoice_outstanding_view WHERE invoice_id = $1"
    ).bind(invoice).fetch_one(&pool).await.unwrap();
    assert_eq!(outstanding, 1_000_000, "invoice fully outstanding again after reversal");
}
```

```rust
// services/inv/tests/cash_app_partial_allocation_test.rs
#[sqlx::test]
async fn one_receipt_splits_across_two_invoices(pool: sqlx::PgPool) {
    let inv_a = seed_invoice(&pool, 600_000).await;
    let inv_b = seed_invoice(&pool, 400_000).await;
    let receipt = seed_receipt(&pool, 1_000_000).await;
    cyberos_inv::cash_app::allocator::allocate(&pool, tenant(), receipt, inv_a, 600_000,
        cyberos_inv::types::AllocationSource::Manual, subject()).await.unwrap();
    cyberos_inv::cash_app::allocator::allocate(&pool, tenant(), receipt, inv_b, 400_000,
        cyberos_inv::types::AllocationSource::Manual, subject()).await.unwrap();

    let inv_a_status: String = sqlx::query_scalar(
        "SELECT effective_status FROM invoice_outstanding_view WHERE invoice_id = $1"
    ).bind(inv_a).fetch_one(&pool).await.unwrap();
    assert_eq!(inv_a_status, "paid");
    let inv_b_status: String = sqlx::query_scalar(
        "SELECT effective_status FROM invoice_outstanding_view WHERE invoice_id = $1"
    ).bind(inv_b).fetch_one(&pool).await.unwrap();
    assert_eq!(inv_b_status, "paid");
}
```

```rust
// services/inv/tests/cash_app_step4_manual_cfo_test.rs
#[tokio::test]
async fn manual_match_by_cfo_succeeds(ctx: TestCtx) {
    let invoice = ctx.create_invoice(1_000_000, "VND").await;
    let receipt = ctx.create_unmatched_receipt(1_000_000, "VND").await;
    let resp = ctx.post_as_cfo("/v1/inv/cash-app/allocate-manual", json!({
        "receipt_id": receipt.id, "invoice_id": invoice.id, "amount_minor": 1_000_000,
        "notes": "Verified payment by phone confirmation 2026-05-15"
    })).await.unwrap();
    assert_eq!(resp.status(), 201);
    let rows = ctx.memory_audit_rows("inv.cash_app_matched_manual").await;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["severity"], "sev-2");
}

#[tokio::test]
async fn manual_match_by_non_cfo_rejected(ctx: TestCtx) {
    let invoice = ctx.create_invoice(1_000_000, "VND").await;
    let receipt = ctx.create_unmatched_receipt(1_000_000, "VND").await;
    let err = ctx.post_as_tenant_admin("/v1/inv/cash-app/allocate-manual", json!({
        "receipt_id": receipt.id, "invoice_id": invoice.id, "amount_minor": 1_000_000, "notes": "test"
    })).await.unwrap_err();
    assert!(format!("{err:?}").contains("permission_denied"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 9 memory row builders follow canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-INV-003** — Stripe webhook (provides receipts).
- **TASK-INV-005** — VietQR webhook (provides receipts + memo parser).

**Downstream:** none at slice 2.

**Cross-module:**
- **TASK-INV-001** — invoices table (FK target + currency + amount_minor).
- **TASK-AUTH-101** — CFO role gate.
- **TASK-AI-003** — memory audit bridge.
- **TASK-MEMORY-111** — PII scrubbing.
- **TASK-OBS-007** — sev-1 alarm on over-allocation + sev-2 on no-match > 24h.
- **TASK-AI-005** — per-tenant fuzzy threshold policy.

---

## §8 — Example payloads

### 8.1 — POST /v1/inv/cash-app/allocate-manual

```json
{
  "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "invoice_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "amount_minor": 4500000,
  "notes": "Customer paid via WeChat instead of VietQR; manual link approved by CFO Q3-2026-042"
}
```

### 8.2 — POST /v1/inv/cash-app/dry-run

```json
{ "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M" }
```

Response:

```json
{
  "would_match": "step3_fuzzy",
  "invoice_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "amount_minor": 970000,
  "fuzzy_percent_difference": 3.0,
  "outstanding_after_match": 30000
}
```

### 8.3 — GET /v1/inv/cash-app/unmatched response

```json
{
  "unmatched": [
    {
      "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
      "amount_minor": 4500000,
      "currency": "VND",
      "transfer_memo": "Thanh toan thang 5 - ACME",
      "received_at": "2026-05-15T10:00:00Z",
      "suggested_matches": [
        { "invoice_id": "9b1deb4d-...", "amount_minor": 4500000, "score": 0.95, "reason": "exact amount match outside date window" },
        { "invoice_id": "8a7c8c80-...", "amount_minor": 4525000, "score": 0.80, "reason": "0.6% fuzzy match" }
      ]
    }
  ]
}
```

### 8.4 — inv.cash_app_matched_step3 memory row

```json
{
  "kind": "inv.cash_app_matched_step3",
  "tenant_id": "5e8f1d2a-...",
  "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "invoice_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "allocation_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8N",
  "allocation_amount_minor": 970000,
  "fuzzy_pct_used": 5,
  "ts_ns": 1747920731000000000
}
```

### 8.5 — inv.cash_app_over_allocation_blocked memory row (sev-1)

```json
{
  "kind": "inv.cash_app_over_allocation_blocked",
  "severity": "sev-1",
  "tenant_id": "5e8f1d2a-...",
  "invoice_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "invoice_amount_minor": 1000000,
  "attempted_sum_minor": 1100000,
  "attempted_by_subject_id_hash16": "9b1deb4d3b7d4bad",
  "ts_ns": 1747920731000000000
}
```

### 8.6 — inv.cash_app_allocation_reversed memory row (sev-2)

```json
{
  "kind": "inv.cash_app_allocation_reversed",
  "severity": "sev-2",
  "tenant_id": "5e8f1d2a-...",
  "original_allocation_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "reversal_allocation_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8N",
  "reason_scrubbed": "Operator linked wrong invoice; corrected via CFO desk 2026-05-16",
  "reversed_by_subject_id_hash16": "8a7c8c8012344567",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Multi-currency cross-allocation (e.g. USD receipt against VND invoice via FX)** — TASK-INV-002 ships SBV FX snapshot; cross-currency cash app is FR-INV-2xx.
- **ML-suggested matches** — slice 3 augmentation of step 4 manual.
- **Bulk import historical allocations** — FR-INV-2xx migration tool.
- **Customer-side allocation hint via API** — TASK-PORTAL-008.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Over-allocation attempted | trigger | P0100 + sev-1 audit | Designed |
| Receipt over-allocated | trigger | P0101 | Designed |
| Manual by non-CFO | role check | 403 | Designed |
| Currency mismatch on manual | handler | 400 currency_mismatch | Designed |
| Invalid amount on manual | handler | 400 | Designed |
| Reversal by non-CFO | role check | 403 | Designed |
| Step 2 ambiguous (multiple candidates) | LIMIT 2 check | NoMatch from step 2; cascades to step 3 | Designed |
| Step 3 ambiguous | same | NoMatch from step 3; falls to manual | Designed |
| Step 3 fuzzy threshold out of bounds | clamp | Threshold capped at [1,20] | Designed |
| Per-tenant policy missing | DEFAULT used | 5% | Designed |
| Receipt > 90 days old | scheduler skip | Only manual handler | None — designed |
| Concurrent scheduler runs | SKIP LOCKED + advisory lock | One wins per receipt | Designed |
| Append-only UPDATE/DELETE from app | SQL grant | permission denied | Designed |
| Currency-mismatch at step 2 | query filter | Skip to step 3 | Designed |
| Currency-mismatch at step 3 | query filter | NoMatch | Designed |
| Invoice auto-mark paid race | UPDATE WHERE status != 'paid' | Idempotent | Designed |
| memory audit emit fail mid-tx | rollback | 500 + retry | memory_writer health |
| OTel span attribute missing | otel_test | CI fails | Fix |
| > 10 unmatched at 24h | OBS rule | sev-2 alarm | Operator review |
| Reversal of already-reversed | sum trigger | OK (net zero is fine; sum stays bounded) | Designed |
| Reversal exceeding original | trigger | P0101 (receipt) or P0100 (invoice) | Designed |
| RLS bypass | USING | 0 rows | Designed |
| Subject deleted while allocations exist | FK RESTRICT | DELETE auth.subjects fails | Soft-delete |
| `cyberos_app` INSERT into allocations | grant absent | permission denied | Use inv_cash_applier role |
| Dry-run side-effect leak | test asserts | CI fails | Fix branching |
| Allocation 0 amount_minor | DB CHECK != 0 | INSERT fails | Designed |
| Positive amount with reverses_allocation_id set | DB CHECK XOR | INSERT fails | Designed |
| Negative amount without reverses_allocation_id | DB CHECK XOR | INSERT fails | Designed |
| Notes > 1000 chars | DB CHECK | INSERT fails | Shorten |
| Memo parser HD-prefix false positive | manual-reversal handler | Operator reverses + re-allocates | Designed |
| Step 1 hit for already-allocated invoice | over-allocation trigger | P0100 | Designed |

---

## §11 — Implementation notes

- **Closed 4-step cascade** — descending confidence, ascending operator effort.
- **Async via 5-min scheduler** — webhook stays fast; matching catches up.
- **Advisory lock per receipt** — multi-statement cascade atomic per receipt.
- **Partial allocation M:N** — VN B2B installment payments reality.
- **Over-allocation block at trigger** — defense in depth.
- **Append-only via SQL grant** — ledger integrity.
- **Reversal as new row** — same correction_to pattern as TASK-TIME-001.
- **CFO-only manual** — segregation of duties.
- **5% fuzzy default** — industry standard.
- **Per-tenant fuzzy override [1,20]%** — flexibility with bounds.
- **9 memory audit kinds** — granular operator queries.
- **Sev-1 alarm on over-allocation** — ledger-corruption signal.
- **Sev-2 escalation on > 10 no-match at 24h** — actionable backlog.
- **Auto-mark `paid` via trigger** — invoice status stays consistent.
- **Outstanding view** — centralised math.
- **Dry-run handler** — pre-commit preview.
- **Currency-mismatch hard skip** — cents-vs-đồng confusion defense.
- **90-day window in scheduler** — backlog avoidance.
- **`inv_cash_applier` SQL role** — defense in depth (reuses TASK-INV-005 role split).
- **DB CHECK on amount_minor signs** — positive iff not reversal.
- **PII scrub notes + memo** — chain holds scrubbed.
- **OBS sev-2 on > 3/h reversals** — high reversal rate signals operator-error or attack.

---

*End of TASK-INV-006.*
