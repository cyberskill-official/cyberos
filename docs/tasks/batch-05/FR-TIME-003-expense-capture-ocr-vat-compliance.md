---
title: "TIME — expense capture, receipt OCR (Vietnamese + English), VAT compliance, INV feed"
author: "@stephen-cheng"
department: finance
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the expense surface inside TIME: **expense entry** with receipt-photo upload + auto-extraction (Tesseract OCR for Vietnamese + English; CUO/CFO-skill structured-extraction for line items, totals, VAT, vendor name); **expense category catalogue** (per-tenant configurable; defaults seeded); **mileage entries** with rate-per-km policy; **per-engagement reimbursable flagging** (consumed by INV in P2 for client billback); **monthly expense submission** flow (Member submits → HR/Ops Lead approves → INV consumes for billable client expenses + payroll for reimbursable team expenses); **Vietnamese VAT compliance** (proper handling of input VAT credit + receipt-data validity checks against the e-invoice format); and **multi-currency** support (USD / EUR / VND / JPY / SGD primary). The module captures travel, software subscriptions billed to a Member's card, client meals, and any other receipt-based expense the team incurs.

## Problem

The team's current expense flow is a Slack message + a photo of a receipt + a spreadsheet entry the founder reconciles monthly. Three failure modes:

- **Receipt loss.** A Member loses a paper receipt; reimbursement is delayed or refused; trust erodes.
- **VAT credit dropped.** Vietnamese tax authority's e-invoice format requires specific fields (vendor tax code, e-invoice serial, line-item VAT split). Receipts that don't match are not eligible for input VAT credit; the company loses ~10% on every miscaptured business expense.
- **Reimbursable client expense lost.** A Member spends $50 on a customer dinner at Acme that the contract reimburses; without structured capture, the expense is forgotten until invoice time and never billed back.

PRD §9.10 ("expense capture") is the canonical commitment; this FR is the implementation.

## Proposed Solution

Schema, capture flow, OCR + structured extraction pipeline, approval workflow, VAT compliance handling, INV feed.

**Schema.**

```sql
CREATE TABLE time.expense_category (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  code TEXT NOT NULL,                          -- "travel_air" | "travel_ground" | "lodging"
                                               -- | "client_meal" | "team_meal" | "office_supplies"
                                               -- | "software_subscription" | "telecom" | "training_external"
                                               -- | "mileage" | "other"
  display_name TEXT NOT NULL,
  default_reimbursable BOOLEAN NOT NULL DEFAULT true,
  default_billable_to_engagement BOOLEAN NOT NULL DEFAULT false,
  requires_receipt BOOLEAN NOT NULL DEFAULT true,
  approval_required_above_amount_minor INT,
  approval_currency TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, code)
);

CREATE TABLE time.expense (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  category_id UUID NOT NULL REFERENCES time.expense_category(id),
  engagement_id UUID,                          -- references proj.engagement
  description TEXT NOT NULL,
  occurred_on DATE NOT NULL,
  amount_minor BIGINT NOT NULL,                -- minor currency units
  currency TEXT NOT NULL,
  exchange_rate_to_tenant_currency REAL,
  amount_in_tenant_currency_minor BIGINT,
  vat_amount_minor BIGINT NOT NULL DEFAULT 0,
  vat_rate REAL,                               -- e.g. 0.10 for 10% Vietnamese VAT
  vendor_name TEXT,
  vendor_tax_code TEXT,                         -- Vietnamese 10/13-digit tax code
  einvoice_serial TEXT,                         -- Vietnamese e-invoice serial
  einvoice_form TEXT,                           -- Vietnamese e-invoice form
  receipt_attachment_id UUID,                   -- references the content-addressed blob store
  receipt_ocr_extracted JSONB,                  -- structured OCR output for review
  is_reimbursable BOOLEAN NOT NULL,
  is_billable_to_engagement BOOLEAN NOT NULL,
  status TEXT NOT NULL DEFAULT 'draft',         -- "draft" | "submitted" | "approved" | "rejected" | "reimbursed"
  submitted_at TIMESTAMPTZ,
  approved_by UUID,
  approved_at TIMESTAMPTZ,
  rejection_reason TEXT,
  reimbursed_at TIMESTAMPTZ,
  reimbursement_method TEXT,                    -- "payroll" | "direct_transfer" | "company_card" | "other"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX expense_member_idx     ON time.expense (tenant_id, member_id, occurred_on);
CREATE INDEX expense_engagement_idx ON time.expense (tenant_id, engagement_id, occurred_on) WHERE is_billable_to_engagement;
CREATE INDEX expense_status_idx     ON time.expense (tenant_id, status, occurred_on);

CREATE TABLE time.mileage_rate (
  tenant_id UUID NOT NULL,
  effective_from DATE NOT NULL,
  effective_to DATE,
  rate_per_km_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  PRIMARY KEY (tenant_id, effective_from)
);
```

**Default seed.**

Categories pre-populated per the table above; `mileage_rate` seeded with the standard Vietnamese business rate (currently ~5,000 VND/km, periodically updated).

**Receipt capture flow.**

1. Member opens `/time/expense/new` (or a mobile-style camera-first surface on small screens).
2. Member uploads / drags a receipt photo (JPG / HEIC / PNG / PDF supported up to 30 MB; the EMAIL chain's ClamAV scan + extension allowlist apply).
3. The OCR pipeline runs:
   - Tesseract OCR (vi + eng languages) extracts raw text.
   - The text + the original image bytes are passed to CUO/CFO-skill via the AI Gateway (FR-AI-001) with a structured-extraction prompt: extract `vendor_name`, `vendor_tax_code`, `einvoice_serial`, `einvoice_form`, `total_amount`, `vat_amount`, `vat_rate`, `currency`, `occurred_on`, `line_items`, `payment_method`. Output as JSON.
   - Confidence per field; high-confidence fields populate the form; low-confidence highlights for the Member to confirm.
4. Member reviews + corrects + saves.

The Vietnamese e-invoice format (Decree 123/2020/NĐ-CP and Circular 78/2021/TT-BTC) defines specific fields; the extractor is calibrated to recognise compliant e-invoices and flag non-compliant receipts for VAT-credit-eligibility review.

OCR latency budget: p95 ≤ 8 s for a 5 MB receipt photo.

**Mileage entries.**

A Member can claim mileage for business travel (bike / car) using their own vehicle:
- Form: date, start address, end address (optional), distance_km, purpose, engagement.
- The rate-per-km from `time.mileage_rate` is applied; `amount_in_tenant_currency_minor` computed automatically.
- `category_id` defaults to "mileage"; `requires_receipt` is false for mileage.

**Submission + approval flow.**

Same shape as FR-TIME-001 but on a monthly cadence:

1. **Last working day of month, 17:00 ICT** — Notify card to each Member: "Submit this month's expenses?".
2. Member submits → all `draft` expenses for the month become `submitted`.
3. HR/Ops Lead reviews: approve / reject / request adjustment.
4. Approved expenses feed:
   - **Reimbursable to Member** (`is_reimbursable: true`, `reimbursement_method: payroll`) — the next month's payroll (FR-REW-001 in P2) consumes.
   - **Billable to Engagement** (`is_billable_to_engagement: true`) — INV (P2) adds to the next invoice for that Engagement.
   - **Company-paid** (paid directly by the company, e.g. SaaS subscription on a company card) — booked to the GL category for accounting reconciliation.

**VAT compliance.**

For each Vietnamese-VAT-eligible expense:
- The OCR-extracted `vendor_tax_code` is validated against checksum + format rules.
- The `einvoice_serial` + `einvoice_form` combination is recorded for input-VAT-credit filing.
- A monthly export `time.vat_credit_export` produces the report HR/Ops Lead files with the Vietnamese tax authority via the company's accountant (Vietnamese tax filing is currently human-assisted; full automation is OQ-FINANCE-VN-FILING).

**Multi-currency.**

- Expense currency is the receipt's currency.
- A daily ECB / SBV (State Bank of Vietnam) exchange-rate poll populates `exchange_rate_to_tenant_currency` at submission time.
- Tenant currency defaults to VND for CyberSkill; per-tenant configurable in P3+.

**Frontend surfaces.**

`/time/expenses` view in the Module-Federation remote (alongside FR-TIME-001's day/week/approval views):

- **My expenses** — list of this month's expenses with totals + status.
- **Camera capture** — primary CTA; opens the upload + OCR flow.
- **Bulk import** — credit-card statement CSV import (P1 supports the major Vietnamese banks: Vietcombank, Techcombank, ACB, BIDV; international banks via generic CSV).
- **Approvals** — HR/Ops Lead view; per-Member submission month + entries.

**MCP tool surface.**

- `cyberos.time.list_expenses(member_id?, status?, since)` — read.
- `cyberos.time.get_expense(id)` — read.
- `cyberos.time.create_expense(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.update_expense(id, patch)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.upload_receipt(image_data)` — `destructive: false`; runs OCR; returns extracted JSON; does not create an expense.
- `cyberos.time.submit_month(member_id, year_month)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.time.approve_expenses(expense_ids)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.time.reject_expenses(expense_ids, reason)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.time.create_mileage(...)` — `destructive: true; requires_confirmation: true`.
- `cyberos.time.set_mileage_rate(rate_per_km, effective_from)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.

CUO scope contracts: read + suggest categorisation allowed; commit-mutations (create / approve / reject) restricted by role + step-up.

## Alternatives Considered

- **Use Expensify / Concur.** Rejected: residency + integration with Engagement primitive + Vietnamese e-invoice handling are not viable in hosted services.
- **No OCR; manual entry only.** Rejected: friction-driven decay; receipt-photo + auto-extraction is the floor.
- **Hosted OCR (Google Document AI, AWS Textract).** Considered. Rejected for default path because per-tenant residency is hard to verify and PII flows out. We use self-hosted Tesseract + AI-Gateway-routed structured extraction (CUO/CFO with the prompt) which keeps data inside residency.
- **Skip e-invoice compliance; let accountant handle VAT.** Rejected: missing the e-invoice fields means the company loses input VAT credit on every miscaptured expense; the structural fields are mandatory for VAT-credit eligibility.
- **Per-day expense submission.** Rejected: monthly cadence aligns with Vietnamese tax filing and the existing accountant relationship.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate progress: ≥ 90% of incurred expenses captured in TIME within 7 days of occurrence; ≥ 95% of Vietnamese-VAT-eligible expenses have valid `einvoice_serial` + `einvoice_form` recorded; OCR auto-extraction acceptance ≥ 70% (Member accepts without correction).
- **Reimbursement latency.** Median time from approval to reimbursement (via payroll path) ≤ 1 month.
- **Latency NFR.** OCR pipeline p95 ≤ 8 s for a 5 MB photo.
- **VAT credit.** Monthly VAT-credit export reconciles 100% with the accountant's filing for 3 consecutive months.

## Scope

**In-scope.**
- `time.expense_category`, `time.expense`, `time.mileage_rate` tables.
- Default seed (10 categories + Vietnamese mileage rate).
- Receipt upload + Tesseract OCR + CUO/CFO structured extraction.
- Mileage entry flow.
- Multi-currency with daily SBV/ECB rate poll.
- Vietnamese VAT compliance (e-invoice fields, validity checks, monthly VAT-credit export).
- Monthly submission + approval flow.
- INV-feed stub (the approved expenses are the data INV-001 consumes in P2).
- Bulk credit-card-statement CSV import for the four major Vietnamese banks.
- The 10 MCP tools.
- Audit integration in scope `time.expense.{tenant}`.

**Out-of-scope (deferred).**
- Direct integration with Vietnamese tax-authority e-invoice portal (OQ-FINANCE-VN-FILING; P3).
- Per-card auto-import via Open-Banking APIs (P3 if the Vietnamese banks adopt PSD2-like APIs).
- Per-Member virtual-card issuance (P4 fintech feature).
- Multi-leg travel expense aggregation (P2).
- Per-engagement budget alerts (P2; INV-001 owns).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-TIME-001 (parent module).
- FR-PROJ-001 (Engagement linkage for billable expenses).
- FR-EMAIL-010 (ClamAV chain reused for receipt scanning).
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002.
- Self-hosted Tesseract OCR with vi + eng language packs on the GPU node from FR-AI-001.
- A Vietnamese tax-code checksum library (curated in-house).
- Compliance: Vietnamese Decree 123/2020/NĐ-CP + Circular 78/2021 (e-invoice format); PDPL Decree 13 (receipts contain personal data); ISO/IEC 27001 (data integrity).
- Locked decisions referenced: DEC-142 (self-hosted Tesseract + CUO structured extraction; no third-party OCR), DEC-143 (monthly cadence), DEC-144 (VAT-credit fields are mandatory for VN-VAT-eligible expenses).

## AI Risk Assessment

OCR + structured extraction are AI surfaces; the extracted data drives reimbursement + VAT credit. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: Member-uploaded receipts. Tesseract is open weights; CUO/CFO runs through the AI Gateway with persona-stamping. Per-tenant residency.

### Human Oversight

- Every OCR result is presented to the Member for review before save.
- Submission + approval is a two-step workflow.
- Adjustment after approval is fully audited.
- VAT-credit export is reviewed by the accountant before filing.

### Failure Modes

- **OCR mis-extracts amount.** Mitigation: low-confidence highlight; the Member sees the discrepancy before save.
- **Wrong currency detected.** Mitigation: per-region default (VND for VN-receipts); the Member overrides if needed.
- **Receipt photo unreadable.** Mitigation: the form gracefully falls back to fully-manual entry; the Member can attach a re-shot photo.
- **Vendor tax code missing on a non-compliant receipt.** Mitigation: the form flags this with a "VAT credit may not be eligible" warning; the Member can still submit.
- **Reimbursement-method mismatch.** Mitigation: HR/Ops Lead reviews method during approval; payroll vs. direct-transfer is HR/Ops Lead's call.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, OCR pipeline, VAT-compliance handling, submission flow, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the Vietnamese e-invoice field set re-verified with the accountant at PR-review.
