---
id: FR-EMAIL-009
title: "EMAIL outbound 1:1 send — DKIM-signed via FR-EMAIL-004 + AM confirm-before-send + queue + bounce handling"
module: EMAIL
priority: MUST
status: ready_to_test
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: 2026-05-23
memory_chain_hash: null
related_frs: [FR-EMAIL-004, FR-EMAIL-010, FR-EMAIL-011, FR-AUTH-101, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-EMAIL-004]
blocks: [FR-EMAIL-010, FR-EMAIL-003, FR-INV-010]

source_pages:
  - website/docs/modules/email.html#outbound

source_decisions:
  - DEC-1480 2026-05-17 — Per-message send: AM/Member composes; FR-EMAIL-004 DKIM-signs; queued via Stalwart SMTP; bounce + complaint tracked
  - DEC-1481 2026-05-17 — Closed enum `send_status` = {drafting, queued, sent, bounced_hard, bounced_soft, complaint, suppressed}; cardinality 7
  - DEC-1482 2026-05-17 — Pre-send confirmation: client UI shows summary; backend requires `confirm_token` (generated at compose-start, valid 5min)
  - DEC-1483 2026-05-17 — Suppression list per-tenant: hard bounces + complaints + manual suppressions; outbound to suppressed → 412
  - DEC-1484 2026-05-17 — memory audit kinds: email.send_queued, email.send_delivered, email.send_bounced, email.send_complaint, email.send_suppressed
  - DEC-1485 2026-05-17 — Rate limit 100 sends/hour/Member (anti-abuse + spam-source prevention)

build_envelope:
  language: rust 1.81
  service: cyberos/services/email/
  new_files:
    - services/email/migrations/0004_outbound_messages.sql
    - services/email/migrations/0005_suppression_list.sql
    - services/email/src/outbound/mod.rs
    - services/email/src/outbound/compose.rs
    - services/email/src/outbound/queue.rs
    - services/email/src/outbound/bounce_handler.rs
    - services/email/src/outbound/suppression.rs
    - services/email/src/audit/outbound_events.rs
    - services/email/src/handlers/outbound_routes.rs
    - services/email/tests/inbound_quarantine_test.rs
    - services/email/tests/outbound_confirm_token_test.rs
    - services/email/tests/inbound_quarantine_test.rs
    - services/email/tests/outbound_bounce_soft_retry_test.rs
    - services/email/tests/outbound_complaint_suppresses_test.rs
    - services/email/tests/inbound_quarantine_test.rs
    - services/email/tests/inbound_quarantine_test.rs
    - services/email/tests/outbound_status_enum_test.rs
    - services/email/tests/audit_row_test.rs

  modified_files:
    - services/email/src/lib.rs

  allowed_tools:
    - file_read: services/email/**
    - file_write: services/email/{src,tests,migrations}/**
    - bash: cd services/email && cargo test outbound

  disallowed_tools:
    - send without confirm_token (per DEC-1482)
    - send to suppressed recipient (per DEC-1483)
    - exceed 100/hour/Member (per DEC-1485)

effort_hours: 4
sub_tasks:
  - "0.4h: 0004 + 0005 migrations + closed enum"
  - "0.3h: outbound/mod.rs"
  - "0.4h: compose.rs (confirm_token gen)"
  - "0.4h: queue.rs (Stalwart SMTP handoff)"
  - "0.5h: bounce_handler.rs (hard/soft logic)"
  - "0.3h: suppression.rs"
  - "0.3h: audit/outbound_events.rs"
  - "0.3h: handlers/outbound_routes.rs"
  - "1.0h: tests — 9 test files"
  - "0.1h: wire-up"

risk_if_skipped: "Without outbound send, EMAIL module receive-only — clients can't reply. Without DEC-1482 confirm, accidental sends ship wrong content. Without DEC-1483 suppression, hard-bounce → repeat send → spam-listed. Without DEC-1485 rate-limit, compromised Member account spams blacklisting our IP."
---

## §1 — Description (BCP-14 normative)

The EMAIL service **MUST** ship outbound 1:1 send at `services/email/src/outbound/` with confirm-token gate, FR-EMAIL-004 DKIM signing, bounce + complaint handling, per-tenant suppression, rate limit, and 5 memory audit kinds.

1. **MUST** define closed `send_status` enum: `('drafting','queued','sent','bounced_hard','bounced_soft','complaint','suppressed')` per DEC-1481. Cardinality 7.

2. **MUST** expose compose `POST /v1/email/outbound/compose` body `{ to, cc, bcc, subject, body_html, body_text, in_reply_to? }`. Handler:
   - Validates recipients not in suppression list per DEC-1483; suppressed → 412 + suppressed reason.
   - Generates `confirm_token` (UUIDv7) with 5-min TTL; cached in Redis.
   - Returns 201 + `{ message_id, confirm_token, expires_at }`.

3. **MUST** expose send `POST /v1/email/outbound/send` body `{ message_id, confirm_token }`. Handler:
   - Validates token + message ownership.
   - Rate-limit check per DEC-1485.
   - Invokes FR-EMAIL-004 DKIM signer.
   - Hands to Stalwart SMTP queue.
   - Persists status='queued'.
   - Emits `email.send_queued` sev-2.

4. **MUST** handle bounce events from Stalwart per DEC-1481:
   - Hard bounce (5xx permanent) → status='bounced_hard' + add recipient to suppression.
   - Soft bounce (4xx temporary) → status='bounced_soft' + Stalwart retries up to 3 days.
   - Emits respective audit kinds.

5. **MUST** handle complaint (Feedback Loop from Gmail/Outlook) per DEC-1481 → status='complaint' + add to suppression + emit `email.send_complaint` sev-1.

6. **MUST** maintain per-tenant suppression list per DEC-1483 with reasons (hard_bounce | complaint | manual). Manual unsuppress endpoint for engagement_admin.

7. **MUST** rate-limit 100 sends/hour/Member per DEC-1485 via Redis sliding-window. Excess → 429.

8. **MUST** emit 5 memory audit kinds per DEC-1484.

9. **MUST** thread trace_id end-to-end.

10. **MUST NOT** send without confirm_token (DEC-1482).

11. **MUST NOT** send to suppressed (DEC-1483).

---

## §2 — Why this design (rationale)

**Why confirm token (DEC-1482)?** Two-step gate prevents accidental sends. UI shows summary before commit.

**Why suppression list (DEC-1483)?** Repeated sends to hard-bounced addresses = spam-reputation damage. Persistent suppression protects sender reputation.

**Why 100/hour/Member (DEC-1485)?** Legitimate Member sends ~10-30/day. 100/hour catches compromised accounts before significant damage.

---

## §3 — API contract

```sql
-- 0004_outbound_messages.sql
CREATE TYPE send_status AS ENUM ('drafting','queued','sent','bounced_hard','bounced_soft','complaint','suppressed');

CREATE TABLE outbound_messages (
  message_id UUID PRIMARY KEY,
  tenant_id UUID NOT NULL,
  sender_subject_id UUID NOT NULL,
  to_addrs TEXT[] NOT NULL,
  cc_addrs TEXT[],
  bcc_addrs TEXT[],
  subject_sha256 CHAR(64) NOT NULL,
  body_sha256 CHAR(64) NOT NULL,
  in_reply_to TEXT,
  status send_status NOT NULL DEFAULT 'drafting',
  queued_at TIMESTAMPTZ,
  sent_at TIMESTAMPTZ,
  bounce_reason TEXT,
  complaint_reason TEXT,
  smtp_message_id TEXT,
  trace_id CHAR(32)
);
CREATE INDEX idx_outbound_sender ON outbound_messages(sender_subject_id, queued_at DESC);
ALTER TABLE outbound_messages ENABLE ROW LEVEL SECURITY;
CREATE POLICY outbound_messages_rls ON outbound_messages
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON outbound_messages FROM cyberos_app;
GRANT UPDATE (status, queued_at, sent_at, bounce_reason, complaint_reason, smtp_message_id)
  ON outbound_messages TO cyberos_app;

-- 0005_suppression_list.sql
CREATE TABLE email_suppression (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  recipient_addr_hash16 TEXT NOT NULL,
  recipient_addr_kms_blob BYTEA NOT NULL,
  reason TEXT NOT NULL CHECK (reason IN ('hard_bounce','complaint','manual')),
  suppressed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  suppressed_by_subject_id UUID,
  unsuppressed_at TIMESTAMPTZ,
  UNIQUE (tenant_id, recipient_addr_hash16)
);
ALTER TABLE email_suppression ENABLE ROW LEVEL SECURITY;
CREATE POLICY email_suppression_rls ON email_suppression
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON email_suppression FROM cyberos_app;
GRANT UPDATE (unsuppressed_at) ON email_suppression TO cyberos_app;
```

Endpoints:
```text
POST   /v1/email/outbound/compose
POST   /v1/email/outbound/send
POST   /v1/admin/email/suppression/unsuppress     (engagement_admin)
GET    /v1/email/outbound?status=...               (sender or admin)
```

---

## §4 — Acceptance criteria

1. **send_status cardinality 7**.
2. **Compose returns confirm_token** valid 5min.
3. **Send without confirm → 400**.
4. **Send with valid confirm** → queued.
5. **DKIM signed before queue** — verified via FR-EMAIL-004.
6. **Hard bounce adds to suppression** + status=bounced_hard.
7. **Soft bounce retried** — Stalwart 3-day retry; status=bounced_soft.
8. **Complaint adds to suppression** + sev-1 audit.
9. **Suppressed recipient blocked** — compose to suppressed → 412.
10. **Rate limit 100/hour** — 101st → 429.
11. **Manual unsuppress** — engagement_admin can re-enable + audit.
12. **5 memory audit kinds emitted**.
13. **Trace_id end-to-end**.
14. **PII scrub** — subject + body sha256 in chain; recipient hash; raw KMS.
15. **Cross-tenant RLS denied**.
16. **In-reply-to preserved** — reply maintains thread.
17. **Bounce notification to sender** — UI surfaces bounce.
18. **Confirm token expires 5min** — past TTL → 412.
19. **Sender required to be Member of tenant**.
20. **Audit on each transition**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn send_requires_confirm_token() {
    let ctx = TestContext::with_member().await;
    let compose = ctx.compose("to@example.com", "test", "body").await;
    let msg_id: Uuid = compose.json::<serde_json::Value>().await.unwrap()["message_id"].as_str().unwrap().parse().unwrap();
    let r = ctx.send_without_token(msg_id).await;
    assert_eq!(r.status(), 400);
}

#[tokio::test]
async fn hard_bounce_adds_to_suppression() {
    let ctx = TestContext::with_member().await;
    let msg_id = ctx.compose_and_send("bouncing@example.com").await;
    ctx.simulate_hard_bounce(msg_id).await;
    let suppressed: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM email_suppression WHERE tenant_id=$1 AND recipient_addr_hash16=$2)"
    ).bind(ctx.tenant_id).bind(hash16("bouncing@example.com")).fetch_one(&ctx.pool).await.unwrap();
    assert!(suppressed);
}

#[tokio::test]
async fn suppressed_recipient_blocked() {
    let ctx = TestContext::with_member().await;
    ctx.add_suppression(ctx.tenant_id, "blocked@example.com", "manual").await;
    let r = ctx.compose("blocked@example.com", "test", "body").await;
    assert_eq!(r.status(), 412);
}

#[tokio::test]
async fn rate_limit_100_per_hour() {
    let ctx = TestContext::with_member().await;
    for _ in 0..100 { ctx.compose_and_send_minimal().await; }
    let r = ctx.compose("more@example.com", "test", "body").await;
    assert_eq!(r.status(), 429);
}

// 5.5..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-EMAIL-004.
**Cross-module:** FR-AUTH-101 (engagement_admin), FR-AI-003, FR-MEMORY-111.
**Downstream:** FR-EMAIL-010, FR-EMAIL-011.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Stalwart unavailable | SMTP error | Status remains queued; retry | Stalwart recovery |
| Confirm token expired | TTL check | 412 | Re-compose |
| Hard bounce | Stalwart event | Suppression + status | Inherent |
| Soft bounce 3-day exceeded | watchdog | Final status=bounced_hard | Inherent |
| Complaint via FBL | feedback consumer | Suppression + sev-1 | Inherent |
| Rate limit | counter | 429 | Member waits |
| Cross-tenant via Member context | RLS | 403 | Inherent |
| DKIM sign fail | per FR-EMAIL-004 | Status=queued but Stalwart rejects | Investigate KMS |
| Recipient address invalid | RFC 5321 check | 400 | Inherent |
| Body > 25 MiB | size check | 413 | Inherent |
| Compromised Member account | rate-limit triggers | Auto-flagged | Sec team review |
| Reply-to thread broken | in_reply_to invalid | Allowed; client may not thread | Inherent |
| FBL not configured for ISP | Microsoft/Yahoo registration | Complaints not detected for those | Manual unsubscribe handling |
| Suppression list grows unbounded | tier review at 1M entries | Indexed lookup remains O(log n) | Inherent |

## §11 — Implementation notes
- §11.1 Confirm token in Redis with 5-min TTL.
- §11.2 Bounce parsing via `mail-parser` Rust crate.
- §11.3 Suppression check at compose time (early reject).
- §11.4 Rate limit Redis sliding-window per (sender_id, hour).
- §11.5 In-reply-to header preserved through Stalwart for threading.

---

*End of FR-EMAIL-009 spec.*
