---
id: TASK-INV-005
title: "INV VietQR / Napas247 webhook handler — HMAC-SHA256 signature + idempotent receipt insert + reference memo parsing + append-only ledger + memory audit"
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
module: inv
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AUTH-003, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-101, TASK-INV-001, TASK-INV-006, TASK-REW-009, TASK-OBS-007]
depends_on: [TASK-AUTH-101]
blocks: [TASK-INV-006, TASK-TEN-102, TASK-REW-009, TASK-ESOP-004]

source_pages:
  - website/docs/modules/inv.html#vietqr-napas247
  - website/docs/modules/inv.html#webhook
  - https://napas.com.vn (Napas247 webhook spec referenced)
source_decisions:
  - DEC-380 (HMAC-SHA256 signature verification mandatory on every webhook; HMAC secret per-tenant rotated 90-day cadence; missing/invalid signature → 401 + sev-2 alarm)
  - DEC-381 (idempotency keyed by Napas247 `transaction_reference` (TXN ref); duplicate → 200 OK with existing receipt id; never double-credit)
  - DEC-382 (REVOKE UPDATE, DELETE on payment_receipts from cyberos_app — append-only at SQL grant)
  - DEC-383 (reference memo parsing — extracts invoice id from `HD<digits>` or `INV<digits>` prefix in `transfer_memo`; unparseable → receipt persisted with `invoice_id=NULL` for manual matching by TASK-INV-006)
  - DEC-384 (closed bank_code enum at 15 Vietnamese banks via Napas247 routing: VCB, BIDV, VIB, TCB, MBB, ACB, CTG, AGB, STB, HDB, VPB, SHB, TPB, EIB, OCB; adding a 16th is an ADR)
  - DEC-385 (currency hard-coded to VND for this handler; multi-currency webhooks route to TASK-INV-003 Stripe / TASK-INV-004 Wise)
  - DEC-386 (amount stored as BIGINT đồng — VND has no minor unit so 1 đồng = 1 minor; FLOAT forbidden per task-audit skill rule 11)
  - DEC-387 (webhook URL is per-tenant — `https://api.cyberos.world/v1/inv/webhooks/vietqr/<tenant_slug>` so Napas247 routes correctly without cross-tenant body inspection)
  - DEC-388 (memory audit kinds: inv.payment_received, inv.webhook_rejected, inv.payment_matched_to_invoice, inv.payment_unmatched, inv.duplicate_webhook_received)
  - DEC-389 (replay window: HMAC payload includes `ts` field; reject webhooks with `ts` more than 5 minutes off server time — prevents replay attacks)
  - DEC-390 (acknowledgement to Napas247 within 5 seconds — webhook handler is fast-path; expensive matching happens async in TASK-INV-006)
  - DEC-391 (sev-2 alarm on > 10 webhook rejections per tenant per hour — suggests HMAC secret rotation issue or attack)
  - PDPL Art. 13 (data minimisation — sender_name/account is PII; scrubbed in memory chain)
  - Decree 53/2022 (VN data localisation — VN-tenant payment data residency-pinned per TASK-AI-016)
  - Napas247 webhook spec (proprietary; not publicly versioned)

language: rust 1.81 + sql
service: cyberos/services/inv/
new_files:
  # payment_receipts table + bank_code enum + RLS + REVOKE writes
  - services/inv/migrations/0010_payment_receipts.sql
  # per-tenant HMAC secret with rotation history
  - services/inv/migrations/0011_webhook_secrets.sql
  # POST /v1/inv/webhooks/vietqr/{tenant_slug} handler
  - services/inv/src/webhook/vietqr.rs
  # HMAC-SHA256 verify + timestamp window check
  - services/inv/src/webhook/hmac.rs
  # Napas247 transaction_reference idempotency cache (Postgres-backed)
  - services/inv/src/webhook/idempotency.rs
  # transfer_memo → invoice_id parser (regex)
  - services/inv/src/parser/memo.rs
  # append-only writer
  - services/inv/src/repo/payment_receipts.rs
  # secret CRUD + rotation
  - services/inv/src/repo/webhook_secrets.rs
  # canonical inv.* memory row builders (5 kinds for this task)
  - services/inv/src/audit/inv_events.rs
  # PaymentReceipt, BankCode (15) enum, ReceiptSource enum
  - services/inv/src/types.rs
  # valid HMAC + new TXN → 200 + payment_receipts row + memory row
  - services/inv/tests/vietqr_webhook_happy_test.rs
  # invalid signature → 401 + inv.webhook_rejected
  - services/inv/tests/vietqr_webhook_hmac_test.rs
  # duplicate TXN ref → 200 + no duplicate row
  - services/inv/tests/vietqr_webhook_idempotent_test.rs
  # ts > 5min old → 401
  - services/inv/tests/vietqr_webhook_replay_test.rs
  # HD123 / INV456 / unparseable cases
  - services/inv/tests/memo_parser_test.rs
  # 15 banks; 16th rejected
  - services/inv/tests/bank_code_closed_test.rs
  # UPDATE/DELETE rejected from cyberos_app
  - services/inv/tests/append_only_receipts_test.rs
  # wrong tenant_slug → 404
  - services/inv/tests/per_tenant_url_test.rs
  # non-VND currency → 400 currency_unsupported
  - services/inv/tests/vnd_only_test.rs
  # ack within 5s p95
  - services/inv/tests/webhook_perf_test.rs
  # every path emits correct memory kind
  - services/inv/tests/audit_emission_test.rs
modified_files:
  # add payment_receipts + webhook_secrets to TENANT_SCOPED_TABLES
  - services/auth/src/rls/templates.rs

allowed_tools:
  - file_read: services/inv/**
  - file_read: services/auth/src/rls/**
  - file_write: services/inv/{src,tests,migrations}/**
  - bash: cd services/inv && cargo test webhook

disallowed_tools:
  - skip HMAC verification (per DEC-380 — every webhook must be authenticated)
  - allow UPDATE on payment_receipts (per DEC-382 — append-only)
  - allow multi-currency in this handler (per DEC-385 — VND only)
  - perform expensive matching synchronously in the webhook handler (per DEC-390 — must ack within 5s)
  - store amounts as FLOAT (per task-audit skill rule 11)
  - accept unsigned webhooks via env override (per DEC-380 — no escape hatch)

effort_hours: 6
subtasks:
  - "0.7h: 0010_payment_receipts.sql — table + bank_code enum + RLS + REVOKE writes"
  - "0.5h: 0011_webhook_secrets.sql — per-tenant HMAC secret + rotation history"
  - "0.7h: webhook/vietqr.rs — handler with per-tenant URL routing"
  - "0.6h: webhook/hmac.rs — HMAC-SHA256 verify + ts window"
  - "0.5h: webhook/idempotency.rs — TXN reference cache"
  - "0.4h: parser/memo.rs — invoice id regex extractor"
  - "0.4h: repo/payment_receipts.rs — append-only writer"
  - "0.3h: repo/webhook_secrets.rs — secret CRUD"
  - "0.4h: audit/inv_events.rs — 5 row builders"
  - "0.3h: types.rs — closed enums"
  - "1.2h: tests — 11 test files"

risk_if_skipped: "Without VietQR/Napas247 webhook capture, every VN-domestic payment requires manual ledger entry — operationally infeasible at any volume. TASK-INV-006 (cash application) needs the receipts table as its input; without DEC-381's idempotency, a Napas247 retry double-credits the invoice; without DEC-380's HMAC verification, attackers spoof payments and credit fraudulent invoices; without DEC-382's append-only ledger, a malicious app handler could rewrite payment history; without DEC-383's memo parser, every receipt requires manual matching (~30 sec each, ~$3 cost per receipt). TASK-REW-009 (payroll batch send) similarly depends on the receipt format. The 6h effort lands the audit-grade payment-capture primitive."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship the VietQR / Napas247 webhook handler at `POST /v1/inv/webhooks/vietqr/{tenant_slug}` with HMAC-SHA256 signature verification + idempotent receipt insertion + reference memo parsing. Each requirement:

1. **MUST** define `payment_receipts` table with: `id UUID PRIMARY KEY`, `tenant_id UUID NOT NULL`, `receipt_source receipt_source NOT NULL` (closed enum: `vietqr_napas247` for this task; `stripe`/`wise` for TASK-INV-003/004), `bank_code bank_code NOT NULL` (closed enum per DEC-384), `transaction_reference TEXT NOT NULL` (Napas247 TXN ref; idempotency key), `amount_minor BIGINT NOT NULL CHECK (amount_minor > 0)` (đồng; per DEC-386), `currency CHAR(3) NOT NULL CHECK (currency = 'VND')`, `sender_account TEXT` (PII; scrubbed in audit), `sender_name TEXT` (PII; scrubbed in audit), `transfer_memo TEXT`, `invoice_id UUID REFERENCES invoices(id)` (nullable; populated by memo parser OR TASK-INV-006 cash application), `received_at TIMESTAMPTZ NOT NULL`, `napas_payload_sha256 CHAR(64) NOT NULL` (hash of original webhook body for forensic replay), `webhook_ts TIMESTAMPTZ NOT NULL` (timestamp claimed by Napas247), `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`. UNIQUE `(tenant_id, transaction_reference)`.

2. **MUST** declare the closed `receipt_source` Postgres enum with exactly 3 values: `'vietqr_napas247'`, `'stripe'`, `'wise'`. Adding a 4th source is an ADR.

3. **MUST** declare the closed `bank_code` Postgres enum with exactly 15 values (per DEC-384 + Napas247 routing): `'VCB', 'BIDV', 'VIB', 'TCB', 'MBB', 'ACB', 'CTG', 'AGB', 'STB', 'HDB', 'VPB', 'SHB', 'TPB', 'EIB', 'OCB'`. Adding a 16th is an ADR.

4. **MUST** ship the `webhook_secrets` table with: `tenant_id UUID PRIMARY KEY`, `secret_kms_blob BYTEA NOT NULL` (KMS-encrypted), `kms_key_id TEXT NOT NULL`, `created_at TIMESTAMPTZ NOT NULL`, `rotated_at TIMESTAMPTZ`, `status TEXT NOT NULL CHECK (status IN ('active','rotated','revoked'))`. Per-tenant HMAC secret with rotation history; partial unique index `(tenant_id) WHERE status='active'` ensures one active secret.

5. **MUST** enforce RLS with both `USING` and `WITH CHECK` on `payment_receipts` and `webhook_secrets`. Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

6. **MUST** be **append-only** on `payment_receipts` (per DEC-382 + task-audit skill rule 12). `REVOKE UPDATE, DELETE ON payment_receipts FROM cyberos_app`. The only mutation is `invoice_id` update which is handled by a privileged `inv_cash_applier` role (granted to TASK-INV-006's job).

7. **MUST** expose `POST /v1/inv/webhooks/vietqr/{tenant_slug}` (per DEC-387). The per-tenant URL routes correctly without cross-tenant body inspection. Unknown slug → 404 `tenant_unknown`. The handler:
- Looks up tenant_id from slug.
- Verifies HMAC-SHA256 signature per §1 #8.
- Checks replay window per §1 #9.
- Checks idempotency per §1 #10.
- Parses transfer_memo for invoice reference per §1 #11.
- INSERT into payment_receipts (atomic with memory audit emit).
- Returns 200 OK within 5s p95 (per DEC-390).

8. **MUST** verify HMAC-SHA256 signature (per DEC-380). Signature in header `X-Napas-Signature: hex(HMAC-SHA256(secret, body))`. The handler:
- Loads per-tenant active secret from `webhook_secrets` (KMS-decrypted).
- Computes HMAC-SHA256 over the raw body bytes (NOT the parsed JSON — preserves byte-for-byte).
- Compares constant-time via `subtle::ConstantTimeEq`.
- Mismatch → 401 `signature_invalid` + emit `inv.webhook_rejected` memory row with `reason='signature_invalid'`.

9. **MUST** validate replay window (per DEC-389). The body MUST contain a `ts` field (ISO-8601 UTC). The handler checks `|server_now - ts| ≤ 5 minutes`. Outside window → 401 `webhook_expired` + emit `inv.webhook_rejected` with `reason='replay_window_exceeded'`.

10. **MUST** enforce idempotency on `transaction_reference` (per DEC-381). Lookup `payment_receipts WHERE tenant_id=$1 AND transaction_reference=$2`. If exists → 200 OK with existing receipt id; NO new row; emit `inv.duplicate_webhook_received` memory row (sev-3 informational). Never double-credit.

11. **MUST** parse `transfer_memo` for invoice reference (per DEC-383). Regex: `^(HD|INV)(\d{6,12})\b`. Match → `invoice_id` looked up via `invoices WHERE tenant_id=$1 AND invoice_number=$2`; if invoice exists in tenant scope, set `invoice_id` column. No match OR invoice not found → `invoice_id = NULL` (TASK-INV-006 cash application handles later).

12. **MUST** emit memory audit rows on every webhook outcome (per DEC-388):
- `inv.payment_received` — happy path; carries `{receipt_id, tenant_id, bank_code, amount_minor, currency, transaction_reference, transfer_memo_scrubbed, invoice_id, ts_ns}`.
- `inv.webhook_rejected` — HMAC fail / replay window / parse error; carries `{tenant_id, reason, source_ip_hash16, payload_sha256, ts_ns}`.
- `inv.payment_matched_to_invoice` — when memo parser matches an invoice; carries `{receipt_id, invoice_id, matched_via='memo_parser', ts_ns}`.
- `inv.payment_unmatched` — when memo parser yields no invoice; carries `{receipt_id, transfer_memo_scrubbed, ts_ns}`.
- `inv.duplicate_webhook_received` — idempotency hit; sev-3.

13. **MUST** PII-scrub `sender_account`, `sender_name`, `transfer_memo` via TASK-MEMORY-111 BEFORE chain commit. Raw values retained in tenant-scoped Postgres rows; chain holds scrubbed.

14. **MUST** acknowledge to Napas247 within 5 seconds p95 (per DEC-390). The handler does minimal synchronous work — HMAC verify + idempotency check + INSERT + audit row. Expensive matching (multiple invoice candidates, fuzzy memo) is delegated to TASK-INV-006's async cash applier.

15. **MUST** alarm at sev-2 on > 10 webhook rejections per tenant per hour (per DEC-391). Counter: `inv_webhook_rejected_total{tenant_id, reason}`; OBS rule fires when rolling-1h sum exceeds 10. Reasons may include rotated secret not yet propagated, attack attempt, or Napas247 bug.

16. **MUST** support HMAC secret rotation via `POST /v1/inv/webhook-secrets/rotate` (caller MUST have role `cfo` per TASK-AUTH-101). Rotation:
- Generates new 32-byte random secret.
- KMS-encrypts with tenant's KMS key.
- INSERT new row with `status='active'`.
- UPDATE prior active row to `status='rotated'` with `rotated_at=now()`.
- Per partial unique index — only one active at a time.
- 60-second overlap: both old + new secrets accepted (operator coordinates with Napas247 portal).
- After 60s, only new secret accepted.

17. **MUST** complete handler in ≤ 200 ms p95 measured server-side (excluding network). Performance test `webhook_perf_test` asserts.

18. **MUST** emit OTel span `inv.webhook.vietqr` with attributes: `tenant_id`, `bank_code`, `outcome` (success | signature_invalid | replay_window_exceeded | tenant_unknown | currency_unsupported | duplicate | parse_error | not_vnd).

19. **MUST** emit OTel metrics:
- `inv_webhook_received_total{tenant_id, bank_code, outcome}` (counter).
- `inv_webhook_rejected_total{tenant_id, reason}` (counter — sev-2 alarm at > 10/h).
- `inv_payment_amount_minor{tenant_id, currency}` (counter — sum of amounts; for revenue dashboards).
- `inv_webhook_latency_ms` (histogram; SLO p95 < 200ms).
- `inv_payment_matched_total{tenant_id, matched_via}` (counter; matched_via ∈ {memo_parser, manual, cash_applier}).

20. **MUST** ensure HMAC secret retrieval is **cached** (in-memory) with 60-second TTL — secret rotation must propagate within 60s. Cache is per-process; multiple gateway instances each maintain their own. The handler ALWAYS validates against both old + new during the 60s overlap window.

21. **MUST** reject non-VND currencies (per DEC-385). Body `currency != 'VND'` → 400 `currency_unsupported` + emit `inv.webhook_rejected` with `reason='currency_unsupported'`. Multi-currency goes through TASK-INV-003 (Stripe) or TASK-INV-004 (Wise).

22. **MUST** validate bank_code against the closed 15-bank enum (per DEC-384). Unknown bank → 400 `unknown_bank_code` + emit `inv.webhook_rejected`.

23. **MUST** persist the raw webhook body's SHA-256 hash (`napas_payload_sha256`) for forensic replay (per §1 #1). The full body is NOT stored (could be large); the hash + the structured fields are sufficient for "did this exact webhook arrive?" reconstruction.

24. **MUST** use the `inv_cash_applier` SQL role for TASK-INV-006's later mutations of `payment_receipts.invoice_id`. The role:
- Has UPDATE on `payment_receipts (invoice_id ONLY)` (column-level grant).
- Cannot UPDATE other columns.
- Cannot DELETE.

25. **MUST** emit a `inv.webhook_rejected` memory row with `reason='tenant_unknown'` even when the slug is invalid (logging-only path; no tenant_id; uses `tenant_id = nil-uuid` for the chain). This catches probing attacks against URL paths.

26. **MUST** ensure `received_at = NOW()` server-side at INSERT (not from Napas payload — Napas's `ts` is in `webhook_ts` column separately). This prevents memo-time spoofing.

---

## §2 — Why this design (rationale for humans)

**Why HMAC-SHA256 with constant-time compare (DEC-380, §1 #8)?** Webhook authentication is the trust boundary. Without HMAC verification, anyone can POST a fake payment and credit a fraudulent invoice. Constant-time comparison (`subtle::ConstantTimeEq`) prevents timing-attack signature recovery. The HMAC secret is per-tenant + KMS-encrypted + rotatable per DEC-380.

**Why per-tenant URL (DEC-387, §1 #7)?** Single URL receiving all tenants' webhooks would force the handler to (a) decode the body before knowing which tenant's secret to use, (b) look up tenant inside the request body (cross-tenant body inspection risk). Per-tenant URL via slug lets the handler resolve tenant + secret BEFORE touching the body. Operationally simpler too — each tenant configures their own URL in Napas247 portal.

**Why idempotency on transaction_reference (DEC-381, §1 #10)?** Napas247 retries on network failure or 5xx response. Without idempotency, retries double-credit. The `transaction_reference` is Napas247's unique identifier per transaction — making it the dedup key. UNIQUE constraint enforces at DB; lookup-then-INSERT pattern in handler.

**Why append-only payment_receipts (DEC-382)?** Payment receipts are financial records. UPDATE in place would let operators silently amend the ledger. Append-only via SQL grant means even a buggy or malicious handler can't rewrite history. The exception (`invoice_id` mutation via privileged `inv_cash_applier` role) is column-level grant — minimum necessary privilege.

**Why memo parser at write time (DEC-383, §1 #11)?** Fast-matching the common case (memo with `HD123456` prefix) at webhook receipt time means the receipt is immediately linked to its invoice — no waiting for the async cash applier. The 80% of receipts with parseable memos go straight to matched state; the 20% with ambiguous or missing memos fall to TASK-INV-006 for manual or fuzzy matching.

**Why closed 15-bank enum (DEC-384, §1 #3, §1 #22)?** Napas247 routes from these 15 major Vietnamese banks. Allowing free-form bank_code would let invalid/typo values into the ledger. Adding a 16th (e.g. a new bank joining Napas247) is an ADR that forces consideration of cross-module impact (rate cards, KYC, etc.).

**Why VND-only for this handler (DEC-385, §1 #21)?** VietQR + Napas247 ARE the VND domestic rail. Stripe + Wise handle multi-currency. Forcing a single handler to handle all currencies would mean (a) different webhook signature schemes, (b) different reference formats, (c) different bank metadata. Separation by source is the correct factoring.

**Why amount as BIGINT đồng (DEC-386, §1 #1)?** task-audit skill rule 11. VND has no minor unit (1 đồng = 1 minor); a 1M-đồng payment is stored as 1000000. FLOAT would introduce rounding in the rare large-payment case (~10B đồng).

**Why 5-minute replay window (DEC-389, §1 #9)?** Replay attacks (attacker captures legitimate webhook + replays later) are mitigated by requiring fresh timestamps. 5 minutes covers clock skew + network delays + reasonable retry windows. Outside that window, the webhook is rejected with `webhook_expired`.

**Why 5-second acknowledgement budget (DEC-390, §1 #14)?** Napas247 considers webhook delivery failed if not ack'd within their timeout. The handler must do minimum work synchronously: HMAC + idempotency + INSERT + audit. Expensive operations (multiple invoice candidates, fuzzy matching, downstream notifications) are async via TASK-INV-006.

**Why sev-2 alarm at > 10 rejections/hour (DEC-391, §1 #15)?** Normal operation produces zero rejections (assuming HMAC secret in sync). A burst suggests (a) operator rotated secret but didn't update Napas247 portal, (b) attacker probing, (c) Napas247 misroute. Sev-2 prompts operator investigation within an hour.

**Why per-tenant HMAC secret with rotation (§1 #4, §1 #16)?** Shared secret across tenants means one tenant's leaked secret compromises all. Per-tenant + rotation gives compartmentalisation + recovery path. The 60-second overlap window during rotation handles the inevitable timing gap between rotating in our system and updating Napas247 portal.

**Why sender_account + sender_name PII-scrubbed in audit (§1 #13)?** Account numbers + names are PII. memory audit chain is more broadly queried than tenant Postgres; storing PII in chain creates PII-everywhere problem. Postgres rows retain raw for in-tenant queries; chain holds scrubbed forms.

**Why `inv.webhook_rejected` memory row even on tenant_unknown (§1 #25)?** Probing attacks scan for valid URLs. Logging probes (with `tenant_id = nil-uuid` since we don't have a real tenant) lets us see attack patterns. Without this, probing would be invisible — the 404 returns silently.

**Why memo parser regex `^(HD|INV)(\d{6,12})\b` (§1 #11)?** VN convention: invoice references prefixed `HD<n>` (hóa đơn) or `INV<n>`. 6-12 digits covers invoice number ranges; word boundary prevents false matches on longer strings. The regex is tight — false-positive matching is worse than missed parses (which fall to manual/fuzzy in TASK-INV-006).

**Why payload SHA-256 stored not the raw body (§1 #23)?** Bodies can be large (~5-20 KB); storing the full payload per receipt inflates the ledger. The SHA-256 is sufficient for "did this exact webhook arrive?" — forensic operations can reconstruct fields from the structured columns. For full body retention, TASK-OBS-006 tail-sampled OTel spans hold the raw body transiently (< 30 days).

**Why `received_at = NOW()` server-side (§1 #26)?** Napas247's `ts` is in `webhook_ts` separately. If `received_at` were from the payload, an attacker (assuming HMAC bypass — defense in depth) could spoof receipt time. Server-side ensures the receipt timestamp is authoritative.

**Why `inv_cash_applier` role separate from `cyberos_app` (§1 #24, §1 #6)?** Cash application is privileged — it links payments to invoices, affecting AR aging + revenue recognition. Splitting roles means the webhook handler (general app role) cannot perform cash app; TASK-INV-006's batch job uses the privileged role. Column-level grant (UPDATE only on `invoice_id`) is minimum necessary privilege.

**Why HMAC secret cached 60s (§1 #20)?** KMS decrypt is ~10-50ms per call; caching per process avoids that latency on every webhook. 60s TTL bounds the propagation delay for rotation. Multiple gateway instances each cache independently — eventual consistency across instances is acceptable.

**Why webhook URL with slug not tenant UUID (§1 #7)?** Slugs are human-readable + URL-safe + short. Napas247 portal operators configuring the URL prefer slugs. The slug → tenant_id lookup is a single indexed query.

---

## §3 — API contract

### 3.1 — Migration 0010 — payment_receipts

```sql
-- services/inv/migrations/0010_payment_receipts.sql

BEGIN;

CREATE TYPE receipt_source AS ENUM ('vietqr_napas247', 'stripe', 'wise');
CREATE TYPE bank_code AS ENUM (
    'VCB', 'BIDV', 'VIB', 'TCB', 'MBB', 'ACB', 'CTG', 'AGB', 'STB',
    'HDB', 'VPB', 'SHB', 'TPB', 'EIB', 'OCB'
);

CREATE TABLE payment_receipts (
    id                       UUID         PRIMARY KEY,
    tenant_id                UUID         NOT NULL,
    receipt_source           receipt_source NOT NULL,
    bank_code                bank_code    NOT NULL,
    transaction_reference    TEXT         NOT NULL,
    amount_minor             BIGINT       NOT NULL CHECK (amount_minor > 0),
    currency                 CHAR(3)      NOT NULL CHECK (currency = 'VND'),
    sender_account           TEXT,
    sender_name              TEXT,
    transfer_memo            TEXT,
    invoice_id               UUID,                                    -- FK to invoices(id) — added in TASK-INV-001 dependency-resolution migration
    received_at              TIMESTAMPTZ  NOT NULL DEFAULT now(),
    napas_payload_sha256     CHAR(64)     NOT NULL CHECK (napas_payload_sha256 ~ '^[0-9a-f]{64}$'),
    webhook_ts               TIMESTAMPTZ  NOT NULL,
    created_at               TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX uniq_payment_txnref ON payment_receipts (tenant_id, transaction_reference);
CREATE INDEX payment_receipts_invoice_idx ON payment_receipts (invoice_id) WHERE invoice_id IS NOT NULL;
CREATE INDEX payment_receipts_tenant_received_idx ON payment_receipts (tenant_id, received_at DESC);

ALTER TABLE payment_receipts ENABLE ROW LEVEL SECURITY;
CREATE POLICY payment_receipts_tenant_isolation ON payment_receipts
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

-- Append-only (DEC-382)
REVOKE UPDATE, DELETE ON payment_receipts FROM cyberos_app;

-- Privileged role for cash application (TASK-INV-006)
CREATE ROLE inv_cash_applier;
GRANT INSERT ON payment_receipts TO cyberos_app;   -- handler inserts; baseline grant
GRANT UPDATE (invoice_id) ON payment_receipts TO inv_cash_applier;   -- only invoice_id mutable
GRANT SELECT ON payment_receipts TO cyberos_app, inv_cash_applier;

COMMIT;
```

### 3.2 — Migration 0011 — webhook_secrets

```sql
-- services/inv/migrations/0011_webhook_secrets.sql

BEGIN;

CREATE TABLE webhook_secrets (
    id                  UUID         PRIMARY KEY,
    tenant_id           UUID         NOT NULL,
    secret_kms_blob     BYTEA        NOT NULL,
    kms_key_id          TEXT         NOT NULL,
    status              TEXT         NOT NULL CHECK (status IN ('active','rotated','revoked')),
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    rotated_at          TIMESTAMPTZ
);

CREATE UNIQUE INDEX uniq_active_secret ON webhook_secrets (tenant_id) WHERE status = 'active';
CREATE INDEX webhook_secrets_tenant_idx ON webhook_secrets (tenant_id, created_at DESC);

ALTER TABLE webhook_secrets ENABLE ROW LEVEL SECURITY;
CREATE POLICY webhook_secrets_tenant_isolation ON webhook_secrets
    USING (tenant_id = current_setting('auth.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);

REVOKE UPDATE, DELETE ON webhook_secrets FROM cyberos_app;
-- A separate `inv_secret_rotator` role used by the rotation handler.
CREATE ROLE inv_secret_rotator;
GRANT INSERT, UPDATE (status, rotated_at) ON webhook_secrets TO inv_secret_rotator;

COMMIT;
```

### 3.3 — HMAC verifier

```rust
// services/inv/src/webhook/hmac.rs
use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use chrono::{DateTime, Duration, Utc};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, thiserror::Error)]
pub enum HmacError {
    #[error("signature_invalid")]
    SignatureInvalid,
    #[error("replay_window_exceeded")]
    ReplayWindowExceeded,
    #[error("missing_signature")]
    MissingSignature,
    #[error("invalid_hex_signature")]
    InvalidHex,
}

pub fn verify(body_bytes: &[u8], signature_hex: &str, ts: DateTime<Utc>, secrets: &[&[u8]]) -> Result<(), HmacError> {
    // Replay window check
    let now = Utc::now();
    let skew = (now - ts).num_seconds().abs();
    if skew > 300 {
        return Err(HmacError::ReplayWindowExceeded);
    }

    let expected = hex::decode(signature_hex).map_err(|_| HmacError::InvalidHex)?;

    // Try each secret (during rotation overlap, both old + new are valid)
    for secret in secrets {
        let mut mac = HmacSha256::new_from_slice(secret).expect("valid secret length");
        mac.update(body_bytes);
        let computed = mac.finalize().into_bytes();
        if computed.as_slice().ct_eq(&expected).into() {
            return Ok(());
        }
    }
    Err(HmacError::SignatureInvalid)
}
```

### 3.4 — Webhook handler

```rust
// services/inv/src/webhook/vietqr.rs
use axum::{Json, body::Bytes, extract::{Path, State}, http::{HeaderMap, StatusCode}};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct NapasWebhookBody {
    pub transaction_reference: String,
    pub bank_code: String,
    pub amount: i64,                    // đồng (VND has no minor)
    pub currency: String,
    pub sender_account: Option<String>,
    pub sender_name: Option<String>,
    pub transfer_memo: Option<String>,
    pub ts: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct WebhookResponse {
    pub status: &'static str,
    pub receipt_id: Uuid,
    pub duplicate: bool,
}

pub async fn vietqr_webhook(
    State(state): State<AppState>,
    Path(tenant_slug): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<WebhookResponse>), ApiError> {
    // 1. Tenant lookup from slug
    let tenant_id = state.repo.find_tenant_by_slug(&tenant_slug).await?
        .ok_or_else(|| {
            // Emit probe-detection audit row even on tenant_unknown (§1 #25)
            tokio::spawn(audit_unknown_tenant(tenant_slug.clone()));
            ApiError::TenantUnknown
        })?;

    // 2. HMAC verify
    let signature = headers.get("X-Napas-Signature")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiError::MissingSignature)?;

    // Parse body for ts FIRST (need it for replay window)
    let parsed: NapasWebhookBody = serde_json::from_slice(&body)
        .map_err(|_| ApiError::InvalidPayload)?;

    // Fetch current + previous secret (60s overlap window)
    let secrets = state.secret_cache.get_active_and_overlap(tenant_id).await?;
    let secret_refs: Vec<&[u8]> = secrets.iter().map(|s| s.as_slice()).collect();

    if let Err(e) = crate::webhook::hmac::verify(&body, signature, parsed.ts, &secret_refs) {
        crate::audit::inv_events::emit_webhook_rejected(tenant_id, format!("{e:?}"), &body, &headers).await;
        return Err(ApiError::from(e));
    }

    // 3. Currency check
    if parsed.currency != "VND" {
        crate::audit::inv_events::emit_webhook_rejected(tenant_id, "currency_unsupported".into(), &body, &headers).await;
        return Err(ApiError::CurrencyUnsupported);
    }

    // 4. Bank code validation
    let bank: crate::types::BankCode = parsed.bank_code.parse().map_err(|_| {
        let body_clone = body.clone();
        tokio::spawn(async move { /* emit unknown_bank_code audit */ });
        ApiError::UnknownBankCode
    })?;

    // 5. Idempotency check
    if let Some(existing_id) = state.repo.find_receipt_by_txn_ref(tenant_id, &parsed.transaction_reference).await? {
        crate::audit::inv_events::emit_duplicate_webhook(tenant_id, existing_id, &parsed.transaction_reference).await;
        return Ok((StatusCode::OK, Json(WebhookResponse {
            status: "ok", receipt_id: existing_id, duplicate: true
        })));
    }

    // 6. Memo parse
    let invoice_id = crate::parser::memo::extract_invoice_id(&parsed.transfer_memo.as_deref().unwrap_or(""))
        .and_then(|inv_num| state.repo.find_invoice_by_number(tenant_id, &inv_num).ok().flatten());

    // 7. Compute payload hash
    let payload_sha = hex::encode(Sha256::digest(&body));

    // 8. INSERT receipt + emit audit
    let receipt_id = Uuid::new_v4();
    let mut tx = state.db.begin().await?;
    sqlx::query(r#"
        INSERT INTO payment_receipts
        (id, tenant_id, receipt_source, bank_code, transaction_reference, amount_minor, currency,
         sender_account, sender_name, transfer_memo, invoice_id, received_at,
         napas_payload_sha256, webhook_ts)
        VALUES ($1, $2, 'vietqr_napas247'::receipt_source, $3::bank_code, $4, $5, 'VND',
                $6, $7, $8, $9, now(), $10, $11)
    "#)
    .bind(receipt_id).bind(tenant_id).bind(bank.as_str())
    .bind(&parsed.transaction_reference).bind(parsed.amount).bind(parsed.sender_account.as_deref())
    .bind(parsed.sender_name.as_deref()).bind(parsed.transfer_memo.as_deref())
    .bind(invoice_id).bind(&payload_sha).bind(parsed.ts)
    .execute(&mut *tx).await?;

    crate::audit::inv_events::emit_payment_received(
        &mut tx, tenant_id, receipt_id, bank, parsed.amount, &parsed.transaction_reference,
        invoice_id, parsed.transfer_memo.as_deref(),
    ).await?;
    if let Some(inv_id) = invoice_id {
        crate::audit::inv_events::emit_matched_to_invoice(&mut tx, tenant_id, receipt_id, inv_id, "memo_parser").await?;
    } else {
        crate::audit::inv_events::emit_payment_unmatched(&mut tx, tenant_id, receipt_id, parsed.transfer_memo.as_deref()).await?;
    }

    tx.commit().await?;

    Ok((StatusCode::OK, Json(WebhookResponse {
        status: "ok", receipt_id, duplicate: false
    })))
}
```

### 3.5 — Memo parser

```rust
// services/inv/src/parser/memo.rs
use once_cell::sync::Lazy;
use regex::Regex;

static MEMO_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)\b(HD|INV)(\d{6,12})\b").unwrap());

pub fn extract_invoice_id(memo: &str) -> Option<String> {
    MEMO_RE.captures(memo).map(|c| {
        let prefix = c.get(1).unwrap().as_str().to_uppercase();
        let number = c.get(2).unwrap().as_str();
        format!("{prefix}{number}")
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_hd_prefix() {
        assert_eq!(extract_invoice_id("Thanh toan HD123456 cho ACME"), Some("HD123456".into()));
    }
    #[test]
    fn parses_inv_prefix() {
        assert_eq!(extract_invoice_id("Payment INV789012"), Some("INV789012".into()));
    }
    #[test]
    fn case_insensitive_prefix() {
        assert_eq!(extract_invoice_id("payment hd456789"), Some("HD456789".into()));
    }
    #[test]
    fn no_match_returns_none() {
        assert_eq!(extract_invoice_id("just some text"), None);
    }
    #[test]
    fn rejects_short_number() {
        assert_eq!(extract_invoice_id("HD123"), None);  // < 6 digits
    }
}
```

---

## §4 — Acceptance criteria

1. **receipt_source enum closed at 3** — `vietqr_napas247`, `stripe`, `wise`.
2. **bank_code enum closed at 15** — exact set per DEC-384.
3. **RLS isolates by tenant** — cross-tenant queries return 0 rows.
4. **POST webhook happy path** — valid HMAC + new TXN → 200 + payment_receipts row + `inv.payment_received` memory row.
5. **HMAC invalid** → 401 `signature_invalid` + `inv.webhook_rejected` memory row.
6. **HMAC missing header** → 401 `missing_signature`.
7. **Replay window exceeded** (ts > 5min off) → 401 `webhook_expired`.
8. **Idempotent on TXN ref** — duplicate POST → 200 + existing receipt_id + no duplicate row + `inv.duplicate_webhook_received` row.
9. **Tenant unknown slug** → 404 + `inv.webhook_rejected` memory row with `reason=tenant_unknown`.
10. **Memo HD123456** → matched_via=memo_parser; `invoice_id` set; `inv.payment_matched_to_invoice` row.
11. **Memo unparseable** → invoice_id NULL; `inv.payment_unmatched` row.
12. **Currency != VND** → 400 `currency_unsupported` + audit row.
13. **Bank code unknown** → 400 `unknown_bank_code` + audit row.
14. **UPDATE payment_receipts blocked from cyberos_app** → permission denied.
15. **DELETE payment_receipts blocked** → permission denied.
16. **UPDATE invoice_id allowed from inv_cash_applier role** — column-level grant works.
17. **UPDATE other columns from inv_cash_applier blocked** — only invoice_id allowed.
18. **Secret rotation creates new active + sets prior to rotated** — partial unique index allows.
19. **60s overlap window** — webhook signed with prior secret still accepted for 60s after rotation.
20. **After 60s only new secret valid** — prior secret rejects.
21. **Webhook ack < 5s p95** — handler returns within budget.
22. **Server-side received_at** — Napas-supplied ts in `webhook_ts`; receipt's received_at is server NOW().
23. **Payload SHA-256 stored** — `napas_payload_sha256` matches `SHA-256(raw body)`.
24. **OTel span emitted** — `inv.webhook.vietqr` with outcome.
25. **Counter `inv_webhook_received_total{bank_code=VCB, outcome=success}` increments** — per webhook.
26. **Counter `inv_webhook_rejected_total{reason=signature_invalid}` increments** on bad HMAC.
27. **Sev-2 alarm at > 10 rejections/h** — OBS rule fires.

---

## §5 — Verification

```rust
// services/inv/tests/vietqr_webhook_hmac_test.rs
#[tokio::test]
async fn invalid_signature_returns_401(ctx: TestCtx) {
    let body = ctx.napas_body_template();
    let bad_sig = "0000000000000000000000000000000000000000000000000000000000000000";
    let resp = ctx.post_webhook(&body, bad_sig).await;
    assert_eq!(resp.status(), 401);
    let rows = ctx.memory_audit_rows("inv.webhook_rejected").await;
    assert!(rows.iter().any(|r| r["reason"] == "signature_invalid"));
}

#[tokio::test]
async fn valid_signature_returns_200(ctx: TestCtx) {
    let body = ctx.napas_body_template();
    let sig = ctx.sign_body(&body).await;
    let resp = ctx.post_webhook(&body, &sig).await;
    assert_eq!(resp.status(), 200);
    let receipts: Vec<PaymentReceipt> = ctx.fetch_receipts().await;
    assert_eq!(receipts.len(), 1);
}
```

```rust
// services/inv/tests/vietqr_webhook_idempotent_test.rs
#[tokio::test]
async fn duplicate_txnref_returns_existing(ctx: TestCtx) {
    let body = ctx.napas_body_with_txnref("TXN12345").await;
    let sig = ctx.sign_body(&body).await;
    let r1: WebhookResponse = ctx.post_webhook_json(&body, &sig).await.unwrap();
    let r2: WebhookResponse = ctx.post_webhook_json(&body, &sig).await.unwrap();
    assert_eq!(r1.receipt_id, r2.receipt_id);
    assert!(!r1.duplicate);
    assert!(r2.duplicate);
    let receipts = ctx.fetch_receipts().await;
    assert_eq!(receipts.len(), 1, "no duplicate row");
    let dup_rows = ctx.memory_audit_rows("inv.duplicate_webhook_received").await;
    assert_eq!(dup_rows.len(), 1);
}
```

```rust
// services/inv/tests/memo_parser_test.rs
#[test]
fn parses_canonical_prefixes() {
    use cyberos_inv::parser::memo::extract_invoice_id;
    assert_eq!(extract_invoice_id("Thanh toan HD123456"), Some("HD123456".into()));
    assert_eq!(extract_invoice_id("Pay INV987654"), Some("INV987654".into()));
}

#[test]
fn rejects_invalid_lengths() {
    use cyberos_inv::parser::memo::extract_invoice_id;
    assert_eq!(extract_invoice_id("HD12345"), None);     // 5 digits < 6
    assert_eq!(extract_invoice_id("HD1234567890123"), None);  // 13 digits > 12
}

#[test]
fn no_match_when_memo_blank() {
    use cyberos_inv::parser::memo::extract_invoice_id;
    assert_eq!(extract_invoice_id(""), None);
    assert_eq!(extract_invoice_id("Just some random text"), None);
}
```

```rust
// services/inv/tests/append_only_receipts_test.rs
#[sqlx::test]
async fn update_blocked_from_cyberos_app(pool: sqlx::PgPool) {
    set_role_app(&pool).await;
    let id = seed_receipt(&pool).await;
    let err = sqlx::query("UPDATE payment_receipts SET amount_minor = 99999 WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}

#[sqlx::test]
async fn cash_applier_can_update_invoice_id(pool: sqlx::PgPool) {
    set_role_cash_applier(&pool).await;
    let receipt = seed_receipt(&pool).await;
    let invoice = seed_invoice(&pool).await;
    sqlx::query("UPDATE payment_receipts SET invoice_id = $2 WHERE id = $1")
        .bind(receipt).bind(invoice).execute(&pool).await.unwrap();
}

#[sqlx::test]
async fn cash_applier_cannot_update_other_columns(pool: sqlx::PgPool) {
    set_role_cash_applier(&pool).await;
    let id = seed_receipt(&pool).await;
    let err = sqlx::query("UPDATE payment_receipts SET amount_minor = 99999 WHERE id = $1")
        .bind(id).execute(&pool).await.unwrap_err();
    assert!(format!("{err}").contains("permission denied"));
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

**Upstream:**
- **TASK-AUTH-101** — RBAC; CFO role for secret rotation.

**Downstream (2 placeholders):**
- **TASK-INV-006** — cash application; reads payment_receipts + matches to invoices.
- **TASK-REW-009** — VietQR payroll batch send; uses the same TXN reference shape.

**Cross-module:**
- **TASK-AI-003** — memory audit bridge.
- **TASK-MEMORY-111** — PII scrubbing.
- **TASK-INV-001** — invoices table provides FK target.
- **TASK-OBS-007** — sev-2 alarm on > 10 rejections/h.

---

## §8 — Example payloads

### 8.1 — POST /v1/inv/webhooks/vietqr/acme-corp

```http
POST /v1/inv/webhooks/vietqr/acme-corp HTTP/1.1
Content-Type: application/json
X-Napas-Signature: a1b2c3d4e5f6...64hex chars total...

{
  "transaction_reference": "TXN20260516001",
  "bank_code": "VCB",
  "amount": 4500000,
  "currency": "VND",
  "sender_account": "1234567890",
  "sender_name": "NGUYEN VAN A",
  "transfer_memo": "Thanh toan HD123456 cho ACME",
  "ts": "2026-05-16T10:00:00Z"
}
```

### 8.2 — 200 OK response

```json
{ "status": "ok", "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M", "duplicate": false }
```

### 8.3 — inv.payment_received memory row

```json
{
  "kind": "inv.payment_received",
  "tenant_id": "5e8f1d2a-...",
  "receipt_id": "01HG7V8B0K8M4Z8Z8M8M8M8M8M",
  "bank_code": "VCB",
  "amount_minor": 4500000,
  "currency": "VND",
  "transaction_reference": "TXN20260516001",
  "transfer_memo_scrubbed": "Thanh toan HD123456 cho [REDACTED-COMPANY]",
  "invoice_id": "HD123456",
  "sender_account_hash16": "abc123",
  "ts_ns": 1747920731000000000
}
```

### 8.4 — inv.webhook_rejected memory row

```json
{
  "kind": "inv.webhook_rejected",
  "tenant_id": "5e8f1d2a-...",
  "reason": "signature_invalid",
  "source_ip_hash16": "def456",
  "payload_sha256": "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
  "ts_ns": 1747920731000000000
}
```

---

## §9 — Open questions

Deferred:
- **Cash application (memo parser fallback fuzzy matching)** — TASK-INV-006.
- **Multi-currency webhooks** — TASK-INV-003 Stripe + TASK-INV-004 Wise.
- **Per-tenant rate limiting on webhook endpoint** — slice 3 (currently rely on edge rate limiter).
- **Webhook secret rotation UI** — slice 3.
- **Bank-specific transaction_reference formats** — task-INV-2xx if needed.

All other questions resolved.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid HMAC signature | constant-time compare | 401 + audit | Designed |
| Missing X-Napas-Signature header | handler | 401 missing_signature | Designed |
| Hex decode failure | hex::decode | 401 invalid_hex_signature | Designed |
| Replay window exceeded (> 5min off) | ts comparison | 401 webhook_expired + audit | Designed |
| Duplicate TXN ref | UNIQUE + handler lookup | 200 + dup audit | Designed |
| Tenant unknown slug | lookup fail | 404 + audit | Designed |
| Currency not VND | handler check | 400 currency_unsupported + audit | Use Stripe/Wise instead |
| Bank code unknown | enum parse | 400 unknown_bank_code + audit | Add bank via ADR |
| UPDATE payment_receipts from app | SQL grant | permission denied | Designed |
| DELETE payment_receipts | SQL grant | permission denied | Designed |
| KMS decrypt failure | error response | 500 internal | KMS health investigation |
| Secret rotation propagation lag | 60s overlap window | None — both old + new accepted | Designed |
| > 10 rejections/h sustained | counter alarm | sev-2 | Operator investigation |
| Payload too large (> 1MB) | body limit | 413 payload_too_large | Designed |
| Memo with > 12 digit invoice number | regex no-match | invoice_id NULL | TASK-INV-006 manual matching |
| Memo with multiple invoice refs | regex first-match | first ref wins | Designed (single-match heuristic) |
| Invoice referenced doesn't exist in tenant | lookup fail | invoice_id NULL | TASK-INV-006 |
| Cross-tenant invoice FK | RLS | 0 rows; treated as no match | Designed |
| memory audit emit fails mid-tx | rollback | 500; webhook retries | memory_writer health |
| Cached secret stale (rotation just happened) | dual-secret check | Designed | None |
| Two webhooks for same TXN concurrent | UNIQUE | One wins; second sees duplicate | Designed |
| Cash applier role tries to UPDATE amount | SQL grant | permission denied | Designed |
| Cache TTL miss + KMS hit storm | 60s TTL bounds | Brief latency spike | None — acceptable |
| Body bytes mutated between HMAC + parse | Bytes is immutable | Designed | None |
| Webhook clock drift > 5min | replay window | 401 | Operator fixes clock |
| Per-tenant URL hijack (slug typo at portal) | wrong tenant_id resolved | 401 (HMAC mismatch) | Operator fixes URL |
| OTel span attribute missing | otel_attrs_test | CI fails | Fix builder |
| Payload SHA-256 mismatch on replay | mismatch test | Sev-3 | Forensic investigation |
| napas_payload_sha256 CHECK violation | DB regex | INSERT fails | Fix hex format |
| Secret KMS blob corrupted | decrypt fail | 500 + sev-1 | Rotate secret |
| Concurrent secret rotation | partial unique | Second rotation fails | Caller retries |

---

## §11 — Implementation notes

- **HMAC-SHA256 with `subtle::ConstantTimeEq`** — timing-attack-resistant comparison.
- **Body bytes (not parsed JSON) for HMAC** — preserves byte-for-byte signature semantics.
- **Per-tenant URL routing** — slug → tenant_id lookup before body touch.
- **60-second overlap during secret rotation** — handles inevitable timing gap with Napas247 portal update.
- **Idempotency keyed by TXN ref + tenant_id** — UNIQUE constraint enforces.
- **Append-only via SQL grant** — `REVOKE UPDATE, DELETE FROM cyberos_app`.
- **`inv_cash_applier` role column-level UPDATE on invoice_id** — minimum necessary privilege.
- **Memo regex tight** — `^(HD|INV)(\d{6,12})\b` — false-negative preferred over false-positive.
- **5-minute replay window** — covers clock skew + retry windows.
- **Payload SHA-256 stored not body** — saves space; sufficient for "did this exact webhook arrive?" replay.
- **Server-side `received_at`** — authoritative; `webhook_ts` separately captures Napas claim.
- **Sev-2 alarm at > 10 rejections/h** — operator investigation prompt.
- **5 memory audit kinds covering all paths** — happy + rejected + matched + unmatched + duplicate.
- **PII scrubbing on sender_account, sender_name, transfer_memo** — chain holds scrubbed.
- **VND hard-coded** — multi-currency goes through Stripe/Wise.
- **15 closed bank codes** — ADR for additions.
- **Per-tenant HMAC secret rotation** — `cfo` role required for rotation handler.
- **5s ack budget** — handler does minimum sync work; TASK-INV-006 handles async matching.
- **Probe-detection audit row even on tenant_unknown** — captures attack patterns.
- **`napas_payload_sha256` CHECK constraint** — enforces hex format.

---

*End of TASK-INV-005.*
