---
id: TASK-RES-004
title: "RES hiring memo CUO draft — skill-gap × CRM pipeline trigger → CEO+CFO review queue with cost-benefit projection"
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
module: res
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 8
slice: 8
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-RES-001, TASK-CUO-101, TASK-CRM-001, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CUO-101, TASK-CRM-001]
blocks: []

source_pages:
  - website/docs/modules/res.html#hiring-memo

source_decisions:
  - DEC-2070 2026-05-17 — Auto-triggered when capacity matrix shows persistent over-allocation (>4 weeks at 110%+) AND CRM pipeline shows new deal velocity; manual trigger by CEO/CHRO
  - DEC-2071 2026-05-17 — Closed enum `hire_recommendation` = {hire_immediate, hire_q_plus_1, defer_pipeline_uncertain, reject_no_business_case}; cardinality 4
  - DEC-2072 2026-05-17 — Memo includes: skill gap analysis, pipeline correlation, cost projection (TASK-HR-002 contract + TASK-REW-004 deductions), 6-month ROI estimate
  - DEC-2073 2026-05-17 — Dual sign-off CEO + CFO required to convert memo to hiring action; TASK-RES-002 marks role as "approved-hire" — slot reserved in capacity matrix
  - DEC-2074 2026-05-17 — memory audit kinds: res.hiring_memo_drafted, res.hiring_memo_signed, res.hiring_memo_dismissed, res.hiring_memo_approved_for_action

language: rust 1.81
service: cyberos/services/res/
new_files:
  - services/res/migrations/0004_hiring_memos.sql
  - services/res/src/hiring/mod.rs
  - services/res/src/hiring/gap_detector.rs
  - services/res/src/hiring/cost_projector.rs
  - services/res/src/hiring/memo_generator.rs
  - services/res/src/handlers/hiring_routes.rs
  - services/res/src/audit/hiring_events.rs
  - services/res/tests/hiring_trigger_at_4w_over_allocated_test.rs
  - services/res/tests/hire_recommendation_enum_cardinality_test.rs
  - services/res/tests/hiring_dual_sign_test.rs
  - services/res/tests/hiring_no_auto_action_test.rs
  - services/res/tests/hiring_audit_emission_test.rs

modified_files:
  - services/res/src/lib.rs

allowed_tools:
  - file_read: services/{res,crm,hr,rew,cuo,ai}/**
  - file_write: services/res/{src,tests,migrations}/**
  - bash: cd services/res && cargo test hiring

disallowed_tools:
  - auto-create hire without dual-sign (per DEC-2073)
  - skip cost projection (per DEC-2072)

effort_hours: 8
subtasks:
  - "0.4h: 0004_hiring_memos.sql"
  - "0.3h: hiring/mod.rs"
  - "0.7h: gap_detector.rs"
  - "0.6h: cost_projector.rs"
  - "0.7h: memo_generator.rs"
  - "0.4h: handlers/hiring_routes.rs"
  - "0.4h: audit/hiring_events.rs"
  - "2.5h: tests — 5 test files"
  - "1.5h: CEO+CFO UI for review + sign"
  - "0.5h: docs"

risk_if_skipped: "Without hiring trigger, persistent over-allocation persists → burnout. Without DEC-2073 dual-sign, headcount adds without CFO budget review. Without DEC-2072 cost projection, decisions intuition-based."
---

## §1 — Description (BCP-14 normative)

The RES service **MUST** ship hiring memo at `services/res/src/hiring/` triggered by skill-gap + pipeline + cost projection + CEO+CFO dual-sign, 4 memory audit kinds.

1. **MUST** validate `hire_recommendation` against closed enum per DEC-2071.

2. **MUST** detect trigger at `gap_detector.rs::detect(tenant)` per DEC-2070:
- SELECT members from TASK-RES-001 matrix where `allocation_flag='over_allocated'` for ≥4 consecutive weeks
- SELECT TASK-CRM-001 deals advancing stage in same period
- If both conditions met: enqueue memo draft

3. **MUST** project cost at `cost_projector.rs::project(role, region, contract_type)` per DEC-2072:
- Base salary band per role (TASK-RES-007 future or hardcoded table for v1)
- TASK-REW-004 statutory deductions per region (BHXH + BHYT + BHTN + PIT)
- Total fully-loaded 6-month cost
- Pipeline-derived revenue offset

4. **MUST** generate memo at `memo_generator.rs::generate(tenant, gap, costs)`:
- Skill gap summary
- Pipeline correlation
- Cost projection table
- 6-month ROI estimate
- AI recommendation (TASK-AI-003) with reasoning

5. **MUST** require CEO + CFO dual-sign per DEC-2073 — same-person rejected.

6. **MUST** define table at migration `0004`:
   ```sql
   CREATE TABLE res_hiring_memos (
     memo_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     trigger_kind TEXT NOT NULL,  -- 'auto_gap_pipeline' | 'manual_ceo' | 'manual_chro'
     role_title TEXT NOT NULL,
     region TEXT NOT NULL,
     proposed_contract_type TEXT,
     memo_body_jsonb JSONB NOT NULL,
     recommendation TEXT NOT NULL
       CHECK (recommendation IN ('hire_immediate','hire_q_plus_1','defer_pipeline_uncertain','reject_no_business_case')),
     status TEXT NOT NULL DEFAULT 'pending_review'
       CHECK (status IN ('pending_review','ceo_signed','cfo_signed','approved_for_action','dismissed')),
     ceo_signed_by UUID,
     ceo_signed_at TIMESTAMPTZ,
     cfo_signed_by UUID,
     cfo_signed_at TIMESTAMPTZ,
     hire_slot_reserved BOOLEAN NOT NULL DEFAULT false,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE res_hiring_memos ENABLE ROW LEVEL SECURITY;
   CREATE POLICY hiring_memos_rls ON res_hiring_memos
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON res_hiring_memos FROM cyberos_app;
   GRANT UPDATE (status, ceo_signed_by, ceo_signed_at, cfo_signed_by, cfo_signed_at, hire_slot_reserved) ON res_hiring_memos TO cyberos_app;
   ```

7. **MUST** expose endpoints:
   ```text
   POST /v1/res/hiring-memos                   (CEO/CHRO manual trigger)
   POST /v1/res/hiring-memos/{id}/ceo-sign
   POST /v1/res/hiring-memos/{id}/cfo-sign
   POST /v1/res/hiring-memos/{id}/dismiss
   GET  /v1/res/hiring-memos                   (list)
   ```

8. **MUST** emit 4 memory audit kinds per DEC-2074. PII per TASK-MEMORY-111: cost projections SHA-256 hashed.

9. **MUST** thread trace_id from trigger → projector → memo → sign → audit.

10. **MUST NOT** auto-action without dual-sign per DEC-2073.

11. **MUST NOT** allow same person to sign both roles per DEC-2073.

---

## §2 — Why this design

**Why dual-sign (DEC-2073)?** Headcount = major financial commitment; CFO budget perspective + CEO strategic both required.

**Why cost projection (DEC-2072)?** Without numbers, debate is opinion; projection grounds discussion.

**Why pipeline correlation (DEC-2070)?** Over-allocation alone may be temporary; pairing with deal velocity confirms sustained demand.

**Why 4 recommendations (DEC-2071)?** Captures real decisions — immediate, next quarter, defer, reject — bounded prevents waffling.

---

## §3 — API contract

Sample memo:
```json
{
  "memo_id": "uuid",
  "role_title": "Senior Backend Engineer",
  "region": "vn-1",
  "memo_body_jsonb": {
    "skill_gap": "Backend capacity at 115% for 6 weeks",
    "pipeline_correlation": "$300k in committed deals requiring backend work in next quarter",
    "cost_projection": {
      "annual_base_vnd": 600000000,
      "annual_si_employer_vnd": 144000000,
      "annual_total_vnd": 744000000,
      "6mo_total_vnd": 372000000
    },
    "roi_estimate": "Break-even at month 2 if pipeline holds",
    "ai_recommendation": "hire_immediate"
  },
  "recommendation": "hire_immediate",
  "status": "pending_review"
}
```

---

## §4 — Acceptance criteria
1. **hire_recommendation enum cardinality 4**. 2. **Auto-trigger at 4w over-allocation + pipeline velocity**. 3. **Manual trigger CEO/CHRO**. 4. **Cost projection includes statutory**. 5. **CEO + CFO dual-sign required**. 6. **Same-person rejected**. 7. **approved_for_action requires both signs**. 8. **hire_slot_reserved set on approval (capacity matrix marker)**. 9. **4 memory audit kinds emitted**. 10. **PII scrubbed (cost SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE except status cols**. 14. **AI failure → minimal memo + sev-2**. 15. **Dismiss allowed at any pre-action stage**. 16. **Status workflow enforced**. 17. **Auto-trigger idempotent per tenant per week**. 18. **Memo body JSONB schema validated**. 19. **Region required (drives cost projection)**. 20. **List endpoint paginated**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn auto_trigger_at_4w_over_allocation_plus_pipeline() {
    let ctx = TestContext::with_4w_over_allocation_and_new_deals().await;
    ctx.run_hiring_detector(ctx.tenant_id).await;
    let memos = ctx.fetch_hiring_memos(ctx.tenant_id).await;
    assert!(!memos.is_empty());
}

#[tokio::test]
async fn dual_sign_required() {
    let ctx = TestContext::with_pending_memo().await;
    ctx.ceo_sign(ctx.memo_id).await;
    let memo = ctx.fetch_memo(ctx.memo_id).await;
    assert_eq!(memo.status, "ceo_signed");
    ctx.cfo_sign(ctx.memo_id).await;
    let memo2 = ctx.fetch_memo(ctx.memo_id).await;
    assert_eq!(memo2.status, "approved_for_action");
    assert!(memo2.hire_slot_reserved);
}

#[tokio::test]
async fn same_person_both_roles_rejected() {
    let ctx = TestContext::with_pending_memo().await;
    ctx.ceo_sign_as(ctx.user_a, ctx.memo_id).await;
    let r = ctx.try_cfo_sign_as(ctx.user_a, ctx.memo_id).await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-CUO-101, TASK-CRM-001. **Cross-module:** TASK-RES-001 (over-allocation signal), TASK-RES-002 (slot reserved), TASK-HR-002 (contract type), TASK-REW-004 (statutory deductions), TASK-AI-003 (LLM), TASK-AUTH-101 (CEO/CFO roles), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| AI timeout | retry 1x | minimal memo + sev-2 | CEO completes manually |
| Cost projection fail | catch | sev-2; use rough estimate | data fix |
| One signer missing | gate | inherent | wait for second |
| Same-person dual-sign | validate | 403 | different signer |
| Auto-trigger noise | rate limit per tenant/week | inherent | inherent |
| Pipeline data stale | warn | proceed with sev-3 | data refresh |
| Cross-tenant memo | RLS | 403 | inherent |
| hire_slot reservation race | UPDATE WHERE | first wins | inherent |
| Stage revert post-approval | manual CEO action | inherent | reject memo |
| Region invalid | validate | 400 | use valid |

## §11 — Implementation notes
- §11.1 Detector cron via TASK-MCP-007 `kind: 'res.hiring_gap_detection'`, weekly.
- §11.2 Cost projector uses TASK-REW-004 SI rate formulas + TASK-HR-005 minimum_wage policy.
- §11.3 AI prompt: structured "Given gap + pipeline + costs, recommend one of: hire_immediate / hire_q+1 / defer / reject. Explain reasoning."
- §11.4 hire_slot_reserved column drives TASK-RES-002 Gantt to show pending hire row.
- §11.5 memory audit body: memo_id, role, region, recommendation; cost SHA256.

---

*End of TASK-RES-004 spec.*
