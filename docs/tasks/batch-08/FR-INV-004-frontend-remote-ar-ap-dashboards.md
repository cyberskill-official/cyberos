---
title: "INV — frontend remote at /inv (AR/AP dashboards, invoice editor, dunning queue, vendor management)"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the INV Module-Federation remote at `/inv` consuming FR-INV-001/002/003. Five primary surfaces: **founder cash-flow dashboard** (`/inv/dashboard`) with AR aging buckets, AP forecast, this-month + next-month revenue projection, late-payer flags + DSO trend; **outbound invoice management** (`/inv/ar`) for drafting + sending + tracking client invoices including the "draft from time" shortcut; **inbound invoice management** (`/inv/ap`) for vendor invoices + PO matching + bank-statement reconciliation; **dunning queue** (`/inv/dunning`) showing CUO/CFO-drafted dunning emails awaiting review; **vendor + asset management** (`/inv/vendors`, `/inv/assets`). Plus admin views for tax-period reconciliation + payment-provider configuration. The frontend is the primary daily-driver surface for HR/Ops Lead + Founder for financial operations.

## Problem

The INV substrate without a frontend is unusable. Three failure modes:

- **No founder cash-flow visibility.** The team's most-asked weekly question is "what's our cash position?"; without a dashboard, this is a 30-minute spreadsheet reconciliation.
- **Manual invoice drafting friction.** Each outbound invoice today is hand-typed in a Google Doc; with 2 active engagements + monthly cadence, that's ~2 hours/month of mechanical work.
- **Dunning queue invisibility.** Without a dedicated queue, dunning drafts get buried in inbox; collection latency stretches.

## Proposed Solution

The shape of the answer is a Vite + React 19 Module-Federation remote at `/inv` consuming the GraphQL surfaces from FR-INV-001/002/003.

**Founder cash-flow dashboard (`/inv/dashboard`).**

For Founder + HR/Ops Lead.

- **Top KPIs.**
  - Cash position (today; ±X% vs. last month).
  - 30-day AR forecast (sum of current + 1-30 day overdue × historical-collection-probability).
  - 30-day AP forecast (sum of inbound invoices + payroll due + recurring SaaS).
  - Net cash flow projection (forecast).
- **AR aging chart.**
  - Stacked bar by bucket (current / 1-30 / 31-60 / 61-90 / 90+).
  - Trend line over the last 6 months.
  - Click a bucket → opens that bucket's invoice list.
- **AP forecast chart.**
  - Stacked bar by week for the next 4 weeks.
  - Categories: payroll, vendor SaaS, contractors, regulatory fees, other.
- **Late-payer flags.**
  - List of accounts with active flags + days-overdue + linked Engagement.
  - Click → opens CRM account 360 (FR-CRM-002).
- **DSO trend.**
  - Days-Sales-Outstanding rolling 90-day; green/yellow/red zones.
- **Recent payments.**
  - Last 10 payment entries (auto-matched + manually-confirmed) with amounts + sources.

Every amount-bearing reveal requires step-up auth (FR-AUTH-003).

**Outbound invoice management (`/inv/ar`).**

- **Invoice list.** Sortable by status, due date, account, amount. Filters: status, account, engagement, currency, period.
- **"Draft new outbound invoice" CTA.**
  - **Time-based draft from Engagement.** Form: select Engagement + period (month / week). The platform pulls approved `time.entry` rows + reimbursable expenses; aggregates per role-rate; produces a draft.
  - **Manual draft.** Free-form for ad-hoc invoices.
  - **Milestone draft.** Fixed-fee milestones from PROJ-007 surface as line items.
- **Invoice editor.** Markdown-style editor for description; line-item table with inline edits; VAT auto-calculated per Vietnamese rules; preview the PDF; preview the e-invoice XML.
- **Send action.** Step-up + destructive confirmation; the email is composed via FR-EMAIL-005 vi-VN composer for Vietnamese clients (with payment-link from FR-INV-003 embedded); audit + CRM activity logged.
- **Invoice detail drawer.** Status timeline; payment history; linked time-entries / expenses; reminders sent; PDF + e-invoice XML download.

**Inbound invoice management (`/inv/ap`).**

- **Invoice list.** Filtered by vendor, status, due date.
- **"Add inbound invoice" CTA.** Manual entry from a vendor's PDF + linkage to PO + cost-center allocation.
- **Bank-statement reconciliation.**
  - Drag-and-drop CSV upload.
  - Matched rows shown in green; unmatched in yellow; suspicious in red.
  - Manual-confirm pane for low-confidence matches.
  - Audit + payment-entry creation upon confirmation.
- **Vendor balance views.** Per-vendor open AP + payment history.

**Dunning queue (`/inv/dunning`).**

For HR/Ops Lead + Founder.

- **Pending review queue.** CUO/CFO-drafted dunning emails awaiting send; sorted by escalation level (90+ first, 30-day last).
- **Per-card.**
  - Invoice context (number, account, amount, days overdue).
  - Draft email (vi-VN or en-US per CRM contact preference).
  - Edit + send actions; "regenerate draft" if the human wants a fresh take.
  - Skip / snooze actions (with reason).
- **Dunning history per invoice.** Past sends + responses (CRM activities).

**Vendor + asset management.**

- **Vendor list.** Active + inactive; per-vendor open POs + open inbound invoices.
- **Vendor detail.** Banking (encrypted; revealed under step-up; HR/Ops + Founder + DPO only).
- **Asset list.** All assets; filter by kind, employee, status; click → asset detail with depreciation schedule + assignment history.

**Tax-period reconciliation (`/inv/admin/tax`).**

- Per-tax-period summary: total input VAT, output VAT, net payable.
- Filing-status workflow: pending → filed → settled.
- Export for the company's external accountant.

**Payment-provider configuration (`/inv/admin/payments`).**

- Per-provider status (Stripe / VNPay / Wise health checks).
- Webhook event log (recent 100 with status).
- Provider-credential rotation reminder (rotation_due_at field).
- Bank-statement import history.

**Vietnamese-locale rendering.**

- vi-VN default for Vietnamese invoices + Vietnamese clients' dunning.
- Currency formatting per Vietnamese convention.
- Invoice PDF template in vi-VN with bilingual fields where the client is international.

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- Cash-flow dashboard p95 ≤ 1 s.
- Invoice list p95 ≤ 600 ms over 500 invoices.
- Invoice editor first-keystroke ≤ 30 ms.
- Step-up reveal ≤ 2 s.

**MCP tool surface (read-only).**

- `cyberos.inv.dashboard_payload` — read; the cash-flow dashboard data; HR/Ops + Founder; step-up.
- `cyberos.inv.list_dunning_drafts` — read; HR/Ops + Founder.
- `cyberos.inv.list_my_assigned_assets` — read; calling Member.

## Alternatives Considered

- **Skip the founder cash-flow dashboard; show only invoice lists.** Rejected: cash visibility is the floor for the founder's daily flow.
- **Embed payment-provider config in `/auth/account`.** Rejected: payment-provider config is INV-scoped admin concern.
- **Flat invoice list without status separation.** Rejected: AR vs. AP have different access patterns + different daily-driver workflows.
- **No bank-statement reconciliation UI; CLI-only.** Rejected: HR/Ops Lead needs a visual tool for the manual-confirm step.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the cash-flow dashboard renders for the canonical CyberSkill tenant; (2) HR/Ops Lead drafts a time-based outbound invoice for an Engagement; (3) the dunning queue surfaces a synthetic 30-day-overdue invoice's draft; (4) bank-statement CSV import auto-matches ≥ 80% of rows.
- **Adoption metric.** Founder uses the cash-flow dashboard ≥ 5 times/week; HR/Ops Lead uses the dunning queue weekly; manual-invoice-typing time reduced by ≥ 75% vs. baseline.
- **Latency NFR.** Per the budgets above; bundle ≤ 50 KB.

## Scope

**In-scope.**
- The Module-Federation remote at `/inv`.
- All 5 primary surfaces.
- Vietnamese-locale rendering + invoice PDF template.
- Step-up enforcement on every encrypted reveal.
- Bank-statement CSV upload + reconciliation UI.
- Tax-period + payment-provider admin views.
- The 3 read-only MCP tools.
- Audit integration in scope `inv.ui.{tenant}`.

**Out-of-scope (deferred).**
- Mobile native (P3).
- Per-Member spending visibility (P3 — Members see their own assigned assets only in P2).
- Multi-tenant aggregated dashboards (forbidden).
- Cap-table view (P3+ — fundraising prep).

## Dependencies

- FR-INV-001 / FR-INV-002 / FR-INV-003.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-EMAIL-001..010 (composer + send path).
- FR-CRM-001 / FR-CRM-002 (account 360 cross-link).
- FR-PROJ-001 / FR-PROJ-007 (Engagement context).
- FR-GENIE-001 / FR-GENIE-002.
- FR-OBS-001 / FR-OBS-002.
- Compliance: PDPL Decree 13; PCI DSS (no card data on the platform; minimised scope); EU AI Act Article 50 (dunning surfaces inherit FR-INV-002's disclosure).
- Locked decisions referenced: DEC-236 (5-surface frontend layout), DEC-237 (per-Engagement time-based invoice draft is the primary AR-creation path).

## AI Risk Assessment

The dunning surface inherits FR-INV-002's classification. Frontend itself is deterministic UI. EU AI Act risk class: `limited` for the consumer surfaces.

### Data Sources

ACL-scoped GraphQL data; payment-provider data via webhook + reconciliation. Per-tenant residency.

### Human Oversight

All financial state changes (send, mark-paid, void, refund) are step-up + multi-party-confirm. AI-derived elements (CUO/CFO dunning drafts) carry the disclosure chip + the human-review gate.

### Failure Modes

- **Step-up bypass.** Mitigated by FR-AUTH-003 server-side enforcement.
- **Reconciliation false-match.** Manual-confirm pane catches; audit-trail allows reversal.
- **Vietnamese e-invoice template drift** (regulator updates). Mitigated by parameter-versioned template; legal counsel + accountant review on any update.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted 5-surface layout, dashboard widgets, dunning queue UX, reconciliation UI, failure modes.
- **Human review:** `@stephen-cheng` reviewed; founder + HR/Ops Lead + external accountant will validate the dashboard + invoice template before P2 production.
