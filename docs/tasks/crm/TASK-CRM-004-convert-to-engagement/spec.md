---
id: TASK-CRM-004
title: "CRM convert-to-engagement — deal.won → PROJ Engagement creation with rate card + billing_currency + recognition_method + AM assignment"
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
module: crm
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CRM-001, TASK-PROJ-005, TASK-INV-002, TASK-INV-011, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CRM-001, TASK-PROJ-005]
blocks: [TASK-CRM-010]

source_pages:
  - website/docs/modules/crm.html#convert-to-engagement

source_decisions:
  - DEC-1640 2026-05-17 — Auto-triggered on deal stage transition to 'won' OR manual via "Create Engagement" button
  - DEC-1641 2026-05-17 — Engagement fields populated from deal: account_id, deal_value, contact_id, currency; user picks rate_card + recognition_method + AM
  - DEC-1642 2026-05-17 — Closed enum `conversion_source` = {auto_deal_won, manual_create, ai_suggested}; cardinality 3
  - DEC-1643 2026-05-17 — Deal backlink: engagement.source_deal_id; deal.converted_engagement_id (bi-directional)
  - DEC-1644 2026-05-17 — Idempotency: deal can convert to engagement only ONCE; subsequent attempts return existing engagement_id
  - DEC-1645 2026-05-17 — memory audit kinds: crm.conversion_initiated, crm.engagement_created, crm.conversion_failed

language: rust 1.81
service: cyberos/services/crm/
new_files:
  - services/crm/migrations/0004_deal_conversion.sql
  - services/crm/src/conversion/mod.rs
  - services/crm/src/conversion/engagement_builder.rs
  - services/crm/src/handlers/conversion_routes.rs
  - services/crm/src/audit/conversion_events.rs
  - services/crm/tests/conversion_auto_on_won_test.rs
  - services/crm/tests/conversion_manual_test.rs
  - services/crm/tests/conversion_idempotent_test.rs
  - services/crm/tests/conversion_backlink_test.rs
  - services/crm/tests/conversion_source_enum_cardinality_test.rs
  - services/crm/tests/conversion_audit_emission_test.rs

modified_files:
  - services/crm/src/deals.rs

allowed_tools:
  - file_read: services/{crm,proj}/**
  - file_write: services/crm/{src,tests,migrations}/**
  - bash: cd services/crm && cargo test conversion

disallowed_tools:
  - convert same deal twice (per DEC-1644)
  - create engagement without rate_card (per DEC-1641)

effort_hours: 6
subtasks:
  - "0.3h: 0004_deal_conversion.sql"
  - "0.3h: conversion/mod.rs"
  - "0.6h: engagement_builder.rs"
  - "0.5h: handlers/conversion_routes.rs"
  - "0.3h: audit/conversion_events.rs"
  - "0.4h: deals.rs hook on stage_change=won"
  - "2.4h: tests — 6 test files"
  - "1.2h: CRO UI conversion picker (rate_card + AM + method)"

risk_if_skipped: "Without auto-conversion, won deals stay in CRM without ops handoff — AM ignorant of new engagement. Without DEC-1644 idempotency, double-conversion creates duplicate engagements (billing chaos). Without DEC-1643 backlink, can't trace engagement → deal origin (sales attribution lost)."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship deal → engagement conversion at `services/crm/src/conversion/` triggered on deal.stage='won' or manual, creating TASK-PROJ-005 engagement with rate card + recognition method + AM, bi-directional backlink, 3 memory audit kinds.

1. **MUST** hook into deal stage transitions at `services/crm/src/deals.rs`. When `new_stage='won'` and no existing conversion: enqueue conversion task; CRO/CDO sees in conversion review queue.

2. **MUST** expose manual conversion endpoint:
   ```text
   POST /v1/crm/deals/{id}/convert-to-engagement
   GET  /v1/crm/deals/{id}/conversion   (status + engagement_id if exists)
   ```

3. **MUST** validate `conversion_source` against closed enum per DEC-1642.

4. **MUST** require user to provide: `rate_card_id` (TASK-PROJ-005), `recognition_method` (TASK-INV-011 enum), `assigned_am_id`, `engagement_start_date`. Auto-pop from deal: `account_id`, `deal_value`, `currency`, `contact_id`.

5. **MUST** call TASK-PROJ-005 engagement create endpoint with assembled payload.

6. **MUST** be idempotent per DEC-1644 — UNIQUE on `(deal_id)`; second call returns existing engagement_id with 200.

7. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE crm_deal_conversions (
     conversion_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     deal_id UUID NOT NULL UNIQUE,  -- idempotent per DEC-1644
     engagement_id UUID NOT NULL,
     conversion_source TEXT NOT NULL
       CHECK (conversion_source IN ('auto_deal_won','manual_create','ai_suggested')),
     rate_card_id UUID NOT NULL,
     recognition_method TEXT NOT NULL,
     assigned_am_id UUID NOT NULL,
     converted_by UUID NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE crm_deal_conversions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY conversions_rls ON crm_deal_conversions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_deal_conversions FROM cyberos_app;

   ALTER TABLE crm_deals ADD COLUMN converted_engagement_id UUID;
   GRANT UPDATE (converted_engagement_id) ON crm_deals TO cyberos_app;
   ```

8. **MUST** set bi-directional backlink per DEC-1643 — `deal.converted_engagement_id` + engagement has `source_deal_id` (TASK-PROJ-005 column).

9. **MUST** emit 3 memory audit kinds per DEC-1645. PII per TASK-MEMORY-111: deal_value SHA256 hashed; ids ok.

10. **MUST** thread trace_id from CRO action / deal-stage trigger → builder → TASK-PROJ-005 call → audit.

11. **MUST NOT** convert same deal twice per DEC-1644.

12. **MUST NOT** auto-create engagement on stage='won' — queue for review per DEC-1640 (rate card + AM selection requires human).

---

## §2 — Why this design

**Why review queue (DEC-1640)?** Rate card + AM assignment + recognition method are operational decisions that can't be inferred from deal alone.

**Why idempotent (DEC-1644)?** Stage change can fire twice (UI race); we must not create two engagements.

**Why bi-directional backlink (DEC-1643)?** Finance audits engagement profitability vs deal expectations; CRO tracks deal → engagement attribution.

**Why TASK-INV-011 recognition method enum (DEC-1641)?** Engagement billing depends on this — pre-binding at creation prevents mid-engagement changes.

---

## §3 — API contract

Sample manual conversion:
```json
POST /v1/crm/deals/{id}/convert-to-engagement
{
  "rate_card_id": "uuid",
  "recognition_method": "time_based",
  "assigned_am_id": "uuid",
  "engagement_start_date": "2026-06-01"
}
```

Response:
```json
{
  "conversion_id": "uuid",
  "engagement_id": "uuid",
  "engagement_url": "/proj/engagements/abc-123",
  "deal_id": "uuid",
  "conversion_source": "manual_create"
}
```

---

## §4 — Acceptance criteria
1. **Deal stage=won queues conversion**. 2. **Manual POST creates conversion**. 3. **3-source enum + cardinality test**. 4. **Required fields enforced (rate_card + method + AM + start_date)**. 5. **Account/value/currency/contact auto-pop from deal**. 6. **Idempotent (UNIQUE on deal_id)**. 7. **Second call returns existing engagement_id**. 8. **Bi-directional backlink set**. 9. **3 memory audit kinds emitted**. 10. **PII scrubbed (deal_value SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **TASK-PROJ-005 create called**. 14. **CRO/CDO role only (TASK-AUTH-101)**. 15. **Append-only conversion table**. 16. **Stage revert from won → no auto-unconvert (audit only)**. 17. **GET endpoint returns conversion status or 404**. 18. **AI-suggested mode: future TASK-CRM-006 can call with conversion_source=ai_suggested**. 19. **Engagement creation failure → conversion=failed; deal stays won**. 20. **Conversion event broadcast for TASK-CRM-002 activity feed**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn manual_convert_creates_engagement() {
    let ctx = TestContext::with_deal().await;
    let r = ctx.convert_deal(ctx.deal_id, ctx.rate_card_id, "time_based", ctx.am_id).await;
    assert!(r.engagement_id.is_some());
    let deal: Deal = ctx.fetch_deal(ctx.deal_id).await;
    assert_eq!(deal.converted_engagement_id, Some(r.engagement_id.unwrap()));
}

#[tokio::test]
async fn idempotent_duplicate_convert() {
    let ctx = TestContext::with_deal().await;
    let r1 = ctx.convert_deal(ctx.deal_id, ctx.rate_card_id, "time_based", ctx.am_id).await;
    let r2 = ctx.convert_deal(ctx.deal_id, ctx.rate_card_id, "time_based", ctx.am_id).await;
    assert_eq!(r1.engagement_id, r2.engagement_id);
}

#[tokio::test]
async fn bidirectional_backlink() {
    let ctx = TestContext::with_deal().await;
    let r = ctx.convert_deal(ctx.deal_id, ctx.rate_card_id, "time_based", ctx.am_id).await;
    let eng = ctx.fetch_engagement(r.engagement_id.unwrap()).await;
    assert_eq!(eng.source_deal_id, Some(ctx.deal_id));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-CRM-001, TASK-PROJ-005. **Cross-module:** TASK-INV-011 (recognition_method enum), TASK-AUTH-101 (CRO/CDO role), TASK-CRM-002 (activity feed event), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Required field missing | validate | 400 | provide field |
| TASK-PROJ-005 create fails | downstream error | conversion=failed; rollback insert | retry |
| Deal stage not won | validate | 409 (conflict) | move deal to won first |
| Duplicate convert race | UNIQUE | second returns existing | inherent |
| rate_card_id doesn't exist | FK | 404 | create rate card |
| AM_id deactivated | warning | proceed (CDO can reassign) | inherent |
| Deal value 0 (free engagement) | warning | proceed | inherent |
| Recognition method invalid | enum check | 400 | use valid |
| Cross-tenant convert attempt | RLS | 403 | inherent |
| Stage revert won → negotiating | audit only | no unconvert | manual eng close |

## §11 — Implementation notes
- §11.1 Builder maps deal fields: deal.value → engagement.contract_value, deal.currency → engagement.billing_currency.
- §11.2 Conversion broadcasts `crm.deal_converted` event for TASK-CRM-002 activity log.
- §11.3 PII: deal_value SHA256 in audit; engagement_id (uuid) ok in chain.
- §11.4 Stage revert post-conversion: audit logged but engagement stays — close manually if needed.
- §11.5 Future AI mode (TASK-CRM-006): suggests rate_card/method/AM based on similar past conversions.

---

*End of TASK-CRM-004 spec.*
