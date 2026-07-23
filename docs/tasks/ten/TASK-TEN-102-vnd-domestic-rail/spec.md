---
id: TASK-TEN-102
title: "VND domestic billing rail — VnPay + Momo + ZaloPay subscription, recurring-charge, refund, dunning + per-PSP webhook bridge for vn-1 tenants"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: ten
priority: p0
status: draft
verify: T
phase: P3
milestone: P3 · billing-substrate-vn
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-001, TASK-TEN-002, TASK-TEN-003, TASK-TEN-004, TASK-TEN-101, TASK-TEN-103, TASK-TEN-104, TASK-INV-005, TASK-INV-006, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-007, TASK-OBS-008]
depends_on: [TASK-TEN-003, TASK-INV-005]
blocks: []

source_pages:
  - website/docs/modules/ten.html#vnd-rail
  # VnPay PSP docs
  - https://sandbox.vnpayment.vn/apis/docs/
  # Momo Open API
  - https://developers.momo.vn/v3/docs/payment/api
  # ZaloPay docs
  - https://docs.zalopay.vn/v2/
  # Decree 123 hóa đơn electronic invoicing
  - https://thuvienphapluat.vn/van-ban/Doanh-nghiep/Nghi-dinh-123-2020-ND-CP-quy-dinh-ve-hoa-don-chung-tu-454681.aspx
  # cross-rail data minimisation parity
  - https://gdpr.eu/article-44-transfers-of-personal-data/

source_decisions:
  - DEC-960 2026-05-17 — VND domestic rail = VnPay + Momo + ZaloPay (3 PSPs); vn-1 tenants pick one PSP at signup; one-of-three (no multi-PSP within a single tenant at slice 2)
  - DEC-961 2026-05-17 — Closed enum `vnd_psp` = {vnpay, momo, zalopay}; CI cardinality test asserts 3
  - DEC-962 2026-05-17 — Recurring monthly billing via PSP-native subscription primitive (VnPay's TokenPay, Momo's recurring-token, ZaloPay's wallet-bind) — NOT manual monthly redirects; user authorises once at signup
  - DEC-963 2026-05-17 — Token-bind flow: signup → PSP-hosted token authorisation (~30s redirect+auth+redirect) → PSP returns a payment_token to our redirect_uri → token KMS-encrypted + persisted in `vnd_payment_tokens` table
  - DEC-964 2026-05-17 — Monthly recurring charge at billing_cycle_anchor (same anchor convention as TASK-TEN-003 DEC-788) via PSP-specific API; result handled by per-PSP webhook (TASK-INV-005 extended for Momo + ZaloPay variants)
  - DEC-965 2026-05-17 — Failed payment dunning: 3 retries over 14 days (parallel to TASK-TEN-003 DEC-790 Stripe pattern); after exhaustion → tenant.dunning_state='suspended' + TASK-TEN-104 suspend
  - DEC-966 2026-05-17 — Refunds via per-PSP refund API; CFO-gated like TASK-TEN-003 DEC-791; sev-1 memory audit; max refund = original charge amount
  - DEC-967 2026-05-17 — Overage charges modeled as one-off VND charges per period_close (NOT metered Subscription Items — VND PSPs lack the Stripe-style metered primitive); aggregated overage = one VND POST at period_end
  - DEC-968 2026-05-17 — Decree 123 hóa đơn điện tử (electronic invoice) generation MANDATORY for every successful VND charge — invoice number, taxpayer info, line items, QR-VietQR for verification per Decree 123 §15; integration with VN tax authority's GDP API
  - DEC-969 2026-05-17 — Per-PSP credentials stored in `vnd_psp_credentials` table, KMS-encrypted; rotation handler with 60s overlap (mirrors TASK-INV-005 secret rotation)
  - DEC-970 2026-05-17 — Idempotency on outbound VND PSP calls: idempotency_key = `vnd.<tenant_id>.<operation>.<period_ts_or_ref>`; PSPs vary in idempotency support (VnPay accepts custom ref; Momo uses `requestId`; ZaloPay uses `app_trans_id`) — adapter maps our key to per-PSP shape
  - DEC-971 2026-05-17 — Per-PSP webhook signature verification (per TASK-INV-005 pattern): VnPay HMAC-SHA512 over query params; Momo HMAC-SHA256 over JSON body; ZaloPay HMAC-SHA256 over key1
  - DEC-972 2026-05-17 — Tax handling: VAT 10% inclusive in VND prices per VN tax law; invoice line items show pre-tax + VAT + total separately per Decree 123 §10
  - DEC-973 2026-05-17 — vn-1 tenant CANNOT cross-rail to Stripe (mirrors TASK-TEN-003 DEC-784 — VND tenants are VND-only)
  - DEC-974 2026-05-17 — Founder tenant skip applies (TASK-TEN-003 DEC-805 derivative) — internal-rail tenants bypass VND rail too
  - DEC-975 2026-05-17 — Per-PSP availability: VnPay 99.5% SLO, Momo 99.5%, ZaloPay 99.0% per published PSP SLAs; failover to alternate PSP at tenant-admin's option (slice 3 — slice 2 is one PSP per tenant locked at signup)
  - DEC-976 2026-05-17 — PSP-charge response asynchronous — recurring-charge API returns 200 + `processing`, real outcome lands via webhook within 5 min (per PSP docs); waiting strategy = 5-min Postgres LISTEN/NOTIFY
  - DEC-977 2026-05-17 — Currency lock applies symmetrically to VND (per TASK-TEN-003 DEC-798): tenant cannot switch from VND to USD post-provisioning
  - DEC-978 2026-05-17 — Per-PSP user-facing branding at signup: each PSP shows their logo + brand at the token-auth redirect (no logo whitelabeling at slice 2)
  - DEC-979 2026-05-17 — memory audit kinds: ten.vnd_token_bind_started, ten.vnd_token_bind_completed, ten.vnd_token_bind_failed, ten.vnd_subscription_charged, ten.vnd_subscription_charge_failed, ten.vnd_overage_charged, ten.vnd_refund_issued, ten.vnd_dunning_advanced, ten.tenant_billing_suspended_vnd, ten.vnd_invoice_issued, ten.vnd_token_revoked, ten.vnd_psp_credential_rotated
  - DEC-980 2026-05-17 — Per-PSP API client per residency: vn-1 residency consumes vnd_psp_credentials with `residency='vn-1'` only (cross-residency credential access blocked per TASK-TEN-103 trip-wire)
  - DEC-981 2026-05-17 — Per-tenant billing_contact_phone REQUIRED at vn-1 signup (PSPs use SMS-OTP for token authorisation); TASK-TEN-101 extended to capture for VND tenants
  - DEC-982 2026-05-17 — VND amounts stored as BIGINT minor (đồng — VND has no minor unit so 1 VND = 1 minor unit; task-audit skill rule 11 satisfied)
  - DEC-983 2026-05-17 — Hóa đơn invoice numbering: prefix `CYBOS-` + YYMMDD + sequential 6-digit padded per Decree 123 §10; gap-free; reset annually
  - DEC-984 2026-05-17 — Hóa đơn signing via VN tax authority's eHĐĐT (electronic invoice) signing service; signed XML stored in `vnd_invoices` table; PDF generated on-demand for tenant download
  - DEC-985 2026-05-17 — Tenant-admin can revoke a payment_token via `POST /v1/admin/tenants/{id}/vnd/token/revoke`; subsequent monthly charges fail until new token authorised (dunning state machine handles)
  - DEC-986 2026-05-17 — Per-PSP webhook dispatch via NATS (mirrors TASK-TEN-003 DEC-793): inv layer captures + verifies, NATS publishes to `tenant.<slug>.ten.vnd.<psp>.<event>`, TEN dispatcher consumes
  - PDPL Law 91/2025 Art. 7 (consent — PSP token auth recorded as explicit consent + versioned at vnd_consents table inline within tenant_consents per TASK-TEN-101)
  - VN Decree 123/2020/NĐ-CP (hóa đơn điện tử — every charge produces an invoice)
  - VN Circular 78/2021/TT-BTC (hóa đơn implementation details — line item format, VAT calc, taxpayer info)
  - PCI DSS — out-of-scope at our endpoint (PSP-hosted card data)
  - SBV Circular 39/2014/TT-NHNN (e-payment service provider regulations — defines PSPs we can integrate)

language: rust 1.81
service: cyberos/services/ten/
new_files:
  # per-tenant PSP token + status + KMS-wrapped
  - services/ten/migrations/0018_vnd_payment_tokens.sql
  # per-PSP API keys + rotation
  - services/ten/migrations/0019_vnd_psp_credentials.sql
  # Decree 123 hóa đơn store + signed XML + PDF link
  - services/ten/migrations/0020_vnd_invoices.sql
  # annual gap-free sequence
  - services/ten/migrations/0021_vnd_invoice_sequence.sql
  # inbound webhook dispatch idempotency
  - services/ten/migrations/0022_vnd_event_dispatch_log.sql
  # VND rail entry
  - services/ten/src/billing/vnd/mod.rs
  # VndPsp trait (3 impls)
  - services/ten/src/billing/vnd/psp_trait.rs
  # VnPay adapter
  - services/ten/src/billing/vnd/vnpay.rs
  # Momo adapter
  - services/ten/src/billing/vnd/momo.rs
  # ZaloPay adapter
  - services/ten/src/billing/vnd/zalopay.rs
  # token-bind flow orchestrator
  - services/ten/src/billing/vnd/token_bind.rs
  # monthly recurring charge
  - services/ten/src/billing/vnd/subscription.rs
  # period_close → one-off charge
  - services/ten/src/billing/vnd/overage.rs
  # CFO-gated refund
  - services/ten/src/billing/vnd/refund.rs
  # VND dunning state machine
  - services/ten/src/billing/vnd/dunning.rs
  # NATS consumer for ten.vnd.<psp>.*
  - services/ten/src/billing/vnd/dispatch.rs
  # per-PSP key adapter
  - services/ten/src/billing/vnd/idempotency.rs
  # hóa đơn issuance + eHĐĐT signing
  - services/ten/src/billing/vnd/hoadon.rs
  # annual gap-free numbering
  - services/ten/src/billing/vnd/hoadon_seq.rs
  - services/ten/src/repo/vnd_payment_tokens.rs
  - services/ten/src/repo/vnd_psp_credentials.rs
  - services/ten/src/repo/vnd_invoices.rs
  # 12 memory row builders
  - services/ten/src/audit/vnd_events.rs
  # POST /v1/signup/vnd/token-bind-start
  - services/ten/src/handlers/vnd_token_bind_start.rs
  # GET callback from PSP
  - services/ten/src/handlers/vnd_token_bind_return.rs
  # POST /v1/admin/tenants/{id}/vnd/token/revoke
  - services/ten/src/handlers/vnd_token_revoke.rs
  # POST /v1/admin/tenants/{id}/vnd/refund (CFO)
  - services/ten/src/handlers/vnd_billing_refund.rs
  # GET /v1/admin/tenants/{id}/vnd/invoices/{id}
  - services/ten/src/handlers/vnd_invoice_get.rs
  - services/ten/tests/vnd_psp_enum_cardinality_test.rs
  - services/ten/tests/vnd_token_bind_happy_test.rs
  - services/ten/tests/vnd_token_bind_failure_test.rs
  - services/ten/tests/vnd_subscription_charge_test.rs
  - services/ten/tests/vnd_async_charge_resolution_test.rs
  - services/ten/tests/vnd_overage_one_off_test.rs
  - services/ten/tests/vnd_dunning_state_machine_test.rs
  - services/ten/tests/vnd_refund_cfo_only_test.rs
  - services/ten/tests/vnd_refund_amount_cap_test.rs
  - services/ten/tests/vnd_token_revoke_test.rs
  - services/ten/tests/vnd_cross_rail_rejection_test.rs
  - services/ten/tests/vnd_founder_skip_test.rs
  - services/ten/tests/vnd_hoadon_issuance_test.rs
  - services/ten/tests/vnd_hoadon_seq_gap_free_test.rs
  - services/ten/tests/vnd_psp_signature_verify_test.rs
  - services/ten/tests/vnd_idempotency_key_per_psp_test.rs
  - services/ten/tests/vnd_dispatcher_idempotency_test.rs
  - services/ten/tests/vnd_residency_isolation_test.rs
  - services/ten/tests/vnd_audit_emission_test.rs

modified_files:
  # for VND tenants, push plan_change as charge-side adjustment (TASK-TEN-002 next period)
  - services/ten/src/handlers/plan_change.rs
  # require billing_contact_phone for VND tenants
  - services/ten/src/handlers/tenant_create.rs
  # +reqwest, +hmac, +sha2 (HMAC-SHA512 for VnPay)
  - services/ten/Cargo.toml
  # add momo.rs + zalopay.rs (VnPay covered in TASK-INV-005)
  - services/inv/src/webhook/
  # POST /v1/inv/webhooks/momo/{tenant_slug}
  - services/inv/src/webhook/momo.rs
  # POST /v1/inv/webhooks/zalopay/{tenant_slug}
  - services/inv/src/webhook/zalopay.rs
  # HMAC-SHA256 over JSON body
  - services/inv/src/webhook/momo_signature.rs
  # HMAC-SHA256 over key1
  - services/inv/src/webhook/zalopay_signature.rs
  # branch — VND rail invokes vnd::overage::charge_for_period
  - services/metering/src/handlers/period_close.rs

allowed_tools:
  - file_read: services/ten/**
  - file_read: services/inv/src/webhook/**
  - file_read: services/metering/src/handlers/**
  - file_write: services/ten/{src,tests,migrations}/**
  - file_write: services/inv/src/webhook/{momo,zalopay}*.rs
  - file_write: services/metering/src/handlers/period_close.rs
  - bash: cd services/ten && cargo test vnd
  - bash: cd services/ten && cargo run --bin cyberos-ten -- vnd-psp-status --residency vn-1

disallowed_tools:
  - charge a non-VND tenant via VND rail (per DEC-973 — symmetric to TASK-TEN-003 cross-rail block)
  - charge the founder tenant via VND rail (per DEC-974)
  - skip hóa đơn issuance on any successful charge (per DEC-968 — VN tax law)
  - store PSP API keys in plaintext (KMS-wrapped only per DEC-969)
  - store payment_token in plaintext (KMS-wrapped only per DEC-963)
  - allow over-refund (per DEC-966 — max = original charge amount)
  - skip per-PSP webhook signature verification (per DEC-971)
  - allow hóa đơn sequence gap (per DEC-983 + Decree 123 §10)

effort_hours: 12
subtasks:
  - "0.6h: 0018_vnd_payment_tokens.sql + 0019_vnd_psp_credentials.sql + RLS + KMS columns"
  - "0.5h: 0020_vnd_invoices.sql + 0021_vnd_invoice_sequence.sql (annual gap-free)"
  - "0.3h: 0022_vnd_event_dispatch_log.sql"
  - "0.6h: billing/vnd/psp_trait.rs + adapter scaffold"
  - "1.0h: vnpay.rs adapter (TokenPay + recurring + refund + signature)"
  - "1.0h: momo.rs adapter (recurring-token + signature)"
  - "1.0h: zalopay.rs adapter (wallet-bind + key1 signature)"
  - "0.6h: token_bind.rs orchestrator (signup redirect flow)"
  - "0.6h: subscription.rs (monthly recurring charge dispatcher)"
  - "0.4h: overage.rs (period_close hook → one-off VND charge)"
  - "0.5h: dunning.rs (state machine mirror of TASK-TEN-003)"
  - "0.5h: refund.rs (CFO-gated; per-PSP refund API)"
  - "0.6h: dispatch.rs (NATS consumer for VND webhooks; idempotency)"
  - "1.0h: hoadon.rs + hoadon_seq.rs (Decree 123 invoice + eHĐĐT signing + gap-free numbering)"
  - "0.4h: idempotency.rs (per-PSP key adapter)"
  - "0.6h: handlers — token_bind_start, token_bind_return, token_revoke, billing_refund, invoice_get"
  - "0.5h: audit/vnd_events.rs (12 builders)"
  - "0.4h: inv/src/webhook/momo.rs + zalopay.rs + signature verifiers"
  - "0.4h: wire-up — plan_change.rs + tenant_create.rs + metering period_close"
  - "2.0h: tests — 19 test files covering all PSPs + happy/failure + dunning + refund + hóa đơn + idempotency"
  - "0.5h: integration smoke against PSP sandbox accounts (VnPay/Momo/ZaloPay test env)"

risk_if_skipped: "Without VND rail, vn-1 tenants cannot pay — every signup in Vietnam fails at the payment step (per TASK-TEN-101 §1 #13's 503 placeholder). This kills the entire VN go-to-market (the founder's home market + the largest single-country opportunity in early commercial). Without DEC-968 hóa đơn issuance, every charge violates VN Decree 123 — illegal billing + tax-authority enforcement risk + commercial-licence revocation. Without DEC-971 per-PSP webhook signature verification, attackers forge fake successful payments. Without DEC-963 KMS-wrapped tokens, a DB leak exposes recurring-charge tokens that can drain customer wallets. Without DEC-983 gap-free hóa đơn numbering, tax authority audit fails (Circular 78 §3 mandates gap-free). Without DEC-967 overage modeling, TASK-TEN-004 metering events for vn-1 tenants are stranded (no rail to bill overages). Without DEC-973 cross-rail block, payment-route logic could attempt Stripe on a VND tenant + leak data cross-region. Without DEC-981's billing_contact_phone capture, PSPs cannot SMS-OTP for token auth = signup fails. The 12h effort lands the VN commercial primitive; without it, vn-1 residency exists but cannot bill."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship the VND domestic billing rail at `services/ten/src/billing/vnd/` — token-bind / subscription / overage / refund / dunning across VnPay + Momo + ZaloPay, NATS-bridged per-PSP webhook dispatch from TASK-INV-005 + new INV webhook handlers, Decree 123 hóa đơn issuance with eHĐĐT signing + annual gap-free numbering, and 12 memory audit kinds.

1. **MUST** define the closed `vnd_psp` Postgres enum at migration `0018`: `('vnpay','momo','zalopay')`. CI cardinality test asserts 3. Adding a fourth PSP requires schema migration + DEC entry.

2. **MUST** define `vnd_payment_tokens` table at migration `0018`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, psp vnd_psp NOT NULL, payment_token_kms_blob BYTEA NOT NULL, kms_key_id TEXT NOT NULL, masked_account_hint TEXT, status TEXT NOT NULL CHECK (status IN ('active','revoked','expired')) DEFAULT 'active', bound_at TIMESTAMPTZ NOT NULL DEFAULT now(), revoked_at TIMESTAMPTZ, expires_at TIMESTAMPTZ)`. Partial unique `(tenant_id) WHERE status='active'` — one active token per tenant at slice 2.

3. **MUST** define `vnd_psp_credentials` table at migration `0019`: `(id BIGSERIAL PRIMARY KEY, psp vnd_psp NOT NULL, residency residency NOT NULL CHECK (residency='vn-1'), credentials_kms_blob BYTEA NOT NULL, kms_key_id TEXT NOT NULL, status TEXT NOT NULL CHECK (status IN ('active','rotated','revoked')) DEFAULT 'active', created_at TIMESTAMPTZ NOT NULL DEFAULT now(), rotated_at TIMESTAMPTZ)`. Partial unique `(psp) WHERE status='active'` — one active credential per PSP. RLS scoped to vn-1 residency per DEC-980 + task-audit skill §8.1d.

4. **MUST** define `vnd_invoices` table at migration `0020`: `(invoice_number TEXT PRIMARY KEY, tenant_id UUID NOT NULL, charge_ref TEXT NOT NULL, issued_at TIMESTAMPTZ NOT NULL DEFAULT now(), pre_tax_amount_vnd BIGINT NOT NULL, vat_amount_vnd BIGINT NOT NULL, total_amount_vnd BIGINT NOT NULL, line_items JSONB NOT NULL, signed_xml_kms_blob BYTEA, ehoadon_tax_authority_ref TEXT, pdf_s3_key TEXT, status TEXT NOT NULL CHECK (status IN ('issued','signed','cancelled')) DEFAULT 'issued')`. Append-only per task-audit skill rule 12 (cancellation = new compensating invoice).

5. **MUST** define `vnd_invoice_sequence` table at migration `0021`: `(year INT PRIMARY KEY, last_sequence INT NOT NULL DEFAULT 0)` for annual gap-free numbering per DEC-983 + Decree 123 §10. Sequence allocation via `SELECT ... FOR UPDATE` to guarantee no gap, no duplicate.

6. **MUST** define `vnd_event_dispatch_log` table at migration `0022`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, psp vnd_psp NOT NULL, psp_event_id TEXT NOT NULL, event_type TEXT NOT NULL, dispatch_status TEXT NOT NULL CHECK (dispatch_status IN ('dispatched','duplicate','failed')), dispatched_at TIMESTAMPTZ NOT NULL DEFAULT now(), UNIQUE(psp, psp_event_id))`. Inbound webhook idempotency per DEC-986. Append-only.

7. **MUST** enforce RLS with both USING and WITH CHECK on all 5 VND tables, scoped to `tenant_id = current_setting('auth.tenant_id')::uuid` AND `current_setting('auth.residency') = 'vn-1'` (TASK-TEN-103 trip-wire integration).

8. **MUST** define the `VndPsp` trait at `billing/vnd/psp_trait.rs`:
    ```rust
    #[async_trait]
    pub trait VndPsp: Send + Sync {
        async fn token_bind_url(&self, tenant_id: Uuid, return_url: &str, requested_at: DateTime<Utc>) -> Result<String, VndError>;
        async fn token_bind_callback(&self, callback_query: &CallbackQuery) -> Result<PaymentToken, VndError>;
        async fn charge(&self, payment_token: &PaymentToken, amount_vnd: i64, description: &str, idempotency_key: &str) -> Result<ChargeAck, VndError>;
        async fn refund(&self, original_charge_ref: &str, amount_vnd: i64, idempotency_key: &str) -> Result<RefundAck, VndError>;
        async fn revoke_token(&self, payment_token: &PaymentToken) -> Result<(), VndError>;
        fn verify_webhook_signature(&self, headers: &HeaderMap, body: &[u8]) -> Result<(), VndError>;
    }
    ```
Implementations: `VnPayClient`, `MomoClient`, `ZaloPayClient`.

9. **MUST** support the token-bind flow at signup per DEC-963. Flow:
1. UI invokes `POST /v1/signup/vnd/token-bind-start` with `{ signup_session_id, psp }`.
2. Handler resolves the chosen PSP adapter; calls `token_bind_url(tenant_id, return_url, ...)` to get the PSP-hosted authorisation URL.
3. UI redirects user to PSP-hosted page.
4. User authorises (SMS-OTP from billing_contact_phone) on PSP page.
5. PSP redirects back to `/v1/signup/vnd/token-bind-return?session_id=...&signed_params=...`.
6. Handler calls `psp.token_bind_callback(query)` to extract + KMS-encrypt the `payment_token`.
7. INSERT into `vnd_payment_tokens` with status='active'.
8. Emit `ten.vnd_token_bind_completed`. Total signup-flow target: <30s same as TASK-TEN-101 SLO, accounting for ~10s PSP redirect+auth time.

10. **MUST** charge monthly subscription at billing_cycle_anchor per DEC-964. The `billing/vnd/subscription.rs::charge_monthly(tenant_id)` job:
- Lookup `tenants.billing_currency = 'VND'` + active payment_token + current plan_tier.
- Compute VND amount = `PRICE_CATALOG.price_for(VND, plan_tier).amount_minor`.
- Issue PSP charge via `psp.charge(token, amount, description, idempotency_key)` with key `vnd.<tenant_id>.subscription.<period_ts>`.
- PSP responds `200 + processing`; real outcome arrives via webhook in <5min per DEC-976.
- Emit `ten.vnd_subscription_charged` on webhook success; `ten.vnd_subscription_charge_failed` on failure.
- On success: issue hóa đơn (per §1 #16); on failure: advance dunning state.

11. **MUST** charge overage at period_close per DEC-967. The `vnd/overage.rs::charge_for_period(tenant_id, period_end)`:
- Compute aggregate overage = sum of (axis_actual - axis_cap) * per_axis_unit_price across 4 axes.
- If aggregate > 0: invoke `psp.charge(token, overage_vnd, "Overage charges for period {period_end}", idempotency_key)` with key `vnd.<tenant_id>.overage.<period_end_unix>`.
- On success: issue hóa đơn + emit `ten.vnd_overage_charged`.
- 1-hour push window (per TASK-TEN-003 DEC-810); 24h retry deadline.

12. **MUST** advance dunning per DEC-965. Mirrors TASK-TEN-003 §1 #11:
- `ok → retry_1 → retry_2 → retry_3 → suspended` on consecutive `vnd_subscription_charge_failed`.
- Recovery: any successful charge → `dunning_state='ok'` + un-suspend if previously suspended.
- On `suspended`: trigger TASK-TEN-104 tenant suspension + emit `ten.tenant_billing_suspended_vnd` sev-1.

13. **MUST** expose `POST /v1/admin/tenants/{id}/vnd/refund` for CFO-gated refunds per DEC-966. Body: `{ original_charge_ref, amount_vnd, reason }`. Validations:
- Caller has `cfo` role per TASK-AUTH-101.
- `amount_vnd ≤ original_charge_amount`.
- `tenant.billing_currency == 'VND'`.
- Invoke `psp.refund(original_charge_ref, amount_vnd, idempotency_key=vnd.<tenant>.refund.<charge_ref>.<amount>)`.
- On success: issue compensating hóa đơn (Decree 123 mandates refund invoices) + emit `ten.vnd_refund_issued` sev-1.

14. **MUST** expose `POST /v1/admin/tenants/{id}/vnd/token/revoke` per DEC-985. Caller has `tenant_admin` role. Handler:
- Lookup active token.
- Call `psp.revoke_token(token)` (best-effort; PSP-side revoke).
- UPDATE `vnd_payment_tokens.status='revoked'` + `revoked_at=now()`.
- Emit `ten.vnd_token_revoked` sev-2.
- Tenant is now without an active token; next monthly charge fails → dunning → suspension unless new token bound.

15. **MUST** support per-PSP webhook ingestion via TASK-INV-005 extension (modified_files: `services/inv/src/webhook/momo.rs` + `zalopay.rs` + their signature verifiers; VnPay already covered by TASK-INV-005). The INV layer:
- Verifies signature per DEC-971 (VnPay HMAC-SHA512 over query; Momo HMAC-SHA256 over body; ZaloPay HMAC-SHA256 over key1).
- Idempotency dedupe on PSP `event_id`.
- NATS-publish to `tenant.<slug>.ten.vnd.<psp>.<event_type>` per DEC-986. The TEN dispatcher (`billing/vnd/dispatch.rs`) consumes the NATS subject + dispatches into per-event handlers (subscription_charged, charge_failed, refund_completed, token_expired). Dispatch idempotency via `vnd_event_dispatch_log` UNIQUE `(psp, psp_event_id)`.

16. **MUST** issue a Decree 123 hóa đơn for every successful charge per DEC-968 + DEC-972 + DEC-983 + DEC-984. The `vnd/hoadon.rs::issue_invoice(tenant_id, charge_ref, total_vnd)`:
- Allocate next invoice number from `vnd_invoice_sequence` (gap-free, annual reset).
- Compute pre_tax = total / 1.10 + vat = total - pre_tax (10% inclusive VAT per VN tax law).
- Generate Decree 123 XML with line items (tenant taxpayer info pre-collected at signup + per-period subscription line + overage lines).
- Sign via VN tax authority's eHĐĐT API; persist signed XML + tax authority ref in `vnd_invoices`.
- Generate PDF on-demand (S3 cache); store S3 key in row.
- Emit `ten.vnd_invoice_issued` sev-2.

17. **MUST** use annual gap-free invoice numbering per DEC-983 + Decree 123 §10. The `hoadon_seq.rs::next_invoice_number(year)`:
- `SELECT last_sequence FROM vnd_invoice_sequence WHERE year=$1 FOR UPDATE`.
- `UPDATE ... SET last_sequence = last_sequence + 1`.
- Return `format!("CYBOS-{:02}{:02}{:02}-{:06}", year_2_digit, month, day, new_sequence)`.
- On rollback (issue_invoice tx fails after sequence allocated): NUMBER IS LOST. This is intentional — gap-free means "no duplicate", not "no skipped". Skipped numbers are auditable with explanation per Decree 123 §10 (the system logs reason for each skip).

18. **MUST** thread idempotency keys per DEC-970 + task-audit skill §8.3b. Per-PSP key adapter at `billing/vnd/idempotency.rs`:
- VnPay: `vnp_TxnRef` field on request.
- Momo: `requestId` field.
- ZaloPay: `app_trans_id` field. Internal canonical key format: `vnd.<tenant_id>.<operation>.<period_ts_or_ref>`. Adapter maps canonical → per-PSP shape. Stored in `vnd_idempotency_cache` (additional table — sub-migration into 0018).

19. **MUST** emit 12 memory audit row kinds per DEC-979 (task-audit skill rule 6 + §8 namespace):
- `ten.vnd_token_bind_started` (sev-3)
- `ten.vnd_token_bind_completed` (sev-2)
- `ten.vnd_token_bind_failed` (sev-2)
- `ten.vnd_subscription_charged` (sev-2)
- `ten.vnd_subscription_charge_failed` (sev-2)
- `ten.vnd_overage_charged` (sev-3)
- `ten.vnd_refund_issued` (sev-1)
- `ten.vnd_dunning_advanced` (sev-1)
- `ten.tenant_billing_suspended_vnd` (sev-1)
- `ten.vnd_invoice_issued` (sev-2)
- `ten.vnd_token_revoked` (sev-2)
- `ten.vnd_psp_credential_rotated` (sev-1) PII-scrubbed via TASK-MEMORY-111: `billing_contact_phone_hash16`, `masked_account_hint_hash16`.

20. **MUST NOT** charge any tenant whose `billing_currency != 'VND'` via this rail per DEC-973 (symmetric to TASK-TEN-003 §1 #23). Guard at `vnd::api_client::call()` entry — cross-rail attempts return `400 + { error: "wrong_billing_rail", expected: "stripe", got: "vnd" }`.

21. **MUST NOT** charge the founder tenant per DEC-974. Guard mirrors TASK-TEN-003 §1 #22 — emits sev-3 `ten.vnd_founder_skip` informational row.

22. **MUST** lock `billing_currency` immutable post-provisioning per DEC-977 (already enforced by TASK-TEN-003's `trg_billing_currency_immutable` trigger).

23. **MUST** require `billing_contact_phone` at vn-1 signup per DEC-981. TASK-TEN-101's `SignupCompleteReq` body extended (in this task's modified_files) with optional `billing_contact_phone: Option<String>`; validation in `vnd::token_bind::start` requires it for VND tenants (returns 400 + `phone_required_for_vnd` if missing).

24. **MUST** PII-scrub all audit rows per task-audit skill rule 18 + DEC-979. Phone hashed as `phone_hash16 = HMAC(global_salt, phone_e164_form)`; masked account hint (last-4 of bank account) hashed.

25. **MUST** thread W3C `traceparent` end-to-end per task-audit skill rule 22-24. Signup → token-bind → callback → INSERT → audit row chain MUST share trace_id.

26. **MUST** support concurrent token-bind safely per task-audit skill §8.3. Per-tenant `SELECT ... FOR UPDATE` on `vnd_payment_tokens` during INSERT; concurrent binds for same tenant return 409 (active token already exists).

27. **SHOULD** observe per-PSP charge latency p95 via OTel histogram `vnd_charge_duration_seconds_by_psp` per task-audit skill rule 22. Alarm sev-2 if p95 > 10s sustained 10 min (PSP-side issue).

---

## §2 — Why this design (rationale for humans)

**Why three PSPs (§1 #1, DEC-960)?** Vietnamese consumer payment landscape is fragmented. VnPay dominates merchant-side (~60% market share); Momo dominates wallet-based payments (~40% of consumer wallets); ZaloPay is the Zalo-ecosystem default (~30% of social-app users). One PSP loses ~40-70% of potential customers depending on which one. Three covers >95% with reasonable integration cost.

**Why one PSP per tenant at slice 2 (§1 #2 partial-unique, DEC-960)?** Multi-PSP-per-tenant adds combinatorial complexity (which PSP for next monthly charge? failover order? per-PSP credentials?) without immediate commercial benefit (a tenant picks one PSP at signup and rarely switches). Slice 3 adds failover; slice 2 keeps the rail simple.

**Why PSP-native subscription primitives over manual monthly redirects (§1 #10, DEC-962)?** Manual monthly redirects mean the user has to authorise every single month — ~30% drop-off per month per industry benchmarks = catastrophic churn. Token-bind authorises once; subsequent charges are headless. All three PSPs support this primitive (VnPay TokenPay, Momo recurring-token, ZaloPay wallet-bind); leveraging them is the conventional path.

**Why hóa đơn issuance mandatory per charge (§1 #16, DEC-968)?** VN Decree 123 §15 requires electronic invoice for every business-to-business transaction. Non-compliance = tax-authority enforcement + commercial-licence revocation. This is not optional commercial UX; it's legal commercial requirement.

**Why gap-free annual sequence (§1 #17, DEC-983)?** Decree 123 §10 + Circular 78 §3 mandate gap-free invoice numbering for tax authority audit. Gaps indicate hidden transactions = fraud signal. The `FOR UPDATE` allocation pattern + the documented "skip = audit log" approach handles rollback semantics legally.

**Why async PSP responses + Postgres LISTEN/NOTIFY (§1 #10, DEC-976)?** VN PSPs return `200 + processing` immediately; real outcome arrives via webhook within seconds-to-5min. Polling = wasted load; LISTEN/NOTIFY = real-time wake-up when the webhook handler INSERTs. Same pattern as TASK-INV-005's webhook dispatch.

**Why per-PSP webhook signature variance (§1 #15, DEC-971)?** Each PSP designed its signature scheme independently — VnPay HMAC-SHA512 over query params; Momo HMAC-SHA256 over JSON body; ZaloPay HMAC-SHA256 over key1. We can't normalise the upstream; the adapter pattern (`VndPsp::verify_webhook_signature`) hides the variance behind a uniform interface.

**Why dunning state machine parallel to TASK-TEN-003 (§1 #12, DEC-965)?** Operational simplicity. Two rails (Stripe + VND) with identical dunning state machines mean operators learn one model. Per-PSP dunning differences would explode the operator-mental-model surface.

**Why per-PSP credentials with rotation (§1 #3, DEC-969)?** Each PSP issues separate API keys. Rotation per PSP is independent (Momo might force-rotate quarterly; VnPay might do annually). Storing them KMS-wrapped + per-PSP table makes rotation independent + auditable.

**Why billing_contact_phone required for VND (§1 #23, DEC-981)?** VN PSPs require SMS-OTP for token authorisation (regulatory + anti-fraud). Without a phone, the token-bind redirect fails at the PSP side — user can't complete signup. Capturing at signup avoids the deadlock.

**Why cross-rail block (§1 #20, DEC-973)?** Tenant `billing_currency` is locked at provisioning (TASK-TEN-003 DEC-798). A VND tenant attempting Stripe rail = bug, either client-side or server-side. Fail closed at the rail entry; trip-wire catches at schema level too (TASK-TEN-103 §1 #8 cross-residency trigger).

**Why VAT 10% inclusive (§1 #16, DEC-972)?** VN consumer-facing prices conventionally show VAT-inclusive. Showing exclusive is unusual + breaks the user's mental model. Decree 123 mandates line-item breakdown (pre-tax + VAT + total) — the invoice carries the split; the price catalog uses inclusive amounts.

**Why hóa đơn signing via tax authority eHĐĐT (§1 #16, DEC-984)?** Decree 123 §13 requires either (a) sign with tax authority's central service OR (b) sign with our own digital cert pre-registered with tax authority. (a) is operationally simpler + always-up-to-date with regulatory changes; (b) requires our cert lifecycle management. Picked (a) for slice 2; (b) is slice 3 if we hit eHĐĐT availability issues.

---

## §3 — API contract

### 3.1 Postgres schema (key migrations)

```sql
-- 0018_vnd_payment_tokens.sql
CREATE TYPE vnd_psp AS ENUM ('vnpay','momo','zalopay');

CREATE TABLE vnd_payment_tokens (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  psp vnd_psp NOT NULL,
  payment_token_kms_blob BYTEA NOT NULL,
  kms_key_id TEXT NOT NULL,
  masked_account_hint TEXT,
  status TEXT NOT NULL DEFAULT 'active'
    CHECK (status IN ('active','revoked','expired')),
  bound_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked_at TIMESTAMPTZ,
  expires_at TIMESTAMPTZ
);
CREATE UNIQUE INDEX uniq_active_vnd_token ON vnd_payment_tokens(tenant_id) WHERE status='active';
ALTER TABLE vnd_payment_tokens ENABLE ROW LEVEL SECURITY;
CREATE POLICY vnd_payment_tokens_rls ON vnd_payment_tokens
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND current_setting('auth.residency') = 'vn-1')
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND current_setting('auth.residency') = 'vn-1');
REVOKE UPDATE, DELETE ON vnd_payment_tokens FROM cyberos_app;
GRANT UPDATE (status, revoked_at, expires_at) ON vnd_payment_tokens TO cyberos_app;

CREATE TABLE vnd_idempotency_cache (
  canonical_key TEXT PRIMARY KEY,
  tenant_id UUID NOT NULL,
  psp vnd_psp NOT NULL,
  per_psp_key TEXT NOT NULL,
  request_sha256 CHAR(64) NOT NULL,
  response_status INT,
  response_body_sha256 CHAR(64),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  ttl_until TIMESTAMPTZ NOT NULL DEFAULT now() + INTERVAL '7 days'
);

-- 0021_vnd_invoice_sequence.sql
CREATE TABLE vnd_invoice_sequence (
  year INT PRIMARY KEY,
  last_sequence INT NOT NULL DEFAULT 0,
  notes JSONB NOT NULL DEFAULT '[]'::jsonb  -- skipped numbers + reasons per Decree 123 §10
);

-- 0020_vnd_invoices.sql
CREATE TABLE vnd_invoices (
  invoice_number TEXT PRIMARY KEY,
  tenant_id UUID NOT NULL,
  charge_ref TEXT NOT NULL,
  issued_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  pre_tax_amount_vnd BIGINT NOT NULL,
  vat_amount_vnd BIGINT NOT NULL,
  total_amount_vnd BIGINT NOT NULL,
  line_items JSONB NOT NULL,
  signed_xml_kms_blob BYTEA,
  ehoadon_tax_authority_ref TEXT,
  pdf_s3_key TEXT,
  status TEXT NOT NULL DEFAULT 'issued'
    CHECK (status IN ('issued','signed','cancelled'))
);
ALTER TABLE vnd_invoices ENABLE ROW LEVEL SECURITY;
CREATE POLICY vnd_invoices_rls ON vnd_invoices
  USING (tenant_id = current_setting('auth.tenant_id')::uuid
         AND current_setting('auth.residency') = 'vn-1')
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid
              AND current_setting('auth.residency') = 'vn-1');
REVOKE UPDATE, DELETE ON vnd_invoices FROM cyberos_app;
GRANT UPDATE (signed_xml_kms_blob, ehoadon_tax_authority_ref, pdf_s3_key, status)
  ON vnd_invoices TO cyberos_app;
```

### 3.2 Rust types (selected)

```rust
// services/ten/src/billing/vnd/psp_trait.rs
#[async_trait]
pub trait VndPsp: Send + Sync {
    fn psp_kind(&self) -> VndPspKind;
    async fn token_bind_url(&self, ctx: &TokenBindCtx) -> Result<String, VndError>;
    async fn token_bind_callback(&self, query: &CallbackQuery) -> Result<PaymentToken, VndError>;
    async fn charge(&self, token: &PaymentToken, amount_vnd: i64, description: &str, idempotency_key: &str) -> Result<ChargeAck, VndError>;
    async fn refund(&self, original_charge_ref: &str, amount_vnd: i64, idempotency_key: &str) -> Result<RefundAck, VndError>;
    async fn revoke_token(&self, token: &PaymentToken) -> Result<(), VndError>;
    fn verify_webhook_signature(&self, headers: &http::HeaderMap, body: &[u8]) -> Result<(), VndError>;
}

pub struct PaymentToken {
    pub tenant_id: Uuid,
    pub psp: VndPspKind,
    pub raw_token: SecretString,  // KMS-decrypted at use
    pub masked_account_hint: Option<String>,
    pub bound_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

pub enum ChargeAck {
    Processing { psp_request_id: String },  // async resolution via webhook
    Completed  { psp_charge_ref: String },  // synchronous (rare)
}
```

### 3.3 REST endpoints

```text
POST   /v1/signup/vnd/token-bind-start              (public, signup-session-bound)
GET    /v1/signup/vnd/token-bind-return             (PSP redirect callback)
POST   /v1/admin/tenants/{id}/vnd/token/revoke      (tenant_admin)
POST   /v1/admin/tenants/{id}/vnd/refund            (cfo)
GET    /v1/admin/tenants/{id}/vnd/invoices          (tenant_admin or cfo — list)
GET    /v1/admin/tenants/{id}/vnd/invoices/{num}    (tenant_admin or cfo — single, PDF download)
```

---

## §4 — Acceptance criteria

1. **vnd_psp cardinality** — enum = exactly `{vnpay, momo, zalopay}`.
2. **Token-bind happy** — start → PSP redirect → return → token persisted KMS-encrypted; `ten.vnd_token_bind_completed` emitted.
3. **Token-bind failure** — PSP returns failure code → token NOT persisted + `ten.vnd_token_bind_failed` emitted.
4. **Monthly subscription charge** — at billing_cycle_anchor, `subscription::charge_monthly(tenant)` invokes PSP charge; webhook resolves to success → `ten.vnd_subscription_charged` + hóa đơn issued.
5. **Async charge resolution** — PSP responds `processing`; webhook arrives 30s later → handler unblocks via LISTEN/NOTIFY.
6. **Overage one-off** — period_close with overage > 0 triggers single VND charge with idempotency_key matching `vnd.<tid>.overage.<period_end>`.
7. **Dunning state machine** — three consecutive `charge_failed` → `retry_3`; fourth → `suspended` + TASK-TEN-104 suspension.
8. **Refund CFO-only** — non-cfo refund attempt returns 403; cfo returns 201 + sev-1 audit.
9. **Refund amount cap** — refund > original charge returns 400 `refund_exceeds_charge`.
10. **Token revoke** — `POST /vnd/token/revoke` → token.status='revoked' + next charge fails.
11. **Cross-rail rejection** — USD tenant invoking VND rail returns 400 `wrong_billing_rail`.
12. **Founder skip** — founder tenant VND attempt is no-op + emits `ten.vnd_founder_skip` sev-3 (not in 12-kind core).
13. **Hóa đơn issuance** — every successful charge produces a `vnd_invoices` row + signed XML.
14. **Hóa đơn sequence gap-free** — 1000 concurrent issue requests produce 1000 unique invoice numbers with no duplicates (skipped on rollback IS allowed but logged in `notes`).
15. **Per-PSP webhook signature verify** — VnPay HMAC-SHA512 + Momo HMAC-SHA256-body + ZaloPay HMAC-SHA256-key1 all verified correctly; bad sig → 401.
16. **Idempotency key per PSP** — same canonical_key → same per-PSP key; replay request returns cached response.
17. **Dispatcher idempotency** — same `(psp, psp_event_id)` twice → one dispatch + one duplicate log row.
18. **Residency isolation** — non-vn-1 handler cannot read `vnd_payment_tokens` (RLS rejects).
19. **Billing_contact_phone required at vn-1 signup** — VND signup without phone returns 400 `phone_required_for_vnd`.
20. **12 memory audit kinds emitted** — happy flow + failure flow produces all 12 kinds across scenarios.

---

## §5 — Verification

### 5.1 `vnd_psp_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn vnd_psp_has_exactly_3_values() {
    let ctx = TestContext::vn1().await;
    let labels: Vec<String> = sqlx::query_scalar("SELECT unnest(enum_range(NULL::vnd_psp))::text")
        .fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels; labels.sort();
    assert_eq!(labels, vec!["momo","vnpay","zalopay"]);
}
```

### 5.2 `vnd_token_bind_happy_test.rs`

```rust
#[tokio::test]
async fn vnpay_token_bind_completes_and_persists_kms_wrapped() {
    let ctx = TestContext::vn1_with_psp(VndPspKind::Vnpay).await;
    let session = ctx.email_verified_signup_session("alice@vn.example", "+84901234567").await;
    let start = ctx.post("/v1/signup/vnd/token-bind-start").json(&json!({
        "signup_session_id": session, "psp": "vnpay"
    })).send().await.unwrap();
    let body: serde_json::Value = start.json().await.unwrap();
    let redirect_url = body["redirect_url"].as_str().unwrap();
    let return_query = ctx.simulate_vnpay_user_auth(redirect_url, "ok").await;
    let r = ctx.get(&format!("/v1/signup/vnd/token-bind-return?{}", return_query)).send().await.unwrap();
    assert_eq!(r.status(), 200);
    let row = sqlx::query("SELECT psp::text, status, payment_token_kms_blob FROM vnd_payment_tokens WHERE tenant_id=$1")
        .bind(ctx.tenant_for_session(session).await)
        .fetch_one(&ctx.pool).await.unwrap();
    let blob: Vec<u8> = row.get("payment_token_kms_blob");
    assert!(blob.len() > 0);
    assert!(!std::str::from_utf8(&blob).map(|s| s.contains("token=")).unwrap_or(false), "plaintext leak");
    assert_eq!(row.get::<String,_>("status"), "active");

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "ten.vnd_token_bind_completed"));
}
```

### 5.3 `vnd_subscription_charge_test.rs`

```rust
#[tokio::test]
async fn monthly_charge_emits_audit_and_invoice() {
    let ctx = TestContext::vn1_with_psp(VndPspKind::Momo).await;
    let tenant = ctx.provision_vnd_tenant_with_token("acme-vn", "+84901234567").await;
    let _ = ctx.run_subscription_charge(tenant, PlanTier::Team).await;
    ctx.simulate_psp_webhook_success(tenant, VndPspKind::Momo).await;

    let charges: Vec<String> = sqlx::query_scalar(
        "SELECT kind FROM memory_rows WHERE tenant_id=$1 AND kind LIKE 'ten.vnd_%'"
    ).bind(tenant).fetch_all(&ctx.pool).await.unwrap();
    assert!(charges.iter().any(|k| k == "ten.vnd_subscription_charged"));
    assert!(charges.iter().any(|k| k == "ten.vnd_invoice_issued"));

    let invoice_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM vnd_invoices WHERE tenant_id=$1"
    ).bind(tenant).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(invoice_count, 1);
}
```

### 5.4 `vnd_async_charge_resolution_test.rs`

```rust
#[tokio::test]
async fn async_charge_resolves_via_listen_notify() {
    let ctx = TestContext::vn1_with_psp(VndPspKind::Zalopay).await;
    let tenant = ctx.provision_vnd_tenant_with_token("zlp-tenant", "+84901234567").await;
    let charge_fut = tokio::spawn(ctx.charge_with_wait(tenant));
    tokio::time::sleep(Duration::from_millis(500)).await;
    ctx.simulate_psp_webhook_success(tenant, VndPspKind::Zalopay).await;
    let result = charge_fut.await.unwrap();
    assert!(result.is_ok());
}
```

### 5.5 `vnd_dunning_state_machine_test.rs`

```rust
#[tokio::test]
async fn vnd_dunning_advances_then_suspends() {
    let ctx = TestContext::vn1_with_psp(VndPspKind::Vnpay).await;
    let tenant = ctx.provision_vnd_tenant_with_token("dunning-test", "+84901234567").await;
    for expected in &[DunningState::Retry1, DunningState::Retry2, DunningState::Retry3] {
        ctx.simulate_psp_webhook_failure(tenant, VndPspKind::Vnpay).await;
        assert_eq!(ctx.load_dunning(tenant).await, *expected);
    }
    ctx.simulate_psp_webhook_failure(tenant, VndPspKind::Vnpay).await;
    assert_eq!(ctx.load_dunning(tenant).await, DunningState::Suspended);

    let status: String = sqlx::query_scalar("SELECT status::text FROM tenants WHERE id=$1")
        .bind(tenant).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(status, "suspended");
}
```

### 5.6 `vnd_hoadon_seq_gap_free_test.rs`

```rust
#[tokio::test]
async fn invoice_sequence_no_duplicates_under_concurrency() {
    let ctx = TestContext::vn1_with_psp(VndPspKind::Vnpay).await;
    let tenants: Vec<_> = (0..100).map(|i| async move {
        ctx.provision_vnd_tenant(&format!("conc-{i}"), "+84901234567").await
    }).collect::<futures::stream::FuturesUnordered<_>>().collect::<Vec<_>>().await;
    let invoices = futures::future::join_all(tenants.into_iter().map(|t| ctx.issue_invoice(t, 100_000))).await;
    let unique: std::collections::HashSet<_> = invoices.iter().filter_map(|r| r.as_ref().ok()).cloned().collect();
    assert_eq!(unique.len(), invoices.iter().filter(|r| r.is_ok()).count(), "duplicate invoice number issued");
}
```

### 5.7 `vnd_refund_cfo_only_test.rs`

```rust
#[tokio::test]
async fn vnd_refund_requires_cfo_role() {
    let ctx = TestContext::vn1_with_psp(VndPspKind::Momo).await;
    let tenant = ctx.provision_vnd_tenant_with_charge("refund-test").await;
    let admin_token = ctx.mint_jwt(tenant, "tenant_admin");
    let r = ctx.post(&format!("/v1/admin/tenants/{tenant}/vnd/refund")).bearer_auth(admin_token)
        .json(&json!({"original_charge_ref": "ch_1", "amount_vnd": 100_000, "reason": "duplicate"}))
        .send().await.unwrap();
    assert_eq!(r.status(), 403);

    let cfo_token = ctx.mint_jwt(tenant, "chief-financial-officer");
    let r = ctx.post(&format!("/v1/admin/tenants/{tenant}/vnd/refund")).bearer_auth(cfo_token)
        .json(&json!({"original_charge_ref": "ch_1", "amount_vnd": 100_000, "reason": "duplicate"}))
        .send().await.unwrap();
    assert_eq!(r.status(), 201);
}
```

### 5.8 `vnd_cross_rail_rejection_test.rs`

```rust
#[tokio::test]
async fn usd_tenant_blocked_from_vnd_rail() {
    let ctx = TestContext::vn1().await;
    let us_tenant = ctx.provision_us_tenant("usd-tenant").await;
    let r = ctx.attempt_vnd_charge(us_tenant).await;
    assert!(matches!(r, Err(VndError::WrongBillingRail { .. })));
}
```

### 5.9 `vnd_psp_signature_verify_test.rs`

```rust
#[tokio::test]
async fn vnpay_signature_validates() {
    let secret = b"vnp_secret_key";
    let body = b"vnp_Amount=10000&vnp_TxnRef=abc";
    let mut mac = Hmac::<Sha512>::new_from_slice(secret).unwrap();
    mac.update(body);
    let sig = hex::encode(mac.finalize().into_bytes());
    let mut headers = HeaderMap::new();
    headers.insert("vnp-SecureHash", sig.parse().unwrap());
    let vnp = VnPayClient::with_secret(secret);
    assert!(vnp.verify_webhook_signature(&headers, body).is_ok());

    // tampered body
    let bad_body = b"vnp_Amount=999999&vnp_TxnRef=abc";
    assert!(vnp.verify_webhook_signature(&headers, bad_body).is_err());
}
```

### 5.10 `vnd_residency_isolation_test.rs`

```rust
#[tokio::test]
async fn non_vn1_handler_cannot_read_vnd_tables() {
    let ctx = TestContext::with_all_residencies().await;
    let vn_tenant = ctx.provision_vnd_tenant("acme-vn", "+84901234567").await;
    let eu_handler = ctx.handler_ctx_in(Residency::Eu1);
    let rows: Vec<(String,)> = sqlx::query_as("SELECT id::text FROM vnd_payment_tokens WHERE tenant_id=$1")
        .bind(vn_tenant).fetch_all(eu_handler.pool()).await.unwrap_or_default();
    assert_eq!(rows.len(), 0, "RLS should block cross-residency read");
}
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton; selected helpers below.)

### 6.1 Hóa đơn sequence allocation (gap-free)

```rust
// services/ten/src/billing/vnd/hoadon_seq.rs
pub async fn next_invoice_number(tx: &mut PgTx<'_>, year: i32, occurred_on: NaiveDate) -> Result<String, HoaDonError> {
    let row = sqlx::query("SELECT last_sequence FROM vnd_invoice_sequence WHERE year=$1 FOR UPDATE")
        .bind(year).fetch_optional(&mut **tx).await?;
    let next = row.map(|r| r.get::<i32,_>(0) + 1).unwrap_or(1);
    sqlx::query(
        "INSERT INTO vnd_invoice_sequence (year, last_sequence) VALUES ($1, $2)
         ON CONFLICT (year) DO UPDATE SET last_sequence = $2"
    ).bind(year).bind(next).execute(&mut **tx).await?;
    Ok(format!(
        "CYBOS-{:02}{:02}{:02}-{:06}",
        year % 100, occurred_on.month(), occurred_on.day(), next
    ))
}
```

### 6.2 Async charge resolution via LISTEN/NOTIFY

```rust
pub async fn charge_with_wait(ctx: &AppCtx, tenant_id: Uuid, plan: PlanTier) -> Result<(), VndError> {
    let mut listener = sqlx::postgres::PgListener::connect_with(&ctx.pool).await?;
    listener.listen(&format!("vnd_charge_{tenant_id}")).await?;
    let psp = ctx.vnd_psp_for(tenant_id).await?;
    let ack = psp.charge(/* ... */).await?;
    match ack {
        ChargeAck::Completed { .. } => Ok(()),
        ChargeAck::Processing { .. } => {
            tokio::time::timeout(Duration::from_secs(300), listener.recv()).await
                .map_err(|_| VndError::PspTimeout)?
                .map_err(VndError::ListenerError)?;
            Ok(())
        }
    }
}
```

### 6.3 NATS dispatcher

```rust
pub async fn run_dispatcher(ctx: AppCtx) {
    let sub = ctx.nats.subscribe("tenant.*.ten.vnd.>").await.unwrap();
    while let Some(msg) = sub.next().await {
        let event: VndWebhookEvent = serde_json::from_slice(&msg.payload).unwrap();
        let dispatched = ctx.repo.vnd_dispatch_log.upsert_idempotent(event.psp, event.psp_event_id).await;
        if !dispatched.is_new { continue; }
        match event.event_type.as_str() {
            "subscription_charge_completed" => handle_charge_completed(&ctx, event).await,
            "subscription_charge_failed"    => handle_charge_failed(&ctx, event).await,
            "refund_completed"              => handle_refund_completed(&ctx, event).await,
            "token_expired"                 => handle_token_expired(&ctx, event).await,
            _ => {}
        }
    }
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-TEN-003** Stripe billing — provides billing-rail abstraction this task consumes (rail trait pattern + plan_change push hook + dunning state machine pattern + refund flow pattern); founder-skip + cross-rail block guards consumed.
- **TASK-INV-005** VietQR webhook — VnPay webhook handler shape consumed; extended for Momo + ZaloPay variants.

**Cross-module (related_tasks):**
- **TASK-TEN-001** Provisioning — billing_contact_phone capture at vn-1 signup.
- **TASK-TEN-002** Plan tiers — VND prices in catalog (PRICE_CATALOG VND entries).
- **TASK-TEN-004** 4-axis metering — period_close hook invokes vnd::overage::charge_for_period.
- **TASK-TEN-101** Self-serve signup — VND path now functional (TASK-TEN-101 §1 #13's 503 placeholder resolved).
- **TASK-TEN-103** Residency provisioning — vn-1 routing for PSP credentials + KMS.
- **TASK-TEN-104** Lifecycle — dunning advances trigger suspension.
- **TASK-INV-006** Cash application — VND receipts reconcile against subscription charges.
- **TASK-AUTH-101** RBAC — cfo + tenant_admin role gates.
- **TASK-AI-003** memory audit — 12 new kinds.
- **TASK-MEMORY-111** PII scrubbing — phone + masked account hashes.
- **TASK-OBS-007** Auto-runbook — sev-1 dunning + signature-failure alerts.

**Downstream (blocks):** None at slice 2.

---

## §8 — Example payloads

### 8.1 `ten.vnd_subscription_charged` memory row

```json
{
  "kind": "ten.vnd_subscription_charged",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "system.ten.vnd",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T03:14:32.847Z",
  "payload": {
    "psp": "momo",
    "amount_vnd": 1_290_000,
    "period_start": "2026-04-17T17:00:00Z",
    "period_end": "2026-05-17T17:00:00Z",
    "psp_charge_ref": "MOMO_TXN_3OabcXYZ",
    "invoice_number": "CYBOS-260517-000042",
    "billing_contact_phone_hash16": "9c4e7a8b6d2f1e3a"
  }
}
```

### 8.2 Token-bind start request/response

```json
// POST /v1/signup/vnd/token-bind-start
{ "signup_session_id": "0190f7c0-8b3c-7a4f-aaaa-000000000001", "psp": "vnpay" }

// response
{ "redirect_url": "https://sandbox.vnpayment.vn/paymentv2/vpcpay.html?vnp_Amount=10000&vnp_TxnRef=...",
  "expires_at": "2026-05-17T03:24:32.847Z" }
```

### 8.3 Hóa đơn (Decree 123 line items)

```json
{
  "invoice_number": "CYBOS-260517-000042",
  "tenant_id": "8a2f...",
  "issued_at": "2026-05-17T03:14:35.221Z",
  "taxpayer": { "tax_id": "0312345678", "name": "ACME Vietnam JSC", "address": "..." },
  "line_items": [
    { "description": "Cyberos Team plan - billing period 2026-04-17 → 2026-05-17",
      "quantity": 1, "pre_tax_amount_vnd": 1_172_727, "vat_rate": 0.10, "vat_amount_vnd": 117_273, "total_vnd": 1_290_000 }
  ],
  "totals": { "pre_tax_vnd": 1_172_727, "vat_vnd": 117_273, "total_vnd": 1_290_000 },
  "ehoadon_tax_authority_ref": "GDT-2026-000123456",
  "signed_xml_sha256": "9c4e7a8b6d2f1e3a..."
}
```

### 8.4 `ten.vnd_refund_issued` memory row

```json
{
  "kind": "ten.vnd_refund_issued",
  "severity": 1,
  "tenant_id": "8a2f...",
  "actor_id": "user.cfo.456",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "psp": "vnpay",
    "original_charge_ref": "VNP_TXN_3OabcXYZ",
    "refund_amount_vnd": 645_000,
    "compensating_invoice_number": "CYBOS-260517-000043",
    "reason": "duplicate_charge"
  }
}
```

---

## §9 — Open questions

All resolved for slice 2. Deferred:

- **Deferred:** Multi-PSP failover within a single tenant — slice 3, task-TEN-2xx (placeholder).
- **Deferred:** PSP-side card whitelabeling (custom branding at PSP redirect) — slice 3.
- **Deferred:** Annual billing cycle for VND — slice 3 (monthly only at slice 2, mirrors TASK-TEN-003).
- **Deferred:** Coupon / promo code support — slice 3.
- **Deferred:** Hóa đơn cancellation flow (Decree 123 §17 conditions) — slice 3, task-TEN-2xx.
- **Deferred:** Bulk hóa đơn export to tax authority's quarterly upload format — slice 3.
- **Deferred:** Multi-PSP-per-tenant for redundancy — slice 3.
- **Deferred:** Recurring-charge advance-notice email (T-3 days reminder) — slice 3, task-EMAIL-1xx.
- **Deferred:** Per-PSP availability monitoring + auto-failover — slice 3, task-OBS-2xx.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| PSP token-bind redirect timeout | 10-min TTL on token-bind session | Bind session expired; user re-clicks "Sign up with VnPay/Momo/ZaloPay" | Re-initiate; idempotent |
| PSP returns failure at callback (user cancelled SMS-OTP) | `psp.token_bind_callback` returns Err | 200 to UI with `{ status: "user_cancelled" }`; signup session remains email_verified for retry | User retries token-bind |
| Webhook signature invalid | HMAC verify fails | 401 + `signature_invalid` + `inv.vnd_event_rejected` | PSP-side rotated credential without our update; ops triages |
| Webhook replay (same psp_event_id) | dispatcher `(psp, event_id)` UNIQUE | 200 + duplicate log row + no side effect | Inherent — idempotency |
| PSP charge API 5xx | api_client retries 3x with backoff | Charge marked failed → dunning advances | Stripe-style auto-retry; persistent → sev-2 alert |
| PSP charge timeout (>5min wait) | LISTEN/NOTIFY timeout fires | Charge marked `processing_timeout`; webhook may arrive later → reconciliation | Reconciliation sweep runs hourly; manual review on persistent timeout |
| Hóa đơn signing service down (eHĐĐT API unavailable) | sign_invoice returns Err | Invoice persisted unsigned (status='issued', signed_xml NULL); retry job re-signs every 5min | eHĐĐT recovery; max 24h before sev-1 alert (Decree 123 §13 grace) |
| Hóa đơn sequence number duplicate (concurrent INSERT) | partial unique on invoice_number | Second INSERT fails; caller retries with next number | Inherent — `FOR UPDATE` + retry on conflict |
| Invoice sequence rollback (tx fails post-allocation) | sequence number "skipped" | Skipped number recorded in `vnd_invoice_sequence.notes` with reason | Auditable per Decree 123 §10 |
| Cross-rail attempt (USD tenant invoking VND) | api_client guard | 400 + `wrong_billing_rail` + `ten.vnd_cross_rail_rejected` sev-2 | Handler bug investigation |
| Founder tenant VND attempt | api_client guard | No-op + sev-3 `ten.vnd_founder_skip` | Inherent guard |
| Tenant has no active payment_token at monthly charge time | charge_monthly preflight | charge skipped + `ten.vnd_subscription_charge_failed` reason='no_token' + dunning advances | Tenant re-binds token via signup-style flow OR support intervention |
| PSP credential rotation overlap expired before rollout | api_client uses old credential | 401 from PSP; sev-1 alert | Tenant_admin re-rotates; usual fix |
| KMS unavailable when decrypting payment_token | KMS timeout | charge skipped + sev-1 `ten.vnd_kms_unavailable` | AWS KMS recovery; charge retried |
| Webhook arrives before charge_monthly job completes | LISTEN/NOTIFY race | Webhook handler runs first; charge_monthly's wait returns immediately on next NOTIFY | Inherent — atomic on (tenant, period) |
| Tenant phone number invalid / OTP undeliverable | PSP returns at token-bind start | 400 + `phone_invalid`; user updates phone | User edits phone in signup flow |
| Cancellation invoice references a non-existent original | hoadon.cancel preflight | 400 + `original_invoice_not_found` | Manual review |
| RLS bypassed (residency drift detected) | TASK-TEN-103 trip-wire | INSERT/UPDATE blocked; sev-1 `ten.cross_residency_write_blocked` | Inherent — defense-in-depth |
| `vnd_invoice_sequence` for new year not yet seeded | year row missing | First-of-year INSERT triggers seed (Jan 1 02:00 UTC scheduled job) | Inherent — seed job |
| eHĐĐT signing succeeds but DB write fails | Postgres write error post-sign | Signed XML temporarily orphaned in eHĐĐT; sev-1 alert; reconciliation job re-links | Reconciliation runs nightly + matches eHĐĐT refs |
| Per-PSP credential leaked (suspicious activity) | OBS anomaly detection | Sev-1 alert; tenant_admin rotates immediately | Standard incident response |
| Hóa đơn line item amounts don't sum to total | pre_save validation | Reject + sev-2 log; charge proceeds without hóa đơn (illegal — operator must fix manually) | Operator validates + reissues |

---

## §11 — Implementation notes

**§11.1** PSP adapters are independently versioned crates inside `services/ten/src/billing/vnd/` — bumping VnPay's API version doesn't touch Momo/ZaloPay.

**§11.2** Per-PSP API endpoints are loaded from `services/ten/src/billing/vnd/psp_endpoints.yaml` (environment-pinned URLs); test/sandbox URLs distinct from prod.

**§11.3** Hóa đơn line-item format follows Circular 78/2021/TT-BTC field-naming convention; XSD validation included in `hoadon.rs` before signing.

**§11.4** The `vnd_invoice_sequence.notes` JSONB stores skipped-number reasons per Decree 123 §10 ("tx rolled back due to KMS unavailable at 2026-05-17T03:14"); auditor can reconstruct gap rationale.

**§11.5** Per-PSP idempotency_key encoding constraints: VnPay `vnp_TxnRef` max 100 chars; Momo `requestId` max 50; ZaloPay `app_trans_id` format `yyMMdd_<num>` max 40. Adapter SHA-1-shortens canonical key when needed; collision risk negligible (~2^60 entropy).

**§11.6** Webhook signature verification timing-safe via `subtle::ConstantTimeEq` to prevent timing attacks.

**§11.7** Per-PSP test environments use sandbox accounts; integration smoke tests run nightly (not in PR CI) to validate signature + recurring-charge end-to-end.

**§11.8** Token expiration handling: VnPay tokens are 365d default; Momo 180d; ZaloPay 365d. Expiry monitor job emails tenant_admin T-30 days; auto-suspends at T+1 if no rebind.

**§11.9** SBV Circular 39/2014 PSP regulations: all 3 chosen PSPs are SBV-licensed e-payment providers; CyberSkill's merchant account requires SBV registration (Stephen handles this commercial workstream out-of-band).

**§11.10** The dunning state machine reuses TASK-TEN-003's `DunningState` enum (Ok/Retry1/Retry2/Retry3/Suspended); same advancement semantics; same un-suspend on success.

**§11.11** Per-PSP error mapping into our `VndError` enum: PSP-specific error codes documented in adapter source files; uniform `VndError::PspError { code, message, retryable }` shape consumed by upstream handlers.

**§11.12** The trace_id thread: signup_session → token_bind → PSP redirect (state param carries trace_id) → callback → INSERT → audit row. PSP-side may not preserve trace_id across redirect; the callback handler RE-INSTATES the original trace_id from server-side session lookup.

**§11.13** The `vnd_invoice_sequence` annual-reset is at Vietnam timezone (UTC+7) midnight, not UTC; matches tax authority convention.

**§11.14** Cross-PSP charge attempts (e.g., tenant bound to VnPay token but operator tries Momo) are prevented by `vnd_payment_tokens` query — there's only one active token per tenant, and its `psp` column dictates which adapter to invoke.

**§11.15** The 7-day idempotency cache TTL (mirror of TASK-TEN-003 DEC-807) — Stripe's idempotency window is 24h, VND PSPs vary (some don't enforce window). 7d is forensically generous + matches existing pattern.

**§11.16** Phone number canonical form for hashing: E.164 (`+84901234567`); user inputs locale-form (`0901 234 567`) normalised at signup.

**§11.17** The `description` field on PSP charge requests is user-visible on bank statement / wallet — `"Cyberos Team plan - 2026-05"` chosen for brand recognition + period clarity.

**§11.18** Refund hóa đơn (compensating invoice per Decree 123) carries negative line items + references original invoice via `original_invoice_ref` column (added in slice 3); slice 2 stores via JSONB free-form note.

**§11.19** OBS dashboard per-PSP success-rate watching alerts on sustained <95% (3 PSP outages within a week = sev-1 escalation per TASK-OBS-007).

**§11.20** Cargo dependency on `hmac` + `sha2` (for VnPay HMAC-SHA512); already in workspace from TASK-INV-005's VietQR pattern.

**§11.21** Per-tenant PSP statistics (`vnd_charges_total`, `vnd_charges_failed_total` Prometheus counters) labelled by PSP for operator visibility.

**§11.22** The `vnd_event_dispatch_log` mirrors TASK-TEN-003's `stripe_event_dispatch_log` table shape; consistent operational model across rails.

---

*End of TASK-TEN-102 spec.*
