---
title: "INV — schema (Vendors, POs, Invoices, AR/AP entries, Assets); consumes TIME + PROJ + REW; Apollo subgraph + RLS"
author: "@stephen-cheng"
department: finance
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the INV (Invoicing + financial flow) module's schema and Apollo Federation subgraph. Six primitives: **Vendor** (companies CyberSkill pays — software subscriptions, contractors, suppliers), **PurchaseOrder** (committed spend before invoice), **Invoice** (both **outbound** to clients via Engagement → Account linkage + **inbound** from vendors), **PaymentEntry** (the actual money movement, AR + AP), **Asset** (depreciable equipment + subscriptions tracked over time), and **TaxLot** (Vietnamese VAT credit accumulation + PIT withholding aggregation feeding REW-004's year-end reconciliation). Consumers: **TIME-001/003** approved time + expenses → outbound invoices via the Engagement rate-card; **PROJ-007** rate-card + budget-burn signals; **REW-003** payroll tracks employee-side flows; **CRM-001** Account → Engagement → outbound-invoice. Integrates with Vietnamese e-invoice format (Decree 123/2020/NĐ-CP) for both directions. Lives in a new `inv` schema (mostly non-secret) + a thin `inv_secure` for vendor banking + payment credentials. Subsequent batch-08 FRs ship lifecycle + dunning (FR-INV-002), payment integrations (FR-INV-003), and the frontend (FR-INV-004).

## Problem

CyberSkill today runs invoicing through a manually-maintained spreadsheet + bank-portal-uploaded VNPay transfers + email-attached PDFs to clients. Three failure modes the platform must structurally avoid:

- **Cash-flow opacity.** Without structured AR aging + AP forecasting, "what does our cash position look like in 30 days?" is answerable only by hand-walking the spreadsheet.
- **Vietnamese e-invoice non-compliance.** Decree 123/2020/NĐ-CP + Circular 78/2021 mandate the e-invoice format with specific fields (vendor tax code, e-invoice serial + form, line-item VAT split). Invoices that don't match are not deductible for VAT credit; missed credits are silent revenue erosion.
- **Disconnected billable-time → invoice.** Without TIME → INV automation, billable hours from FR-TIME-001 + reimbursable expenses from FR-TIME-003 are invoiced manually each month; the team's existing two-long-term-engagement billing has historically had a 1-2 day reconciliation overhead per invoice.

PRD §9.16 names "Invoice lifecycle; AR aging; Stripe / Wise / VND-PSP integration; tax compliance." This FR ships the substrate.

## Proposed Solution

The shape of the answer is `inv.*` schema (most data) + `inv_secure.*` schema (vendor banking + payment credentials; same KMS pattern as FR-HR-001's `hr_secure`) + an Apollo Federation v2 subgraph + RLS + cross-module consumers.

**Schema (`inv` — non-secret).**

```sql
CREATE SCHEMA inv;

-- Vendor: a company we pay.
CREATE TABLE inv.vendor (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  name TEXT NOT NULL,
  slug TEXT NOT NULL,
  legal_name TEXT,
  vendor_kind TEXT NOT NULL,                                       -- "saas_subscription" | "contractor" | "supplier"
                                                                   -- | "professional_services" | "regulatory_fee"
  vendor_country TEXT NOT NULL,                                    -- ISO 3166-1
  vendor_tax_code TEXT,                                             -- VN MST or equivalent
  primary_contact_email TEXT,
  primary_contact_name TEXT,
  default_currency TEXT NOT NULL,
  default_payment_terms_days INT NOT NULL DEFAULT 30,
  default_payment_method TEXT,                                       -- "bank_transfer" | "stripe" | "wise" | "vnpay" | "card"
  status TEXT NOT NULL DEFAULT 'active',                              -- "active" | "inactive" | "blocked"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ,
  UNIQUE (tenant_id, slug)
);

-- Purchase order — committed spend.
CREATE TABLE inv.purchase_order (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  vendor_id UUID NOT NULL REFERENCES inv.vendor(id),
  po_number TEXT NOT NULL,                                           -- "PO-2026-001"
  description_md TEXT,
  amount_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  expected_delivery_date DATE,
  cost_center_engagement_id UUID,                                     -- when bookable to a client engagement (rare; mostly internal)
  cost_center_kind TEXT NOT NULL,                                     -- "billable_to_engagement" | "engineering_overhead"
                                                                     -- | "operations" | "sales_marketing" | "people_team"
                                                                     -- | "hardware" | "software_subscription"
  status TEXT NOT NULL DEFAULT 'draft',                                -- "draft" | "approved" | "fulfilled" | "cancelled"
  approved_by UUID,
  approved_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, po_number)
);

-- Invoice (outbound to clients OR inbound from vendors).
CREATE TABLE inv.invoice (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  invoice_number TEXT NOT NULL,                                       -- "INV-2026-001" or vendor-side "VINV-..."
  direction TEXT NOT NULL,                                             -- "outbound_to_client" | "inbound_from_vendor"
  -- For outbound:
  client_account_id UUID,                                              -- references crm.account
  client_engagement_id UUID,                                           -- references proj.engagement
  -- For inbound:
  vendor_id UUID,
  purchase_order_id UUID,
  -- Common:
  invoice_date DATE NOT NULL,
  due_date DATE NOT NULL,
  amount_subtotal_minor BIGINT NOT NULL,
  vat_amount_minor BIGINT NOT NULL DEFAULT 0,
  amount_total_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  -- Vietnamese e-invoice fields (mandatory for VN-VAT-eligible invoices):
  einvoice_form TEXT,                                                  -- e-invoice form code per Decree 123
  einvoice_serial TEXT,                                                -- e-invoice serial
  einvoice_signed_xml_blob_id UUID,                                    -- references the signed XML in the blob store
  -- Lifecycle:
  status TEXT NOT NULL DEFAULT 'draft',                                 -- "draft" | "sent" | "viewed_by_client"
                                                                       -- | "partially_paid" | "paid" | "overdue"
                                                                       -- | "void" | "disputed"
  sent_at TIMESTAMPTZ,
  paid_at TIMESTAMPTZ,
  payment_method TEXT,
  payment_reference TEXT,
  -- Source-of-billing for outbound:
  source_kind TEXT,                                                    -- "time_based_monthly" | "expense_passthrough"
                                                                       -- | "fixed_fee_milestone" | "ad_hoc"
  source_period_start DATE,
  source_period_end DATE,
  pdf_blob_id UUID,                                                    -- the signed-PDF artefact
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, invoice_number)
);

CREATE INDEX invoice_direction_status_idx ON inv.invoice (tenant_id, direction, status);
CREATE INDEX invoice_engagement_idx        ON inv.invoice (tenant_id, client_engagement_id) WHERE direction = 'outbound_to_client';
CREATE INDEX invoice_vendor_idx            ON inv.invoice (tenant_id, vendor_id) WHERE direction = 'inbound_from_vendor';
CREATE INDEX invoice_due_date_idx          ON inv.invoice (tenant_id, due_date) WHERE status NOT IN ('paid', 'void');

-- Line items (for both directions).
CREATE TABLE inv.invoice_line_item (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  invoice_id UUID NOT NULL REFERENCES inv.invoice(id) ON DELETE CASCADE,
  position INT NOT NULL,
  description TEXT NOT NULL,
  quantity NUMERIC NOT NULL,
  unit_label TEXT,                                                     -- "hours", "items", "GB", etc.
  unit_price_minor BIGINT NOT NULL,
  amount_minor BIGINT NOT NULL,                                         -- quantity * unit_price
  vat_rate_pct REAL NOT NULL DEFAULT 0,                                  -- 10% for typical Vietnamese VAT
  vat_amount_minor BIGINT NOT NULL DEFAULT 0,
  -- Linkage to source data:
  source_kind TEXT,                                                    -- "time_entries" | "expense_records" | "milestone_payment"
  source_refs JSONB,                                                   -- e.g. { time_entry_ids: [...], expense_ids: [...] }
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Payment entries (AR + AP money movements).
CREATE TABLE inv.payment_entry (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  invoice_id UUID NOT NULL REFERENCES inv.invoice(id),
  direction TEXT NOT NULL,                                             -- "ar_received" | "ap_sent"
  amount_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  occurred_at DATE NOT NULL,
  payment_method TEXT NOT NULL,                                        -- "bank_transfer" | "stripe" | "wise" | "vnpay" | "card"
  payment_reference TEXT,                                              -- bank-transfer ref or Stripe charge ID
  reconciled_at TIMESTAMPTZ,                                            -- when matched to bank statement
  reconciliation_method TEXT,                                          -- "manual" | "auto_match"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX payment_entry_invoice_idx ON inv.payment_entry (tenant_id, invoice_id);

-- Asset (depreciable items + recurring subscriptions tracked over time).
CREATE TABLE inv.asset (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  asset_kind TEXT NOT NULL,                                            -- "hardware_laptop" | "hardware_other"
                                                                       -- | "software_subscription" | "office_furniture"
  name TEXT NOT NULL,
  vendor_id UUID REFERENCES inv.vendor(id),
  acquisition_date DATE NOT NULL,
  acquisition_cost_minor BIGINT NOT NULL,
  currency TEXT NOT NULL,
  expected_useful_life_months INT,                                      -- for depreciation calc
  current_book_value_minor BIGINT,                                       -- recomputed monthly via straight-line
  assigned_employee_id UUID,                                              -- when assigned to a Member (e.g. laptop)
  status TEXT NOT NULL DEFAULT 'active',                                  -- "active" | "retired" | "disposed" | "lost"
  retirement_date DATE,
  retirement_reason_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tax lot (VAT credit accumulation per tax-period).
CREATE TABLE inv.tax_lot (
  tenant_id UUID NOT NULL,
  tax_period TEXT NOT NULL,                                             -- "2026-09" or "2026-Q3"
  jurisdiction TEXT NOT NULL DEFAULT 'VN',
  total_input_vat_minor BIGINT NOT NULL DEFAULT 0,                       -- VAT we paid on inbound
  total_output_vat_minor BIGINT NOT NULL DEFAULT 0,                      -- VAT we collected on outbound
  net_vat_payable_minor BIGINT NOT NULL DEFAULT 0,                        -- output - input (when positive, owed)
  filing_status TEXT NOT NULL DEFAULT 'pending',                          -- "pending" | "filed" | "settled"
  filing_signed_off_by_accountant_ref TEXT,
  filed_at DATE,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, tax_period, jurisdiction)
);
```

**Schema (`inv_secure` — payment credentials encrypted under the same KMS-key pattern as `hr_secure`).**

```sql
CREATE SCHEMA inv_secure;

CREATE TABLE inv_secure.vendor_banking (
  tenant_id UUID NOT NULL,
  vendor_id UUID NOT NULL UNIQUE REFERENCES inv.vendor(id) ON DELETE CASCADE,
  bank_account_number_encrypted BYTEA,
  bank_routing_encrypted BYTEA,
  bank_swift_encrypted BYTEA,
  vat_id_encrypted BYTEA,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, vendor_id)
);

-- Payment-provider credentials (encrypted; never logged):
CREATE TABLE inv_secure.payment_provider_credential (
  tenant_id UUID NOT NULL,
  provider TEXT NOT NULL,                                                -- "stripe" | "wise" | "vnpay"
  credential_kind TEXT NOT NULL,                                          -- "api_key" | "webhook_secret"
  credential_value_encrypted BYTEA NOT NULL,
  rotation_due_at DATE,
  last_rotated_at DATE,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, provider, credential_kind)
);
```

**Cross-module consumers.**

- **TIME-001 → outbound invoices.** A scheduled monthly job at the end-of-month walks `time.entry` rows with `status: approved` + `is_billable: true` + `engagement_id` matching active `proj.engagement` rows. Aggregates per Engagement; produces a draft invoice with line items per Member-rate-card-role (the rate card from `proj.engagement.rate_card`).
- **TIME-003 → outbound invoices.** Same pattern for `time.expense` rows with `is_billable_to_engagement: true`. Each expense becomes a line item with VAT separated.
- **PROJ-007 budget burn.** A periodic job aggregates outbound-invoiced amounts per Engagement; compares to `proj.engagement.budget_amount_minor`; surfaces a Notify if budget burn exceeds 80%.
- **REW-003 → AP payroll.** Payroll cycle's bank-disbursement file (FR-REW-003) creates an AP entry per cycle (one aggregate inbound invoice from "Internal: Payroll Disbursement" representing the gross paid).
- **CRM-001 → outbound dunning.** Overdue AR triggers FR-INV-002's dunning email path with the CRM contact's preferred language (vi-VN / en-US).

**Federation directives.**

- `InvInvoice @key(fields: "id")`.
- `InvVendor @key(fields: "id")`.
- `Member @key(fields: "id") @external` references AUTH.
- `CrmAccount @key(fields: "id") @external` cross-references CRM (FR-CRM-001).
- `ProjEngagement @key(fields: "id") @external` cross-references PROJ (FR-PROJ-001).

**RLS + ACL.**

- `inv.*` (non-secret): tenant RLS + ACL — Account Manager + HR/Ops Lead + Founder + Auditor see all; Member sees their own assigned assets only; client-engagement-specific views accessible to that engagement's primary owner.
- `inv_secure.*`: tenant RLS + tighter ACL — only HR/Ops Lead + Founder + DPO. Every read writes an audit row with `field_kind` + `purpose` (mandatory).

**GraphQL subgraph.**

```graphql
type Query {
  invVendors(status: String, kind: String): [InvVendor!]!
  invVendor(id: ID, slug: String): InvVendor
  invPurchaseOrders(status: String, vendorId: ID): [InvPurchaseOrder!]!
  invPurchaseOrder(id: ID!): InvPurchaseOrder
  invInvoices(direction: String, status: [String!], engagementId: ID, vendorId: ID,
              dueBefore: Date, dueAfter: Date, first: Int = 50): InvInvoiceConnection!
  invInvoice(id: ID!): InvInvoice
  invAgingReport(asOf: Date, direction: String): InvAgingReport!
  invPaymentEntries(invoiceId: ID): [InvPaymentEntry!]!
  invAssets(employeeId: ID, kind: String, status: String): [InvAsset!]!
  invTaxLots(period: String, jurisdiction: String): [InvTaxLot!]!
  # Secure-tier queries with @stepUp:
  invVendorBanking(vendorId: ID!): InvVendorBanking
}

type Mutation {
  invCreateVendor(input: InvVendorInput!): InvVendor!
  invUpdateVendor(id: ID!, patch: InvVendorPatch!): InvVendor!
  invUpsertVendorBanking(vendorId: ID!, patch: InvVendorBankingPatch!): InvVendorBanking!
  invCreatePurchaseOrder(input: InvPurchaseOrderInput!): InvPurchaseOrder!
  invApprovePurchaseOrder(id: ID!): InvPurchaseOrder!
  invCreateInvoice(input: InvInvoiceInput!): InvInvoice!
  invDraftOutboundInvoiceFromTime(engagementId: ID!, periodStart: Date!, periodEnd: Date!): InvInvoice!
  invSendInvoice(id: ID!): InvInvoice!
  invMarkInvoicePaid(id: ID!, payment: InvPaymentEntryInput!): InvInvoice!
  invVoidInvoice(id: ID!, reason: String!): InvInvoice!
  invCreatePaymentEntry(input: InvPaymentEntryInput!): InvPaymentEntry!
  invReconcilePayment(paymentId: ID!): InvPaymentEntry!
  invCreateAsset(input: InvAssetInput!): InvAsset!
  invRetireAsset(id: ID!, reason: String!): InvAsset!
}
```

Persisted-queries discipline applies. Secure-tier queries + mutations carry the `@stepUp` directive (FR-AUTH-003).

**MCP tool surface.**

Read tools (everyone with appropriate ACL):

- `cyberos.inv.list_invoices(direction?, status?, engagement_id?)` — read.
- `cyberos.inv.get_invoice(id)` — read.
- `cyberos.inv.aging_report(as_of?, direction?)` — read; HR/Ops + Founder + Auditor.
- `cyberos.inv.list_assets(employee_id?, kind?)` — read.
- `cyberos.inv.list_my_assigned_assets` — read; calling Member.
- `cyberos.inv.list_vendors(status?, kind?)` — read.
- `cyberos.inv.list_pos(status?)` — read.
- `cyberos.inv.tax_lot_summary(period)` — read.

Mutation tools (HR/Ops + Founder; multi-step destructive-confirmation):

- `cyberos.inv.draft_outbound_invoice_from_time(engagement_id, period_start, period_end)` — `destructive: false; idempotent: true` (drafts only; sending is separate).
- `cyberos.inv.send_invoice(id)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.inv.mark_invoice_paid(id, payment)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.inv.void_invoice(id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.inv.create_vendor(input)` — `destructive: true; requires_confirmation: true`.
- `cyberos.inv.upsert_vendor_banking(vendor_id, patch)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.

CUO scope contract: read all + draft-from-time allowed; commit-mutations forbidden (compensation-adjacent + payment-movement is human-only).

**Audit integration.** `inv.{tenant}` audit scope; `inv_secure.{tenant}` for secure-tier reads + writes with `field_kind` + `purpose`.

**BRAIN denylist + structural exclusion.**

- `inv_secure.*` is structurally excluded from BRAIN ingestion.
- `inv.invoice.amount_*_minor` + `inv.payment_entry.amount_minor` are denylisted at ingestion (compensation-adjacent monetary values).
- Vendor banking + payment-provider credentials never enter BRAIN.

## Alternatives Considered

- **Use a hosted invoicing tool (QuickBooks, Xero, MISA).** Rejected: residency + Engagement linkage + Vietnamese e-invoice integration require platform ownership.
- **Skip e-invoice integration; manually upload to MPS portal.** Rejected: VAT credit eligibility + tax-period reconciliation depend on it.
- **Auto-send invoices on schedule.** Rejected: human-in-the-loop floor; HR/Ops Lead reviews + sends.
- **Combine all invoicing data in a single `invoice` table without `direction` distinction.** Rejected: the AR + AP semantics differ enough to warrant explicit direction modelling.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) HR/Ops Lead creates 5+ vendors + their banking under step-up; (2) the time-based monthly invoice draft for one Engagement is generated correctly from `time.entry` aggregates; (3) Vietnamese e-invoice fields populate + the signed XML is generated; (4) AR aging report renders correctly; (5) RLS denies a non-Member read of vendor banking.
- **Compliance metric.** Zero invoice values appear in BRAIN; zero vendor-banking-data reads without `field_kind` + `purpose` audit-row.
- **Latency NFR.** Aging report p95 ≤ 800 ms over 500 invoices.

## Scope

**In-scope.**
- The 6 schema additions in `inv` + 2 in `inv_secure`.
- Federation directives + cross-module references.
- TIME → outbound invoice draft generator.
- TIME → reimbursable expense passthrough.
- PROJ → budget-burn signal.
- REW → AP payroll entry.
- Vietnamese e-invoice field plumbing.
- AR/AP aging report.
- The 8 read MCP tools + 6 mutation MCP tools.
- Audit integration in `inv.{tenant}` + `inv_secure.{tenant}`.

**Out-of-scope (deferred to FR-INV-002 / FR-INV-003 / FR-INV-004).**
- Invoice lifecycle automation + dunning email drafts (FR-INV-002).
- Stripe + Wise + VNPay integrations + reconciliation (FR-INV-003).
- Frontend remote at /inv (FR-INV-004).
- Asset depreciation auto-compute (P3 — accountant-driven for now).
- AP automation beyond the schema (P3).
- Multi-tenant invoicing federation (forbidden by design).

## Dependencies

- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-HR-001 (the `hr_secure` KMS-key pattern reused for `inv_secure`; separate per-tenant key for `inv_secure`).
- FR-TIME-001 (approved time entries).
- FR-TIME-003 (approved expenses).
- FR-PROJ-001 / FR-PROJ-007 (Engagement + rate-card + budget signals).
- FR-REW-003 (payroll AP entry).
- FR-CRM-001 (Account → outbound invoice).
- FR-CP-001 (Compliance Cockpit panel).
- HashiCorp Vault for the `inv_secure` per-tenant KMS key.
- Compliance: Vietnamese Decree 123/2020/NĐ-CP + Circular 78/2021 (e-invoice format); PDPL Decree 13; SOC 2 CC6 (logical access on `inv_secure`); ISO 27001.
- Locked decisions referenced: DEC-227 (invoice direction modelled explicitly), DEC-228 (separate `inv_secure` schema with per-tenant KMS key), DEC-229 (Vietnamese e-invoice field plumbing in P2).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The schema + federation are deterministic; AI surfaces (CUO/CFO dunning drafts, AR-prediction, reconciliation suggestions) ship in FR-INV-002 with their own classification.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
