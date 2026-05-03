---
title: "INV — Stripe + VNPay + Wise integrations; payment-link generation; webhook reconciliation; AR/AP automation"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P2 / 2027-Q3"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire payment-provider integrations: **Stripe** for international clients (USD / EUR / SGD / GBP), **VNPay** for Vietnamese-domestic VND payments, **Wise** for cross-border wires + multi-currency receivables; **payment-link generation** per outbound invoice (the client clicks a per-invoice URL → pays → status auto-transitions); **webhook receivers** with HMAC verification + idempotency; **bank-statement reconciliation** for non-online channels (Vietnamese bank exports — Vietcombank, Techcombank, ACB, BIDV); **payment-entry auto-creation** from confirmed payments; **dispute + refund handling** routed via FR-INV-002's lifecycle. Provider credentials live in `inv_secure` (FR-INV-001 §"Schema (`inv_secure`)") under the per-tenant KMS key. **Never moves money on the user's behalf without explicit human action** — payment links are generated for the client to pay; outbound vendor payments are initiated by HR/Ops Lead through the bank's portal manually (banks lack reliable APIs in 2026); the platform tracks + reconciles, but does not initiate AP transfers.

## Problem

Without payment-integration, every outbound invoice's payment is reconciled by hand from the bank statement; the median latency from "client paid" to "marked paid in our system" is currently 2-3 days. Three failure modes:

- **Slow paid-marking.** Marking invoices paid late distorts AR aging + cash forecast.
- **Manual reconciliation errors.** A wire memo that doesn't match an invoice number leaves the payment unmatched + flags as suspicious.
- **No client-side ease of payment.** International clients pay via wire transfer because we don't offer a click-to-pay option; wire transfer is slow + costly + painful for the client.

PRD §9.16 names "Stripe / Wise / VND-PSP integration" + PRD §14.3.1 P2 scope explicitly: "INV module — invoice issuance, AR aging, dunning email drafts, Stripe + VNPay integration." This FR ships the integrations.

## Proposed Solution

The shape of the answer is a `cyberos-inv-payments` service + per-provider adapter modules + webhook receivers + the bank-statement reconciler.

**Provider catalogue.**

| Provider | Direction | Currencies | Use case |
|---|---|---|---|
| **Stripe** | Inbound (AR) | USD, EUR, GBP, SGD, AUD | International clients; click-to-pay via Stripe Checkout |
| **VNPay** | Inbound (AR) | VND | Vietnamese clients; QR + bank-redirect |
| **Wise** | Inbound (AR) + Outbound (AP cross-border) | Multi-currency | International AR (alternative to Stripe for clients preferring wire); cross-border vendor wire (Founder + HR/Ops manual initiation) |
| **Bank exports** | Both (reconciliation only) | VND | Vietcombank/Techcombank/ACB/BIDV statement CSV import |

**Provider adapter pattern.**

Each provider implements `PaymentProvider`:
```rust
trait PaymentProvider {
    async fn create_payment_link(&self, invoice: &Invoice, return_url: &str) -> Result<PaymentLink>;
    async fn verify_webhook(&self, payload: &[u8], signature: &str) -> Result<Webhook>;
    async fn process_webhook(&self, webhook: &Webhook) -> Result<PaymentEvent>;
    async fn refund(&self, charge_id: &str, amount_minor: i64, reason: &str) -> Result<Refund>;
    async fn list_recent_charges(&self, since: DateTime<Utc>) -> Result<Vec<Charge>>;
}
```

**Stripe adapter.**

- **Setup.** A per-tenant Stripe account; the API key + webhook secret stored in `inv_secure.payment_provider_credential` under the tenant's `inv_secure` KMS key.
- **Payment-link creation.** When an outbound invoice is sent (FR-INV-002 `invSendInvoice`) and the client account has `metadata.preferred_payment_provider: "stripe"` or the invoice currency is non-VND, the platform calls `POST /v1/checkout/sessions` with: amount, currency, invoice metadata (the invoice ID + tenant ID + a HMAC for verification), success/cancel URLs. Returns a payment URL embedded in the invoice email + the invoice PDF.
- **Webhook receiver** at `https://payments.cyberos.world/webhook/stripe/{tenant-slug}`. On `checkout.session.completed` or `payment_intent.succeeded`, the receiver verifies the signature, looks up the invoice by metadata, creates a `payment_entry` with `direction: ar_received`, transitions invoice status, fires Notify card to the founder.
- **Refund.** A founder + DPO + step-up authorised refund flows through `POST /v1/refunds`; the refund creates a negative `payment_entry`.
- **Disputes.** A `charge.dispute.created` webhook auto-transitions the invoice to `disputed`; CUO/CFO drafts the dispute response (read-only AI; human reviews + sends).

**VNPay adapter.**

- **Setup.** Per-tenant VNPay merchant account; the merchant ID + secret key stored in `inv_secure.payment_provider_credential`.
- **Payment-link creation.** VNPay's API generates a payment URL with QR code; integrated banks: Vietcombank, Techcombank, ACB, BIDV, Sacombank, MB Bank, others.
- **Webhook receiver** at `https://payments.cyberos.world/webhook/vnpay/{tenant-slug}`. On confirmation, same flow as Stripe.
- **Vietnamese-locale** payment page (the client sees Vietnamese-language flow if their CRM contact's `language_default` is vi-VN; otherwise English).

**Wise adapter.**

- **Setup.** Per-tenant Wise Business account; the Wise API token stored in `inv_secure.payment_provider_credential`.
- **Payment-link creation.** Wise's "Request Money" API generates a request the client can pay with a Wise account or via SWIFT-style wire to the Wise virtual account.
- **Outbound (AP).** Used by Founder + HR/Ops Lead for cross-border vendor payments. The platform doesn't auto-initiate; the human reviews + initiates from Wise's web portal; the platform records the initiation as an `inv.payment_entry` with `direction: ap_sent`, links to the Wise transfer ID, marks `reconciled_at` when Wise's webhook confirms settlement.

**Bank-statement reconciliation.**

For VND bank transfers (the largest channel for Vietnamese clients):

1. HR/Ops Lead exports the bank statement CSV from Vietcombank / Techcombank / ACB / BIDV iBank monthly (or weekly during high-volume periods).
2. Uploads via `/inv/admin/reconcile` (FR-INV-004).
3. The reconciler service:
   - Parses each row.
   - Matches against open invoices: by exact-amount match within ±0.5%, then by transfer-memo invoice-number-pattern match, then by sender-name-to-CRM-account match.
   - High-confidence matches auto-create `payment_entry` with `direction: ar_received`, `reconciliation_method: auto_match`.
   - Low-confidence matches surface as Notify cards for HR/Ops Lead manual confirmation.
   - Unmatched rows are flagged for review.
4. Audit row per match + per manual confirmation.

**Webhook security.**

Every webhook receiver:
- HMAC verification using the provider-specific algorithm (Stripe: HMAC-SHA256 with timestamp; VNPay: HMAC-SHA512; Wise: HMAC-SHA256).
- Idempotency by event ID — duplicate events are deduped against `inv.webhook_event_log`.
- Network policy: only the provider's documented IP ranges can reach the receiver endpoint (Cloudflare WAF rule).
- Audit row per webhook received (success or failure).

**Schema additions.**

```sql
CREATE TABLE inv.webhook_event_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  provider TEXT NOT NULL,
  event_id TEXT NOT NULL,                                             -- provider's event ID; unique
  event_kind TEXT NOT NULL,
  payload JSONB NOT NULL,
  received_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  processed_at TIMESTAMPTZ,
  processing_status TEXT NOT NULL DEFAULT 'pending',                   -- "pending" | "processed" | "failed" | "duplicate"
  related_invoice_id UUID,
  related_payment_entry_id UUID,
  error_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, provider, event_id)
);

CREATE TABLE inv.bank_statement_import (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  bank TEXT NOT NULL,                                                  -- "vietcombank" | "techcombank" | "acb" | "bidv" | "sacombank"
  account_last4 TEXT NOT NULL,
  period_start DATE NOT NULL,
  period_end DATE NOT NULL,
  total_rows INT NOT NULL,
  matched_rows INT NOT NULL DEFAULT 0,
  manually_resolved_rows INT NOT NULL DEFAULT 0,
  unmatched_rows INT NOT NULL DEFAULT 0,
  uploaded_by UUID NOT NULL,
  uploaded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  status TEXT NOT NULL DEFAULT 'processing',                            -- "processing" | "needs_review" | "complete"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**Payment-link UX.**

The outbound invoice email + PDF includes:
- A bold "Pay online" button → the per-provider payment URL.
- For multi-currency clients, the platform offers Stripe (card) + Wise (wire) options.
- For Vietnamese clients, VNPay (QR + bank redirect) + manual bank transfer (with account number + memo).

The links are short (e.g. `https://pay.cyberos.world/i/<short-token>`) for inclusion in invoice PDFs + emails; the short URL redirects to the full Stripe/VNPay/Wise checkout.

**MCP tool surface (read-only — money movement is human-only).**

- `cyberos.inv.list_payment_provider_status` — read; HR/Ops + Founder; provider health.
- `cyberos.inv.get_payment_link(invoice_id)` — read.
- `cyberos.inv.list_recent_webhooks(provider?, since)` — read.
- `cyberos.inv.list_unmatched_payments` — read.
- `cyberos.inv.match_payment(payment_id, invoice_id)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`.
- `cyberos.inv.unmatch_payment(payment_id, reason)` — `destructive: true; requires_confirmation: true; sensitivity: medium`.
- `cyberos.inv.import_bank_statement(file_blob_id, bank, period)` — `destructive: false; sensitivity: medium`.

CUO scope contracts: read all + match-suggestion (AI-suggested via reconciler heuristic) allowed; commit-mutations forbidden — payment-entry creation is human + step-up.

## Alternatives Considered

- **Auto-initiate AP transfers via Stripe / Wise APIs.** Rejected: Vietnamese banks lack reliable payment-out APIs; Stripe + Wise's payouts to vendors require human-in-the-loop fund management; the architectural rule "platform never moves money on user's behalf" applies.
- **Skip VNPay; only Stripe.** Rejected: ~70% of the team's clients are Vietnamese; VNPay is the natural channel.
- **Polling-based reconciliation instead of webhooks.** Rejected: webhook-driven is real-time; polling adds latency.
- **Single payment-link domain (no per-tenant subdomain).** Accepted — `pay.cyberos.world` shared with tenant routing via short-token; per-tenant subdomain not needed for the payment-link surface.

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate progress: ≥ 80% of outbound invoices in the period get paid via payment-link (not manual wire) for non-Vietnamese clients; ≥ 60% for Vietnamese clients (slower adoption due to bank-transfer culture).
- **Latency metric.** Time from "client pays" to "invoice marked paid in CyberOS" ≤ 60 seconds p95 (vs. 2-3 days baseline).
- **Reconciliation accuracy.** Auto-match rate ≥ 90% on bank-statement imports for known clients with consistent memo patterns.

## Scope

**In-scope.**
- Stripe adapter (international AR).
- VNPay adapter (Vietnamese AR).
- Wise adapter (cross-border AR + AP tracking).
- Bank-statement reconciler (Vietcombank / Techcombank / ACB / BIDV / Sacombank).
- Webhook receivers with HMAC + idempotency + WAF.
- Payment-link generation per outbound invoice.
- Auto-paid status transitions on confirmed webhooks.
- Refund + dispute handling.
- The 2 schema additions (`webhook_event_log`, `bank_statement_import`).
- The 7 MCP tools.
- Audit integration in scope `inv.payment.{tenant}`.

**Out-of-scope (deferred).**
- Auto-initiated AP transfers (P3 if Vietnamese banks publish reliable APIs).
- Subscription billing for recurring revenue (P3 — not the team's current model).
- Multi-tenant payment-aggregator reseller pattern (P4).
- Payment-method-on-file for repeat clients (P3 — currently every invoice generates a fresh link).

## Dependencies

- FR-INV-001 / FR-INV-002.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-EMAIL-001..010 (invoice email send path).
- FR-CRM-001 / FR-CRM-002 (account preferred-payment-provider metadata).
- HashiCorp Vault for `inv_secure` per-tenant KMS key.
- Stripe + VNPay + Wise merchant accounts per tenant (CyberSkill in P2).
- Cloudflare WAF rules for webhook IP allowlisting.
- Compliance: PCI DSS (Stripe + VNPay handle card data; the platform doesn't store card data — PCI scope minimised); PDPL Decree 13 (payment data is personal data); GDPR; ISO 27001 Annex A.
- Locked decisions referenced: DEC-233 (Stripe + VNPay + Wise as the 3 providers in P2), DEC-234 (no auto-AP transfers; human-initiated), DEC-235 (bank-statement reconciler covers 5 major Vietnamese banks).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Payment integration is deterministic + webhook-driven. The reconciler's heuristic matching is rule-based, not AI.
