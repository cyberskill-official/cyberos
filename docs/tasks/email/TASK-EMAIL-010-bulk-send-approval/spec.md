---
id: TASK-EMAIL-010
title: "EMAIL bulk send (≥ 10 recipients) — AM + CFO/marketing dual-approval token + suppression-list filter + rate-pacing"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: EMAIL
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-EMAIL-009, TASK-EMAIL-011, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-EMAIL-009]
blocks: []

source_pages:
  - website/docs/modules/email.html#bulk

source_decisions:
  - DEC-1490 2026-05-17 — Bulk send threshold ≥ 10 distinct recipients triggers approval workflow; below = ordinary 1:1 via TASK-EMAIL-009
  - DEC-1491 2026-05-17 — Dual-approval: AM (`engagement_admin`) + CFO/marketing (`cfo` or `marketing_admin`) must both sign before send; CHECK constraint distinct signatures
  - DEC-1492 2026-05-17 — Per-tenant rate-pace: max 5000 recipients/hour (avoid bulk-relay reputation damage)
  - DEC-1493 2026-05-17 — Suppression filter applied at send-time per recipient (TASK-EMAIL-009 suppression list); suppressed recipients silently skipped + counted
  - DEC-1494 2026-05-17 — Closed enum `bulk_status` = {drafting, pending_am_sign, pending_cfo_sign, ready_to_send, sending, completed, cancelled, failed}; cardinality 8
  - DEC-1495 2026-05-17 — Each recipient gets own outbound_message row (per TASK-EMAIL-009) for tracking; bulk row aggregates status
  - DEC-1496 2026-05-17 — memory audit kinds: email.bulk_drafted, email.bulk_am_signed, email.bulk_cfo_signed, email.bulk_sending_started, email.bulk_completed, email.bulk_cancelled, email.bulk_recipient_suppressed_skipped

build_envelope:
  language: rust 1.81
  service: cyberos/services/email/
  new_files:
    - services/email/migrations/0006_bulk_sends.sql
    - services/email/src/bulk/mod.rs
    - services/email/src/bulk/draft.rs
    - services/email/src/bulk/sign.rs
    - services/email/src/bulk/dispatch.rs
    - services/email/src/bulk/rate_pacer.rs
    - services/email/src/audit/bulk_events.rs
    - services/email/src/handlers/bulk_routes.rs
    - services/email/tests/bulk_threshold_test.rs
    - services/email/tests/bulk_dual_sign_test.rs
    - services/email/tests/bulk_same_person_blocked_test.rs
    - services/email/tests/bulk_suppression_filter_test.rs
    - services/email/tests/bulk_rate_pace_5000_per_hour_test.rs
    - services/email/tests/bulk_status_enum_test.rs
    - services/email/tests/bulk_cancellation_test.rs
    - services/email/tests/bulk_audit_emission_test.rs

  modified_files:
    - services/email/src/lib.rs

  allowed_tools:
    - file_read: services/email/**
    - file_write: services/email/{src,tests,migrations}/**
    - bash: cd services/email && cargo test bulk

  disallowed_tools:
    - skip dual-sign for ≥ 10 recipients (per DEC-1491)
    - exceed 5000/hour (per DEC-1492)
    - bypass suppression filter (per DEC-1493)

effort_hours: 5
subtasks:
  - "0.4h: 0006_bulk_sends.sql + closed enum"
  - "0.3h: bulk/mod.rs"
  - "0.4h: draft.rs"
  - "0.5h: sign.rs (AM + CFO dual)"
  - "0.5h: dispatch.rs (per-recipient TASK-EMAIL-009 spawn)"
  - "0.4h: rate_pacer.rs"
  - "0.3h: audit/bulk_events.rs"
  - "0.4h: handlers/bulk_routes.rs"
  - "1.3h: tests — 8 test files"
  - "0.5h: integration smoke"

risk_if_skipped: "Without bulk approval, mass-send accidents (wrong template, wrong recipient list) reach 1000+ recipients before anyone notices. Without DEC-1492 rate-pacing, ISP reputation damaged + delivery blocked. Without DEC-1493 suppression filter, every previously-bounced recipient gets re-spam'd. Without DEC-1491 dual-sign, single compromised AM can spam org's contact list."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship bulk send (≥10 recipients) at `services/email/src/bulk/` with AM+CFO dual-signature + 5000/hour rate-pacing + suppression filter + 8-status FSM + 7 memory audit kinds.

1. **MUST** define closed `bulk_status` enum per DEC-1494. Cardinality 8.

2. **MUST** define `bulk_sends` table at migration `0006`: `(bulk_id UUID PRIMARY KEY, tenant_id UUID NOT NULL, drafter_subject_id UUID NOT NULL, recipients_list_kms_blob BYTEA NOT NULL, recipient_count INT NOT NULL CHECK (recipient_count >= 10), subject_sha256 CHAR(64) NOT NULL, body_sha256 CHAR(64) NOT NULL, status bulk_status NOT NULL DEFAULT 'drafting', am_signer_subject_id UUID, am_signed_at TIMESTAMPTZ, cfo_signer_subject_id UUID, cfo_signed_at TIMESTAMPTZ, dispatch_started_at TIMESTAMPTZ, dispatch_completed_at TIMESTAMPTZ, sent_count INT DEFAULT 0, suppressed_count INT DEFAULT 0, failed_count INT DEFAULT 0, trace_id CHAR(32), CHECK (am_signer_subject_id IS NULL OR cfo_signer_subject_id IS NULL OR am_signer_subject_id != cfo_signer_subject_id))`.

3. **MUST** expose draft `POST /v1/email/bulk/draft` body `{ to_addrs[], subject, body }`. Validates `to_addrs.len() >= 10` (below uses TASK-EMAIL-009). Status='pending_am_sign'. Emits `email.bulk_drafted` sev-2.

4. **MUST** AM sign `POST /v1/email/bulk/{id}/sign-am`. Engagement_admin. Status → pending_cfo_sign. Emits `email.bulk_am_signed` sev-1.

5. **MUST** CFO sign `POST /v1/email/bulk/{id}/sign-cfo`. `cfo` or `marketing_admin`. CHECK distinct from AM. Status → ready_to_send. Emits `email.bulk_cfo_signed` sev-1.

6. **MUST** dispatch `POST /v1/email/bulk/{id}/dispatch`. Either signer. Transitions to 'sending'. For each recipient:
   - Check suppression list per DEC-1493; skip + count suppressed.
   - Rate-pace per DEC-1492 (max 5000/hour/tenant via Redis sliding-window).
   - Invoke TASK-EMAIL-009 send.
   - Track sent_count/failed_count.
   - Final: status='completed'.
   - Emits `email.bulk_sending_started` + per-suppressed `email.bulk_recipient_suppressed_skipped` (sampled 1%) + `email.bulk_completed`.

7. **MUST** support cancel `POST /v1/email/bulk/{id}/cancel`. AM or CFO. Allowed in {pending_am_sign, pending_cfo_sign, ready_to_send}; not in sending/completed. Emits `email.bulk_cancelled` sev-1.

8. **MUST** rate-pace 5000/hour/tenant per DEC-1492. Pacer delays dispatch when exceeded.

9. **MUST** emit 7 memory audit kinds per DEC-1496.

10. **MUST NOT** allow same person AM+CFO sign (CHECK).

11. **MUST NOT** bypass suppression filter.

---

## §2 — Why this design

**Why dual-sign (DEC-1491)?** Bulk send blast radius makes single-approver failure catastrophic. Two distinct roles = defense-in-depth.

**Why 5000/hour cap (DEC-1492)?** ISP gateway limits + reputation pacing; above triggers anti-spam responses.

**Why suppression filter at send (DEC-1493)?** Re-spamming bounced recipients = primary reputation damage source.

---

## §3 — API contract

```sql
CREATE TYPE bulk_status AS ENUM ('drafting','pending_am_sign','pending_cfo_sign','ready_to_send','sending','completed','cancelled','failed');

CREATE TABLE bulk_sends (
  bulk_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  drafter_subject_id UUID NOT NULL,
  recipients_list_kms_blob BYTEA NOT NULL,
  recipient_count INT NOT NULL CHECK (recipient_count >= 10),
  subject_sha256 CHAR(64) NOT NULL,
  body_sha256 CHAR(64) NOT NULL,
  status bulk_status NOT NULL DEFAULT 'drafting',
  am_signer_subject_id UUID,
  am_signed_at TIMESTAMPTZ,
  cfo_signer_subject_id UUID,
  cfo_signed_at TIMESTAMPTZ,
  dispatch_started_at TIMESTAMPTZ,
  dispatch_completed_at TIMESTAMPTZ,
  sent_count INT NOT NULL DEFAULT 0,
  suppressed_count INT NOT NULL DEFAULT 0,
  failed_count INT NOT NULL DEFAULT 0,
  cancellation_reason TEXT,
  trace_id CHAR(32),
  CHECK (am_signer_subject_id IS NULL OR cfo_signer_subject_id IS NULL
         OR am_signer_subject_id != cfo_signer_subject_id)
);
ALTER TABLE bulk_sends ENABLE ROW LEVEL SECURITY;
CREATE POLICY bulk_sends_rls ON bulk_sends
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON bulk_sends FROM cyberos_app;
GRANT UPDATE (status, am_signer_subject_id, am_signed_at, cfo_signer_subject_id, cfo_signed_at,
              dispatch_started_at, dispatch_completed_at, sent_count, suppressed_count, failed_count,
              cancellation_reason) ON bulk_sends TO cyberos_app;
```

---

## §4 — Acceptance criteria
1. **bulk_status cardinality 8**. 2. **< 10 recipients routes to TASK-EMAIL-009 path**. 3. **Drafted status set**. 4. **AM sign transitions**. 5. **CFO sign requires distinct subject**. 6. **Same person AM+CFO blocked (CHECK)**. 7. **Dispatch filters suppression**. 8. **Rate-pacing at 5000/hour**. 9. **Cancel before send works**. 10. **Cancel during send blocked**. 11. **7 memory audit kinds emitted**. 12. **Per-recipient TASK-EMAIL-009 row created**. 13. **Sent/suppressed/failed counts accurate**. 14. **PII scrub** subject/body/recipients via TASK-MEMORY-111. 15. **Trace_id end-to-end**. 16. **Cross-tenant RLS**. 17. **Marketing_admin role accepted for CFO slot**. 18. **Recipient_count CHECK ≥ 10 enforced**. 19. **Recipients KMS-encrypted at rest**. 20. **Audit on each transition**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn dual_sign_distinct_required() {
    let ctx = TestContext::new().await;
    let bulk = ctx.draft_bulk(12).await;
    ctx.as_am().sign_am(bulk).await;
    let r = ctx.as_am().sign_cfo(bulk).await;  // same person
    assert_eq!(r.status(), 400);  // CHECK fires at DB
}

#[tokio::test]
async fn suppression_filter_at_dispatch() {
    let ctx = TestContext::new().await;
    ctx.add_suppression(ctx.tenant_id, "blocked@example.com", "manual").await;
    let bulk = ctx.draft_bulk_with(vec!["a@x.com", "b@x.com", "blocked@example.com", /*7 more*/]).await;
    ctx.as_am().sign_am(bulk).await;
    ctx.as_cfo().sign_cfo(bulk).await;
    ctx.dispatch(bulk).await;
    let row: (i32, i32) = sqlx::query_as("SELECT sent_count, suppressed_count FROM bulk_sends WHERE bulk_id=$1")
        .bind(bulk).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(row.1, 1);  // blocked counted as suppressed
}

#[tokio::test]
async fn rate_pace_5000_per_hour() {
    let ctx = TestContext::new().await;
    let bulk = ctx.draft_bulk(6000).await;
    ctx.dual_sign(bulk).await;
    let start = Instant::now();
    ctx.dispatch(bulk).await;
    // First 5000 dispatched quickly; remaining 1000 paced over next hour
    assert!(ctx.dispatched_count_at(bulk, Duration::from_secs(60)).await >= 4500);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-EMAIL-009.
**Cross-module:** TASK-AUTH-101 (chief-financial-officer/marketing_admin roles), TASK-AI-003, TASK-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Recipient list invalid emails | per-recipient validate | Skipped + counted as failed | Inherent |
| Rate-pace stuck | watchdog | Sev-2 alert | Operator investigates |
| Cancel during dispatch | tx isolation | Allowed cancel marks remaining as cancelled | Inherent |
| TASK-EMAIL-009 send error per recipient | per-recipient try | failed_count++; continues | Inherent |
| Single recipient huge bulk (1 million) | size cap 100k | 413 + size_exceeded | Inherent slice-2 enhancement |
| AM revokes signature | not supported | Cancel + new draft | Inherent |
| Concurrent dispatch attempts | partial unique on (bulk_id, status='sending') | Second 409 | Inherent |
| Suppression list grows mid-dispatch | per-recipient query | Caught at send-time | Inherent |
| KMS unavailable decrypting recipients | timeout | Sev-1 + dispatch halts | KMS recovery |
| Cross-tenant via subject context | RLS | Inherent | None |
| Marketing_admin role missing | role check | 403 | Inherent |

## §11 — Implementation notes
- §11.1 Recipients KMS-encrypted at draft; decrypted at dispatch only.
- §11.2 Pacing via tokio::time::sleep between batches.
- §11.3 Per-recipient TASK-EMAIL-009 spawned async with bounded concurrency (50).
- §11.4 memory row per recipient sampled 1% (else volume explodes).
- §11.5 Cancellation tokens propagate to in-flight dispatch tasks.

---

*End of TASK-EMAIL-010 spec.*
