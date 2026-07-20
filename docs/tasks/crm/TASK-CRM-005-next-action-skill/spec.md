---
id: TASK-CRM-005
title: "CRM CUO crm.next-action@1 skill — AI-ranked top-3 next moves per open deal with rationale and deep-links"
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
module: CRM
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-CRM-001, TASK-CRM-002, TASK-CUO-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-CRM-001, TASK-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/crm.html#next-action

source_decisions:
  - DEC-1650 2026-05-17 — Skill exposed via CUO at `crm.next-action@1`; called per-deal returning top-3 ranked actions
  - DEC-1651 2026-05-17 — Closed enum `next_action_kind` = {send_email, schedule_call, share_proposal, request_intro, escalate_to_decision_maker, follow_up, mark_lost}; cardinality 7
  - DEC-1652 2026-05-17 — Suggestions grounded in: deal age, last activity (TASK-CRM-002), contact engagement signals, account history, similar deal patterns
  - DEC-1653 2026-05-17 — Each suggestion includes: kind, summary, rationale (1-2 sentences), confidence_score (0-1), deep_link to relevant CRM record
  - DEC-1654 2026-05-17 — Per-user rate limit: 100 calls/day to prevent runaway AI cost
  - DEC-1655 2026-05-17 — memory audit kinds: crm.next_action_suggested, crm.next_action_executed, crm.next_action_dismissed

language: rust 1.81
service: cyberos/services/crm/
new_files:
  - services/crm/migrations/0005_next_action_suggestions.sql
  - services/crm/src/next_action/mod.rs
  - services/crm/src/next_action/context_builder.rs
  - services/crm/src/next_action/ranker.rs
  - services/crm/src/next_action/skill_handler.rs
  - services/crm/src/audit/next_action_events.rs
  - services/crm/tests/next_action_returns_top_3_test.rs
  - services/crm/tests/next_action_kind_enum_cardinality_test.rs
  - services/crm/tests/next_action_rate_limit_test.rs
  - services/crm/tests/next_action_dismiss_test.rs
  - services/crm/tests/next_action_audit_emission_test.rs

modified_files:
  - services/crm/src/lib.rs

allowed_tools:
  - file_read: services/{crm,cuo,ai}/**
  - file_write: services/crm/{src,tests,migrations}/**
  - bash: cd services/crm && cargo test next_action

disallowed_tools:
  - return >3 suggestions (per DEC-1650)
  - exceed 100 calls/day per user (per DEC-1654)

effort_hours: 6
subtasks:
  - "0.3h: 0005_next_action_suggestions.sql"
  - "0.3h: next_action/mod.rs"
  - "0.6h: context_builder.rs (gather deal + activity + contact)"
  - "0.7h: ranker.rs (TASK-AI-003 prompt)"
  - "0.4h: skill_handler.rs (CUO registration)"
  - "0.3h: audit/next_action_events.rs"
  - "1.8h: tests — 5 test files"
  - "1.6h: CDO UI panel for top-3 display + execute/dismiss buttons"

risk_if_skipped: "Without next-action ranking, CDO scans CRM manually — open deals stagnate. Without DEC-1653 rationale, suggestions feel arbitrary (low adoption). Without DEC-1654 rate limit, runaway AI cost on hot users."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship `crm.next-action@1` skill at `services/crm/src/next_action/` returning top-3 AI-ranked actions per deal, grounded in deal context, rate-limited, 3 memory audit kinds.

1. **MUST** register skill at CUO via `skill_handler.rs::register()` per DEC-1650 — invoked via CUO `crm.next-action@1`.

2. **MUST** validate `next_action_kind` against closed enum per DEC-1651.

3. **MUST** build context at `context_builder.rs::build(deal_id)`:
- Deal record (stage, value, age, owner)
- Last 20 activities from TASK-CRM-002
- Contact engagement (last email reply gap, etc.)
- Account history (won/lost recent deals)

4. **MUST** rank at `ranker.rs::rank(context)` via TASK-AI-003 with structured prompt:
- Output JSON array of 3 entries, each `{kind, summary, rationale, confidence_score, deep_link}` per DEC-1653.
- Validation: each kind in enum; confidence 0-1; deep_link non-empty.

5. **MUST** enforce rate limit per DEC-1654 — 100 calls/user/day; return 429 when exceeded.

6. **MUST** define table at migration `0005`:
   ```sql
   CREATE TABLE crm_next_action_suggestions (
     suggestion_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     deal_id UUID NOT NULL,
     suggestions JSONB NOT NULL,
     requested_by UUID NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending'
       CHECK (status IN ('pending','executed','dismissed','expired')),
     executed_kind TEXT
       CHECK (executed_kind IS NULL OR executed_kind IN
         ('send_email','schedule_call','share_proposal','request_intro','escalate_to_decision_maker','follow_up','mark_lost')),
     executed_at TIMESTAMPTZ,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX next_action_user_day_idx
     ON crm_next_action_suggestions(tenant_id, requested_by, created_at DESC);
   ALTER TABLE crm_next_action_suggestions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY next_action_rls ON crm_next_action_suggestions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_next_action_suggestions FROM cyberos_app;
   GRANT UPDATE (status, executed_kind, executed_at) ON crm_next_action_suggestions TO cyberos_app;
   ```

7. **MUST** auto-expire pending suggestions after 7 days (cron via TASK-MCP-007).

8. **MUST** emit 3 memory audit kinds per DEC-1655. PII per TASK-MEMORY-111: rationale text SHA-256 hashed.

9. **MUST** thread trace_id from CUO call → context → AI → audit.

10. **MUST NOT** return >3 suggestions per DEC-1650.

11. **MUST NOT** exceed user rate limit per DEC-1654.

---

## §2 — Why this design

**Why top-3 (DEC-1650)?** Cognitive load research: >5 options paralyzes; 3 optimizes selection.

**Why rationale required (DEC-1653)?** CDO won't follow black-box AI; explainability drives adoption.

**Why rate limit (DEC-1654)?** AI cost per call is non-trivial; runaway usage breaks budget.

**Why context from TASK-CRM-002 (DEC-1652)?** Activity feed is the truth source; without it AI hallucinates.

---

## §3 — API contract

```text
POST   /v1/crm/next-action       body: {deal_id}
POST   /v1/crm/next-action/{id}/execute    body: {kind}  (records executed_kind)
POST   /v1/crm/next-action/{id}/dismiss
```

Sample response:
```json
{
  "suggestion_id": "uuid",
  "deal_id": "uuid",
  "suggestions": [
    {
      "kind": "send_email",
      "summary": "Follow up on proposal — no reply in 5 days",
      "rationale": "Last email Jun 1 unanswered; account history shows 7-day response pattern.",
      "confidence_score": 0.85,
      "deep_link": "/email/threads/abc-123"
    },
    {
      "kind": "schedule_call",
      "summary": "Push for decision call this week",
      "rationale": "Deal age 45d, stage 'proposal' for 14d; similar deals close after exec call.",
      "confidence_score": 0.75,
      "deep_link": "/calendar/new?contact_id=..."
    },
    {
      "kind": "request_intro",
      "summary": "Ask current contact to introduce CFO",
      "rationale": "Decision-maker not yet engaged; CFO buyer signals from similar deals.",
      "confidence_score": 0.65,
      "deep_link": "/crm/contacts/...../add-stakeholder"
    }
  ]
}
```

---

## §4 — Acceptance criteria
1. **CUO skill registered as crm.next-action@1**. 2. **Returns exactly 3 suggestions (or fewer if AI can't fill)**. 3. **Enum 7 + cardinality test**. 4. **Each suggestion has all 5 fields**. 5. **Rationale 1-2 sentences**. 6. **Confidence_score 0-1**. 7. **Deep_link non-empty**. 8. **Context built from TASK-CRM-002 activities**. 9. **Rate limit 100/user/day**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (rationale SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Execute records executed_kind**. 15. **Dismiss → status=dismissed**. 16. **7-day expiry via cron**. 17. **Append-only suggestions table**. 18. **AI returns invalid JSON → sev-2 + retry once**. 19. **No deal_id (closed deal) → 404**. 20. **CDO/CRO role required**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn returns_top_3() {
    let ctx = TestContext::with_open_deal_and_activities(20).await;
    let r = ctx.next_action(ctx.deal_id).await;
    assert_eq!(r.suggestions.len(), 3);
    for s in &r.suggestions {
        assert!(!s.rationale.is_empty());
        assert!(s.confidence_score >= 0.0 && s.confidence_score <= 1.0);
    }
}

#[tokio::test]
async fn rate_limit_enforced() {
    let ctx = TestContext::with_user().await;
    for _ in 0..100 {
        ctx.next_action_for_random_deal().await;
    }
    let r = ctx.next_action_for_random_deal().await;
    assert_eq!(r.status_code, 429);
}

#[tokio::test]
async fn execute_records_kind() {
    let ctx = TestContext::with_suggestion().await;
    ctx.execute_suggestion(ctx.suggestion_id, "send_email").await;
    let row = ctx.fetch_suggestion(ctx.suggestion_id).await;
    assert_eq!(row.status, "executed");
    assert_eq!(row.executed_kind.as_deref(), Some("send_email"));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-CRM-001, TASK-CUO-101. **Cross-module:** TASK-CRM-002 (activity context), TASK-AI-003 (LLM), TASK-MCP-007 (expiry cron), TASK-AUTH-101 (role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| AI returns <3 suggestions | accept fewer | use what AI returned | inherent |
| AI invalid JSON | parse error | retry 1x, fallback sev-2 | inherent |
| AI hallucinated kind (not in enum) | filter | drop, fewer suggestions | inherent |
| Deal closed (won/lost) | check stage | 404 | inherent |
| Rate limit window edge | day boundary check | rolling 24h or calendar day | inherent |
| Context too large (>50k tokens) | truncate to last 10 acts | inherent | inherent |
| AI provider quota | rate limit upstream | sev-2; degrade gracefully | inherent |
| Expiry cron skipped | next run catches | inherent | manual run |
| Execute kind not in original suggestions | accept (user override) | record anyway | inherent |
| Cross-tenant suggestion lookup | RLS | 404 | inherent |

## §11 — Implementation notes
- §11.1 AI prompt: includes deal context as structured JSON, asks for `[{kind, summary, rationale, confidence_score, deep_link}]`.
- §11.2 Rate limit via Redis sliding-window: 100 ops per 24h per user.
- §11.3 memory audit body: deal_id, kinds[]; rationale SHA256.
- §11.4 Expiry cron at 02:00 tenant_tz: `UPDATE suggestions SET status='expired' WHERE status='pending' AND created_at < now() - interval '7 days'`.
- §11.5 Deep_link templates per kind: send_email → /email/compose?to=..., schedule_call → /calendar/new?contact=..., etc.

---

*End of TASK-CRM-005 spec.*
