---
id: TASK-LEARN-004
title: "LEARN Hội đồng Chuyên môn (Specialist Council) — 3-5 judges + multi-dim scoring + per-judge anonymity within council"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: LEARN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CHRO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-LEARN-001, TASK-LEARN-005, TASK-LEARN-006, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-LEARN-001]
blocks: [TASK-LEARN-005, TASK-LEARN-006]

source_pages:
  - website/docs/modules/learn.html#specialist-council

source_decisions:
  - DEC-2110 2026-05-17 — Specialist Council: 3-5 judges score candidate against multi-dim rubric (technical / leadership / impact / collaboration / growth-potential)
  - DEC-2111 2026-05-17 — Closed enum `score_dimension` = {technical, leadership, impact, collaboration, growth_potential}; cardinality 5
  - DEC-2112 2026-05-17 — Closed enum `council_status` = {convened, scoring, completed, dismissed}; cardinality 4
  - DEC-2113 2026-05-17 — Per-judge scores ONLY visible to council members during scoring; TASK-LEARN-005 enforces post-completion isolation
  - DEC-2114 2026-05-17 — Aggregate output: median per-dimension + overall recommendation (promote / hold / decline)
  - DEC-2115 2026-05-17 — memory audit kinds: learn.council_convened, learn.judge_assigned, learn.score_submitted, learn.council_completed, learn.council_dismissed

build_envelope:
  language: rust 1.81
  service: cyberos/services/learn/
  new_files:
    - services/learn/migrations/0004_councils.sql
    - services/learn/src/council/mod.rs
    - services/learn/src/council/scorer.rs
    - services/learn/src/council/aggregator.rs
    - services/learn/src/handlers/council_routes.rs
    - services/learn/src/audit/council_events.rs
    - services/learn/tests/council_3_to_5_judges_test.rs
    - services/learn/tests/score_dimension_enum_cardinality_test.rs
    - services/learn/tests/council_status_enum_cardinality_test.rs
    - services/learn/tests/council_judge_scoring_test.rs
    - services/learn/tests/council_aggregate_test.rs
    - services/learn/tests/council_audit_emission_test.rs

  modified_files:
    - services/learn/src/lib.rs

  allowed_tools:
    - file_read: services/{learn,auth}/**
    - file_write: services/learn/{src,tests,migrations}/**
    - bash: cd services/learn && cargo test council

  disallowed_tools:
    - judges < 3 (per DEC-2110)
    - judges > 5 (per DEC-2110)
    - mutate prior score (per DEC-2113)

effort_hours: 10
subtasks:
  - "0.4h: 0004_councils.sql"
  - "0.4h: council/mod.rs"
  - "0.7h: scorer.rs"
  - "0.6h: aggregator.rs"
  - "0.5h: handlers/council_routes.rs"
  - "0.4h: audit/council_events.rs"
  - "3.5h: tests — 6 test files"
  - "2.0h: judge UI for scoring"
  - "1.5h: docs"

risk_if_skipped: "Without council, mastery + promotion decisions ad-hoc. Without DEC-2110 3-5 bounds, single judge becomes single point of failure. Without DEC-2114 median aggregation, outlier scores swing decisions."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship Hội đồng Chuyên môn at `services/learn/src/council/` with 3-5 judges + 5-dim scoring + median aggregation, 5 memory audit kinds.

1. **MUST** validate `score_dimension` against closed enum per DEC-2111, `council_status` per DEC-2112.

2. **MUST** enforce 3-5 judges per DEC-2110 at council convene.

3. **MUST** define tables at migration `0004`:
   ```sql
   CREATE TABLE learn_councils (
     council_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     candidate_member_id UUID NOT NULL,
     skill_id UUID,
     promotion_target_level INT,
     status TEXT NOT NULL DEFAULT 'convened'
       CHECK (status IN ('convened','scoring','completed','dismissed')),
     convened_by UUID NOT NULL,
     convened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     completed_at TIMESTAMPTZ,
     trace_id CHAR(32)
   );
   ALTER TABLE learn_councils ENABLE ROW LEVEL SECURITY;
   CREATE POLICY councils_rls ON learn_councils
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_councils FROM cyberos_app;
   GRANT UPDATE (status, completed_at) ON learn_councils TO cyberos_app;

   CREATE TABLE learn_council_judges (
     council_id UUID NOT NULL REFERENCES learn_councils(council_id),
     judge_id UUID NOT NULL,
     tenant_id UUID NOT NULL,
     assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     PRIMARY KEY (council_id, judge_id)
   );
   ALTER TABLE learn_council_judges ENABLE ROW LEVEL SECURITY;
   CREATE POLICY judges_rls ON learn_council_judges
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_council_judges FROM cyberos_app;

   CREATE TABLE learn_council_scores (
     score_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     council_id UUID NOT NULL REFERENCES learn_councils(council_id),
     judge_id UUID NOT NULL,
     dimension TEXT NOT NULL
       CHECK (dimension IN ('technical','leadership','impact','collaboration','growth_potential')),
     score INT NOT NULL CHECK (score >= 1 AND score <= 5),
     rationale TEXT,
     submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     trace_id CHAR(32),
     UNIQUE (council_id, judge_id, dimension)
   );
   ALTER TABLE learn_council_scores ENABLE ROW LEVEL SECURITY;
   CREATE POLICY scores_rls ON learn_council_scores
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_council_scores FROM cyberos_app;
   ```

4. **MUST** aggregate at `aggregator.rs::aggregate(council)` per DEC-2114:
   - For each dimension: median of judge scores
   - Overall recommendation: promote if median >= 4 on technical + leadership; hold if median 3; decline if median <3.

5. **MUST** mark complete only when all judges submitted all 5 dimensions.

6. **MUST** expose endpoints:
   ```text
   POST /v1/learn/councils                           (CHRO convenes)
   POST /v1/learn/councils/{id}/judges               (add judge — 3-5 enforced)
   POST /v1/learn/councils/{id}/scores               (judge submits)
   GET  /v1/learn/councils/{id}                      (status + aggregate if completed)
   POST /v1/learn/councils/{id}/dismiss
   ```

7. **MUST** emit 5 memory audit kinds per DEC-2115. PII per TASK-MEMORY-111: rationale SHA256; scores ok (small integers).

8. **MUST** thread trace_id from convene → scoring → complete → audit.

9. **MUST NOT** allow <3 or >5 judges per DEC-2110.

10. **MUST NOT** mutate prior score per DEC-2113.

11. **MUST NOT** double-score same dimension by same judge (UNIQUE).

---

## §2 — Why this design

**Why 3-5 judges (DEC-2110)?** Odd numbers + bounded — prevents tie + cognitive overhead.

**Why 5 dimensions (DEC-2111)?** Industry-standard rubric (Google L-ladder pattern).

**Why median (DEC-2114)?** Robust to outlier judges; mean would let one extreme score sway.

**Why per-judge anonymity (DEC-2113)?** Reduces social pressure; TASK-LEARN-005 enforces post-completion isolation.

---

## §3 — API contract

Sample council convene:
```json
POST /v1/learn/councils
{
  "candidate_member_id": "uuid",
  "skill_id": "uuid",
  "promotion_target_level": 4
}
```

Sample judge score:
```json
POST /v1/learn/councils/{id}/scores
{
  "dimension": "technical",
  "score": 4,
  "rationale": "Strong system design; needs more breadth in distributed systems."
}
```

Aggregate (when completed):
```json
{
  "council_id": "uuid",
  "status": "completed",
  "aggregate": {
    "technical": 4,
    "leadership": 3,
    "impact": 4,
    "collaboration": 4,
    "growth_potential": 4,
    "overall_recommendation": "promote"
  },
  "judges_count": 5
}
```

---

## §4 — Acceptance criteria
1. **score_dimension enum cardinality 5**. 2. **council_status enum cardinality 4**. 3. **3-5 judges enforced**. 4. **Score CHECK 1-5**. 5. **UNIQUE(council, judge, dimension)**. 6. **Median aggregation**. 7. **Completed when all judges × all dims submitted**. 8. **5 memory audit kinds emitted**. 9. **PII scrubbed (rationale SHA256)**. 10. **RLS denies cross-tenant**. 11. **CHRO-only convene**. 12. **Trace_id preserved**. 13. **Append-only via REVOKE except status cols**. 14. **Dismiss endpoint allowed**. 15. **Judge cannot self-score (member ≠ judge)**. 16. **Same-tenant judges only**. 17. **promotion_target_level optional (general assessment OK)**. 18. **Aggregate hidden until completed**. 19. **History queryable per candidate**. 20. **Overall recommendation logic per spec**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn 3_judges_minimum() {
    let ctx = TestContext::with_council_convened().await;
    ctx.add_judge(ctx.judge1).await;
    ctx.add_judge(ctx.judge2).await;
    let r = ctx.try_start_scoring(ctx.council_id).await;
    assert!(r.is_err());  // need 3+
    ctx.add_judge(ctx.judge3).await;
    let r2 = ctx.try_start_scoring(ctx.council_id).await;
    assert!(r2.is_ok());
}

#[tokio::test]
async fn median_aggregation() {
    let ctx = TestContext::with_5_judges_scored([3, 4, 4, 5, 5], "technical").await;
    ctx.complete_council().await;
    let agg = ctx.fetch_aggregate(ctx.council_id).await;
    assert_eq!(agg.technical, 4);  // median of [3,4,4,5,5]
}

#[tokio::test]
async fn double_score_rejected() {
    let ctx = TestContext::with_council_scoring().await;
    ctx.submit_score(ctx.judge_id, "technical", 4).await;
    let r = ctx.submit_score(ctx.judge_id, "technical", 5).await;
    assert!(r.is_err());  // UNIQUE
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-LEARN-001.
**Downstream:** TASK-LEARN-005 (isolation), TASK-LEARN-006 (promotion approval).
**Cross-module:** TASK-AUTH-101 (CHRO + judge roles), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Judge count <3 | validate | reject start | add judge |
| Judge count >5 | validate | reject add | inherent |
| Score out of range | CHECK | 400 | use 1-5 |
| Double score | UNIQUE | 409 | inherent |
| Judge = candidate | validate | reject | use different judge |
| Cross-tenant judge | RLS | 0 rows | inherent |
| Premature complete (missing scores) | check | reject | submit remaining |
| Judge withdraws mid-council | manual reassign | inherent | replacement judge |
| Decimal precision N/A | integers | inherent | inherent |
| Concurrent score | UNIQUE | first wins | inherent |

## §11 — Implementation notes
- §11.1 Median: even-count handled by floor(avg) for stability.
- §11.2 Overall recommendation matrix configurable in future; v1 hardcoded.
- §11.3 memory audit body: council_id, judge_id, dimension, score; rationale SHA256.
- §11.4 Judge UI shows other judges' status (submitted/pending) but NOT their scores.
- §11.5 TASK-LEARN-005 enforces post-completion data isolation outside council.

---

*End of TASK-LEARN-004 spec.*
