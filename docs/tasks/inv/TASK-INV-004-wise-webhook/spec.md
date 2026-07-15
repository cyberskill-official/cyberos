---
id: TASK-INV-004
title: "Wise webhook handler for multi-currency receipts (USD / EUR / GBP / SGD / JPY)"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: INV
priority: p1
status: ready_to_implement
accepted_at: 2026-05-16
accepted_by: Stephen Cheng
verify: T
phase: P2
milestone: P2 · billing-substrate
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-INV-001, TASK-INV-002, TASK-INV-003, TASK-INV-005, TASK-INV-006, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-AUTH-101]
blocks: []

source_pages:
  - website/docs/modules/inv.html#wise-webhook
source_decisions:
  - DEC-840 2026-05-16 — Wise webhook signature verified via X-Signature-SHA256 + per-profile public key (Wise uses RSA-SHA256, not HMAC)
  - DEC-841 2026-05-16 — Wise public key fetched from `/v1/profiles/{profile_id}/webhook-subscriptions/{subscription_id}/public-key` once + cached + rotated on signature failure
  - DEC-842 2026-05-16 — Closed `wise_event_type` enum (transfers#state-change, balances#credit, balances#update) — exhaustive at Wise's 2025-Q1 API
  - DEC-843 2026-05-16 — Idempotency via UNIQUE (profile_id, event_id) — Wise event_id is 36-char UUID
  - DEC-844 2026-05-16 — Replay defense: reject events with `data.occurred_at` more than 5 days old (Wise retry window)
  - DEC-845 2026-05-16 — Append-only `wise_webhook_events` via REVOKE UPDATE/DELETE + privileged `inv_wise_writer` role
  - DEC-846 2026-05-16 — Multi-currency match: receipt currency MUST equal invoice currency; never auto-convert at receipt time (FX is TASK-INV-002 SBV daily snapshot)
  - DEC-847 2026-05-16 — Per-tenant `wise_profile_id` configured at provisioning (TASK-TEN-001); one profile per tenant
  - DEC-848 2026-05-16 — Wise public key rotated yearly per Wise policy; on rotation, fetch new key + verify signature against both old and new for 24h overlap window
  - DEC-849 2026-05-16 — 7 memory audit kinds with TASK-MEMORY-111 PII scrubbing
  - DEC-850 2026-05-16 — Webhook endpoint returns 200 + empty body within 5 seconds (Wise SLA); processing offloaded to background job via WAL queue
  - DEC-851 2026-05-16 — Dead-letter queue for events that fail processing 3 times; ops sev-1 alarm + manual restore path
  - DEC-852 2026-05-16 — Currency-mismatch receipt (e.g., USD wire to a VND-denominated invoice) is held in `unmatched_receipts` table for CFO review, NOT auto-matched
  - DEC-853 2026-05-16 — Closed 5-value `wise_receipt_state` enum (received, matched, currency_mismatch, dead_lettered, manually_resolved)

build_envelope:
  language: rust 1.81
  service: cyberos/services/inv/
  new_files:
    - services/inv/src/wise/mod.rs
    - services/inv/src/wise/handler.rs
    - services/inv/src/wise/signature.rs
    - services/inv/src/wise/parser.rs
    - services/inv/src/wise/public_key.rs
    - services/inv/src/wise/processor.rs
    - services/inv/migrations/0020_wise_webhook_events.sql
    - services/inv/migrations/0021_wise_unmatched_receipts.sql
    - services/inv/tests/wise_signature_test.rs
    - services/inv/tests/wise_idempotency_test.rs
    - services/inv/tests/wise_currency_mismatch_test.rs
    - services/inv/tests/wise_replay_test.rs
    - services/inv/tests/wise_key_rotation_test.rs
  modified_files:
    - services/inv/src/lib.rs (mount /v1/webhooks/wise)
  allowed_tools:
    - file_read: services/inv/**
    - file_write: services/inv/{src,tests,migrations}/**
    - bash: cargo test -p cyberos-inv
    - net: GET https://api.wise.com/v1/profiles/{id}/webhook-subscriptions/{sub}/public-key
  disallowed_tools:
    - hand-roll RSA-SHA256 verification (use ring or rsa crate)
    - log full Wise event body at any level (carries customer transfer details)
    - skip signature verification on any event
    - auto-convert currency at receipt time (FX is TASK-INV-002 SBV authoritative)

effort_hours: 6
subtasks:
  - "0.5h: wise_webhook_events migration + grants + REVOKE"
  - "0.5h: wise_unmatched_receipts migration"
  - "1.0h: RSA-SHA256 signature verification with public key cache + rotation"
  - "0.5h: closed event_type enum + parser per type"
  - "1.0h: handler — fast 200 response + WAL push to background processor"
  - "1.0h: background processor — idempotency check + invoice match (delegates to TASK-INV-006) + currency-mismatch hold"
  - "0.5h: dead-letter queue + restore handler"
  - "0.5h: 7 memory audit kinds emission + TASK-MEMORY-111 scrubbing"
  - "0.5h: integration tests with fixture events"
risk_if_skipped: "Without Wise support, international tenants paying in USD/EUR/GBP must rely on manual reconciliation; we lose the automated cash-application loop for foreign-currency receipts. TASK-INV-006 cash-app cascade is incomplete without Wise as a receipt source for international transfers."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** accept Wise webhook events at `POST /v1/webhooks/wise/{profile_id}`, verify the RSA-SHA256 signature against the per-profile public key, decode the event body, persist append-only, and dispatch to TASK-INV-006 cash application or hold for CFO review on mismatch.

1. **MUST** verify the `X-Signature-SHA256` header on every incoming webhook before any processing (DEC-840). Wise signs the request body with RSA-SHA256 using a per-profile private key; the public key is fetched from `/v1/profiles/{profile_id}/webhook-subscriptions/{subscription_id}/public-key` and cached. Verification failure returns `401 invalid_signature` + sev-2 memory audit `inv.wise_signature_invalid` (NEVER tells Wise it was invalid — return 401 generically).

2. **MUST** fetch + cache the Wise public key in-process for 24 hours (DEC-841). On the first webhook for a profile, the handler fetches the key and stores `(profile_id, pem_key, fetched_at, expires_at)` in process memory. Subsequent webhooks reuse the cache.

3. **MUST** support public-key rotation (DEC-848). On signature-verification failure, the handler re-fetches the public key once, retries verification, and accepts if the new key validates. The 24h overlap window covers Wise's key-rotation contract. Persistent failure after re-fetch is treated per clause #1.

4. **MUST** define a closed 3-value `wise_event_type` enum (`transfers_state_change`, `balances_credit`, `balances_update`) per DEC-842. Event types outside this enum are persisted to dead-letter queue with reason `unknown_event_type` + sev-2 audit. CI cardinality test asserts exactly 3.

5. **MUST** enforce idempotency via UNIQUE constraint on `(profile_id, event_id)` (DEC-843). The Wise `event_id` is a 36-char UUID. Duplicate event MUST return success without re-processing AND without emitting a duplicate memory audit row.

6. **MUST** reject events with `data.occurred_at` more than 5 days in the past (DEC-844). Wise's retry window is 5 days; older events are presumed stale. Stale event returns `200 OK` (Wise's retry stops) + sev-2 audit `inv.wise_stale_event`. The body is logged for forensic but no processing happens.

7. **MUST** persist every received event (valid + invalid) to the append-only `wise_webhook_events` table (DEC-845). The table has REVOKE UPDATE, DELETE FROM cyberos_app; only `inv_wise_writer` role holds INSERT.

8. **MUST** return `200 OK` with empty body within 5 seconds (DEC-850 + Wise SLA). The handler performs signature verification + persistence + WAL-queue push, then returns. Heavy processing (invoice matching, memory emission) happens in a background processor reading from the WAL queue.

9. **MUST** define a closed 5-value `wise_receipt_state` enum (`received`, `matched`, `currency_mismatch`, `dead_lettered`, `manually_resolved`) per DEC-853. CI cardinality test asserts exactly 5.

10. **MUST** match incoming transfers to invoices by exact-currency comparison (DEC-846 + DEC-852). The processor delegates to TASK-INV-006 cash-application cascade ONLY when the transfer currency matches an outstanding invoice in the same currency. Currency mismatch → state `currency_mismatch`, row written to `unmatched_receipts` table, CFO surfaces in dashboard. Never auto-convert.

11. **MUST** verify the tenant binding before processing. The webhook URL carries `profile_id`; the processor looks up `tenants.wise_profile_id = $1` (DEC-847). Profile_id not found → dead-letter + sev-1 audit `inv.wise_profile_unknown` (this should never happen for legitimate Wise traffic; signature already passed).

12. **MUST** route failed-3-times events to the dead-letter queue (DEC-851). The processor tracks retry count per event; on 3rd failure, the event moves to `dead_lettered` state + sev-1 audit `inv.wise_dead_lettered` + ops alarm. Manual restore is available via `POST /v1/admin/wise-events/{id}/restore` for CFO role.

13. **MUST** scrub all Wise event body fields containing customer names, account numbers, or bank references through TASK-MEMORY-111 before memory audit emission (DEC-849). Postgres holds raw (RLS-scoped); chain holds scrubbed.

14. **MUST** emit 7 closed memory audit kinds:
    - `inv.wise_received` (sev-3, per valid event)
    - `inv.wise_matched` (sev-3, on successful cash-app match)
    - `inv.wise_signature_invalid` (sev-2, per verification fail)
    - `inv.wise_stale_event` (sev-2, per occurred_at > 5d)
    - `inv.wise_currency_mismatch` (sev-2, per mismatch)
    - `inv.wise_dead_lettered` (sev-1, per 3-fail)
    - `inv.wise_profile_unknown` (sev-1, per unmapped profile_id)
    - `inv.wise_key_rotated` (sev-2, per cache refresh due to verification failure)

15. **MUST** support concurrent webhook delivery without deadlock. Each request opens a short transaction (insert + WAL push). The background processor uses `pg_advisory_xact_lock(event_id)` to prevent double-processing of the same event from a retry storm.

16. **MUST** validate request body schema against the Wise event structure for each event_type. Missing required fields (e.g., `data.resource.id` for transfers#state-change) → 400 Bad Request + sev-2 audit `inv.wise_schema_invalid`.

17. **MUST** support per-tenant Wise profile rotation (e.g., tenant changes their Wise account). The handler reads the active `wise_profile_id` from `tenants` at request time; the previous profile's webhook URL gracefully returns 410 GONE after a 7-day deprecation window managed via `tenants.wise_profile_deprecated_at` column.

18. **MUST** expose `GET /v1/admin/wise-events` for CFO role to list events with filters (state, tenant_id, date range). RLS-scoped to caller's tenant unless CyberSkill founder.

19. **MUST** expose `GET /v1/admin/wise-events/{id}` for CFO to inspect a single event including raw body (Postgres-side, RLS-scoped). The handler scrubs the body through TASK-MEMORY-111 only for chain-bound audit rows; the API response shows the raw row content to the authorized CFO.

20. **MUST** record currency-mismatch events in `unmatched_receipts (id, tenant_id, source, source_event_id, currency, amount_minor, occurred_at, notes, resolved_at, resolved_by)`. The CFO has a `POST /v1/admin/unmatched-receipts/{id}/resolve` endpoint to mark resolution + record a note (≥ 10 chars).

21. **MUST** validate the per-tenant `wise_profile_id` at tenant provisioning (TASK-TEN-001 integration). The profile_id is a 12-digit integer per Wise; format-validated at policy save.

22. **MUST** reject webhook URLs where the path's `profile_id` does not match the body's `data.resource.profile_id` for transfer events. URL-vs-body profile_id mismatch returns `400 profile_id_mismatch` + sev-2 audit `inv.wise_profile_mismatch`.

23. **MUST** enforce TLS 1.3 on the webhook endpoint (no TLS 1.2 fallback). Wise sends from documented IP ranges; we additionally allowlist the Wise IP ranges at the Cloudflare/edge layer as a defense-in-depth bonus (out of scope for this task, listed in implementation notes for downstream wiring).

24. **MUST** apply rate limiting per profile_id: max 100 webhooks per second per profile. Burst beyond returns `429 TOO_MANY_REQUESTS` with `Retry-After: 1` header. Wise's documented peak rate is well under 100/s; the limit defends against malformed retries or hostile traffic.

25. **MUST** record a `webhook_received_at` and `webhook_processed_at` timestamp pair per event for SLA observability. The pair lets operators identify processing-lag incidents distinct from delivery delays.

---

## §2 — Rationale (informative — preserve all 22 paragraphs)

**§2.1  Why RSA-SHA256 signature and not HMAC.** DEC-840. Wise's design choice. Unlike Stripe (HMAC-SHA256 with shared secret) or Napas247 (HMAC-SHA256 too), Wise uses asymmetric signing — Wise holds the private key, we hold the public key. The benefit: a leaked secret on our side cannot forge signatures. The cost: we have to fetch + rotate public keys. The ring crate handles RSA-SHA256 verification safely.

**§2.2  Why fetch + cache the public key.** DEC-841 + clause #2. Each profile_id has a distinct public key. Caching means we don't fetch on every webhook (Wise's API rate limits + our latency budget). 24h TTL is generous given Wise rotates yearly; the refresh-on-failure pattern (clause #3) covers in-window rotation.

**§2.3  Why retry verification with re-fetched key on failure.** DEC-848 + clause #3. Wise's documented key rotation gives a 24h overlap window. If the cached key fails verification, the most likely cause is rotation. Re-fetching and retrying gracefully handles the rotation without ops intervention.

**§2.4  Why closed event-type enum.** DEC-842 + clause #4. Wise's API has a fixed set of event types we care about. Hardcoding the closed set ensures the handler can't be tricked into running unknown-type processing logic. New event types from Wise upgrades require a code change + DEC entry.

**§2.5  Why idempotency on (profile_id, event_id).** DEC-843 + clause #5. Wise retries delivery up to 5 days on 5xx responses. Without idempotency, every retry would re-process the same event. The UNIQUE constraint at INSERT is the cheapest enforcement; the handler treats duplicate-key as success.

**§2.6  Why 5-day staleness rejection.** DEC-844 + clause #6. Wise's documented retry window. Events older than 5 days won't be retried by Wise; if we receive one, it's either a misconfiguration or an attacker replaying. Either way, processing it serves no legitimate purpose; sev-2 audit makes the rejection visible.

**§2.7  Why background processor not synchronous.** DEC-850 + clause #8. Wise SLA: 5-second response window or webhook is retried (which then deduplicates but adds load). Synchronous processing (signature → DB → match → audit) risks the 5-second budget under DB latency or audit subprocess hangs. Fast 200 + background processing eliminates the timeout vector.

**§2.8  Why currency-mismatch held for CFO review.** DEC-852 + clause #10. A USD wire to a VND invoice is a real situation: an international customer pays in their currency for an invoice we issued in VND. We could auto-convert at the receipt time, but the FX rate used at conversion is policy-significant (SBV daily vs spot vs T+1). Holding for CFO review keeps the call human + auditable.

**§2.9  Why append-only events.** DEC-845 + clause #7. Webhook events are the wire-level forensic record. Any UPDATE/DELETE would create a "did we receive that event?" gap. Append-only + REVOKE GRANT is the defense.

**§2.10  Why dead-letter queue at 3 failures.** DEC-851 + clause #12. Transient failures (DB blip, TASK-INV-006 race) resolve with retry. Persistent failures (corrupted event, downstream bug) need human attention; auto-retrying forever consumes capacity without resolving. 3 attempts is the standard "transient vs persistent" boundary.

**§2.11  Why per-profile rate limit.** Clause #24. A misconfigured Wise customer or an attacker could try to bury us under spurious webhooks. Per-profile (not global) means one bad actor doesn't block legitimate traffic from other profiles. 100/s is well above Wise's documented peak.

**§2.12  Why URL-vs-body profile_id cross-check.** Clause #22. The URL's profile_id is part of our routing; the body's profile_id is part of the signed payload. Mismatch indicates a forged URL or a confused integration. Reject + audit.

**§2.13  Why 7-day deprecation for profile rotation.** Clause #17. Wise profile changes are operational events. 7 days lets the tenant complete in-flight transfers under the old profile before the URL goes 410. After 7 days, the old URL is hard-removed; any straggler webhooks are dead-lettered.

**§2.14  Why CFO surface for unmatched receipts.** Clause #20. Currency-mismatch is a business decision (which invoice does this USD receipt go against?). The CFO has the visibility into the customer relationship needed to make the call. Sales/AR roles see the receipt but cannot resolve it without CFO sign-off (separation of duty).

**§2.15  Why we don't auto-convert FX at receipt time.** DEC-846 + clause #10. FX timing matters for revenue recognition. Receiving USD on day 1, posting against a VND invoice on day 5: which day's FX rate? Auto-conversion forces an answer; CFO review lets the answer match the company's policy (TASK-INV-002 SBV daily snapshot, contract terms, or spot-at-receipt).

**§2.16  Why we never log full event body at any level.** Build-envelope `disallowed_tools`. Wise event bodies contain customer names, bank reference, sometimes invoice references. Bulk logging would create a PII-rich log corpus we don't want. Postgres-side raw storage is RLS-scoped; TASK-MEMORY-111 scrubs before chain emission; the API response surface for raw body is CFO-gated.

**§2.17  Why webhook returns 401 generically on signature failure.** Clause #1. Returning a more specific error ("signature mismatch", "expired key", "wrong profile") would give probing attackers feedback. Generic 401 + sev-2 memory audit (operator-visible) is the right split.

**§2.18  Why we don't share-credential authenticate (basic-auth, etc).** Wise's signed-webhook contract is sufficient; layering basic-auth would be redundant and complicate Wise's configuration. The signed body already proves origin + integrity.

**§2.19  Why per-tenant wise_profile_id and not platform-wide.** DEC-847 + clause #11. Tenants control their own Wise accounts; the webhook needs to route to the right tenant's invoices. Per-tenant profile mapping is the natural pattern.

**§2.20  Why we use `ring` or `rsa` crate not hand-rolled RSA.** Build-envelope `disallowed_tools`. RSA signature verification has subtle pitfalls (padding scheme, hash function, key parsing). Battle-tested crates avoid PKCS#1 v1.5 pitfalls and side-channel issues.

**§2.21  Why TLS 1.3 minimum.** Clause #23. TLS 1.2 is acceptable for many integrations but 1.3 is faster + has fewer cipher suite vulnerabilities. Wise supports 1.3; mandating it costs us nothing.

**§2.22  Why we don't store the raw body indefinitely.** The `wise_webhook_events.body` column carries the full event body in JSONB. Retention policy: 2 years (matches typical financial-record retention). After 2 years, a periodic job purges body to NULL while preserving the audit metadata (profile_id, event_id, event_type, occurred_at, state). The memory chain row stays forever — the chain is the long-term forensic record.

---

## §3 — API & schema

### §3.1 — Migration 0020: wise_webhook_events

```sql
-- services/inv/migrations/0020_wise_webhook_events.sql

CREATE TYPE wise_event_type AS ENUM ('transfers_state_change', 'balances_credit', 'balances_update');
CREATE TYPE wise_receipt_state AS ENUM ('received', 'matched', 'currency_mismatch', 'dead_lettered', 'manually_resolved');

CREATE TABLE wise_webhook_events (
    id                    UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    profile_id            BIGINT NOT NULL,
    event_id              CHAR(36) NOT NULL,  -- Wise UUID
    event_type            wise_event_type NOT NULL,
    tenant_id             UUID REFERENCES tenants(id),
    received_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    processed_at          TIMESTAMPTZ,
    occurred_at           TIMESTAMPTZ NOT NULL,
    signature             TEXT NOT NULL,
    body                  JSONB NOT NULL,
    state                 wise_receipt_state NOT NULL DEFAULT 'received',
    retry_count           INTEGER NOT NULL DEFAULT 0,
    matched_invoice_id    UUID,
    matched_allocation_id UUID,
    error_reason          TEXT,
    memory_chain_hash      CHAR(64) NOT NULL CHECK (memory_chain_hash ~ '^[0-9a-f]{64}$'),
    CONSTRAINT idempotent UNIQUE (profile_id, event_id)
);

CREATE INDEX wise_events_state ON wise_webhook_events (state, received_at) WHERE state IN ('received', 'currency_mismatch');
CREATE INDEX wise_events_tenant ON wise_webhook_events (tenant_id, received_at DESC);

REVOKE UPDATE, DELETE ON wise_webhook_events FROM cyberos_app;
GRANT INSERT, SELECT, UPDATE(processed_at, state, retry_count, matched_invoice_id, matched_allocation_id, error_reason)
    ON wise_webhook_events TO inv_wise_writer;
GRANT SELECT ON wise_webhook_events TO inv_wise_reader, cfo;

ALTER TABLE wise_webhook_events ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON wise_webhook_events
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);

-- Per-tenant wise profile mapping
ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS wise_profile_id BIGINT UNIQUE,
    ADD COLUMN IF NOT EXISTS wise_profile_deprecated_at TIMESTAMPTZ;
```

### §3.2 — Migration 0021: unmatched_receipts

```sql
-- services/inv/migrations/0021_wise_unmatched_receipts.sql

CREATE TABLE unmatched_receipts (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    source          TEXT NOT NULL CHECK (source IN ('wise', 'vietqr', 'stripe', 'manual')),
    source_event_id TEXT NOT NULL,
    currency        CHAR(3) NOT NULL,
    amount_minor    BIGINT NOT NULL CHECK (amount_minor > 0),
    occurred_at     TIMESTAMPTZ NOT NULL,
    received_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes           TEXT,
    resolved_at     TIMESTAMPTZ,
    resolved_by     UUID REFERENCES subjects(id),
    resolution_notes TEXT CHECK (resolution_notes IS NULL OR length(resolution_notes) >= 10),
    memory_chain_hash CHAR(64) NOT NULL
);

CREATE INDEX unmatched_receipts_tenant ON unmatched_receipts (tenant_id, resolved_at) WHERE resolved_at IS NULL;

REVOKE UPDATE, DELETE ON unmatched_receipts FROM cyberos_app;
GRANT INSERT, SELECT, UPDATE(resolved_at, resolved_by, resolution_notes) ON unmatched_receipts TO inv_wise_writer;
GRANT SELECT ON unmatched_receipts TO cfo;

ALTER TABLE unmatched_receipts ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON unmatched_receipts
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);
```

### §3.3 — Signature verification

```rust
// services/inv/src/wise/signature.rs

use ring::signature::{UnparsedPublicKey, RSA_PKCS1_2048_8192_SHA256};
use base64::{engine::general_purpose::STANDARD, Engine};

pub fn verify_signature(public_key_pem: &[u8], body: &[u8], signature_b64: &str) -> Result<(), SignatureError> {
    let signature = STANDARD.decode(signature_b64).map_err(|_| SignatureError::InvalidEncoding)?;
    let der_key = pem_to_der(public_key_pem)?;
    let key = UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, &der_key);
    key.verify(body, &signature).map_err(|_| SignatureError::VerifyFailed)
}
```

### §3.4 — Webhook handler

```rust
// services/inv/src/wise/handler.rs

pub async fn handle_wise_webhook(
    state: State<AppState>,
    Path(profile_id): Path<u64>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let started = Instant::now();
    let sig_header = headers.get("X-Signature-SHA256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // §1 #2-3 fetch + verify with re-try on failure
    let public_key = state.wise_keys.get_or_fetch(profile_id).await?;
    if let Err(_) = signature::verify_signature(&public_key, &body, sig_header) {
        let refreshed = state.wise_keys.force_refresh(profile_id).await?;
        if signature::verify_signature(&refreshed, &body, sig_header).is_err() {
            emit_memory_audit(MemoryKind::WiseSignatureInvalid, profile_id).await;
            return StatusCode::UNAUTHORIZED;
        }
        emit_memory_audit(MemoryKind::WiseKeyRotated, profile_id).await;
    }

    // Parse + idempotency-check at INSERT
    let event: WiseEvent = match serde_json::from_slice(&body) {
        Ok(e) => e,
        Err(_) => {
            emit_memory_audit(MemoryKind::WiseSchemaInvalid, profile_id).await;
            return StatusCode::BAD_REQUEST;
        }
    };
    // §1 #6 staleness check
    if (Utc::now() - event.data.occurred_at).num_days() > 5 {
        emit_memory_audit(MemoryKind::WiseStaleEvent, profile_id).await;
        return StatusCode::OK;
    }
    // §1 #22 URL-vs-body profile_id cross-check
    if let Some(body_profile_id) = event.data.resource.profile_id {
        if body_profile_id != profile_id {
            emit_memory_audit(MemoryKind::WiseProfileMismatch, profile_id).await;
            return StatusCode::BAD_REQUEST;
        }
    }

    // Persist; INSERT race wins via UNIQUE
    let tenant_id = state.pool.tenant_for_profile(profile_id).await?;
    let insert_result = sqlx::query!(
        r#"INSERT INTO wise_webhook_events
              (profile_id, event_id, event_type, tenant_id, occurred_at, signature, body, memory_chain_hash)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
           ON CONFLICT (profile_id, event_id) DO NOTHING"#,
        profile_id as i64, &event.id, event.event_type as _, tenant_id, event.data.occurred_at, sig_header, body_as_jsonb(&body), &memory_hash
    ).execute(&state.pool).await;

    // §1 #8 WAL push then fast 200
    state.wal.push_for_processor(profile_id, event.id.clone()).await;
    if started.elapsed() > Duration::from_secs(4) {
        // SLA warning — log but still 200
        tracing::warn!(elapsed = ?started.elapsed(), "wise webhook approaching 5s SLA");
    }
    StatusCode::OK
}
```

---

## §4 — Acceptance criteria

1. `wise_event_type` enum exactly 3 values.
2. `wise_receipt_state` enum exactly 5 values.
3. Webhook with invalid signature returns 401 + sev-2 audit; no row written.
4. Webhook with valid signature returns 200 within 5s + row inserted.
5. Duplicate (profile_id, event_id) returns 200 + no second row + no second audit.
6. Event with `occurred_at` 6+ days ago returns 200 + sev-2 stale audit + no processing.
7. Event_type outside enum → dead-lettered + sev-2 audit `unknown_event_type`.
8. URL profile_id ≠ body profile_id → 400 + sev-2 audit.
9. Body schema invalid → 400 + sev-2 audit.
10. Public key cache TTL = 24h; refresh on signature failure once.
11. Signature retry after re-fetched key succeeds → sev-2 audit `wise_key_rotated`.
12. Persistent signature failure after re-fetch → 401 + sev-2.
13. Currency mismatch (USD receipt to VND invoice) → state `currency_mismatch` + unmatched_receipts row + sev-2 audit.
14. Currency match → delegates to TASK-INV-006 cash-app → sev-3 `wise_matched` audit on success.
15. 3rd processing failure → state `dead_lettered` + sev-1 audit + ops alarm.
16. CFO restore via `POST /v1/admin/wise-events/{id}/restore` resets state to `received` + new processing attempt.
17. `GET /v1/admin/wise-events` RLS-scoped to caller's tenant.
18. `wise_webhook_events` REVOKE UPDATE/DELETE confirmed at `\dp`.
19. Cross-tenant SELECT returns empty (RLS).
20. Per-tenant `wise_profile_id` validated at provisioning (12-digit integer).
21. Profile deprecation: 7-day window then URL returns 410 GONE.
22. Background processor uses `pg_advisory_xact_lock(event_id)` to prevent double-processing.
23. Per-profile rate limit: 101st request/second returns 429 with Retry-After=1.
24. TLS 1.3 enforced; TLS 1.2 client gets connection refused.
25. CFO can read raw event body via `GET /v1/admin/wise-events/{id}` (RLS-scoped); other roles cannot.
26. `unmatched_receipts.resolution_notes` ≥ 10 chars enforced; resolve handler requires CFO role.
27. memory audit row emitted per 8 closed kinds.
28. All scrubbable text routed through TASK-MEMORY-111 before chain emission.
29. Webhook latency p99 < 4s (signature verify + DB insert + WAL push).
30. Body retention 2 years; cleanup job NULLs body column while preserving metadata.

---

## §5 — Verification (CI tests)

- `cardinality_test_event_type` — 3.
- `cardinality_test_receipt_state` — 5.
- `signature_valid_test` — fixture event + correct key → 200 + row.
- `signature_invalid_test` — wrong key → 401 + sev-2.
- `signature_rotated_test` — old cached key fails + new key succeeds → 200 + sev-2.
- `idempotency_test` — second POST with same event_id → 200 + no second row.
- `staleness_test` — occurred_at = -6d → 200 + sev-2 stale + no processing.
- `unknown_event_type_test` — `transfers#unsupported` → dead-lettered + sev-2.
- `profile_mismatch_test` — URL profile != body profile → 400.
- `schema_invalid_test` — missing required field → 400.
- `currency_mismatch_test` — USD receipt + VND invoice → unmatched_receipts row + sev-2.
- `currency_match_test` — delegates to TASK-INV-006 → matched + sev-3.
- `dead_letter_test` — fail 3× → dead_lettered + sev-1.
- `restore_acl_test` — non-CFO restore → 403; CFO restore → 200 + state=received.
- `rls_isolation_test` — two tenants, cross-query empty.
- `append_only_test` — REVOKE inspection.
- `profile_id_format_test` — non-12-digit profile_id at provisioning → 400.
- `deprecation_window_test` — at +7d, URL returns 410.
- `advisory_lock_test` — concurrent processor invocations → no double-processing.
- `rate_limit_test` — 101st request/sec → 429.
- `tls_1_3_test` — TLS 1.2 client → connection refused.
- `cfo_raw_body_test` — CFO sees full body; other roles get 403.
- `resolution_notes_length_test` — < 10 chars → 400.
- `latency_test` — p99 < 4s under sustained load.

---

## §6 — File skeleton

```
services/inv/
├── src/
│   ├── wise/
│   │   ├── mod.rs              # pub re-exports
│   │   ├── handler.rs          # POST /v1/webhooks/wise/{profile_id} (§3.4)
│   │   ├── signature.rs        # RSA-SHA256 verify (§3.3)
│   │   ├── parser.rs           # event_type discriminated parser
│   │   ├── public_key.rs       # fetch + cache + rotation
│   │   ├── processor.rs        # background WAL consumer + match delegate
│   │   ├── unmatched.rs        # unmatched_receipts persistence
│   │   ├── audit.rs            # 8 memory kinds
│   │   └── error.rs            # SignatureError + ProcessError
│   └── admin/
│       ├── wise_list.rs        # GET /v1/admin/wise-events
│       ├── wise_show.rs        # GET /v1/admin/wise-events/{id}
│       ├── wise_restore.rs     # POST /v1/admin/wise-events/{id}/restore
│       └── unmatched_resolve.rs # POST /v1/admin/unmatched-receipts/{id}/resolve
├── migrations/
│   ├── 0020_wise_webhook_events.sql
│   └── 0021_wise_unmatched_receipts.sql
└── tests/
    ├── wise_signature_test.rs
    ├── wise_idempotency_test.rs
    ├── wise_currency_mismatch_test.rs
    ├── wise_replay_test.rs
    └── wise_key_rotation_test.rs
```

---

## §7 — Dependencies & blast-radius

**Depends on**: TASK-AUTH-101 (RBAC for CFO role + tenant_admin gates).

**Blocks**: TASK-INV-006 (cash-application — wise receipts feed the cascade as one source).

**Blast radius if broken**:
- **Signature bypass**: forged transfer events credit unearned invoices; sev-1.
- **Idempotency bug**: same payment counted twice on retry; CFO catches in reconciliation.
- **Currency auto-convert at receipt time**: FX timing controversy + revenue misstatement.
- **Dead-letter accumulation**: queue grows unbounded; ops alarm + manual review.

---

## §8 — Payload examples

### §8.1 — Webhook delivery

```
POST /v1/webhooks/wise/12345678
X-Signature-SHA256: BASE64_RSA_SIGNATURE
Content-Type: application/json

{
  "data": {
    "resource": {
      "id": 90019283,
      "profile_id": 12345678,
      "type": "transfer"
    },
    "current_state": "outgoing_payment_sent",
    "previous_state": "processing",
    "occurred_at": "2026-05-16T10:30:00Z"
  },
  "subscription_id": "sub-abc",
  "event_type": "transfers#state-change",
  "schema_version": "2.0.0",
  "sent_at": "2026-05-16T10:30:01Z"
}

200 OK
```

### §8.2 — Invalid signature

```
POST /v1/webhooks/wise/12345678
X-Signature-SHA256: WRONG

401 Unauthorized
```

### §8.3 — Unmatched receipt list

```
GET /v1/admin/unmatched-receipts

200 OK
[
  {
    "id": "ur_xyz",
    "source": "wise",
    "currency": "USD",
    "amount_minor": 50000,  // $500
    "occurred_at": "2026-05-16T10:30:00Z",
    "resolved_at": null
  }
]
```

### §8.4 — Restore dead-lettered event

```
POST /v1/admin/wise-events/{id}/restore
Authorization: Bearer <cfo>

200 OK
{ "state": "received", "next_attempt_at": "2026-05-16T11:00:00Z" }
```

---

## §9 — Open questions

- **OQ-1** (closed by DEC-846): no auto-convert at receipt time.
- **OQ-2** (closed by DEC-852): currency mismatch held for CFO.
- **OQ-3** (open): support Wise's `balances#statement-update` event-type? Not in scope for initial slice; revisit when CFO requests balance-reconciliation tooling.
- **OQ-4** (open): support multi-profile per tenant (e.g., one Wise profile per legal entity)? Currently one per tenant; the legal-entity workflow would need additional schema work.

---

## §10 — Failure modes (32 rows)

| # | Failure | Detection | Sev | Handler |
|---|---------|-----------|-----|---------|
| 1 | Signature verification failure | RSA verify | 2 | 401 + sev-2 |
| 2 | Public key fetch fails | HTTP error | 2 | Use cached key if still valid; sev-2 audit |
| 3 | Public key expired + fetch fails | both stale | 1 | 401 + sev-1; ops alarm |
| 4 | Event body not valid JSON | parser error | 2 | 400 + sev-2 |
| 5 | Event body schema mismatch | required field missing | 2 | 400 + sev-2 |
| 6 | URL profile_id ≠ body profile_id | cross-check | 2 | 400 + sev-2 |
| 7 | event_type outside closed enum | match | 2 | Dead-lettered + sev-2 |
| 8 | Idempotency collision (legitimate retry) | UNIQUE | 3 | 200 + no second row |
| 9 | occurred_at > 5 days old | timestamp | 2 | 200 + sev-2 stale audit |
| 10 | Currency mismatch (USD→VND) | currency compare | 2 | unmatched_receipts + sev-2 |
| 11 | TASK-INV-006 cash-app match fails | downstream error | 2 | retry_count++; eventual dead-letter |
| 12 | 3rd cash-app failure → dead_lettered | retry counter | 1 | sev-1 + ops alarm |
| 13 | profile_id not in tenants table | lookup miss | 1 | Dead-letter + sev-1 (should never happen post-sig) |
| 14 | Webhook latency > 5s | SLA monitor | 2 | Wise retries; idempotency covers |
| 15 | Wise key rotation mid-delivery | sig fail → re-fetch | 2 | Re-fetch + retry; sev-2 audit |
| 16 | TLS 1.2 client | TLS handshake | 3 | Connection refused |
| 17 | Rate limit exceeded (>100/s/profile) | rate limiter | 3 | 429 + Retry-After |
| 18 | DB outage during INSERT | sqlx error | 1 | 500; Wise retries within 5d |
| 19 | memory audit subprocess fails | timeout | 1 | WAL retry; sev-1 if exhausted |
| 20 | Background processor lock contention | pg_advisory_xact_lock | 3 | Skip + retry next tick |
| 21 | Restore by non-CFO role | RBAC | 2 | 403 + sev-2 |
| 22 | Restore with no pending event | application check | 3 | 404 |
| 23 | unmatched_receipts.resolution_notes < 10 chars | CHECK | 3 | 400 |
| 24 | Cross-tenant RLS leak | rls_isolation_test | 1 | CI blocks |
| 25 | wise_profile_id format invalid | format check | 3 | 400 at provisioning |
| 26 | wise_profile_deprecated_at expired | timestamp + middleware | 3 | 410 GONE |
| 27 | Duplicate UNIQUE INSERT race | ON CONFLICT DO NOTHING | 3 | Treated as duplicate; no error |
| 28 | Background processor dies mid-event | WAL replay on restart | 2 | Resume from advisory lock release |
| 29 | memory_chain_hash regex fails | CHECK | 1 | INSERT rejected |
| 30 | Public-key cache size unbounded | LRU max 10000 | 3 | Eviction |
| 31 | Body > 1 MiB (Wise unlikely but defensive) | request body limit | 3 | 413 Payload Too Large |
| 32 | Resolved unmatched receipt re-resolved | resolved_at check | 3 | 409 already_resolved |

---

## §11 — Implementation notes

**§11.1** RSA verification uses the `ring` crate (`RSA_PKCS1_2048_8192_SHA256` algorithm). The PEM-to-DER conversion happens once per key fetch.

**§11.2** The public key cache is `Arc<RwLock<HashMap<u64, (Vec<u8>, Instant)>>>` keyed on profile_id; the per-profile 24h TTL is per-entry.

**§11.3** The Wise API endpoint to fetch the public key requires authentication with a tenant-scoped Wise API token, stored encrypted in `tenants.wise_api_token_kms_blob` (analogous to TASK-AUTH-103 SP signing key). KMS decryption happens once at cache populate.

**§11.4** The webhook handler is mounted at `POST /v1/webhooks/wise/{profile_id}` with no auth middleware — Wise authenticates via signature. The request size limit is 1 MiB (defensive; Wise events are typically < 4 KiB).

**§11.5** The background processor is a single tokio task per gateway process consuming the WAL channel; on event arrival, it acquires `pg_advisory_xact_lock(event_id)` and processes.

**§11.6** The processor's retry policy: exponential backoff (1m, 5m, 30m); 3rd failure → dead_lettered.

**§11.7** The dead-letter restore endpoint resets retry_count to 0 + state to `received` + emits sev-2 `wise_restored` audit (under `wise_dead_lettered` kind with action=restored sub-field).

**§11.8** The `unmatched_receipts` table is shared across sources (wise, vietqr, stripe, manual); the `source` column discriminates.

**§11.9** The CFO's `wise_show` endpoint that surfaces raw body has a sev-3 `wise_raw_body_viewed` audit emission (operator-introspection visibility).

**§11.10** The 24h public-key cache TTL + the 1-hour overlap window cover ~99% of Wise rotation events without intervention.

**§11.11** Tests use a fixture RSA key pair (generated at test setup) and fixture event payloads from Wise documentation.

**§11.12** The advisory lock key is `hashtext(event_id)::bigint`; collisions are bounded by the 64-bit hash space.

**§11.13** The processor uses the `inv_wise_writer` role (not the public `cyberos_app`) so it can UPDATE the state column.

**§11.14** Rate limiting is implemented via `governor` crate per-profile token bucket; 100/s burst with 100/s refill.

**§11.15** TLS 1.3 enforcement is at the gateway/edge layer (Cloudflare); the application doesn't re-enforce — it trusts the edge for transport. Documented for ops in the deployment runbook.

**§11.16** The `wise_webhook_events.body` column has a periodic NULL-out job at 2 years (matches financial-record retention); the cleanup preserves all non-body columns.

**§11.17** The 5-day staleness check uses `Utc::now() - event.data.occurred_at` (not `received_at - occurred_at`) so a delayed delivery that was Wise-side queued isn't penalized.

**§11.18** Wise profile rotation: the previous URL stays alive for 7 days returning 410. After 7 days, the URL returns 404. The deprecation window is a tenant-side operational signal.

**§11.19** The processor delegates the cash-application match to TASK-INV-006 via a function call (not HTTP) — same Rust binary; no network hop.

**§11.20** The `processed_at` timestamp is set when the processor finishes (success or dead-letter). The pair (received_at, processed_at) computes per-event processing latency.

**§11.21** Cross-tenant CFO view requires `cyberskill_founder` role + a SECURITY DEFINER function that sets tenant_id to a sentinel value, emitting a sev-3 audit per call.

**§11.22** The webhook URL pattern (`/v1/webhooks/wise/{profile_id}`) is configured at Wise side per profile. Changing the URL requires updating the Wise subscription (out-of-band ops task).

---

*End of TASK-INV-004 spec.*
