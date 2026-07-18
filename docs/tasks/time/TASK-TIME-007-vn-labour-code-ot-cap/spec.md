---
id: TASK-TIME-007
title: "TIME VN Labour Code Art. 107 OT cap — hard-block at entry write when monthly OT > 40h or yearly OT > 200h (300h with regulator approval)"
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
module: TIME
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CLO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-TIME-002, TASK-TIME-003, TASK-HR-001, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-TIME-001]
blocks: []

source_pages:
  - website/docs/modules/time.html#vn-labour-code
  # Art. 107
  - https://thuvienphapluat.vn/van-ban/Lao-dong-Tien-luong/Bo-luat-lao-dong-2019-333670.aspx

source_decisions:
  - DEC-1390 2026-05-17 — VN Labour Code Art. 107: OT capped at 40h/month + 200h/year (300h/year with regulator approval); standard workday 8h; OT = hours beyond 8/day or 48/week
  - DEC-1391 2026-05-17 — Hard-block at entry write — refuses to record TIME entry that would breach; soft warnings at 80% threshold
  - DEC-1392 2026-05-17 — Closed enum `ot_breach_kind` = {monthly_40h_breach, yearly_200h_breach, yearly_300h_breach_no_approval, daily_4h_breach, weekly_12h_breach}; CI cardinality asserts 5
  - DEC-1393 2026-05-17 — Per-Member `vn_ot_approval` flag = 'standard' (200h cap) or 'extended' (300h cap with regulator approval); HR-001 owns
  - DEC-1394 2026-05-17 — Applies ONLY to vn-1 residency Members (DEC-1390 is VN-specific); other residencies subject to their own labour-law tasks (future)
  - DEC-1395 2026-05-17 — Daily/weekly hard caps: 4h OT/day, 12h OT/week per Art. 107(2); breach blocks entry
  - DEC-1396 2026-05-17 — memory audit kinds: time.ot_warning_issued, time.ot_breach_blocked, time.ot_approval_changed

language: rust 1.81
service: cyberos/services/time/
new_files:
  - services/time/migrations/0003_vn_ot_tracking.sql
  - services/time/src/vn_labour/mod.rs
  - services/time/src/vn_labour/cap_check.rs
  - services/time/src/vn_labour/aggregator.rs
  - services/time/src/audit/vn_ot_events.rs
  - services/time/tests/vn_ot_40h_monthly_blocked_test.rs
  - services/time/tests/vn_ot_200h_yearly_blocked_test.rs
  - services/time/tests/vn_ot_300h_with_approval_test.rs
  - services/time/tests/vn_ot_4h_daily_blocked_test.rs
  - services/time/tests/vn_ot_12h_weekly_blocked_test.rs
  - services/time/tests/vn_ot_warning_80pct_test.rs
  - services/time/tests/vn_ot_non_vn_skip_test.rs
  - services/time/tests/vn_ot_breach_enum_cardinality_test.rs
  - services/time/tests/vn_ot_audit_emission_test.rs

modified_files:
  # invoke cap_check pre-write
  - services/time/src/entry/create.rs

allowed_tools:
  - file_read: services/{time,hr}/**
  - file_write: services/time/{src,tests,migrations}/**
  - bash: cd services/time && cargo test vn_ot

disallowed_tools:
  - allow entry write that breaches caps (per DEC-1391 — hard block)
  - skip approval validation for 300h tier (per DEC-1393)
  - apply VN caps to non-vn-1 Members (per DEC-1394)

effort_hours: 4
subtasks:
  - "0.3h: 0003_vn_ot_tracking.sql + closed enum"
  - "0.5h: vn_labour/aggregator.rs (per-Member daily/weekly/monthly/yearly OT sums)"
  - "0.5h: vn_labour/cap_check.rs (4-tier breach detection)"
  - "0.3h: audit/vn_ot_events.rs"
  - "0.3h: integration into entry/create.rs"
  - "1.5h: tests — 9 test files covering all breach types + approval tiers + warnings"
  - "0.6h: HR-side approval flag wiring"

risk_if_skipped: "Without VN Labour Code Art. 107 enforcement, vn-1 tenants accumulate uncompensated OT liability + risk MOLISA inspections (fines up to VND 75M per violation). Without DEC-1391 hard-block, soft-only warnings get ignored → systemic violation. Without DEC-1393 approval tier, 300h-approved Members can't legitimately use their headroom OR 200h-cap Members illegitimately log up to 300h. Without DEC-1394 residency scoping, non-VN Members get rejected for hours legal in their jurisdiction. The 4h effort lands the labour-compliance gate that protects every VN tenant from regulatory risk."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship VN Labour Code Art. 107 OT cap enforcement at `services/time/src/vn_labour/` with 4-tier hard-blocks (monthly + yearly + daily + weekly), per-Member approval tier flag, 80% soft warnings, vn-1-residency scoping, and 3 memory audit kinds.

1. **MUST** define closed `ot_breach_kind` enum: `('monthly_40h_breach','yearly_200h_breach','yearly_300h_breach_no_approval','daily_4h_breach','weekly_12h_breach')` per DEC-1392. Cardinality 5.

2. **MUST** apply caps ONLY when `Member's tenant.residency = 'vn-1'` per DEC-1394. Non-VN Members skip cap check entirely (other labour-law tasks handle).

3. **MUST** compute OT = `MAX(0, daily_total_hours - 8)` per Art. 107. Standard workday baseline 8h; everything beyond is OT.

4. **MUST** aggregate OT at 4 granularities via `vn_labour/aggregator.rs`:
   - **Daily**: SUM(OT hours) WHERE entry_date = $today.
   - **Weekly**: SUM(OT hours) WHERE entry_date >= $week_start (Monday-Sunday).
   - **Monthly**: SUM(OT hours) WHERE entry_month = $current_month.
   - **Yearly**: SUM(OT hours) WHERE entry_year = $current_year.

5. **MUST** check 4 hard caps pre-entry-write per DEC-1391 + DEC-1395:
   - **Daily**: new_daily_ot > 4h → `daily_4h_breach`.
   - **Weekly**: new_weekly_ot > 12h → `weekly_12h_breach`.
   - **Monthly**: new_monthly_ot > 40h → `monthly_40h_breach`.
   - **Yearly (standard tier)**: new_yearly_ot > 200h AND approval='standard' → `yearly_200h_breach`.
   - **Yearly (extended tier)**: new_yearly_ot > 300h → `yearly_300h_breach_no_approval` (regardless of approval; 300h is absolute cap).
   - First-matching breach returned; entry write blocked with 412 + breach kind.

6. **MUST** emit 80% soft warning per DEC-1391. If new_total > 0.80 × cap AND new_total ≤ cap: write proceeds + emits `time.ot_warning_issued` sev-3 + UI banner.

7. **MUST** consume per-Member `vn_ot_approval` flag per DEC-1393 from `hr_members.vn_ot_approval`. TASK-HR-001 owns this field; CLO sets after MOLISA approval received.

8. **MUST** define `vn_ot_tracking` table at migration `0003` for materialised aggregates: `(member_subject_id UUID, year INT, month INT, day DATE, week_start DATE, daily_ot_seconds INT, weekly_ot_seconds INT, monthly_ot_seconds INT, yearly_ot_seconds INT, updated_at TIMESTAMPTZ, PRIMARY KEY (member_subject_id, day))`. Updated transactionally on each TIME entry write. RLS scoped.

9. **MUST** expose approval-change endpoint `POST /v1/admin/members/{member_id}/vn-ot-approval` body `{ tier, molisa_doc_ref, expires_at }`. Caller has `clo` role. Updates `hr_members.vn_ot_approval` + emits `time.ot_approval_changed` sev-1.

10. **MUST** emit 3 memory audit kinds per DEC-1396:
    - `time.ot_warning_issued` (sev-3)
    - `time.ot_breach_blocked` (sev-2 — material entry rejection)
    - `time.ot_approval_changed` (sev-1 — compliance event)

11. **MUST** thread trace_id end-to-end.

12. **MUST NOT** allow entry write that breaches per DEC-1391.

13. **MUST NOT** apply VN caps to non-vn-1 Members per DEC-1394.

---

## §2 — Why this design (rationale)

**Why hard-block (§1 #5, DEC-1391)?** Soft warnings get ignored. Hard-block forces conversation: either Member legitimately stops, or admin investigates rate-card/staffing issue, or extended approval requested.

**Why 4 cap granularities (§1 #4-5)?** Art. 107 specifies all 4 (daily, weekly, monthly, yearly). Single yearly aggregate would let Member burn 300h in January and crash; granular limits enforce smooth distribution.

**Why approval tier flag (§1 #7, DEC-1393)?** MOLISA-approved 300h tier is per-Member (not org-wide). Per-Member flag lets CLO grant on case-by-case basis without org-wide policy change.

**Why residency scoping (§1 #2, DEC-1394)?** Non-VN Members have their own labour laws (EU 48h/week, US no federal cap, etc.). Applying VN caps to non-VN Members is wrong; future tasks cover other jurisdictions.

---

## §3 — API contract

```sql
-- 0003_vn_ot_tracking.sql
CREATE TYPE ot_breach_kind AS ENUM ('monthly_40h_breach','yearly_200h_breach','yearly_300h_breach_no_approval','daily_4h_breach','weekly_12h_breach');

CREATE TABLE vn_ot_tracking (
  member_subject_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  day DATE NOT NULL,
  week_start DATE NOT NULL,
  year INT NOT NULL,
  month INT NOT NULL,
  daily_ot_seconds INT NOT NULL DEFAULT 0,
  weekly_ot_seconds INT NOT NULL DEFAULT 0,
  monthly_ot_seconds INT NOT NULL DEFAULT 0,
  yearly_ot_seconds INT NOT NULL DEFAULT 0,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (member_subject_id, day)
);
ALTER TABLE vn_ot_tracking ENABLE ROW LEVEL SECURITY;
CREATE POLICY vn_ot_tracking_rls ON vn_ot_tracking
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON vn_ot_tracking FROM cyberos_app;
GRANT UPDATE (daily_ot_seconds, weekly_ot_seconds, monthly_ot_seconds, yearly_ot_seconds, updated_at)
  ON vn_ot_tracking TO cyberos_app;
```

Endpoints:
```text
POST   /v1/admin/members/{id}/vn-ot-approval                  (clo)
GET    /v1/time/vn-ot/status?member_id=...                     (member own; or hr_admin)
```

---

## §4 — Acceptance criteria

1. **ot_breach_kind cardinality 5**.
2. **Monthly 40h hard-block** — 40h+1min entry → 412 + monthly_40h_breach.
3. **Yearly 200h block (standard)** — Member at 200h0min, +1min entry → 412 + yearly_200h_breach.
4. **300h with approval** — Member with tier='extended', at 250h, +30min entry → succeeds.
5. **300h absolute cap** — Member with tier='extended', at 300h0min, +1min → 412 + yearly_300h_breach_no_approval.
6. **Daily 4h breach** — single day 4h+1min OT → 412.
7. **Weekly 12h breach** — sum across week 12h+1min → 412.
8. **80% warning emitted** — 32h monthly (80% of 40h) → entry succeeds + audit warning.
9. **Non-VN Member skipped** — sg-1 Member 50h monthly OT → succeeds without check.
10. **Approval change requires CLO** — non-CLO call → 403.
11. **Approval change emits sev-1 audit** — CLO updates tier → `time.ot_approval_changed`.
12. **OT calculation correct** — 9h day = 1h OT.
13. **Trace_id end-to-end**.
14. **PII scrub** — audit row carries Member hash only.
15. **Aggregator transactional** — entry insert + tracking update atomic.
16. **3 memory audit kinds emitted** in full lifecycle.
17. **RLS isolation** — cross-tenant tracking invisible.
18. **Concurrent entries respect cap** — race-safe via SELECT FOR UPDATE on tracking row.
19. **Week boundary correctly Monday** — entry on Sunday counts to that week.
20. **Year rollover** — Jan 1 entry counts to new year; Dec 31 to old.

---

## §5 — Verification

```rust
#[tokio::test]
async fn monthly_40h_hard_blocked() {
    let ctx = TestContext::with_vn_member().await;
    ctx.seed_ot_for_month(ctx.member_id, 40 * 3600).await;  // exactly 40h
    let r = ctx.create_entry_with_ot(ctx.member_id, 60).await;  // +1min
    assert_eq!(r.status(), 412);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["breach_kind"], "monthly_40h_breach");
}

#[tokio::test]
async fn extended_approval_300h_works() {
    let ctx = TestContext::with_vn_member_with_extended_approval().await;
    ctx.seed_ot_for_year(ctx.member_id, 250 * 3600).await;
    let r = ctx.create_entry_with_ot(ctx.member_id, 30 * 60).await;
    assert_eq!(r.status(), 201);
}

#[tokio::test]
async fn non_vn_member_skipped() {
    let ctx = TestContext::with_sg_member().await;
    ctx.seed_ot_for_month(ctx.member_id, 50 * 3600).await;
    let r = ctx.create_entry_with_ot(ctx.member_id, 8 * 3600).await;
    assert_eq!(r.status(), 201);
}

#[tokio::test]
async fn 80pct_warning_emitted() {
    let ctx = TestContext::with_vn_member().await;
    ctx.seed_ot_for_month(ctx.member_id, 30 * 3600).await;  // 75% of 40h
    let r = ctx.create_entry_with_ot(ctx.member_id, 3 * 3600).await;  // pushes to 33h (82.5%)
    assert_eq!(r.status(), 201);
    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "time.ot_warning_issued"));
}

// 5.5..5.10: daily/weekly breaches, approval gate, aggregator transactional, race-safety
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-001 (entry write path).
**Cross-module:** TASK-HR-001 (vn_ot_approval flag), TASK-TIME-002 (timer integration), TASK-AUTH-101 (clo role), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.ot_breach_blocked`:
```json
{
  "kind": "time.ot_breach_blocked",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.member.456",
  "trace_id": "...",
  "payload": {
    "member_subject_id_hash16": "f8a1...",
    "breach_kind": "monthly_40h_breach",
    "attempted_ot_seconds": 60,
    "current_monthly_ot_seconds": 144000,
    "cap_seconds": 144000
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Non-VN labour codes (EU 48h, US OSHA) — slice 2 / per-jurisdiction tasks.
- **Deferred:** Tet/National holiday triple-pay flagging — slice 2.
- **Deferred:** Per-engagement OT cap (some clients want stricter) — slice 2.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Aggregator out-of-date | SELECT FOR UPDATE locks tracking row | Inherent consistency | None needed |
| Approval expired mid-year | check expires_at | Treated as 'standard' tier | CLO renews |
| Concurrent entry race | row lock | First wins | Inherent |
| Bulk import bypasses check | import handler must invoke same check | Sev-2 if missing | Code review |
| Cap reached exactly | inclusive vs exclusive | 412 at +1s past cap | Inherent |
| Member transferred to vn-1 mid-year | check at write-time only | Subsequent entries enforced | History grandfathered |
| Daily cap reached but week not | order of checks | Daily wins (more granular) | Inherent |
| Year-end rollover entry | timestamp check | Counts to entry's date year | Inherent |
| Negative OT (correction reduces total) | aggregator handles signed | Subtracts correctly | Inherent |
| Tracking table desync vs entries | nightly reconciliation job | Sev-2 if drift detected | Rebuild from entries |
| Cross-tenant attempt | RLS | 0 rows | Inherent |
| Approval ref doc lost | TEXT field; no FK | Soft loss; CLO re-attaches | Inherent |
| Member changes residency from vn-1 to sg-1 | residency immutable per TASK-TEN-103 | Cannot happen | Inherent |
| Bulk entry retroactively breaches | check at write per entry | Each entry checked individually | Inherent |
| Approval tier changed mid-write | tx isolation | Last-write semantics | Race rare |

---

## §11 — Implementation notes

**§11.1** Aggregator uses materialized table for O(1) cap lookup; transactional update on every TIME entry write.

**§11.2** Week boundary: Monday 00:00 tenant timezone (default Asia/Ho_Chi_Minh).

**§11.3** Year boundary: Jan 1 00:00 tenant timezone.

**§11.4** Cap order checked: daily → weekly → monthly → yearly (most granular first).

**§11.5** OT computed per entry: `OT = MAX(0, entry_seconds_in_day - (8*3600 - prior_today_seconds))`.

**§11.6** Concurrent insert race: `SELECT ... FROM vn_ot_tracking WHERE member_id=$1 AND day=$2 FOR UPDATE` before write.

**§11.7** Approval expires_at check: if expired, fallback to 'standard' tier silently + emit sev-3 warning.

**§11.8** Bulk import handler MUST invoke same cap_check per entry; CI lint enforces.

**§11.9** Nightly reconciliation: SUM(entries.duration) vs tracking.daily_ot_seconds; sev-2 alert on drift > 60s.

**§11.10** PII: Member identifier hashed to 16-hex in chain; raw retained in DB.

---

*End of TASK-TIME-007 spec.*
