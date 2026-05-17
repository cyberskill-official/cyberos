---
id: FR-CRM-002
title: "CRM activity feed — auto-log inbound email + outbound send + chat mention + calendar event to per-contact timeline"
module: CRM
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CRO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-CRM-001, FR-EMAIL-006, FR-EMAIL-009, FR-CHAT-005, FR-AI-003, FR-BRAIN-111]
depends_on: [FR-CRM-001, FR-EMAIL-006]
blocks: []

source_pages:
  - website/docs/modules/crm.html#activity-feed

source_decisions:
  - DEC-1620 2026-05-17 — Activity feed is per-contact + per-account; aggregates inbound email, outbound email, chat mentions, calendar events, deal stage changes
  - DEC-1621 2026-05-17 — Append-only by design — activity rows never updated; corrections via new "correction" row
  - DEC-1622 2026-05-17 — Closed enum `activity_kind` = {email_inbound, email_outbound, chat_mention, calendar_meeting, deal_stage_change, note_added, call_logged}; cardinality 7
  - DEC-1623 2026-05-17 — Source URLs preserved: each activity row has source_kind + source_id + deep_link for "open original"
  - DEC-1624 2026-05-17 — Cross-source dedup: same email_thread_id from inbound + convert-to-issue + reply does NOT produce 3 activities; one row per logical event
  - DEC-1625 2026-05-17 — BRAIN audit kinds: crm.activity_logged, crm.activity_dedup_skipped, crm.activity_correction_added; PII scrub via FR-BRAIN-111

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/migrations/0002_activity_feed.sql
    - services/crm/src/activity/mod.rs
    - services/crm/src/activity/logger.rs
    - services/crm/src/activity/dedup.rs
    - services/crm/src/activity/event_subscribers.rs
    - services/crm/src/handlers/activity_routes.rs
    - services/crm/src/audit/activity_events.rs
    - services/crm/tests/activity_email_logged_test.rs
    - services/crm/tests/activity_chat_mention_test.rs
    - services/crm/tests/activity_calendar_test.rs
    - services/crm/tests/activity_dedup_test.rs
    - services/crm/tests/activity_kind_enum_cardinality_test.rs
    - services/crm/tests/activity_correction_test.rs
    - services/crm/tests/activity_audit_emission_test.rs

  modified_files:
    - services/crm/src/lib.rs

  allowed_tools:
    - file_read: services/{crm,email,chat}/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test activity

  disallowed_tools:
    - mutate prior activity (per DEC-1621)
    - log inactive contact without create (per DEC-1620 — contact must exist)

effort_hours: 8
sub_tasks:
  - "0.3h: 0002_activity_feed.sql"
  - "0.3h: activity/mod.rs"
  - "0.6h: logger.rs"
  - "0.6h: dedup.rs"
  - "1.0h: event_subscribers.rs (EMAIL/CHAT/calendar)"
  - "0.4h: handlers/activity_routes.rs"
  - "0.3h: audit/activity_events.rs"
  - "2.4h: tests — 7 test files"
  - "1.5h: CRM UI activity timeline component"
  - "0.6h: docs"

risk_if_skipped: "Without activity feed, CRO sees CRM as static record — no temporal context for engagement health. Without DEC-1624 dedup, multi-source ingestion duplicates each interaction (noise). Without DEC-1621 append-only, edits create audit gap (compliance failure)."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship activity feed at `services/crm/src/activity/` subscribing to EMAIL/CHAT/calendar/deal events, deduplicating cross-source, logging per-contact + per-account, 3 BRAIN audit kinds.

1. **MUST** subscribe to events via `event_subscribers.rs`:
   - `email.inbound_processed` (FR-EMAIL-006 fires after link) → `activity_kind=email_inbound`
   - `email.message_sent` (FR-EMAIL-009) → `email_outbound`
   - `chat.mention_received` (FR-CHAT-005) → `chat_mention`
   - `calendar.meeting_scheduled` → `calendar_meeting`
   - `crm.deal_stage_changed` → `deal_stage_change`

2. **MUST** validate `activity_kind` against closed enum per DEC-1622.

3. **MUST** dedup at `dedup.rs::is_duplicate(activity)` per DEC-1624 — same `(contact_id, source_kind, source_id, kind)` within 60s = skip + emit `crm.activity_dedup_skipped`.

4. **MUST** define table at migration `0002`:
   ```sql
   CREATE TABLE crm_activities (
     activity_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     contact_id UUID,
     account_id UUID,
     deal_id UUID,
     kind TEXT NOT NULL
       CHECK (kind IN ('email_inbound','email_outbound','chat_mention','calendar_meeting','deal_stage_change','note_added','call_logged')),
     summary TEXT NOT NULL,
     source_kind TEXT NOT NULL,
     source_id UUID,
     deep_link TEXT NOT NULL,
     actor_id UUID,
     occurred_at TIMESTAMPTZ NOT NULL,
     trace_id CHAR(32),
     correction_to UUID,  -- points to prior activity_id if this is a correction
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX crm_activities_contact_time_idx
     ON crm_activities(tenant_id, contact_id, occurred_at DESC)
     WHERE contact_id IS NOT NULL;
   CREATE INDEX crm_activities_account_time_idx
     ON crm_activities(tenant_id, account_id, occurred_at DESC)
     WHERE account_id IS NOT NULL;
   CREATE INDEX crm_activities_dedup_idx
     ON crm_activities(tenant_id, source_kind, source_id, kind, occurred_at);
   ALTER TABLE crm_activities ENABLE ROW LEVEL SECURITY;
   CREATE POLICY activities_rls ON crm_activities
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_activities FROM cyberos_app;
   -- No UPDATE grant — append-only per DEC-1621
   ```

5. **MUST** preserve source deep_link per DEC-1623 — each row has clickable origin (e.g. `/email/threads/{id}`, `/chat/messages/{id}`).

6. **MUST** support manual `note_added` + `call_logged` via POST endpoint:
   ```text
   POST /v1/crm/activities  (CRO/AM via FR-AUTH-101)
   GET  /v1/crm/contacts/{id}/activities?limit=50  (paginated, desc time)
   GET  /v1/crm/accounts/{id}/activities?limit=50
   ```

7. **MUST** emit 3 BRAIN audit kinds per DEC-1625. PII: summary text SHA-256 hashed per FR-BRAIN-111.

8. **MUST** thread trace_id from source event → subscriber → logger → audit.

9. **MUST NOT** mutate prior activity per DEC-1621 — use `correction_to` column to chain.

10. **MUST NOT** log to contact that doesn't exist — return early; sev-3 audit.

---

## §2 — Why this design

**Why event subscribers (DEC-1620)?** Pull-based scanning misses real-time view; subscribers fire as events happen.

**Why dedup (DEC-1624)?** Multi-source ingestion (e.g. inbound email + convert-to-issue + reply) produces 3 events for one logical interaction; CRM would be noisy.

**Why append-only (DEC-1621)?** Audit lineage requires unmutable history; corrections via new row preserve trail.

**Why deep_link (DEC-1623)?** CRO clicks to see original; without link, activity feed is just trivia.

---

## §3 — API contract

Sample activity:
```json
{
  "activity_id": "uuid",
  "contact_id": "uuid",
  "account_id": "uuid",
  "kind": "email_inbound",
  "summary": "Re: Q3 pricing question — replied with proposal",
  "source_kind": "email_thread",
  "source_id": "uuid",
  "deep_link": "/email/threads/abc-123",
  "actor_id": "uuid-of-receiver",
  "occurred_at": "2026-05-17T10:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **Email inbound logged**. 2. **Email outbound logged**. 3. **Chat mention logged**. 4. **Calendar meeting logged**. 5. **Deal stage change logged**. 6. **Manual note/call POST works**. 7. **Closed enum 7 + cardinality test**. 8. **Dedup skip within 60s window**. 9. **3 BRAIN audit kinds emitted**. 10. **PII scrubbed (summary SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Deep_link present + clickable**. 14. **Pagination (desc time)**. 15. **Per-contact filter**. 16. **Per-account filter (rolls up contacts)**. 17. **Correction_to chains**. 18. **Append-only (REVOKE UPDATE)**. 19. **Activity to nonexistent contact rejected**. 20. **Source_kind enum (open: email_thread/chat/calendar/manual/etc.)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn email_inbound_creates_activity() {
    let ctx = TestContext::with_tracked_domain_and_contact().await;
    ctx.receive_inbound("jane@acme.com", "Q3 question").await;
    tokio::time::sleep(Duration::from_secs(1)).await;  // wait for subscriber
    let acts = ctx.fetch_contact_activities(ctx.contact_id).await;
    assert!(acts.iter().any(|a| a.kind == "email_inbound"));
}

#[tokio::test]
async fn dedup_within_60s() {
    let ctx = TestContext::new().await;
    let src = Uuid::new_v4();
    ctx.log_activity_raw(ctx.contact_id, "email_inbound", src).await;
    ctx.log_activity_raw(ctx.contact_id, "email_inbound", src).await;
    let acts = ctx.fetch_contact_activities(ctx.contact_id).await;
    assert_eq!(acts.iter().filter(|a| a.source_id == Some(src)).count(), 1);
}

#[tokio::test]
async fn append_only_no_update() {
    let ctx = TestContext::with_activity().await;
    let result = ctx.try_update_activity(ctx.activity_id, "tampered").await;
    assert!(result.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-CRM-001, FR-EMAIL-006.
**Cross-module:** FR-EMAIL-009 (outbound event), FR-CHAT-005 (mention), FR-AUTH-101 (manual logger), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Subscriber lost event | retry via DLQ | sev-2 audit; manual replay | inherent |
| Source event missing fields | validate | skip + sev-3 | data fix |
| Concurrent dedup race | UNIQUE on (src_kind,src_id,kind,occurred_at) | second skipped | inherent |
| Contact deleted post-event | FK NULL or skip | log to account only | inherent |
| Deep_link invalid | per-source validation | log with note | inherent |
| High-volume tenant (>10k acts/day) | pagination + index | inherent | inherent |
| Manual log spam | rate-limit per user | 429 | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Backlog event during outage | replay queue | inherent | manual run |
| Correction chain too deep (>10) | sanity check | sev-3; allow | inherent |

## §11 — Implementation notes
- §11.1 Subscribers via internal event bus or polling FR-BRAIN audit log; choose per-deployment.
- §11.2 Dedup window 60s — short enough to catch races, long enough to allow distinct re-sends.
- §11.3 Summary auto-generated: kind-specific template (`"Email from {sender}: {subject_first_60_chars}"`).
- §11.4 BRAIN audit body: kind, contact_id, source ids; summary SHA256.
- §11.5 UI timeline shows kind icons + deep_link + relative time.

---

*End of FR-CRM-002 spec.*
