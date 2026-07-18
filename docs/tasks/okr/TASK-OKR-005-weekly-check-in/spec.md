---
id: TASK-OKR-005
title: "OKR weekly check-in — 1-10 confidence + rationale per KR with rolling 4-week history + trend visualization"
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
module: OKR
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CEO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OKR-001, TASK-OKR-006, TASK-MEMORY-111]
depends_on: [TASK-OKR-001]
blocks: [TASK-OKR-006]

source_pages:
  - website/docs/modules/okr.html#weekly-check-in

source_decisions:
  - DEC-2000 2026-05-17 — Weekly check-in: 1-10 confidence + free-text rationale; submitted Monday by KR owner
  - DEC-2001 2026-05-17 — Closed enum `confidence_trend` = {improving, steady, declining, unknown}; cardinality 4 (derived from 4-week rolling avg)
  - DEC-2002 2026-05-17 — Per-week immutable; corrections via new row with same ISO week (UNIQUE(kr_id, iso_week, version))
  - DEC-2003 2026-05-17 — Reminder cron Monday 09:00 tenant_tz; missing check-in = sev-3 alert to KR owner
  - DEC-2004 2026-05-17 — memory audit kinds: okr.checkin_submitted, okr.checkin_corrected, okr.checkin_missing_alert

build_envelope:
  language: rust 1.81
  service: cyberos/services/okr/
  new_files:
    - services/okr/migrations/0005_weekly_checkins.sql
    - services/okr/src/checkin/mod.rs
    - services/okr/src/checkin/trend_calculator.rs
    - services/okr/src/checkin/reminder_cron.rs
    - services/okr/src/handlers/checkin_routes.rs
    - services/okr/src/audit/checkin_events.rs
    - services/okr/tests/checkin_confidence_range_test.rs
    - services/okr/tests/checkin_trend_enum_cardinality_test.rs
    - services/okr/tests/checkin_immutability_test.rs
    - services/okr/tests/checkin_correction_via_new_row_test.rs
    - services/okr/tests/checkin_reminder_test.rs
    - services/okr/tests/checkin_audit_emission_test.rs

  modified_files:
    - services/okr/src/lib.rs

  allowed_tools:
    - file_read: services/okr/**
    - file_write: services/okr/{src,tests,migrations}/**
    - bash: cd services/okr && cargo test checkin

  disallowed_tools:
    - mutate prior check-in (per DEC-2002)
    - confidence outside 1-10 (per DEC-2000)

effort_hours: 5
subtasks:
  - "0.3h: 0005_weekly_checkins.sql"
  - "0.3h: checkin/mod.rs"
  - "0.4h: trend_calculator.rs"
  - "0.4h: reminder_cron.rs"
  - "0.4h: handlers/checkin_routes.rs"
  - "0.3h: audit/checkin_events.rs"
  - "2.0h: tests — 6 test files"
  - "0.6h: Member UI + trend viz + docs"
  - "0.3h: cron registration"

risk_if_skipped: "Without weekly check-ins, KR owners disengage → progress drifts unseen. Without DEC-2002 immutability, KR owners revise history (audit weakness). Without DEC-2003 reminder, check-ins skipped → no signal."
---

## §1 — Description (BCP-14 normative)

The OKR service **MUST** ship weekly check-in at `services/okr/src/checkin/` with 1-10 confidence + rationale + 4-week trend + Monday reminder, 3 memory audit kinds.

1. **MUST** validate confidence in 1-10 per DEC-2000 (CHECK constraint).

2. **MUST** validate `confidence_trend` against closed enum per DEC-2001.

3. **MUST** compute trend at `trend_calculator.rs::trend(kr, current_week)`:
   - Average confidence of last 4 weeks (excluding current).
   - improving: current > avg + 1
   - declining: current < avg - 1
   - steady: within ±1
   - unknown: <2 prior weeks of data

4. **MUST** define table at migration `0005`:
   ```sql
   CREATE TABLE okr_weekly_checkins (
     checkin_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     kr_id UUID NOT NULL,
     iso_week CHAR(8) NOT NULL,  -- 'YYYY-Www' e.g. '2026-W20'
     version INT NOT NULL DEFAULT 1,
     confidence INT NOT NULL CHECK (confidence >= 1 AND confidence <= 10),
     rationale TEXT NOT NULL,
     submitted_by UUID NOT NULL,
     submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     trace_id CHAR(32),
     UNIQUE (tenant_id, kr_id, iso_week, version)
   );
   CREATE INDEX checkins_kr_week_idx ON okr_weekly_checkins(tenant_id, kr_id, iso_week DESC);
   ALTER TABLE okr_weekly_checkins ENABLE ROW LEVEL SECURITY;
   CREATE POLICY checkins_rls ON okr_weekly_checkins
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON okr_weekly_checkins FROM cyberos_app;
   ```

5. **MUST** correct via new row per DEC-2002 — `version` increments; original preserved.

6. **MUST** run Monday reminder cron per DEC-2003 — for each KR owner missing this-week check-in, emit `okr.checkin_missing_alert`.

7. **MUST** expose endpoints:
   ```text
   POST /v1/okr/krs/{id}/checkins              body: {confidence, rationale}
   GET  /v1/okr/krs/{id}/checkins              (history desc time)
   GET  /v1/okr/krs/{id}/trend                 (derived from history)
   ```

8. **MUST** emit 3 memory audit kinds per DEC-2004. PII per TASK-MEMORY-111: rationale SHA-256 hashed.

9. **MUST** thread trace_id from submit → audit.

10. **MUST NOT** mutate prior check-in per DEC-2002.

11. **MUST NOT** accept confidence outside 1-10 per DEC-2000.

---

## §2 — Why this design

**Why 1-10 + rationale (DEC-2000)?** Number alone is noise; rationale captures the why for retro/learning.

**Why immutable with version (DEC-2002)?** Audit lineage; can't backdate confidence to look good in retros.

**Why ISO week (DEC-2002)?** Calendar weeks standard; UNIQUE constraint per week+version.

**Why Monday reminder (DEC-2003)?** Common cadence; gives owner full Monday to complete.

---

## §3 — API contract

Sample check-in:
```json
POST /v1/okr/krs/{id}/checkins
{
  "confidence": 7,
  "rationale": "On track; one risk is Q3 hire delay impacting milestone 4."
}
```

Response with trend:
```json
{
  "checkin_id": "uuid",
  "confidence": 7,
  "iso_week": "2026-W20",
  "trend": "steady",
  "rolling_4w_avg": 6.8
}
```

---

## §4 — Acceptance criteria
1. **Confidence 1-10 CHECK**. 2. **confidence_trend enum cardinality 4**. 3. **Rationale required**. 4. **Per-week immutable**. 5. **Correction via version+1**. 6. **UNIQUE(kr_id, iso_week, version)**. 7. **Trend calc from 4-week rolling**. 8. **<2 prior weeks → trend=unknown**. 9. **Monday reminder cron**. 10. **3 memory audit kinds emitted**. 11. **PII scrubbed (rationale SHA256)**. 12. **RLS denies cross-tenant**. 13. **KR owner-only submit**. 14. **Trace_id preserved**. 15. **Append-only via REVOKE**. 16. **History query desc time**. 17. **Trend recompute on each new check-in**. 18. **Missing check-in alert sev-3**. 19. **ISO week format enforced**. 20. **Rationale length capped 2000 chars**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn confidence_range_enforced() {
    let r = ctx.submit_checkin(ctx.kr_id, 11, "test").await;
    assert!(r.is_err());
    let r2 = ctx.submit_checkin(ctx.kr_id, 0, "test").await;
    assert!(r2.is_err());
}

#[tokio::test]
async fn correction_creates_v2() {
    let ctx = TestContext::with_checkin_v1().await;
    ctx.submit_checkin_correction(ctx.kr_id, 8, "corrected").await;
    let history = ctx.fetch_checkins(ctx.kr_id, "2026-W20").await;
    assert_eq!(history.len(), 2);
    assert_eq!(history[1].version, 2);
}

#[tokio::test]
async fn trend_improving_after_3w_climb() {
    let ctx = TestContext::with_checkin_history(vec![5, 6, 7]).await;
    ctx.submit_checkin(ctx.kr_id, 9, "great week").await;
    let trend = ctx.fetch_trend(ctx.kr_id).await;
    assert_eq!(trend, "improving");
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-OKR-001.
**Downstream:** TASK-OKR-006 (Monday digest uses check-ins).
**Cross-module:** TASK-MCP-007 (reminder cron), TASK-AUTH-101 (KR owner role), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Confidence out of range | CHECK | 400 | 1-10 |
| Rationale empty | validate | 400 | provide text |
| Duplicate version race | UNIQUE | 409 | retry with v+1 |
| Trend with <2 data | unknown | inherent | inherent |
| Cron skipped | catch-up | sev-3 | inherent |
| KR owner inactive | skip reminder | inherent | reassign |
| Cross-tenant submit | RLS | 403 | inherent |
| ISO week parse fail | validate | 400 | YYYY-Www format |
| Rationale > 2000 chars | validate | 400 | shorten |
| Concurrent submit | UNIQUE | second 409 | retry |

## §11 — Implementation notes
- §11.1 Trend calc pure function: `(history: Vec<(week, confidence)>, current) → Trend`.
- §11.2 Reminder cron via TASK-MCP-007 `kind: 'okr.checkin_reminder'`, Monday 09:00.
- §11.3 ISO week computed via `chrono::IsoWeek`.
- §11.4 memory audit body: kr_id, iso_week, confidence; rationale SHA256.
- §11.5 UI: shows 8-week sparkline + current confidence + trend arrow.

---

*End of TASK-OKR-005 spec.*
