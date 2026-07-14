---
id: TASK-CRM-006
title: "CRM AI lead scoring — contact-creation-time score + nightly refresh based on activity signals, account tier, engagement history"
module: CRM
priority: SHOULD
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
  - website/docs/modules/crm.html#lead-scoring

source_decisions:
  - DEC-1660 2026-05-17 — Score 0-100 + closed enum tier {cold/warm/hot/qualified}; computed on contact create + nightly refresh
  - DEC-1661 2026-05-17 — Closed enum `lead_score_tier` = {cold, warm, hot, qualified}; cardinality 4; thresholds 0-24/25-49/50-74/75-100
  - DEC-1662 2026-05-17 — Signals: title seniority, engagement count last 30d, response rate, account tier match, deal-history similarity
  - DEC-1663 2026-05-17 — Score immutable snapshot per period — current_score + scored_at; history preserved in score_snapshots
  - DEC-1664 2026-05-17 — Per-tenant scoring weights configurable; default ships proven defaults
  - DEC-1665 2026-05-17 — memory audit kinds: crm.lead_scored_initial, crm.lead_score_refreshed, crm.lead_score_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/crm/
  new_files:
    - services/crm/migrations/0006_lead_scoring.sql
    - services/crm/src/scoring/mod.rs
    - services/crm/src/scoring/signal_builder.rs
    - services/crm/src/scoring/scorer.rs
    - services/crm/src/scoring/nightly_refresh.rs
    - services/crm/src/audit/scoring_events.rs
    - services/crm/tests/scoring_initial_test.rs
    - services/crm/tests/scoring_nightly_refresh_test.rs
    - services/crm/tests/scoring_tier_enum_cardinality_test.rs
    - services/crm/tests/scoring_snapshot_immutable_test.rs
    - services/crm/tests/scoring_audit_emission_test.rs

  modified_files:
    - services/crm/src/contacts.rs

  allowed_tools:
    - file_read: services/{crm,cuo,ai}/**
    - file_write: services/crm/{src,tests,migrations}/**
    - bash: cd services/crm && cargo test scoring

  disallowed_tools:
    - mutate prior score snapshot (per DEC-1663)
    - score without signals (per DEC-1662)

effort_hours: 5
subtasks:
  - "0.3h: 0006_lead_scoring.sql"
  - "0.3h: scoring/mod.rs"
  - "0.5h: signal_builder.rs"
  - "0.6h: scorer.rs (AI scoring call)"
  - "0.5h: nightly_refresh.rs (TASK-MCP-007 cron)"
  - "0.3h: audit/scoring_events.rs"
  - "0.3h: contacts.rs hook on create"
  - "1.7h: tests — 5 test files"
  - "0.5h: CRM UI score badge + tier color"

risk_if_skipped: "Without lead scoring, CDO prioritizes randomly — high-value leads neglected. Without DEC-1663 snapshot, score history lost (can't analyze score-correlation with deal win). Without DEC-1664 weights, can't tune for industry."
---

## §1 — Description (BCP-14 normative)

The CRM service **MUST** ship lead scoring at `services/crm/src/scoring/` computing 0-100 score + tier on contact-create + nightly refresh, immutable snapshots, 3 memory audit kinds.

1. **MUST** hook into contact creation (`services/crm/src/contacts.rs`): on insert, call `scoring::score_initial(contact_id)` asynchronously per DEC-1660.

2. **MUST** run nightly refresh per DEC-1660 at `nightly_refresh.rs::refresh_all(tenant)` triggered by TASK-MCP-007 cron at 02:00 tenant_tz — refresh all active contacts.

3. **MUST** build signals at `signal_builder.rs::build(contact_id)` per DEC-1662:
   - Title seniority (Director / VP / C-suite weight higher)
   - Engagement count last 30d (from TASK-CRM-002 activity feed)
   - Response rate (replies / sends ratio)
   - Account tier match (account-level signal)
   - Deal-history similarity (similar past contacts that won)

4. **MUST** score at `scorer.rs::score(signals, weights)` returning 0-100 int + tier per DEC-1661.

5. **MUST** validate `lead_score_tier` against closed enum (cardinality 4).

6. **MUST** define tables at migration `0006`:
   ```sql
   ALTER TABLE crm_contacts ADD COLUMN current_score INT
     CHECK (current_score IS NULL OR (current_score >= 0 AND current_score <= 100));
   ALTER TABLE crm_contacts ADD COLUMN current_tier TEXT
     CHECK (current_tier IS NULL OR current_tier IN ('cold','warm','hot','qualified'));
   ALTER TABLE crm_contacts ADD COLUMN scored_at TIMESTAMPTZ;
   GRANT UPDATE (current_score, current_tier, scored_at) ON crm_contacts TO cyberos_app;

   CREATE TABLE crm_score_snapshots (
     snapshot_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     contact_id UUID NOT NULL,
     score INT NOT NULL CHECK (score >= 0 AND score <= 100),
     tier TEXT NOT NULL CHECK (tier IN ('cold','warm','hot','qualified')),
     signals JSONB NOT NULL,
     weights_version TEXT NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX score_snapshots_contact_time_idx
     ON crm_score_snapshots(tenant_id, contact_id, created_at DESC);
   ALTER TABLE crm_score_snapshots ENABLE ROW LEVEL SECURITY;
   CREATE POLICY snap_rls ON crm_score_snapshots
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON crm_score_snapshots FROM cyberos_app;
   -- No GRANT UPDATE — snapshots immutable per DEC-1663

   CREATE TABLE crm_scoring_weights (
     tenant_id UUID PRIMARY KEY,
     weights JSONB NOT NULL,
     version TEXT NOT NULL,
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE crm_scoring_weights ENABLE ROW LEVEL SECURITY;
   CREATE POLICY weights_rls ON crm_scoring_weights
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT UPDATE (weights, version, updated_at) ON crm_scoring_weights TO cyberos_app;
   ```

7. **MUST** support per-tenant weight override via PUT endpoint (CDO/CRO only); default weights ship in code.

8. **MUST** emit 3 memory audit kinds per DEC-1665. PII per TASK-MEMORY-111: signals JSONB hashed (may contain title text); ids ok.

9. **MUST** thread trace_id from create/cron → builder → scorer → audit.

10. **MUST NOT** mutate prior snapshot per DEC-1663 — append-only.

11. **MUST NOT** score without all required signals — return error.

---

## §2 — Why this design

**Why 0-100 + 4 tiers (DEC-1661)?** Numeric continuous + categorical band — UX shows badge color (tier) + number for fine-grained filter.

**Why snapshots (DEC-1663)?** Score history = analytical truth ("scored 75 last week, 40 today — what changed?"). Mutation loses lineage.

**Why nightly refresh (DEC-1660)?** Signals decay (engagement count rolls), score must follow.

**Why per-tenant weights (DEC-1664)?** B2B SaaS values title seniority; agency values response rate.

---

## §3 — API contract

```text
POST   /v1/crm/contacts/{id}/rescore          (manual rescore, CDO)
GET    /v1/crm/contacts/{id}/score-history    (snapshot list)
PUT    /v1/crm/scoring/weights                (CDO/CRO only)
GET    /v1/crm/scoring/weights
```

Sample score:
```json
{
  "contact_id": "uuid",
  "current_score": 72,
  "current_tier": "hot",
  "scored_at": "2026-05-17T02:00:00Z",
  "signals": {
    "title_seniority": 8,
    "engagement_count_30d": 5,
    "response_rate": 0.6,
    "account_tier_match": "strategic",
    "similar_deal_win_rate": 0.4
  }
}
```

---

## §4 — Acceptance criteria
1. **Initial score on contact create**. 2. **Nightly refresh via cron 02:00**. 3. **Score 0-100 range enforced (CHECK)**. 4. **Tier enum 4 + cardinality test**. 5. **Tier thresholds applied (0-24=cold, 25-49=warm, 50-74=hot, 75+=qualified)**. 6. **All signals present required**. 7. **Snapshot immutable (no UPDATE/DELETE grant)**. 8. **Per-tenant weights respected**. 9. **Default weights ship in code**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (signals JSON hashed)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Manual rescore via POST**. 15. **History GET returns desc time**. 16. **Append-only snapshots**. 17. **Weight update increments version field**. 18. **Cron skip when tenant has 0 active contacts**. 19. **Score 100 cap (not 101)**. 20. **Signal missing → score=null + sev-2 audit (don't lie)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn initial_score_on_create() {
    let ctx = TestContext::with_account_signals().await;
    let contact = ctx.create_contact("john@acme.com").await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let c: Contact = ctx.fetch_contact(contact.id).await;
    assert!(c.current_score.is_some());
    assert!(c.current_tier.is_some());
}

#[tokio::test]
async fn snapshots_immutable() {
    let ctx = TestContext::with_scored_contact().await;
    let snap = ctx.fetch_latest_snapshot(ctx.contact_id).await;
    let r = ctx.try_mutate_snapshot(snap.snapshot_id, 0).await;
    assert!(r.is_err());
}

#[tokio::test]
async fn nightly_refresh_updates_all() {
    let ctx = TestContext::with_3_contacts().await;
    ctx.simulate_time_passage_signal_drift().await;
    ctx.run_nightly_scoring(ctx.tenant_id).await;
    for cid in ctx.contact_ids() {
        let history = ctx.fetch_score_history(cid).await;
        assert!(history.len() >= 2);  // initial + refresh
    }
}

// 5.4..5.10 — tier thresholds, weight version, manual rescore
```

---

## §7 — Dependencies
**Upstream:** TASK-CRM-001, TASK-CUO-101.
**Cross-module:** TASK-CRM-002 (signals), TASK-AI-003 (LLM scoring), TASK-MCP-007 (cron), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Signals missing | early validate | score=null + sev-2 audit | rescore later |
| AI scoring timeout | retry 1x | sev-2; previous score retained | next cron |
| Score out of range | CHECK constraint | DB rejects | bug fix |
| Tier mismatch with score | validation | reject + sev-1 | bug fix |
| Snapshot table grows large | partition by month | inherent | maintenance |
| Weight update mid-refresh | version pinning | inherent | inherent |
| Default weights miss tenant nuance | per-tenant override | manual tune | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Cron skipped | next run catches | inherent | inherent |
| 0 contacts tenant | skip silently | inherent | inherent |

## §11 — Implementation notes
- §11.1 Scorer uses TASK-AI-003 with structured prompt: "Score this contact 0-100 based on signals. Output JSON {score, tier, rationale}."
- §11.2 Default weights: title 30%, engagement 25%, response_rate 20%, account_tier 15%, deal_similarity 10%.
- §11.3 Snapshots persisted per refresh; useful for trend analysis.
- §11.4 memory audit body: contact_id, score, tier; signals SHA256.
- §11.5 Nightly cron via TASK-MCP-007 `kind: 'crm.lead_scoring_refresh'`, per-tenant fanout.

---

*End of TASK-CRM-006 spec.*
