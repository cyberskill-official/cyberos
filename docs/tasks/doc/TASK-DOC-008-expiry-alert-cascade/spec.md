---
id: TASK-DOC-008
title: "DOC expiry alert cascade — 90/30/7-day notifications to parties + CLO with deduplication and snooze support"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: DOC
priority: p0
status: draft
verify: T
phase: P2
milestone: P2 · slice 1
slice: 1
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-DOC-007, TASK-EMAIL-009, TASK-CHAT-005, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-DOC-007]
blocks: []

source_pages:
  - website/docs/modules/doc.html#expiry-alerts

source_decisions:
  - DEC-1720 2026-05-17 — Three alert thresholds: 90d, 30d, 7d before expiry; notify all parties + tenant CLO
  - DEC-1721 2026-05-17 — Closed enum `alert_threshold` = {d90, d30, d7}; cardinality 3
  - DEC-1722 2026-05-17 — Dedup: one alert per (document_id, threshold) sent once; UNIQUE constraint enforces
  - DEC-1723 2026-05-17 — Notification channels: TASK-EMAIL-009 (party emails) + TASK-CHAT-005 (internal CLO)
  - DEC-1724 2026-05-17 — CLO can snooze alerts per-document (postpones all remaining thresholds until snooze expires)
  - DEC-1725 2026-05-17 — memory audit kinds: doc.expiry_alert_scheduled, doc.expiry_alert_sent, doc.expiry_alert_snoozed, doc.expiry_alert_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/doc/
  new_files:
    - services/doc/migrations/0003_expiry_alerts.sql
    - services/doc/src/expiry/mod.rs
    - services/doc/src/expiry/scanner.rs
    - services/doc/src/expiry/notifier.rs
    - services/doc/src/handlers/expiry_routes.rs
    - services/doc/src/audit/expiry_events.rs
    - services/doc/tests/expiry_90_30_7_thresholds_test.rs
    - services/doc/tests/expiry_dedup_test.rs
    - services/doc/tests/expiry_snooze_test.rs
    - services/doc/tests/expiry_threshold_enum_cardinality_test.rs
    - services/doc/tests/expiry_audit_emission_test.rs

  modified_files:
    - services/doc/src/lib.rs

  allowed_tools:
    - file_read: services/{doc,email,chat}/**
    - file_write: services/doc/{src,tests,migrations}/**
    - bash: cd services/doc && cargo test expiry

  disallowed_tools:
    - send duplicate alert (per DEC-1722)
    - ignore snooze (per DEC-1724)

effort_hours: 4
subtasks:
  - "0.3h: 0003_expiry_alerts.sql"
  - "0.3h: expiry/mod.rs"
  - "0.5h: scanner.rs (daily threshold check)"
  - "0.5h: notifier.rs (email+chat dispatch)"
  - "0.4h: handlers/expiry_routes.rs"
  - "0.3h: audit/expiry_events.rs"
  - "1.6h: tests — 5 test files"
  - "0.1h: docs"

risk_if_skipped: "Without expiry alerts, contracts auto-expire unnoticed → service interruption + customer relationship damage. Without DEC-1722 dedup, parties receive 90 emails as scanner re-runs (spam). Without DEC-1724 snooze, irrelevant alerts during renewal negotiation noise the queue."
---

## §1 — Description (BCP-14 normative)

The DOC service **MUST** ship expiry alert cascade at `services/doc/src/expiry/` triggered daily, sending at 90/30/7-day thresholds via TASK-EMAIL-009 + TASK-CHAT-005, deduplicated, snooze-able, 4 memory audit kinds.

1. **MUST** schedule daily scan at 06:00 tenant_tz per DEC-1720 via TASK-MCP-007 cron.

2. **MUST** validate `alert_threshold` against closed enum per DEC-1721.

3. **MUST** scan at `scanner.rs::scan(tenant, today)`:
   - SELECT documents WHERE expiry_date IS NOT NULL AND lifecycle_status NOT IN ('expired','terminated').
   - Check if `expiry_date - today` matches d90/d30/d7 ±0d.
   - Filter snoozed docs per DEC-1724.

4. **MUST** dedup per DEC-1722 via UNIQUE constraint — skip if `(document_id, threshold)` row exists.

5. **MUST** dispatch at `notifier.rs::notify(doc, threshold, parties)`:
   - For each party with email → TASK-EMAIL-009 send
   - Tenant CLO → TASK-CHAT-005 message
   - Use template: "Contract {title} expires in {days}d. Renew or terminate?"

6. **MUST** define table at migration `0003`:
   ```sql
   CREATE TABLE doc_expiry_alerts (
     alert_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     threshold TEXT NOT NULL CHECK (threshold IN ('d90','d30','d7')),
     sent_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     recipients_count INT NOT NULL,
     trace_id CHAR(32),
     UNIQUE (tenant_id, document_id, threshold)
   );
   ALTER TABLE doc_expiry_alerts ENABLE ROW LEVEL SECURITY;
   CREATE POLICY alerts_rls ON doc_expiry_alerts
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON doc_expiry_alerts FROM cyberos_app;

   CREATE TABLE doc_expiry_snoozes (
     snooze_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     document_id UUID NOT NULL,
     snoozed_until TIMESTAMPTZ NOT NULL,
     reason TEXT,
     snoozed_by UUID NOT NULL,
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, document_id)
   );
   ALTER TABLE doc_expiry_snoozes ENABLE ROW LEVEL SECURITY;
   CREATE POLICY snoozes_rls ON doc_expiry_snoozes
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT DELETE ON doc_expiry_snoozes TO cyberos_app;  -- CLO can un-snooze
   ```

7. **MUST** support CLO snooze per DEC-1724:
   ```text
   POST   /v1/doc/documents/{id}/snooze-alerts   body: {until, reason}
   DELETE /v1/doc/documents/{id}/snooze-alerts   (un-snooze)
   ```

8. **MUST** emit 4 memory audit kinds per DEC-1725. PII per TASK-MEMORY-111: title hashed; threshold + counts ok.

9. **MUST** thread trace_id from cron → scanner → notifier → audit.

10. **MUST NOT** send duplicate alert per DEC-1722.

11. **MUST NOT** ignore active snooze per DEC-1724.

---

## §2 — Why this design

**Why 90/30/7 (DEC-1720)?** Industry standard escalation cadence; gives renewal negotiation runway.

**Why dedup (DEC-1722)?** Daily scan re-checks; without UNIQUE, same threshold fires repeatedly.

**Why snooze (DEC-1724)?** Active renewal negotiation makes alerts noise; CLO suppresses until they need them again.

**Why both email + chat (DEC-1723)?** Parties (external) need email; CLO (internal) needs chat.

---

## §3 — API contract

```text
POST   /v1/doc/documents/{id}/snooze-alerts   body: {until: ISO8601, reason}
DELETE /v1/doc/documents/{id}/snooze-alerts
GET    /v1/doc/expiry-alerts                  (list sent + scheduled)
POST   /v1/doc/expiry-scan                    (CLO manual trigger)
```

---

## §4 — Acceptance criteria
1. **Daily scan at 06:00 tenant_tz**. 2. **3-threshold enum + cardinality test**. 3. **Dedup via UNIQUE constraint**. 4. **Snooze suppresses all thresholds until snoozed_until**. 5. **Un-snooze (DELETE) re-enables**. 6. **Email to each party with email field**. 7. **Chat to tenant CLO**. 8. **4 memory audit kinds emitted**. 9. **PII scrubbed (title SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Expired docs excluded from scan**. 13. **Terminated docs excluded from scan**. 14. **Boundary day exact match (90d = scan day matches)**. 15. **CLO manual trigger via POST**. 16. **Append-only alerts table**. 17. **Snooze can be re-set (UNIQUE on doc_id replaces)**. 18. **Failed send → status=failed; retry**. 19. **High-volume tenant (1000+ docs/day) handled**. 20. **No alert if expiry=null (legacy docs)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn alerts_at_exact_thresholds() {
    let ctx = TestContext::doc_expires_in_days(90).await;
    ctx.run_expiry_scan(today()).await;
    let alerts = ctx.fetch_alerts(ctx.doc_id).await;
    assert_eq!(alerts.len(), 1);
    assert_eq!(alerts[0].threshold, "d90");
}

#[tokio::test]
async fn dedup_skips_second_scan() {
    let ctx = TestContext::doc_expires_in_days(90).await;
    ctx.run_expiry_scan(today()).await;
    ctx.run_expiry_scan(today()).await;  // same day, same threshold
    let alerts = ctx.fetch_alerts(ctx.doc_id).await;
    assert_eq!(alerts.len(), 1);
}

#[tokio::test]
async fn snooze_suppresses_alerts() {
    let ctx = TestContext::doc_expires_in_days(30).await;
    ctx.snooze_alerts(ctx.doc_id, today() + Duration::days(60)).await;
    ctx.run_expiry_scan(today()).await;
    let alerts = ctx.fetch_alerts(ctx.doc_id).await;
    assert_eq!(alerts.len(), 0);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-DOC-007.
**Cross-module:** TASK-EMAIL-009 (email send), TASK-CHAT-005 (chat notify), TASK-MCP-007 (cron), TASK-AUTH-101 (CLO role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Email send fails | retry 3x | failed audit | next scan retries |
| Chat send fails | retry | failed audit | next scan retries |
| No party emails | warn | CLO chat only | inherent |
| Scanner missed run | last_run check | catch-up next boot | inherent |
| High doc volume | pagination | batched | inherent |
| Snooze past expiry | OK | won't alert post-expiry | inherent |
| Duplicate snooze race | UNIQUE | second wins (replace) | inherent |
| Doc has no expiry_date | filter | excluded | inherent |
| Doc status terminated | filter | excluded | inherent |
| Cross-tenant snooze | RLS | 0 rows | inherent |

## §11 — Implementation notes
- §11.1 Cron via TASK-MCP-007 `kind: 'doc.expiry_scan'`, daily 06:00.
- §11.2 Threshold check: `(expiry_date - today) IN (90,30,7)` (exact match — runs daily so each day-X-from-expiry triggers once).
- §11.3 Notifier batches per-doc: all parties get same email body in one call.
- §11.4 memory audit body: doc_id, threshold, recipients_count; title SHA256.
- §11.5 Snooze: stored row with snoozed_until; scanner filters where snooze IS NULL OR snoozed_until < now().

---

*End of TASK-DOC-008 spec.*
