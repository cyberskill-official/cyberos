---
id: TASK-LEARN-005
title: "LEARN per-judge score isolation — never exit LEARN boundary; HR receives only summary + recommendation"
module: LEARN
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 7
slice: 7
owner: Stephen Cheng (CISO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-LEARN-004, TASK-LEARN-006, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-LEARN-004]
blocks: []

source_pages:
  - website/docs/modules/learn.html#per-judge-isolation

source_decisions:
  - DEC-2120 2026-05-17 — Per-judge scores NEVER leave LEARN service boundary post-completion; HR sees aggregate + recommendation only
  - DEC-2121 2026-05-17 — Closed enum `external_disclosure` = {none, aggregate_only, recommendation_only, judge_identities_only}; cardinality 4
  - DEC-2122 2026-05-17 — Cross-service read via TASK-LEARN-005 export endpoint applies disclosure filter automatically; raw scores rejected
  - DEC-2123 2026-05-17 — HR + REW consumers receive aggregate_only; CEO/CHRO receive recommendation_only; CISO can audit who saw what
  - DEC-2124 2026-05-17 — memory audit kinds: learn.external_disclosure_requested, learn.disclosure_filtered, learn.disclosure_denied, learn.unauthorized_raw_access_attempt

build_envelope:
  language: rust 1.81
  service: cyberos/services/learn/
  new_files:
    - services/learn/migrations/0005_disclosure_log.sql
    - services/learn/src/disclosure/mod.rs
    - services/learn/src/disclosure/filter.rs
    - services/learn/src/disclosure/access_gate.rs
    - services/learn/src/handlers/disclosure_routes.rs
    - services/learn/src/audit/disclosure_events.rs
    - services/learn/tests/disclosure_enum_cardinality_test.rs
    - services/learn/tests/raw_scores_denied_outside_learn_test.rs
    - services/learn/tests/hr_gets_aggregate_only_test.rs
    - services/learn/tests/disclosure_audit_emission_test.rs

  modified_files:
    - services/learn/src/handlers/council_routes.rs

  allowed_tools:
    - file_read: services/{learn,auth,hr,rew}/**
    - file_write: services/learn/{src,tests,migrations}/**
    - bash: cd services/learn && cargo test disclosure

  disallowed_tools:
    - expose raw per-judge scores via any cross-service API (per DEC-2120)
    - bypass filter (per DEC-2122)

effort_hours: 5
subtasks:
  - "0.3h: 0005_disclosure_log.sql"
  - "0.3h: disclosure/mod.rs"
  - "0.5h: filter.rs"
  - "0.5h: access_gate.rs"
  - "0.4h: handlers/disclosure_routes.rs"
  - "0.3h: audit/disclosure_events.rs"
  - "1.8h: tests — 4 test files"
  - "0.7h: docs + CISO audit dashboard"
  - "0.2h: council_routes.rs internal-only flag"

risk_if_skipped: "Without isolation, HR managers see raw judge scores → council judges feel exposed → score honesty drops. Without DEC-2123 role-based disclosure, full data leaks to wrong consumers. Without DEC-2124 audit, CISO can't detect leak."
---

## §1 — Description (BCP-14 normative)

The LEARN service **MUST** ship per-judge score isolation at `services/learn/src/disclosure/` enforcing disclosure filter on all cross-service reads, 4 memory audit kinds.

1. **MUST** validate `external_disclosure` against closed enum per DEC-2121.

2. **MUST** filter at `filter.rs::apply(council_data, requester_role)` per DEC-2120:
   - aggregate_only: median scores per dim + recommendation
   - recommendation_only: just promote/hold/decline + reasoning summary
   - judge_identities_only: who served (for transparency about who reviewed)
   - none: 403

3. **MUST** gate at `access_gate.rs::check(requester, council)` per DEC-2123:
   - HR consumers (TASK-HR-008): aggregate_only
   - REW consumers: aggregate_only
   - CEO + CHRO: recommendation_only
   - CISO: full access for audit (logged)
   - Other roles: none (403)

4. **MUST** define disclosure log at migration `0005`:
   ```sql
   CREATE TABLE learn_disclosure_log (
     log_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     council_id UUID NOT NULL,
     requester_id UUID NOT NULL,
     requester_role TEXT NOT NULL,
     disclosure_kind TEXT NOT NULL
       CHECK (disclosure_kind IN ('none','aggregate_only','recommendation_only','judge_identities_only')),
     succeeded BOOLEAN NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX disclosure_council_idx ON learn_disclosure_log(tenant_id, council_id, created_at DESC);
   ALTER TABLE learn_disclosure_log ENABLE ROW LEVEL SECURITY;
   CREATE POLICY disclosure_rls ON learn_disclosure_log
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON learn_disclosure_log FROM cyberos_app;
   ```

5. **MUST** expose disclosure endpoint:
   ```text
   GET /v1/learn/councils/{id}/disclosure?kind=aggregate_only
   ```
   Returns filtered data; rejects unauthorized roles.

6. **MUST** internal council scoring endpoints (TASK-LEARN-004) marked `internal_only` — direct cross-service raw access blocked.

7. **MUST** emit 4 memory audit kinds per DEC-2124. PII per TASK-MEMORY-111: scores never in memory chain.

8. **MUST** thread trace_id from request → gate → filter → audit.

9. **MUST NOT** expose raw per-judge scores via any cross-service API per DEC-2120.

10. **MUST NOT** bypass filter per DEC-2122 (no admin override outside CISO audit).

---

## §2 — Why this design

**Why isolation (DEC-2120)?** Judge honesty depends on confidence scores won't haunt them politically; exposure breaks the system.

**Why role-based disclosure (DEC-2123)?** Different consumers need different views — HR needs scores, CEO needs decisions, others need nothing.

**Why CISO full-access (DEC-2123)?** Audit must verify isolation works; CISO access is logged → traceable.

**Why disclosure log (DEC-2124)?** Audit "who saw what about whom"; investigates suspected leaks.

---

## §3 — API contract

```text
GET /v1/learn/councils/{id}/disclosure?kind=aggregate_only
```

Sample aggregate_only response:
```json
{
  "council_id": "uuid",
  "candidate_member_id": "uuid",
  "aggregate": {
    "technical": 4,
    "leadership": 3,
    "impact": 4,
    "collaboration": 4,
    "growth_potential": 4
  },
  "overall_recommendation": "promote",
  "judges_count": 5
}
```

Sample 403 attempt:
```json
{
  "error": "disclosure_denied",
  "reason": "role 'engineer' cannot access learn.council disclosures"
}
```

---

## §4 — Acceptance criteria
1. **external_disclosure enum cardinality 4**. 2. **HR gets aggregate_only**. 3. **REW gets aggregate_only**. 4. **CEO+CHRO get recommendation_only**. 5. **CISO gets full (logged)**. 6. **Other roles 403**. 7. **Raw scores never exposed externally**. 8. **4 memory audit kinds emitted**. 9. **PII scrubbed (scores never in chain)**. 10. **RLS denies cross-tenant**. 11. **Disclosure log per access**. 12. **Trace_id preserved**. 13. **Append-only log via REVOKE**. 14. **Internal endpoint marker prevents external use**. 15. **CISO audit dashboard accessible**. 16. **Unauthorized attempt → sev-2 audit**. 17. **Filter pure function**. 18. **Cross-tenant attempt rejected**. 19. **Per-council disclosure history queryable**. 20. **Aggregate_only excludes per-judge breakdown**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn raw_scores_denied_to_hr() {
    let ctx = TestContext::with_completed_council_and_hr_role().await;
    let r = ctx.try_read_raw_scores_as_hr(ctx.council_id).await;
    assert_eq!(r.status_code, 403);
    let log = ctx.fetch_disclosure_log(ctx.council_id).await;
    assert!(log.iter().any(|l| !l.succeeded));
}

#[tokio::test]
async fn hr_gets_aggregate_only() {
    let ctx = TestContext::with_completed_council_and_hr_role().await;
    let r = ctx.read_disclosure_as_hr(ctx.council_id, "aggregate_only").await;
    assert!(r.contains("technical"));
    assert!(!r.contains("judge_id"));  // raw judge IDs not exposed
}

#[tokio::test]
async fn unauthorized_attempt_audited() {
    let ctx = TestContext::with_engineer_role().await;
    ctx.try_read_disclosure(ctx.council_id).await;
    let audits = ctx.fetch_memory_audits("learn.disclosure_denied").await;
    assert!(!audits.is_empty());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-LEARN-004.
**Downstream:** TASK-LEARN-006 (promotion uses recommendation_only).
**Cross-module:** TASK-AUTH-101 (role check), TASK-HR-008 (HR consumer), FR-REW (REW consumer), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Role not mapped | gate default | 403 + sev-2 | add role mapping |
| Disclosure kind not in enum | reject | 400 | use valid |
| Cross-tenant read | RLS | 403 | inherent |
| Internal endpoint accessed externally | flag check | 403 + sev-1 | inherent |
| Filter bug exposes scores | code review + tests | sev-1 | fix |
| Disclosure log table fills | partition by month | inherent | maintenance |
| CISO audit dashboard slow | index | tune | inherent |
| Concurrent disclosure | inherent | both logged | inherent |
| Cross-service raw query attempt | gate | 403 + sev-2 | inherent |
| Council not completed | check | 412 | wait |

## §11 — Implementation notes
- §11.1 Filter is pure function: `(council_full, disclosure_kind) → filtered_view`.
- §11.2 Internal endpoint marker: `#[router(internal_only)]` macro; gateway rejects external requests.
- §11.3 memory audit body: council_id, requester_role, disclosure_kind, succeeded; no score data.
- §11.4 CISO dashboard: SELECT * FROM disclosure_log ORDER BY created_at DESC LIMIT 100.
- §11.5 Unauthorized attempts trigger sev-2 alert to CISO via TASK-CHAT-005.

---

*End of TASK-LEARN-005 spec.*
