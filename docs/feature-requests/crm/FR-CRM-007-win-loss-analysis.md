---
id: FR-CRM-007
title: "CRM win/loss analysis CUO draft — auto-generate analysis at deal close + memory memory persistence for institutional learning"
module: CRM
priority: SHOULD
status: ready_to_implement
verify: T
phase: P1
milestone: P1 · slice 6
slice: 6
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-CRM-001, FR-CRM-002, FR-CUO-101, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-CRM-001, FR-CUO-101]
blocks: []

source_pages:
  - website/docs/modules/crm.html#win-loss

source_decisions:
  - DEC-1670 2026-05-17 — Auto-triggered on deal.stage transition to won/lost; produces structured analysis (what worked, what didn't, lessons, follow-ups)
  - DEC-1671 2026-05-17 — Closed enum `wl_outcome` = {won, lost_no_decision, lost_competitor, lost_budget, lost_timing, lost_other}; cardinality 6
  - DEC-1672 2026-05-17 — Analysis includes: outcome, key turning points (from activity feed), customer feedback (from messages), competitor mentions, lessons, future action items
  - DEC-1673 2026-05-17 — Analysis persists as memory memory (kind=lessons) via FR-MEMORY-111 — searchable for similar future deals
  - DEC-1674 2026-05-17 — Draft requires CDO review before memory persistence — never auto-persist (PII + accuracy risk)
  - DEC-1675 2026-05-17 — memory audit kinds: crm.wl_draft_created, crm.wl_draft_approved, crm.wl_memory_memory_written, crm.wl_dismissed

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/migrations/0007_win_loss_drafts.sql
    - services/crm/src/win_loss/mod.rs
    - services/crm/src/win_loss/draft_generator.rs
    - services/crm/src/win_loss/memory_writer.rs
    - services/crm/src/handlers/win_loss_routes.rs
    - services/crm/src/audit/win_loss_events.rs
    - services/crm/tests/wl_auto_on_close_test.rs
    - services/crm/tests/wl_outcome_enum_cardinality_test.rs
    - services/crm/tests/wl_no_auto_persist_test.rs
    - services/crm/tests/wl_memory_write_on_approve_test.rs
    - services/crm/tests/wl_audit_emission_test.rs

  modified_files:
    - services/crm/src/deals.rs

  allowed_tools:
    - file_read: services/{crm,memory,cuo,ai}/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test win_loss

  disallowed_tools:
    - auto-persist memory memory (per DEC-1674)
    - skip outcome categorization (per DEC-1671)

effort_hours: 5
sub_tasks:
  - "0.3h: 0007_win_loss_drafts.sql"
  - "0.3h: win_loss/mod.rs"
  - "0.6h: draft_generator.rs"
  - "0.5h: memory_writer.rs (memory memory persist)"
  - "0.4h: handlers/win_loss_routes.rs"
  - "0.3h: audit/win_loss_events.rs"
  - "0.3h: deals.rs hook"
  - "1.6h: tests — 5 test files"
  - "0.7h: CDO UI for draft review + approval"

risk_if_skipped: "Without win/loss analysis, lessons evaporate at deal close — no institutional learning. Without DEC-1674 manual review, AI hallucination persists wrong narrative as 'truth'. Without DEC-1673 memory persistence, future similar deals don't benefit."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship win/loss analysis at `services/crm/src/win_loss/` triggered on deal close, drafted via FR-AI-003, queued for CDO review, persisted to memory on approval, 4 memory audit kinds.

1. **MUST** hook into deal stage transition (`services/crm/src/deals.rs`): on transition to `won` or `lost_*`, enqueue draft generation per DEC-1670.

2. **MUST** validate `wl_outcome` against closed enum per DEC-1671.

3. **MUST** draft via `draft_generator.rs::draft(deal, activities)` calling FR-AI-003 with structured prompt — output: `{outcome, turning_points[], customer_feedback, competitor_mentions[], lessons[], future_actions[]}`.

4. **MUST** queue draft for CDO review per DEC-1674 — NEVER auto-persist to memory.

5. **MUST** on approval, call `memory_writer.rs::write(draft)` to persist memory memory per DEC-1673 — `kind=lessons`, searchable via FR-MEMORY-108 query.

6. **MUST** define table at migration `0007`:
   ```sql
   CREATE TABLE crm_win_loss_drafts (
     draft_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     deal_id UUID NOT NULL UNIQUE,
     outcome TEXT NOT NULL
       CHECK (outcome IN ('won','lost_no_decision','lost_competitor','lost_budget','lost_timing','lost_other')),
     analysis_body JSONB NOT NULL,
     status TEXT NOT NULL DEFAULT 'pending_review'
       CHECK (status IN ('pending_review','approved','dismissed','persisted')),
     reviewed_by UUID,
     reviewed_at TIMESTAMPTZ,
     memory_memory_path TEXT,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE crm_win_loss_drafts ENABLE ROW LEVEL SECURITY;
   CREATE POLICY wl_drafts_rls ON crm_win_loss_drafts
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_win_loss_drafts FROM cyberos_app;
   GRANT UPDATE (status, reviewed_by, reviewed_at, memory_memory_path) ON crm_win_loss_drafts TO cyberos_app;
   ```

7. **MUST** emit 4 memory audit kinds per DEC-1675. PII per FR-MEMORY-111: analysis_body text hashed.

8. **MUST** thread trace_id from deal close → draft → CDO review → memory write → audit.

9. **MUST NOT** auto-persist memory memory per DEC-1674.

10. **MUST NOT** skip outcome categorization per DEC-1671 — required field on draft.

---

## §2 — Why this design

**Why on close (DEC-1670)?** Knowledge is freshest immediately — capture before memory fades.

**Why manual review (DEC-1674)?** AI may misattribute reasons; CDO confirms accuracy before institutional memory.

**Why memory persistence (DEC-1673)?** Future similar deals query lessons via FR-MEMORY-108; without persist, lessons stay siloed in deal record.

**Why 6-outcome enum (DEC-1671)?** Industry-standard win/loss taxonomy; enables aggregate reporting ("we lose 40% on budget").

---

## §3 — API contract

```text
GET    /v1/crm/win-loss/drafts             (list pending review)
POST   /v1/crm/win-loss/drafts/{id}/approve  (writes to memory)
POST   /v1/crm/win-loss/drafts/{id}/dismiss
```

Sample draft:
```json
{
  "draft_id": "uuid",
  "deal_id": "uuid",
  "outcome": "lost_competitor",
  "analysis_body": {
    "outcome": "lost_competitor",
    "turning_points": [
      {"date": "2026-04-10", "event": "Competitor demo went well"},
      {"date": "2026-04-20", "event": "Customer asked for our price match"}
    ],
    "customer_feedback": "Liked our product but went with cheaper option",
    "competitor_mentions": ["CompetitorX"],
    "lessons": [
      "Price-sensitive segment requires upfront pricing discussion",
      "Demo-to-close gap of 10d gives competitors window"
    ],
    "future_actions": ["Build price-match playbook", "Tighten demo-to-proposal cycle"]
  }
}
```

---

## §4 — Acceptance criteria
1. **Auto-drafted on close**. 2. **6-outcome enum + cardinality test**. 3. **Queued for review, never auto-persist**. 4. **Approve writes to memory memory**. 5. **Dismiss → status=dismissed**. 6. **4 memory audit kinds emitted**. 7. **PII scrubbed (analysis_body SHA256)**. 8. **RLS denies cross-tenant**. 9. **CDO/CRO role only**. 10. **Trace_id preserved**. 11. **Idempotent (UNIQUE on deal_id)**. 12. **Append-only via REVOKE except review/path cols**. 13. **AI failure → status=failed + sev-2 + retry**. 14. **memory_memory_path populated post-approve**. 15. **GET endpoint lists pending drafts**. 16. **Outcome required**. 17. **Lessons text non-empty**. 18. **Customer feedback may be empty if no signals**. 19. **Re-open deal → no auto-redraft (manual trigger)**. 20. **memory memory tagged `kind=lessons` searchable via FR-MEMORY-108**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn auto_drafts_on_close() {
    let ctx = TestContext::with_deal_in_proposal().await;
    ctx.change_deal_stage(ctx.deal_id, "lost_competitor").await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let drafts = ctx.fetch_drafts_for_deal(ctx.deal_id).await;
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].outcome, "lost_competitor");
}

#[tokio::test]
async fn never_auto_persists() {
    let ctx = TestContext::with_closed_deal().await;
    tokio::time::sleep(Duration::from_secs(2)).await;
    let memory_memories = ctx.memory_memory_count_for_deal(ctx.deal_id).await;
    assert_eq!(memory_memories, 0);
    let drafts = ctx.fetch_drafts_for_deal(ctx.deal_id).await;
    assert_eq!(drafts[0].status, "pending_review");
}

#[tokio::test]
async fn approve_writes_memory() {
    let ctx = TestContext::with_pending_draft().await;
    ctx.approve_draft(ctx.draft_id).await;
    let row = ctx.fetch_draft(ctx.draft_id).await;
    assert_eq!(row.status, "persisted");
    assert!(row.memory_memory_path.is_some());
    let mem = ctx.memory_fetch(row.memory_memory_path.unwrap()).await;
    assert!(mem.body.contains("lessons"));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-CRM-001, FR-CUO-101.
**Cross-module:** FR-CRM-002 (activity context), FR-AI-003 (draft), FR-MEMORY-111 (PII + memory write), FR-AUTH-101 (CDO role), FR-MEMORY-108 (search).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| AI timeout | retry 1x | sev-2 fallback to skeleton | manual fill |
| Duplicate draft race | UNIQUE on deal_id | skip second | inherent |
| Outcome enum mismatch | validate | reject | bug fix |
| memory write fail on approve | sev-1; status=approved-pending-retry | retry | inherent |
| User dismisses then re-opens deal | new draft only on manual trigger | inherent | inherent |
| Activity feed empty | proceed with deal-only context | inherent | inherent |
| AI hallucinates competitor | CDO catches in review | inherent | inherent |
| Lost stage but no specific reason | use 'lost_other' default | inherent | inherent |
| Cross-tenant approve | RLS | 403 | inherent |
| Concurrent approve | UPDATE WHERE pending | first wins | inherent |

## §11 — Implementation notes
- §11.1 AI prompt: structured win/loss template; force JSON output.
- §11.2 memory memory path: `memories/lessons/<deal_id_prefix>/<draft_id>.md`.
- §11.3 memory memory body: human-readable analysis text + structured tags (outcome, competitors, lessons).
- §11.4 memory audit body: deal_id, outcome enum; analysis SHA256.
- §11.5 Search via FR-MEMORY-108: `kind=lessons AND outcome=lost_competitor` returns similar lost-to-competitor analyses.

---

*End of FR-CRM-007 spec.*
