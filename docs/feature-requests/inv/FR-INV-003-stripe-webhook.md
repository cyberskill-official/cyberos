---
id: FR-INV-003
title: "INV Stripe webhook handler — Stripe-Signature verify + closed event-type allowlist + idempotent receipt insert + multi-currency + append-only ledger + memory audit"
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
related_frs: [FR-AUTH-101, FR-AI-003, FR-MEMORY-101, FR-INV-005, FR-INV-006, FR-TEN-003, FR-OBS-007]
depends_on: [FR-AUTH-101]
blocks: [FR-INV-006, FR-TEN-003]

source_pages:
  - website/docs/modules/inv.html#stripe
  - https://stripe.com/docs/webhooks/signatures
  - https://stripe.com/docs/api/events/types
source_decisions:
  - DEC-460 (Stripe-Signature verification per Stripe's v1 signing scheme — HMAC-SHA256 over `{timestamp}.{payload}`; constant-time compare; rotate webhook secret quarterly)
  - DEC-461 (closed event-type allowlist at 8 values: payment_intent.succeeded · payment_intent.payment_failed · charge.refunded · invoice.payment_succeeded · invoice.payment_failed · invoice.finalized · customer.subscription.updated · customer.subscription.deleted — unknown event_type → log + 200 OK ack; never process unknown)
  - DEC-462 (idempotency keyed by Stripe `event.id` — Stripe guarantees unique event_id; duplicate webhook → 200 OK ack with existing receipt_id reference; never double-credit)
  - DEC-463 (REVOKE UPDATE, DELETE on stripe_event_log from cyberos_app — append-only at SQL grant)
  - DEC-464 (replay window 5 minutes per Stripe spec; outside → 401 + audit)
  - DEC-465 (per-tenant webhook URL routing — Stripe webhook endpoint per tenant: `https://api.cyberos.world/v1/inv/webhooks/stripe/<tenant_slug>`; secret per tenant)
  - DEC-466 (multi-currency support — Stripe sends amount + currency; stored as BIGINT minor + CHAR(3) currency per feature-request-audit skill rule 11; supports USD, EUR, SGD, GBP at slice 1)
  - DEC-467 (memory audit kinds: inv.stripe_event_received, inv.stripe_event_rejected, inv.stripe_payment_received, inv.stripe_payment_refunded, inv.stripe_subscription_changed, inv.duplicate_stripe_event_received)
  - DEC-468 (Stripe-Signature header format: `t=<unix_timestamp>,v1=<hex_signature>[,v0=<deprecated>]`; we accept v1 only; v0 deprecated by Stripe)
  - DEC-469 (per-tenant webhook secret stored in `stripe_webhook_secrets` table KMS-encrypted; rotation handler with 60s overlap window)
  - DEC-470 (alarm sev-2 on > 10 rejections/h per tenant — same threshold as VietQR FR-INV-005)
  - DEC-471 (Stripe Connect / multi-account scenarios deferred to FR-INV-2xx — slice 2 is platform-account only)
  - DEC-472 (`payment_intent.metadata` field contains `invoice_id` or `tenant_invoice_ref` — extracted into receipt's `invoice_id` column for FR-INV-006 cash app)
  - DEC-473 (PCI SAQ-A compliance — Stripe never sends raw card numbers to our webhook; PCI scope rests with Stripe)
  - PDPL Art. 13 (data minimisation — customer email + name PII-scrubbed in memory chain)
  - PCI DSS SAQ-A (Stripe-hosted card data — out-of-scope at our endpoint)

language: rust 1.81 + sql
service: cyberos/services/inv/
new_files:
  - services/inv/migrations/0012_stripe_event_log.sql            # append-only event log + RLS
  - services/inv/migrations/0013_stripe_webhook_secrets.sql       # per-tenant secret + rotation history
  - services/inv/src/webhook/stripe.rs                            # POST handler
  - services/inv/src/webhook/stripe_signature.rs                  # Stripe v1 signing scheme verification
  - services/inv/src/webhook/stripe_event_dispatch.rs             # closed event-type → handler routing
  - services/inv/src/webhook/stripe_idempotency.rs                # event.id idempotency cache
  - services/inv/src/types.rs                                     # StripeEventKind enum (closed 8 values)
  - services/inv/src/repo/stripe_event_log.rs                     # append-only writer
  - services/inv/src/repo/stripe_secrets.rs                       # secret CRUD + rotation
  - services/inv/src/audit/stripe_events.rs                       # 6 memory row builders
  - services/inv/tests/stripe_signature_test.rs
  - services/inv/tests/stripe_event_allowlist_test.rs
  - services/inv/tests/stripe_idempotent_test.rs
  - services/inv/tests/stripe_replay_window_test.rs
  - services/inv/tests/stripe_multi_currency_test.rs
  - services/inv/tests/stripe_metadata_invoice_link_test.rs
  - services/inv/tests/stripe_append_only_log_test.rs
  - services/inv/tests/stripe_per_tenant_url_test.rs
  - services/inv/tests/stripe_rotation_overlap_test.rs
  - services/inv/tests/stripe_audit_emission_test.rs
  - services/inv/tests/stripe_perf_test.rs
modified_files:
  - services/inv/src/types.rs                                     # extend receipt_source enum (already added in FR-INV-005)

allowed_tools:
  - file_read: services/inv/**
  - file_write: services/inv/{src,tests,migrations}/**
  - bash: cd services/inv && cargo test stripe

disallowed_tools:
  - skip Stripe-Signature verification (per DEC-460)
  - accept v0 signature scheme (per DEC-468 — deprecated)
  - process events outside the closed 8-value allowlist (per DEC-461)
  - allow UPDATE on stripe_event_log (per DEC-463)
  - store amounts as FLOAT (per feature-request-audit skill rule 11)

effort_hours: 8
sub_tasks:
  - "0.6h: 0012_stripe_event_log.sql — append-only + RLS + REVOKE"
  - "0.4h: 0013_stripe_webhook_secrets.sql — per-tenant KMS-encrypted secrets + rotation"
  - "0.8h: webhook/stripe_signature.rs — v1 scheme HMAC + replay window"
  - "0.8h: webhook/stripe.rs — POST handler with per-tenant URL"
  - "0.8h: webhook/stripe_event_dispatch.rs — closed allowlist routing"
  - "0.5h: webhook/stripe_idempotency.rs — event.id cache"
  - "0.5h: types.rs — StripeEventKind closed enum"
  - "0.5h: repo/stripe_event_log.rs — append-only writer"
  - "0.4h: repo/stripe_secrets.rs"
  - "0.5h: audit/stripe_events.rs — 6 builders"
  - "2.2h: tests — 11 test files"

risk_if_skipped: "Without Stripe webhook capture, international tenant payments (USD/EUR/SGD) bypass our ledger — every payment becomes manual entry. FR-TEN-003 (Stripe billing for international tenants) cannot ship without webhook receipts. FR-INV-006 (cash application) needs the same Stripe receipt shape as VietQR receipts. Without DEC-460's Stripe-Signature verification, attackers forge fake payments (high-value attack — credit fraudulent invoices). Without DEC-461's closed event-type allowlist, novel Stripe events arrive + processing logic forks into untested branches. Without DEC-462's idempotency on Stripe event.id, network retries double-credit. Without DEC-466's BIGINT-minor multi-currency, USD cents → FLOAT rounding corrupts invoice math at scale. The 8h effort lands the audit-grade international-payment capture primitive."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship the Stripe webhook handler at `POST /v1/inv/webhooks/stripe/{tenant_slug}` with Stripe-Signature v1 verification + closed event-type allowlist + event.id idempotency + multi-currency + append-only ledger. Each requirement:

1. **MUST** extend the `payment_receipts` table (from FR-INV-005) — `receipt_source` enum already includes `'stripe'`; no schema change here. New rows from Stripe handler use `receipt_source='stripe'` and populate `bank_code='STRIPE'` (Stripe is added as a new value to bank_code enum via a small migration ALTER TYPE).

2. **MUST** define `stripe_event_log` table: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, stripe_event_id TEXT NOT NULL, event_type TEXT NOT NULL, livemode BOOLEAN NOT NULL, payload_sha256 CHAR(64) NOT NULL, payment_intent_id TEXT, invoice_id_stripe TEXT, customer_id_stripe TEXT, amount_minor BIGINT, currency CHAR(3), outcome TEXT NOT NULL CHECK (outcome IN ('processed','rejected','duplicate','unknown_event_type')), failure_reason TEXT, ts TIMESTAMPTZ NOT NULL DEFAULT now())`. UNIQUE `(tenant_id, stripe_event_id)`.

3. **MUST** define `stripe_webhook_secrets` table: `(tenant_id UUID PRIMARY KEY, secret_kms_blob BYTEA NOT NULL, kms_key_id TEXT NOT NULL, created_at TIMESTAMPTZ, rotated_at TIMESTAMPTZ, status TEXT NOT NULL CHECK (status IN ('active','rotated','revoked')))`. Partial unique `(tenant_id) WHERE status='active'`.

4. **MUST** enforce RLS with both `USING` and `WITH CHECK` on both new tables. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

5. **MUST** be **append-only** on `stripe_event_log` at SQL grant (per DEC-463 + feature-request-audit skill rule 12). `REVOKE UPDATE, DELETE ON stripe_event_log FROM cyberos_app`.

6. **MUST** expose `POST /v1/inv/webhooks/stripe/{tenant_slug}` (per DEC-465). Per-tenant URL routing. Unknown slug → 404 `tenant_unknown` + emit `inv.stripe_event_rejected` memory row.

7. **MUST** verify Stripe-Signature header per Stripe's v1 scheme (per DEC-460 + DEC-468):
   - Header format: `t=<unix_timestamp>,v1=<hex_signature>[,v0=<deprecated>]`.
   - Parse `t` + `v1` values (ignore v0).
   - Compute `expected_signature = HMAC-SHA256(secret, "{t}.{raw_body}")`.
   - Constant-time compare `v1` vs `expected_signature`.
   - Mismatch → 401 `stripe_signature_invalid` + emit `inv.stripe_event_rejected` memory row with `reason='signature_invalid'`.

8. **MUST** validate replay window (per DEC-464). `|server_now - t| ≤ 5 minutes`. Outside → 401 `stripe_replay_window_exceeded` + emit `inv.stripe_event_rejected` with `reason='replay_window_exceeded'`.

9. **MUST** enforce **closed event-type allowlist** (per DEC-461 + §1 #14). The 8 allowed values:
    - `payment_intent.succeeded` — successful payment.
    - `payment_intent.payment_failed` — payment attempt failed.
    - `charge.refunded` — refund issued.
    - `invoice.payment_succeeded` — subscription invoice paid.
    - `invoice.payment_failed` — subscription invoice failed.
    - `invoice.finalized` — invoice ready for collection.
    - `customer.subscription.updated` — subscription state change.
    - `customer.subscription.deleted` — subscription cancelled.
   Unknown `event.type` → INSERT `stripe_event_log` row with `outcome='unknown_event_type'` + return 200 OK (Stripe expects 2xx ack for all received events; non-2xx triggers retry).

10. **MUST** enforce idempotency on `event.id` (per DEC-462). Lookup `stripe_event_log WHERE tenant_id=$1 AND stripe_event_id=$2`. If exists → return 200 OK with existing log id; emit `inv.duplicate_stripe_event_received` memory row (sev-3 informational). Never double-process.

11. **MUST** route allowed events to per-type handlers (per DEC-461):
    - `payment_intent.succeeded` → INSERT `payment_receipts` row with `receipt_source='stripe'`, `bank_code='STRIPE'`, link to invoice via `payment_intent.metadata.invoice_id` (per DEC-472).
    - `payment_intent.payment_failed` → log only; emit `inv.stripe_event_rejected` with `reason='payment_failed'`.
    - `charge.refunded` → INSERT compensating payment_receipts row with negative `amount_minor`; emit `inv.stripe_payment_refunded` memory row.
    - `invoice.payment_succeeded` → INSERT payment_receipts + match to FR-INV-001 invoice via `stripe_invoice.metadata.invoice_id`.
    - `invoice.payment_failed` → log; emit `inv.stripe_event_rejected`.
    - `invoice.finalized` → log only (slice 1; subscription state tracking ships in FR-TEN-003).
    - `customer.subscription.{updated,deleted}` → emit `inv.stripe_subscription_changed` memory row + log.

12. **MUST** support multi-currency (per DEC-466). Stripe sends `amount` (BIGINT minor units) + `currency` (3-letter ISO-4217 lowercase per Stripe — we uppercase). Supported at slice 1: USD, EUR, SGD, GBP. Unsupported currency → 400 `currency_unsupported` + emit `inv.stripe_event_rejected`.

13. **MUST** PII-scrub `customer.email`, `customer.name`, `description` via FR-MEMORY-111 BEFORE chain commit. Raw values retained in tenant Postgres rows; memory chain holds hashed forms (`customer_email_hash16`, `customer_name_hash16`).

14. **MUST** declare the closed `StripeEventKind` Rust enum with exactly 8 values mapping to the 8 allowed event_type strings (per §1 #9). Adding a 9th is an ADR.

15. **MUST** persist `payload_sha256` (SHA-256 of raw body) for forensic replay. Full body NOT stored (potentially large + may contain PII); hash + structured fields suffice.

16. **MUST** complete handler in ≤ 200 ms p95 (Stripe expects 2xx within seconds). `stripe_perf_test`.

17. **MUST** acknowledge to Stripe within 5 seconds (per Stripe's retry policy — non-2xx OR > 30s = retry). The handler does minimum sync work: signature verify + idempotency check + event dispatch + INSERT + audit emit.

18. **MUST** support webhook secret rotation via `POST /v1/inv/stripe-secrets/rotate`. Caller MUST have role `cfo` per FR-AUTH-101. Rotation flow same shape as FR-INV-005 — generates new 32-byte secret, KMS-encrypts, INSERT new active + UPDATE prior to rotated; 60-second overlap window where both old + new accepted.

19. **MUST** emit 6 memory audit row kinds (per DEC-467):
    - `inv.stripe_event_received` — every successful processing.
    - `inv.stripe_event_rejected` — signature fail / replay / unsupported_currency / unknown_tenant.
    - `inv.stripe_payment_received` — payment_intent.succeeded or invoice.payment_succeeded.
    - `inv.stripe_payment_refunded` — charge.refunded.
    - `inv.stripe_subscription_changed` — customer.subscription.{updated,deleted}.
    - `inv.duplicate_stripe_event_received` — idempotency hit; sev-3.

20. **MUST** alarm sev-2 on > 10 rejections per tenant per hour (per DEC-470). OBS rule in FR-OBS-007's set.

21. **MUST** emit OTel span `inv.webhook.stripe` with attributes: `tenant_id`, `event_type`, `currency`, `outcome` (success | signature_invalid | replay_window_exceeded | tenant_unknown | currency_unsupported | duplicate | unknown_event_type | payment_failed).

22. **MUST** emit OTel metrics:
    - `inv_stripe_event_received_total{tenant_id, event_type, outcome}` (counter).
    - `inv_stripe_event_rejected_total{tenant_id, reason}` (counter — sev-2 alarm at > 10/h).
    - `inv_stripe_payment_amount_minor{tenant_id, currency}` (counter — sum of payment amounts).
    - `inv_stripe_refund_amount_minor{tenant_id, currency}` (counter).
    - `inv_stripe_webhook_latency_ms` (histogram; SLO p95 < 200ms).

23. **MUST** validate that `livemode` matches the environment — production endpoint accepts only `livemode=true`; staging/dev only `livemode=false`. Mismatch → 400 `livemode_mismatch` + emit `inv.stripe_event_rejected`.

24. **MUST** route the dispatch by `event.type` using a closed `match` statement on `StripeEventKind`. Default arm: log `unknown_event_type` + ack 200 (Stripe must not retry); never panic or 500 on unknown types.

25. **MUST** populate `payment_receipts.invoice_id` from `event.data.object.metadata.invoice_id` (per DEC-472) when present + the invoice exists in tenant scope. Missing metadata or invoice not found → `invoice_id=NULL`; FR-INV-006 cash app handles matching async.

26. **MUST** record `customer_id_stripe` + `payment_intent_id` + `invoice_id_stripe` in `stripe_event_log` for cross-event reconciliation (e.g. finding all events for one customer + verifying lifecycle order).

---

## §2 — Why this design (rationale for humans)

**Why Stripe's v1 signature scheme (DEC-460, §1 #7)?** Stripe's signing scheme binds `timestamp + payload` under HMAC-SHA256 — preventing both signature forgery AND replay (timestamp validation). v0 is deprecated; v1 is the current production scheme. Constant-time compare prevents timing-attack signature recovery.

**Why closed event-type allowlist of 8 (DEC-461, §1 #9)?** Stripe ships ~200 event types. Most are irrelevant (e.g. `radar.early_fraud_warning.created`). Processing only the 8 we explicitly understand means novel events don't fork into untested logic. Unknown events still get 200-ack'd (Stripe requires) but logged for review.

**Why idempotency on event.id (DEC-462, §1 #10)?** Stripe retries webhooks on non-2xx response — without idempotency, a slow-but-eventually-success path retried 3 times = 3 payment_receipts rows for one payment. Stripe guarantees event.id uniqueness; we use it as dedup key.

**Why 200 OK ack on unknown_event_type (DEC-461, §1 #9)?** Stripe's retry policy treats non-2xx as transient failure → retries with exponential backoff for up to 3 days. If we 4xx on novel events, our endpoint flood-receives retries for events we don't understand AND those events get stuck. 200-ack ends the retry; the log row preserves the unknown event for later review.

**Why 5-minute replay window (DEC-464, §1 #8)?** Same logic as VietQR — Stripe's signing includes timestamp; we validate against current time + 5min skew. Stripe spec is `tolerance=300` seconds (5min).

**Why per-tenant URL (DEC-465, §1 #6)?** Tenant URL → per-tenant secret lookup without body inspection. Stripe portal lets each tenant configure their own webhook URL + secret — operational simplicity.

**Why per-tenant secret rotation with 60s overlap (DEC-469, §1 #18)?** Same logic as VietQR webhook. Operator rotates secret in our system + updates Stripe portal in sequence; 60s window handles the timing gap.

**Why multi-currency stored as BIGINT minor (DEC-466, §1 #12)?** USD cents, EUR cents, SGD cents, GBP pence — all minor-unit currencies. feature-request-audit skill rule 11. FLOAT for amount would round-error at scale.

**Why supported currencies USD/EUR/SGD/GBP at slice 1 (§1 #12)?** Covers ~95% of international SaaS billing; adding more is an ADR (each new currency may need FX rate setup + invoicing template).

**Why payload_sha256 stored not full body (§1 #15)?** Stripe payloads include line items, addresses, metadata — can be 5-50 KB. Storing all ledgers bloats. Hash is sufficient for "did this exact event arrive?" replay.

**Why livemode validation (§1 #23)?** Production endpoint receiving test-mode events = developer testing leaked to production OR misconfigured portal. Test-mode endpoint receiving live events = real money treated as test data. Hard reject prevents both.

**Why charge.refunded creates negative payment_receipts (§1 #11)?** Accounting: a refund reverses a payment. Negative-amount row in the same ledger preserves the cause-effect link (refund row references the original payment_intent_id via the event's `payment_intent` field). Sum aggregates naturally net out.

**Why customer.subscription.{updated,deleted} logged but not state-tracked at slice 1 (§1 #11)?** Subscription state lives in FR-TEN-003 (Stripe billing). This FR persists the event; FR-TEN-003 consumes for subscription tier transitions.

**Why metadata.invoice_id linking (DEC-472, §1 #25)?** Stripe lets us attach arbitrary key-value metadata to payment intents + invoices. We populate `invoice_id` (our internal FR-INV-001 invoice UUID) at payment_intent creation time; the webhook extracts it to match payment back to invoice.

**Why default arm = log + 200 (§1 #24)?** Defense against future Stripe API additions. New event_types appear regularly; our handler doesn't 5xx on them. Operators see the unknown events in logs + decide whether to add to allowlist.

**Why customer email + name PII-scrubbed in memory chain (§1 #13)?** Customer PII is sensitive; memory chain is queried broadly. Hashed forms suffice for forensic queries; raw retained in tenant-scoped Postgres rows under RLS.

**Why no Stripe Connect at slice 1 (DEC-471)?** Connect (Stripe's marketplace pattern with multiple Stripe accounts) adds substantial complexity (oauth flow, on-behalf-of headers, application_fee). Slice 1 = single platform Stripe account; Connect ships as FR-INV-2xx when marketplace use case arrives.

**Why PCI SAQ-A scope (DEC-473)?** Stripe-hosted card data means we never touch card numbers — we're PCI SAQ-A (self-assessment questionnaire A, the simplest tier). No card numbers in our webhook = nothing to encrypt at rest beyond what we do already.

---

## §3 — API contract

### 3.1 — Migration 0012 — stripe_event_log + extend bank_code

```sql
-- services/inv/migrations/0012_stripe_event_log.sql

BEGIN;

-- Extend bank_code enum to include STRIPE (was missing from FR-INV-005's 15 VN banks)
ALTER TYPE bank_code ADD VALUE 'STRIPE';

CREATE TABLE stripe_event_log (
    id                     BIGSERIAL    PRIMARY KEY,
    tenant_id              UUID         NOT NULL,
    stripe_event_id        TEXT         NOT NULL,
    event_type             TEXT         NOT NULL,
    livemode               BOOLEAN      NOT NULL,
    payload_sha256         CHAR(64)     NOT NULL CHECK (payload_sha256 ~ '^[0-9a-f]{64}$'),
    payment_intent_id      TEXT,
    invoice_id_stripe      TEXT,
    customer_id_stripe     TEXT,
    amount_minor           BIGINT,
    currency               CHAR(3),
    outcome                TEXT         NOT NULL CHECK (outcome IN ('processed','rejected','duplicate','unknown_event_type')),
    failure_reason         TEXT,
    ts                     TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uniq_stripe_event_id ON stripe_event_log (tenant_id, stripe_event_id);
CREATE INDEX stripe_event_log_tenant_ts_idx ON stripe_event_log (tenant_id, ts DESC);
CREATE INDEX stripe_event_log_customer_idx ON stripe_event_log (tenant_id, customer_id_stripe) WHERE customer_id_stripe IS NOT NULL;
CREATE INDEX stripe_event_log_pi_idx ON stripe_event_log (tenant_id, payment_intent_id) WHERE payment_intent_id IS NOT NULL;

ALTER TABLE stripe_event_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY stripe_event_log_tenant_iso ON stripe_event_log
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON stripe_event_log FROM cyberos_app;

COMMIT;
```

### 3.2 — Migration 0013 — stripe_webhook_secrets

```sql
-- services/inv/migrations/0013_stripe_webhook_secrets.sql

BEGIN;

CREATE TABLE stripe_webhook_secrets (
    id                UUID         PRIMARY KEY,
    tenant_id         UUID         NOT NULL,
    secret_kms_blob   BYTEA        NOT NULL,
    kms_key_id        TEXT         NOT NULL,
    status            TEXT         NOT NULL CHECK (status IN ('active','rotated','revoked')),
    created_at        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    rotated_at        TIMESTAMPTZ
);

CREATE UNIQUE INDEX uniq_active_stripe_secret ON stripe_webhook_secrets (tenant_id) WHERE status = 'active';
CREATE INDEX stripe_secrets_tenant_idx ON stripe_webhook_secrets (tenant_id, created_at DESC);

ALTER TABLE stripe_webhook_secrets ENABLE ROW LEVEL SECURITY;
CREATE POLICY stripe_secrets_tenant_iso ON stripe_webhook_secrets
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON stripe_webhook_secrets FROM cyberos_app;
CREATE ROLE inv_stripe_secret_rotator;
GRANT INSERT, UPDATE (status, rotated_at) ON stripe_webhook_secrets TO inv_stripe_secret_rotator;

COMMIT;
```

### 3.3 — Stripe signature verifier

```rust
// services/inv/src/webhook/stripe_signature.rs
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use chrono::Utc;

type HmacSha256 = Hmac<Sha256>;

const REPLAY_WINDOW_SECONDS: i64 = 300;  // 5 minutes per Stripe spec

#[derive(Debug, thiserror::Error)]
pub enum StripeSigError {
    #[error("signature_header_missing")]
    HeaderMissing,
    #[error("signature_header_malformed")]
    HeaderMalformed,
    #[error("signature_invalid")]
    SignatureInvalid,
    #[error("replay_window_exceeded")]
    ReplayWindowExceeded,
}

pub fn verify(body: &[u8], signature_header: &str, secrets: &[&[u8]]) -> Result<(), StripeSigError> {
    // Parse "t=<ts>,v1=<sig>" (ignore v0)
    let mut t: Option<i64> = None;
    let mut v1: Option<String> = None;
    for pair in signature_header.split(',') {
        let mut parts = pair.splitn(2, '=');
        let k = parts.next().ok_or(StripeSigError::HeaderMalformed)?.trim();
        let v = parts.next().ok_or(StripeSigError::HeaderMalformed)?.trim();
        match k {
            "t" => t = v.parse().ok(),
            "v1" => v1 = Some(v.to_string()),
            _ => continue,   // ignore v0 etc.
        }
    }
    let ts = t.ok_or(StripeSigError::HeaderMalformed)?;
    let v1_hex = v1.ok_or(StripeSigError::HeaderMalformed)?;

    // Replay window
    let now = Utc::now().timestamp();
    if (now - ts).abs() > REPLAY_WINDOW_SECONDS {
        return Err(StripeSigError::ReplayWindowExceeded);
    }

    // Compute expected signature: HMAC-SHA256(secret, "{t}.{body}")
    let signed_payload = format!("{ts}.").into_bytes();
    let expected = hex::decode(&v1_hex).map_err(|_| StripeSigError::HeaderMalformed)?;

    for secret in secrets {
        let mut mac = HmacSha256::new_from_slice(secret).expect("valid hmac key");
        mac.update(&signed_payload);
        mac.update(body);
        let computed = mac.finalize().into_bytes();
        if computed.as_slice().ct_eq(&expected).into() {
            return Ok(());
        }
    }
    Err(StripeSigError::SignatureInvalid)
}
```

### 3.4 — Event kind closed enum

```rust
// services/inv/src/types.rs (additions)
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StripeEventKind {
    PaymentIntentSucceeded,
    PaymentIntentFailed,
    ChargeRefunded,
    InvoicePaymentSucceeded,
    InvoicePaymentFailed,
    InvoiceFinalized,
    SubscriptionUpdated,
    SubscriptionDeleted,
}

impl StripeEventKind {
    pub const ALL: &'static [StripeEventKind] = &[
        StripeEventKind::PaymentIntentSucceeded,
        StripeEventKind::PaymentIntentFailed,
        StripeEventKind::ChargeRefunded,
        StripeEventKind::InvoicePaymentSucceeded,
        StripeEventKind::InvoicePaymentFailed,
        StripeEventKind::InvoiceFinalized,
        StripeEventKind::SubscriptionUpdated,
        StripeEventKind::SubscriptionDeleted,
    ];

    pub fn as_stripe_str(self) -> &'static str {
        match self {
            StripeEventKind::PaymentIntentSucceeded => "payment_intent.succeeded",
            StripeEventKind::PaymentIntentFailed    => "payment_intent.payment_failed",
            StripeEventKind::ChargeRefunded         => "charge.refunded",
            StripeEventKind::InvoicePaymentSucceeded => "invoice.payment_succeeded",
            StripeEventKind::InvoicePaymentFailed   => "invoice.payment_failed",
            StripeEventKind::InvoiceFinalized       => "invoice.finalized",
            StripeEventKind::SubscriptionUpdated    => "customer.subscription.updated",
            StripeEventKind::SubscriptionDeleted    => "customer.subscription.deleted",
        }
    }
}

impl FromStr for StripeEventKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, ()> {
        for k in Self::ALL { if k.as_stripe_str() == s { return Ok(*k); } }
        Err(())
    }
}
```

### 3.5 — Webhook handler

```rust
// services/inv/src/webhook/stripe.rs
use axum::{body::Bytes, extract::{Path, State}, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use uuid::Uuid;

const SUPPORTED_CURRENCIES: &[&str] = &["USD", "EUR", "SGD", "GBP"];

#[derive(Deserialize)]
pub struct StripeEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub livemode: bool,
    pub created: i64,
    pub data: StripeEventData,
}

#[derive(Deserialize)]
pub struct StripeEventData {
    pub object: Value,    // type-dependent shape; parsed per event_type
}

#[derive(Serialize)]
pub struct StripeWebhookResponse {
    pub status: &'static str,
    pub log_id: i64,
    pub duplicate: bool,
    pub unknown_event_type: bool,
}

pub async fn stripe_webhook(
    State(state): State<AppState>,
    Path(tenant_slug): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    // 1. Resolve tenant
    let tenant_id = match state.repo.find_tenant_by_slug(&tenant_slug).await {
        Ok(Some(id)) => id,
        _ => {
            tokio::spawn(audit_unknown_tenant(tenant_slug.clone()));
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"tenant_unknown"}))).into_response();
        }
    };

    // 2. Verify signature (with 60s rotation overlap)
    let sig_header = match headers.get("Stripe-Signature").and_then(|h| h.to_str().ok()) {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"signature_header_missing"}))).into_response(),
    };
    let secrets = match state.stripe_secret_cache.get_active_and_overlap(tenant_id).await {
        Ok(s) => s,
        _ => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"secret_lookup_failed"}))).into_response(),
    };
    let secret_refs: Vec<&[u8]> = secrets.iter().map(|s| s.as_slice()).collect();

    if let Err(e) = crate::webhook::stripe_signature::verify(&body, sig_header, &secret_refs) {
        crate::audit::stripe_events::emit_rejected(tenant_id, format!("{e:?}"), &body).await;
        let code = match e {
            crate::webhook::stripe_signature::StripeSigError::ReplayWindowExceeded => StatusCode::UNAUTHORIZED,
            _ => StatusCode::UNAUTHORIZED,
        };
        return (code, Json(serde_json::json!({"error": format!("{e:?}")}))).into_response();
    }

    // 3. Parse event
    let evt: StripeEvent = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(_) => {
            crate::audit::stripe_events::emit_rejected(tenant_id, "malformed_event_body".into(), &body).await;
            return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"malformed_event_body"}))).into_response();
        }
    };

    // 4. Livemode check
    if evt.livemode != state.config.expect_livemode {
        crate::audit::stripe_events::emit_rejected(tenant_id, "livemode_mismatch".into(), &body).await;
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"livemode_mismatch"}))).into_response();
    }

    // 5. Idempotency check
    if let Some(existing_id) = state.repo.find_stripe_event_log(tenant_id, &evt.id).await.ok().flatten() {
        crate::audit::stripe_events::emit_duplicate(tenant_id, existing_id, &evt.id).await;
        return (StatusCode::OK, Json(StripeWebhookResponse {
            status: "ok", log_id: existing_id, duplicate: true, unknown_event_type: false,
        })).into_response();
    }

    // 6. Parse event_type against closed enum
    let kind = match evt.event_type.parse::<crate::types::StripeEventKind>() {
        Ok(k) => k,
        Err(_) => {
            // Unknown event_type — 200 ack but log it
            let payload_sha = hex::encode(Sha256::digest(&body));
            let log_id = state.repo.insert_stripe_event_log_unknown(
                tenant_id, &evt.id, &evt.event_type, evt.livemode, &payload_sha,
            ).await.unwrap_or(0);
            return (StatusCode::OK, Json(StripeWebhookResponse {
                status: "ok", log_id, duplicate: false, unknown_event_type: true,
            })).into_response();
        }
    };

    // 7. Dispatch by kind
    let payload_sha = hex::encode(Sha256::digest(&body));
    let result = crate::webhook::stripe_event_dispatch::dispatch(
        &state, tenant_id, &evt, kind, &payload_sha,
    ).await;

    match result {
        Ok(log_id) => (StatusCode::OK, Json(StripeWebhookResponse {
            status: "ok", log_id, duplicate: false, unknown_event_type: false,
        })).into_response(),
        Err(e) => {
            tracing::error!("stripe dispatch failed: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error":"dispatch_failed"}))).into_response()
        }
    }
}
```

### 3.6 — Event dispatcher

```rust
// services/inv/src/webhook/stripe_event_dispatch.rs
use uuid::Uuid;
use crate::types::StripeEventKind;
use crate::webhook::stripe::StripeEvent;

pub async fn dispatch(
    state: &crate::AppState,
    tenant_id: Uuid,
    evt: &StripeEvent,
    kind: StripeEventKind,
    payload_sha: &str,
) -> anyhow::Result<i64> {
    use StripeEventKind::*;
    match kind {
        PaymentIntentSucceeded | InvoicePaymentSucceeded => handle_payment_succeeded(state, tenant_id, evt, payload_sha).await,
        PaymentIntentFailed | InvoicePaymentFailed => handle_payment_failed(state, tenant_id, evt, payload_sha).await,
        ChargeRefunded => handle_charge_refunded(state, tenant_id, evt, payload_sha).await,
        InvoiceFinalized => handle_invoice_finalized(state, tenant_id, evt, payload_sha).await,
        SubscriptionUpdated | SubscriptionDeleted => handle_subscription_event(state, tenant_id, evt, kind, payload_sha).await,
    }
}

async fn handle_payment_succeeded(
    state: &crate::AppState, tenant_id: Uuid, evt: &StripeEvent, payload_sha: &str,
) -> anyhow::Result<i64> {
    let amount_minor = evt.data.object.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
    let currency = evt.data.object.get("currency").and_then(|v| v.as_str()).unwrap_or("USD").to_uppercase();
    if !crate::webhook::stripe::SUPPORTED_CURRENCIES.iter().any(|c| *c == currency) {
        crate::audit::stripe_events::emit_rejected(tenant_id, "currency_unsupported".into(), &[]).await;
        anyhow::bail!("currency_unsupported");
    }
    let pi_id = evt.data.object.get("id").and_then(|v| v.as_str()).map(String::from);
    let invoice_id_stripe = evt.data.object.get("invoice").and_then(|v| v.as_str()).map(String::from);
    let customer = evt.data.object.get("customer").and_then(|v| v.as_str()).map(String::from);
    let invoice_id = evt.data.object.get("metadata")
        .and_then(|m| m.get("invoice_id"))
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let mut tx = state.db.begin().await?;
    // (1) Log event
    let log_id: i64 = sqlx::query_scalar(r#"
        INSERT INTO stripe_event_log
        (tenant_id, stripe_event_id, event_type, livemode, payload_sha256,
         payment_intent_id, invoice_id_stripe, customer_id_stripe, amount_minor, currency, outcome)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, 'processed')
        RETURNING id
    "#)
    .bind(tenant_id).bind(&evt.id).bind(&evt.event_type).bind(evt.livemode).bind(payload_sha)
    .bind(&pi_id).bind(&invoice_id_stripe).bind(&customer).bind(amount_minor).bind(&currency)
    .fetch_one(&mut *tx).await?;

    // (2) Insert payment_receipts row (mirrors FR-INV-005 shape)
    let receipt_id = Uuid::new_v4();
    let txn_ref = pi_id.clone().unwrap_or_else(|| evt.id.clone());
    sqlx::query(r#"
        INSERT INTO payment_receipts
        (id, tenant_id, receipt_source, bank_code, transaction_reference, amount_minor, currency,
         sender_name, transfer_memo, invoice_id, received_at, napas_payload_sha256, webhook_ts)
        VALUES ($1, $2, 'stripe'::receipt_source, 'STRIPE'::bank_code, $3, $4, $5,
                $6, $7, $8, now(), $9, $10)
    "#)
    .bind(receipt_id).bind(tenant_id).bind(&txn_ref).bind(amount_minor).bind(&currency)
    .bind(&customer).bind(&invoice_id_stripe).bind(&invoice_id)
    .bind(payload_sha)
    .bind(chrono::DateTime::<chrono::Utc>::from_timestamp(evt.created, 0))
    .execute(&mut *tx).await?;

    crate::audit::stripe_events::emit_payment_received(
        &mut tx, tenant_id, receipt_id, amount_minor, &currency, &evt.id, invoice_id.as_ref(),
    ).await?;

    tx.commit().await?;
    Ok(log_id)
}

async fn handle_charge_refunded(
    state: &crate::AppState, tenant_id: Uuid, evt: &StripeEvent, payload_sha: &str,
) -> anyhow::Result<i64> {
    let amount_minor = -evt.data.object.get("amount_refunded").and_then(|v| v.as_i64()).unwrap_or(0);
    // ... similar shape with negative amount_minor + emit inv.stripe_payment_refunded
    todo!()
}

async fn handle_payment_failed(state: &crate::AppState, tenant_id: Uuid, evt: &StripeEvent, payload_sha: &str) -> anyhow::Result<i64> {
    // INSERT log row with outcome='rejected' + failure_reason; emit inv.stripe_event_rejected
    todo!()
}

async fn handle_invoice_finalized(state: &crate::AppState, tenant_id: Uuid, evt: &StripeEvent, payload_sha: &str) -> anyhow::Result<i64> {
    // INSERT log row only; FR-TEN-003 consumes for subscription tier transitions
    todo!()
}

async fn handle_subscription_event(state: &crate::AppState, tenant_id: Uuid, evt: &StripeEvent, kind: StripeEventKind, payload_sha: &str) -> anyhow::Result<i64> {
    // INSERT log row + emit inv.stripe_subscription_changed
    todo!()
}
```

---

## §4 — Acceptance criteria

1. **StripeEventKind closed at 8** — exact set per DEC-461.
2. **bank_code enum extended with STRIPE** — receipt rows from Stripe carry `bank_code='STRIPE'`.
3. **RLS isolates by tenant** — cross-tenant queries return 0 rows.
4. **POST happy path payment_intent.succeeded** — valid sig + new event.id → 200 + log row + payment_receipts row + `inv.stripe_payment_received` memory row.
5. **Signature invalid** → 401 `signature_invalid` + `inv.stripe_event_rejected` memory row.
6. **Signature header missing** → 401 `signature_header_missing`.
7. **Replay window exceeded** (`t` > 5min off) → 401 + audit.
8. **Idempotent on event.id** — duplicate POST → 200 + existing log_id + no duplicate row + `inv.duplicate_stripe_event_received` row.
9. **Unknown event_type** → 200 (Stripe must not retry) + log row with `outcome='unknown_event_type'`.
10. **Tenant unknown slug** → 404 + audit row.
11. **Currency unsupported** (e.g. JPY) → 400 + audit.
12. **Livemode mismatch** (test event on prod endpoint) → 400 + audit.
13. **Multi-currency receipts** — USD, EUR, SGD, GBP all process; amount_minor stored as BIGINT.
14. **metadata.invoice_id extracted** — receipt's `invoice_id` populated when present.
15. **Charge refunded creates negative amount_minor** — `inv.stripe_payment_refunded` row.
16. **UPDATE/DELETE stripe_event_log blocked from cyberos_app** — permission denied.
17. **Secret rotation creates new active + sets prior to rotated** — partial unique index allows.
18. **60s overlap window** — webhook signed with prior secret still accepted for 60s.
19. **Webhook ack < 200ms p95** — perf test.
20. **OTel span `inv.webhook.stripe` emitted** with `outcome` attr.
21. **Counter `inv_stripe_event_received_total{event_type=payment_intent.succeeded, outcome=success}` increments**.
22. **Sev-2 alarm at > 10 rejections/h** — OBS rule.
23. **PII-scrubbed customer email + name in memory row**.
24. **payload_sha256 stored** — matches SHA-256 of raw body.
25. **Subscription events log only at slice 1** — `inv.stripe_subscription_changed` emitted; no state transition.
26. **PCI SAQ-A scope** — no card numbers in any DB column or memory row (test scans).
27. **6 memory audit kinds emit correctly** — one per path.

---

## §5 — Verification

```rust
// services/inv/tests/stripe_signature_test.rs
#[test]
fn valid_v1_signature_accepted() {
    let secret = b"whsec_test_secret";
    let body = br#"{"id":"evt_123","type":"payment_intent.succeeded","created":1700000000}"#;
    let ts = chrono::Utc::now().timestamp();
    let sig = compute_sig(secret, ts, body);
    let header = format!("t={ts},v1={sig}");
    cyberos_inv::webhook::stripe_signature::verify(body, &header, &[secret]).unwrap();
}

#[test]
fn invalid_signature_rejected() {
    let body = br#"{"id":"evt_123"}"#;
    let ts = chrono::Utc::now().timestamp();
    let bad_sig = "00".repeat(32);
    let header = format!("t={ts},v1={bad_sig}");
    assert!(matches!(
        cyberos_inv::webhook::stripe_signature::verify(body, &header, &[b"secret"]),
        Err(cyberos_inv::webhook::stripe_signature::StripeSigError::SignatureInvalid)
    ));
}

#[test]
fn v0_signature_ignored() {
    // Only v1 accepted; v0 in header must not match
    let body = br#"{"id":"evt_123"}"#;
    let ts = chrono::Utc::now().timestamp();
    let v0_sig = "ff".repeat(32);
    let header = format!("t={ts},v0={v0_sig}");
    assert!(matches!(
        cyberos_inv::webhook::stripe_signature::verify(body, &header, &[b"secret"]),
        Err(cyberos_inv::webhook::stripe_signature::StripeSigError::HeaderMalformed)
    ));
}

#[test]
fn replay_window_exceeded() {
    let body = br#"{"id":"evt_123"}"#;
    let old_ts = chrono::Utc::now().timestamp() - 400;   // > 5min
    let sig = compute_sig(b"secret", old_ts, body);
    let header = format!("t={old_ts},v1={sig}");
    assert!(matches!(
        cyberos_inv::webhook::stripe_signature::verify(body, &header, &[b"secret"]),
        Err(cyberos_inv::webhook::stripe_signature::StripeSigError::ReplayWindowExceeded)
    ));
}
```

```rust
// services/inv/tests/stripe_event_allowlist_test.rs
#[tokio::test]
async fn unknown_event_type_acked_200(ctx: TestCtx) {
    let body = ctx.make_event_body("radar.early_fraud_warning.created");
    let sig = ctx.sign_body(&body).await;
    let resp = ctx.post_stripe_webhook(&body, &sig).await;
    assert_eq!(resp.status(), 200);
    let r: StripeWebhookResponse = resp.json().await.unwrap();
    assert!(r.unknown_event_type);
    // Log row with outcome='unknown_event_type'
    let row = ctx.fetch_latest_event_log().await;
    assert_eq!(row.outcome, "unknown_event_type");
    // Stripe must NOT retry — verified by the 200 status
}

#[tokio::test]
async fn payment_intent_succeeded_processed(ctx: TestCtx) {
    let body = ctx.make_payment_intent_succeeded(/* amount */ 4999, /* currency */ "USD");
    let sig = ctx.sign_body(&body).await;
    let resp = ctx.post_stripe_webhook(&body, &sig).await;
    assert_eq!(resp.status(), 200);
    let receipts = ctx.fetch_payment_receipts().await;
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0].receipt_source, "stripe");
    assert_eq!(receipts[0].amount_minor, 4999);
    assert_eq!(receipts[0].currency, "USD");
    let rows = ctx.memory_audit_rows("inv.stripe_payment_received").await;
    assert_eq!(rows.len(), 1);
}
```

```rust
// services/inv/tests/stripe_idempotent_test.rs
#[tokio::test]
async fn duplicate_event_id_returns_existing(ctx: TestCtx) {
    let body = ctx.make_payment_intent_succeeded(1000, "USD");
    let sig = ctx.sign_body(&body).await;
    let r1: StripeWebhookResponse = ctx.post_stripe_webhook_json(&body, &sig).await;
    let r2: StripeWebhookResponse = ctx.post_stripe_webhook_json(&body, &sig).await;
    assert_eq!(r1.log_id, r2.log_id);
    assert!(!r1.duplicate);
    assert!(r2.duplicate);
    let dup_rows = ctx.memory_audit_rows("inv.duplicate_stripe_event_received").await;
    assert_eq!(dup_rows.len(), 1);
    let receipts = ctx.fetch_payment_receipts().await;
    assert_eq!(receipts.len(), 1, "no duplicate receipt row");
}
```

```rust
// services/inv/tests/stripe_multi_currency_test.rs
#[tokio::test]
async fn supported_currencies_all_process(ctx: TestCtx) {
    for currency in ["USD", "EUR", "SGD", "GBP"] {
        let body = ctx.make_payment_intent_succeeded_with_unique_id(1000, currency);
        let sig = ctx.sign_body(&body).await;
        let resp = ctx.post_stripe_webhook(&body, &sig).await;
        assert_eq!(resp.status(), 200);
    }
    let receipts = ctx.fetch_payment_receipts().await;
    assert_eq!(receipts.len(), 4);
}

#[tokio::test]
async fn unsupported_currency_rejected(ctx: TestCtx) {
    let body = ctx.make_payment_intent_succeeded(1000, "JPY");
    let sig = ctx.sign_body(&body).await;
    let resp = ctx.post_stripe_webhook(&body, &sig).await;
    assert_eq!(resp.status(), 400);
    let rows = ctx.memory_audit_rows("inv.stripe_event_rejected").await;
    assert!(rows.iter().any(|r| r["reason"] == "currency_unsupported"));
}
```

```rust
// services/inv/tests/stripe_append_only_log_test.rs
#[sqlx::test]
async fn event_log_update_blocked(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_stripe_event_log(&pool).await;
    let err = sqlx::query("UPDATE stripe_event_log SET outcome = 'duplicate' WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton; 6 memory row builders follow the canonical pattern.)

---

## §7 — Dependencies

**Upstream:**
- **FR-AUTH-101** — CFO role for secret rotation.

**Downstream (2 placeholders):**
- **FR-INV-006** — cash application reads payment_receipts.
- **FR-TEN-003** — Stripe billing integration consumes the event log for subscription state.

**Cross-module:**
- **FR-INV-001** — invoices table FK target for `invoice_id`.
- **FR-INV-005** — shares the `payment_receipts` schema + `receipt_source` enum.
- **FR-AI-003** — memory audit bridge.
- **FR-MEMORY-111** — PII scrubbing.
- **FR-OBS-007** — sev-2 alarm rule.

---

## §8 — Example payloads

### 8.1 — Stripe webhook event (payment_intent.succeeded)

```json
{
  "id": "evt_1NXxx2L8aBfH9qZK",
  "object": "event",
  "type": "payment_intent.succeeded",
  "livemode": true,
  "created": 1700000000,
  "data": {
    "object": {
      "id": "pi_3NXxx2L8aBfH9qZK",
      "amount": 4999,
      "currency": "usd",
      "customer": "cus_NgYx2L8aBfH9qZ",
      "metadata": { "invoice_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d" }
    }
  }
}
```

### 8.2 — Stripe-Signature header

```
Stripe-Signature: t=1700000050,v1=5257a869e7ecebeda32affa62cdca3fa51cad7e77a0e56ff536d0ce8e108d8bd
```

### 8.3 — inv.stripe_payment_received memory row

```json
{
  "kind": "inv.stripe_payment_received",
  "tenant_id": "5e8f1d2a-...",
  "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "stripe_event_id": "evt_1NXxx2L8aBfH9qZK",
  "amount_minor": 4999,
  "currency": "USD",
  "invoice_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "customer_id_stripe": "cus_NgYx2L8aBfH9qZ",
  "customer_email_hash16": "abc123def4567890",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — inv.stripe_event_rejected memory row

```json
{
  "kind": "inv.stripe_event_rejected",
  "tenant_id": "5e8f1d2a-...",
  "stripe_event_id": "evt_1NXxx2L8aBfH9qZK",
  "reason": "signature_invalid",
  "payload_sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "ts_ns": 1747920731000000000
}
```

### 8.5 — 200 OK response

```json
{
  "status": "ok",
  "log_id": 12345,
  "duplicate": false,
  "unknown_event_type": false
}
```

---

## §9 — Open questions

Deferred:
- **Stripe Connect (multi-account marketplace)** — FR-INV-2xx.
- **Cash application logic** — FR-INV-006.
- **Subscription state tracking** — FR-TEN-003.
- **Cross-currency FX reconciliation** — FR-INV-002.
- **Refund partial-allocation** — FR-INV-2xx.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid signature | constant-time compare | 401 + audit | Designed |
| Missing Stripe-Signature | handler | 401 | Designed |
| Header malformed (no v1) | parser | 401 | Designed |
| Replay window exceeded | ts check | 401 + audit | Designed |
| Duplicate event.id | UNIQUE + handler lookup | 200 + dup audit | Designed |
| Tenant unknown slug | lookup | 404 + audit | Designed |
| Unknown event_type | enum parse | 200 + log unknown | Designed |
| Currency unsupported | enum check | 400 + audit | Add via ADR |
| Livemode mismatch | env check | 400 + audit | Designed |
| Malformed JSON body | serde error | 400 + audit | Designed |
| UPDATE stripe_event_log from app | SQL grant | permission denied | Designed |
| KMS decrypt fail | aws-sdk error | 500 + sev-1 | KMS health |
| Secret rotation timing gap | 60s overlap window | Designed | None |
| > 10 rejections/h | counter alarm | sev-2 | Operator investigation |
| Invoice metadata missing | optional field | invoice_id=NULL | FR-INV-006 cash app |
| Customer FK missing | optional field | customer column NULL | None |
| memory audit fail mid-tx | rollback | 500; Stripe retries | memory_writer health |
| OTel span attribute missing | otel_test | CI fails | Fix |
| Refund without original payment_intent | log + emit | Manual reconciliation | FR-INV-006 |
| Cross-tenant FK | RLS | 0 rows | Designed |
| Cached secret stale during rotation | dual-secret check | Designed | None |
| Concurrent webhook for same event_id | UNIQUE | One wins; second sees duplicate | Designed |
| Stripe API breaking change | log unknown event_type | 200 ack + log | Add to allowlist via ADR |
| payload > 1MB | body limit | 413 | Designed |
| `data.object` shape changes | per-handler parser tolerant | Designed | None |
| Stripe portal URL drift | per-tenant URL stable | None | Designed |
| Concurrent rotation attempts | partial unique | Second fails | Caller retries |
| Receipt amount negative on non-refund | dispatch enforces | Should never happen | Test asserts |
| Refund larger than original | charge.amount_refunded validation | Allowed (Stripe handles) | None |
| Webhook arrives before payment_intent (race) | metadata.invoice_id lookup may fail | invoice_id=NULL | FR-INV-006 |

---

## §11 — Implementation notes

- **Stripe v1 signature scheme** — `HMAC-SHA256(secret, "{t}.{body}")`; constant-time compare.
- **v0 deprecated** — only v1 parsed; v0 header → malformed.
- **Per-tenant URL** — slug routes to per-tenant secret without body inspection.
- **Closed 8-event allowlist** — closed enum; unknown event_types still 200-ack'd but logged.
- **200 OK on unknown event_type** — Stripe must not retry; we just log for review.
- **Idempotency on event.id** — Stripe-guaranteed uniqueness; UNIQUE constraint enforces.
- **5-minute replay window** — Stripe spec default.
- **60s secret rotation overlap** — same pattern as VietQR webhook.
- **Multi-currency BIGINT minor** — USD/EUR/SGD/GBP at slice 1; ADR to add.
- **payload_sha256 stored, not body** — bodies can be 5-50KB.
- **Livemode validation** — production-vs-test environment separation.
- **metadata.invoice_id linking** — fast-path receipt-to-invoice match.
- **Refunds as negative amount_minor** — preserves cause-effect in ledger.
- **Subscription events log only at slice 1** — FR-TEN-003 consumes for state.
- **6 memory audit kinds** — received / rejected / payment_received / refunded / subscription_changed / duplicate.
- **PII scrub customer.email + name** — chain holds hashed.
- **Sev-2 at > 10 rejections/h** — matches VietQR threshold.
- **PCI SAQ-A scope** — no card data anywhere; Stripe carries PCI scope.
- **`bank_code='STRIPE'`** — extends enum via ALTER TYPE.
- **Default arm = log + 200 in match** — defense against future Stripe additions.
- **No Stripe Connect at slice 1** — single platform account.

---

*End of FR-INV-003.*
