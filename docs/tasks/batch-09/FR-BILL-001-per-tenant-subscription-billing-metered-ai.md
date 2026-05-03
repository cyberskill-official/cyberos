---
title: "BILL — per-tenant subscription tiers, metered AI usage, Stripe + VNPay subscription integration, billing-failure → suspension"
author: "@stephen-cheng"
department: finance
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: true
template: feature_request@1
---

# Summary

Stand up the **Billing module** for external paying tenants per PRD §14.4.1: **subscription tiers** (T1 Starter, T2 Growth, T3 Enterprise) with per-tier feature flags + per-tier seat caps + per-tier AI usage allowances; **metered AI usage** (per-tenant rolling token count + per-tenant cost cap; overage triggers either Notify-card warnings or hard-cap downgrades to Haiku-only); **Stripe + VNPay subscription billing** (re-using FR-INV-003's payment integrations; Stripe handles international subscriptions; VNPay handles Vietnamese tenants); **automated invoice generation** per tenant per period with PDF + e-invoice XML; **billing-failure → suspension flow** with FR-TEN-002's lifecycle (3-payment-failure auto-suspends with 7-day grace + Notify cards); **tenant-admin billing portal** at `/tenant/admin/billing` for plan management + payment method + invoice history. Subscription billing is the platform's commercial-readiness substrate (PRD §4.1 G6).

## Customer Quotes

<untrusted_content source="founder_anticipation">
"Until billing works automatically + transparently — what plan we're on, what we're using, what we're paying — every customer interaction has a billing-conversation overhead the customer doesn't want." — anticipated by Stephen
</untrusted_content>

# Problem

PRD §14.4.1 P3 scope: "Billing — per-tenant subscription tiers; metered usage for AI calls; Stripe integration for non-VN, VNPay for VN." Three failure modes the platform must structurally avoid:

- **Manual billing chaos.** A tenant signs up; the platform doesn't auto-bill; the founder manually invoices each month. Operational tax + cash-flow lag.
- **AI cost runaway per tenant.** A high-volume tenant exceeds expected AI usage; without metering + cost caps, the platform's gross margin collapses on a single tenant.
- **Plan-change ambiguity.** A tenant upgrades from T1 → T2 mid-cycle; without clean prorating + plan-feature flipping, the experience is friction-heavy.

# Proposed Solution

**Schema.**

```sql
CREATE SCHEMA bill;

-- Subscription tier catalogue.
CREATE TABLE bill.plan_tier (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tier_code TEXT NOT NULL UNIQUE,                                       -- "T1_starter" | "T2_growth" | "T3_enterprise"
  display_name TEXT NOT NULL,
  monthly_price_minor BIGINT NOT NULL,                                   -- per seat
  annual_price_minor BIGINT NOT NULL,                                    -- per seat (typical 17% discount vs. monthly × 12)
  currency TEXT NOT NULL,                                                -- "USD" for international; "VND" for VN tenants
  max_seats INT,                                                          -- null = unlimited
  features JSONB NOT NULL,                                                -- per-feature on/off flags
  ai_token_allowance_monthly BIGINT NOT NULL,                             -- per-seat-included AI tokens
  ai_overage_token_rate_minor BIGINT NOT NULL,                            -- per-1K-tokens overage rate
  storage_allowance_gb_monthly REAL,
  storage_overage_rate_minor BIGINT,                                      -- per-GB overage
  is_active BOOLEAN NOT NULL DEFAULT true,
  effective_from DATE NOT NULL,
  superseded_by UUID REFERENCES bill.plan_tier(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Per-tenant subscription.
CREATE TABLE bill.subscription (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL UNIQUE REFERENCES cyberos_meta.tenant(id),
  plan_tier_id UUID NOT NULL REFERENCES bill.plan_tier(id),
  billing_cadence TEXT NOT NULL DEFAULT 'monthly',                        -- "monthly" | "annual"
  billing_currency TEXT NOT NULL,
  current_seats INT NOT NULL DEFAULT 1,
  status TEXT NOT NULL DEFAULT 'trial',                                    -- "trial" | "active" | "past_due"
                                                                         -- | "suspended" | "cancelled" | "ended"
  trial_ends_at TIMESTAMPTZ,
  current_period_start DATE NOT NULL,
  current_period_end DATE NOT NULL,
  next_billing_date DATE,
  payment_provider TEXT,                                                   -- "stripe" | "vnpay" | "manual_wire"
  payment_provider_subscription_id TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Plan-change history.
CREATE TABLE bill.subscription_change (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  subscription_id UUID NOT NULL REFERENCES bill.subscription(id),
  change_kind TEXT NOT NULL,                                              -- "upgrade" | "downgrade" | "seat_increase" | "seat_decrease"
                                                                         -- | "cadence_change" | "plan_change"
  from_tier_id UUID,
  to_tier_id UUID,
  from_seats INT,
  to_seats INT,
  proration_amount_minor BIGINT NOT NULL,                                  -- positive = customer owes; negative = credit
  proration_period_days INT NOT NULL,
  effective_at TIMESTAMPTZ NOT NULL,
  initiated_by_member_id UUID NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Per-tenant usage tracking (separate from per-call ai_call to keep this hot table small).
CREATE TABLE bill.usage_period (
  tenant_id UUID NOT NULL,
  period_month TEXT NOT NULL,                                              -- "2027-12"
  seat_count_avg REAL NOT NULL,
  ai_tokens_consumed BIGINT NOT NULL DEFAULT 0,
  ai_tokens_allowance BIGINT NOT NULL,
  ai_tokens_overage BIGINT NOT NULL DEFAULT 0,
  ai_overage_charge_minor BIGINT NOT NULL DEFAULT 0,
  storage_gb_avg REAL,
  storage_overage_charge_minor BIGINT NOT NULL DEFAULT 0,
  total_charges_minor BIGINT NOT NULL,
  invoice_id UUID,                                                          -- references inv.invoice when invoiced
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, period_month)
);
```

**Plan tier seed.**

| Tier | Seats | Monthly per-seat | Annual per-seat | AI tokens/seat/month | Storage/seat |
|---|---|---|---|---|---|
| **T1 Starter** | 5 max | $29 USD / 580K VND | $290 / 5.8M | 100K | 1 GB |
| **T2 Growth** | 50 max | $79 / 1.6M | $790 / 16M | 500K | 5 GB |
| **T3 Enterprise** | unlimited | $149 / 3M | $1,490 / 30M | 2M | 25 GB |

Pricing reviewed annually + parameter-versioned. Vietnamese VND pricing accounts for purchasing-power parity + Vietnamese-domestic-customer affordability.

**Per-tier feature flags.**

T1 features: core platform (BRAIN + CHAT + EMAIL + PROJ + KB), 3 CUO C-skills (CEO + COO + CTO), basic OBS, Stripe-only payment, no custom domain, Powered-by footer.
T2 adds: HR + REW + LEARN + ESOP + INV + RES + OKR; 6 CUO C-skills (+ CRO + CFO + CHRO); KB/360 advanced features; custom domain.
T3 adds: white-label option (no Powered-by footer); QES e-signature tier; emergent C-skills (CAIO + CXO + CSO-Sus + CLO); per-tenant database isolation upgrade option; dedicated support.

**Metered AI usage.**

- Each AI Gateway call (FR-AI-001) writes per-tenant token consumption to `cyberos_meta.ai_call`.
- A daily roll-up aggregates into `bill.usage_period.ai_tokens_consumed`.
- When consumption exceeds `ai_tokens_allowance × current_seats`:
  - At 80% allowance: Notify card to tenant admin "approaching AI quota; consider upgrading or scaling-back".
  - At 100% allowance: overage starts; per-1K-token rate applies; admin sees a banner.
  - At 110% allowance: hard cap on Sonnet calls; gateway routes Haiku-only for the rest of the period.
  - At 200% allowance: hard cap on all AI; tenant must upgrade or wait next period.
- Tenant admin can configure per-tier overage policy: "always allow overage" or "hard cap at allowance".

**Subscription billing flow.**

For Stripe (international tenants):

1. Tenant admin selects plan tier + billing cadence at provisioning (FR-TEN-002 §"Provisioning flow").
2. Stripe Checkout session creates the subscription with the tenant's Stripe customer.
3. The subscription's webhook events (`invoice.payment_succeeded`, `invoice.payment_failed`, `customer.subscription.updated`) flow to FR-INV-003's webhook receiver + create per-period `bill.usage_period` records linked to `inv.invoice`.
4. Each billing-period close: usage is computed; an `inv.invoice` is generated (per FR-INV-001) with the subscription charge + overage charges; Stripe charges the customer's card.

For VNPay (Vietnamese tenants):

1. VNPay doesn't have native subscription support; the platform implements subscription on top: each period generates a fresh VNPay payment link; the tenant admin pays via QR code or bank redirect; the payment confirms the next period's access.
2. Optionally: T2/T3 Vietnamese tenants can elect annual prepayment via bank wire (manual reconciliation per FR-INV-003 bank-statement reconciler).

For T3 enterprise tenants (regardless of currency): annual contracts via signed envelope (FR-DOC-001) + manual wire transfer + per-anniversary renewal flow.

**Billing-failure → suspension.**

When Stripe reports `invoice.payment_failed` (3 attempts):
1. After 1st failure: Notify card to tenant admin + payment-method-update CTA + 3-day grace before retry.
2. After 2nd failure: 4-day grace; tenant admin email reminders daily.
3. After 3rd failure: subscription status `→ past_due`; 3-day grace before suspension.
4. Suspension: FR-TEN-002 lifecycle transitions tenant → `suspended`; Notify card explains; payment-method-update + retry path remains open.
5. Resumption: payment succeeds → tenant unsuspends; the missed period's invoice still owed.

**Self-service plan changes.**

Tenant admin at `/tenant/admin/billing` can:
- Upgrade plan: takes effect at next billing period (immediate effect with prorated charge for the rest of the current period).
- Downgrade plan: takes effect at next billing period (no proration credit; existing-period charges remain).
- Add/remove seats: prorated charge (or credit if removing) within the current period.
- Switch cadence (monthly ↔ annual): annual switch typically gives a discount; cadence change effective at next renewal.
- Cancel: subscription continues to period end; tenant is moved to `archive_pending` lifecycle state at period end (FR-TEN-002).

All plan changes write `bill.subscription_change` rows + audit rows.

**Frontend at `/tenant/admin/billing`.**

For tenant admins:
- Current plan + seats + next-billing-date.
- This-period usage (AI tokens consumed vs. allowance + storage + seats).
- Plan-comparison + upgrade CTAs.
- Payment method update.
- Invoice history (downloadable PDFs + e-invoice XML for Vietnamese tenants).
- Cancel-subscription action (with FR-TEN-002 archive flow explained).

**MCP tool surface.**

- `cyberos.bill.my_subscription_status` — read; tenant admin.
- `cyberos.bill.my_usage_current_period` — read.
- `cyberos.bill.list_my_invoices(since?)` — read.
- `cyberos.bill.list_plan_tiers` — read; everyone.
- `cyberos.bill.upgrade_plan(to_tier_code, cadence?)` — `destructive: true; requires_confirmation: true; sensitivity: medium` (changes billing).
- `cyberos.bill.adjust_seats(new_seat_count)` — `destructive: true; requires_confirmation: true`.
- `cyberos.bill.cancel_subscription(reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.

Mutation tools require tenant-admin role. The platform-side billing-administration (the founder of CyberSkill granting custom plans for partners or strategic tenants) is UI-only at `/admin/tenants/<id>/billing` for the platform team.

# Alternatives Considered

- **Build subscription on top of Stripe Tax + Stripe Billing only.** Considered + accepted for international tenants; for VN tenants, Stripe Tax doesn't handle Vietnamese e-invoice format well, so VNPay + manual e-invoice generation via FR-INV-001 templates is the floor.
- **Skip metered AI usage.** Rejected: tenant cost variance is too high; metering is the floor.
- **Auto-charge overages without warning.** Rejected: surprise bills damage trust; the 80%/100%/110% Notify ladder is the floor.
- **One-size-fits-all plan tier.** Rejected: T1/T2/T3 segmentation matches the customer landscape.

# Sales/CS Summary

CyberOS bills per-seat per-month with an annual discount. Three plans: Starter (small teams), Growth (mid-market), Enterprise (white-label + dedicated support). AI usage is included up to a per-seat allowance; predictable overage rates apply if you exceed. Customers in Vietnam pay via VNPay in VND; international customers via Stripe in USD. Plan changes happen self-service at any time. If billing fails we'll Notify before suspending — never silently cut off your team. Cancel at any time; your data archives for 90 days for export, then deletes irrevocably with cryptographic erasure.

# Success Metrics

- **Primary metric.** P3 → P4 exit-gate progress: ≥ 1 paying external tenant on each tier; subscription cycle runs end-to-end via Stripe + VNPay; metered usage accurately tracked + invoiced.
- **Cash-flow metric.** Day-Sales-Outstanding on subscription invoices ≤ 7 days p95 (auto-charge via card).
- **Plan-change latency.** Self-service plan upgrade takes effect within 30 seconds of confirmation.

# Scope

**In-scope.**
- The 4 schema additions (`plan_tier`, `subscription`, `subscription_change`, `usage_period`).
- Plan-tier seed (T1/T2/T3 with parameter-version sign chain).
- Stripe subscription integration (re-using FR-INV-003 webhook receiver).
- VNPay subscription pattern (payment-link-per-period).
- Metered AI usage with 80%/100%/110%/200% Notify + cap ladder.
- Self-service plan changes with proration.
- Billing-failure → suspension with grace period.
- Tenant-admin billing portal.
- Platform-admin billing oversight.
- The 7 MCP tools.
- Audit integration in scope `bill.{tenant}`.

**Out-of-scope (deferred).**
- Volume discounts (P4).
- Per-feature add-on packages (P4).
- Marketplace + partner-led billing (P4+).
- Trial-to-paid conversion analytics (P4).
- Subscription pause (P4).

# Dependencies

- FR-TEN-001 / FR-TEN-002.
- FR-INV-001 / FR-INV-003 (invoicing + payment integrations).
- FR-AI-001 (per-tenant cost accounting in `cyberos_meta.ai_call`).
- FR-AUTH-001 / FR-MCP-001.
- FR-DOC-001 (T3 enterprise contracts).
- Stripe Tax + Stripe Billing subscriptions configured per-platform-account.
- VNPay merchant account (per FR-INV-003).
- Compliance: Vietnamese tax law (VAT on subscription billing); GDPR (per-tenant payment-data residency); PCI DSS (handled by Stripe + VNPay).
- Locked decisions referenced: DEC-282 (3 plan tiers T1/T2/T3), DEC-283 (metered AI usage with overage ladder), DEC-284 (billing-failure → 3-strike suspension flow).

# AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Billing is deterministic.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
