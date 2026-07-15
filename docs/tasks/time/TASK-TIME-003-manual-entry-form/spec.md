---
id: TASK-TIME-003
title: "TIME manual entry form — retroactive time logging with date validation + per-day total cap + TASK-TIME-007 VN Labour Code cap integration"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
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
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TIME-001, TASK-TIME-002, TASK-TIME-007, TASK-AUTH-101, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-TIME-001]
blocks: []

source_pages:
  - website/docs/modules/time.html#manual-entry

source_decisions:
  - DEC-1400 2026-05-17 — Manual entry path for retroactive logging (forgot to start timer, off-network periods, mobile bulk-add); date must be within last 30 days unless engagement_admin overrides
  - DEC-1401 2026-05-17 — Per-day total cap: 24h total entries per Member per day; soft-block at 16h with admin override required; hard-block at 24h
  - DEC-1402 2026-05-17 — Validates via TASK-TIME-007 OT cap chain (vn-1 Members); rejects entries that would breach
  - DEC-1403 2026-05-17 — Closed enum `manual_entry_reason` = {forgot_to_start_timer, off_network, mobile_bulk_add, correction, retroactive_invoiced}; CI cardinality asserts 5
  - DEC-1404 2026-05-17 — Past 90 days requires `engagement_admin` approval; past 1 year requires `cfo` approval (older entries are usually corrections)
  - DEC-1405 2026-05-17 — memory audit kinds: time.manual_entry_created, time.manual_entry_blocked_24h_cap, time.manual_entry_overrode_16h_softblock, time.manual_entry_past_90d_approved, time.manual_entry_past_1y_approved

build_envelope:
  language: rust 1.81
  service: cyberos/services/time/
  new_files:
    - services/time/src/manual_entry/mod.rs
    - services/time/src/manual_entry/date_validator.rs
    - services/time/src/manual_entry/per_day_cap.rs
    - services/time/src/audit/manual_entry_events.rs
    - services/time/src/handlers/manual_entry_routes.rs
    - services/time/web/manual-entry-form.ts
    - services/time/tests/manual_entry_happy_test.rs
    - services/time/tests/manual_entry_30d_limit_test.rs
    - services/time/tests/manual_entry_24h_per_day_test.rs
    - services/time/tests/manual_entry_16h_softblock_test.rs
    - services/time/tests/manual_entry_90d_admin_approval_test.rs
    - services/time/tests/manual_entry_1y_cfo_approval_test.rs
    - services/time/tests/manual_entry_vn_ot_integration_test.rs
    - services/time/tests/manual_entry_reason_enum_cardinality_test.rs
    - services/time/tests/manual_entry_audit_emission_test.rs

  modified_files:
    - services/time/src/lib.rs

  allowed_tools:
    - file_read: services/time/**
    - file_write: services/time/{src,tests,web}/**
    - bash: cd services/time && cargo test manual_entry

  disallowed_tools:
    - allow date > 30d past without engagement_admin approval (per DEC-1404)
    - allow past 1 year without cfo approval (per DEC-1404)
    - bypass TASK-TIME-007 cap (per DEC-1402)
    - exceed 24h per-day cap (per DEC-1401)

effort_hours: 6
subtasks:
  - "0.4h: manual_entry/mod.rs + closed reason enum"
  - "0.4h: date_validator.rs (30d/90d/1y tiers)"
  - "0.5h: per_day_cap.rs (24h hard + 16h soft)"
  - "0.4h: TASK-TIME-007 integration"
  - "0.3h: audit/manual_entry_events.rs"
  - "0.4h: handlers/manual_entry_routes.rs"
  - "0.5h: web/manual-entry-form.ts"
  - "1.5h: tests — 9 test files"
  - "1.1h: integration smoke + cross-task test with TASK-TIME-007"

risk_if_skipped: "Without manual entry, Members can't log time they forgot to track via timer — lost billable revenue + audit incompleteness. Without DEC-1400 30-day default limit, fraudulent backdating risks (logging fake hours months later). Without DEC-1404 escalating approval tiers, no governance on suspicious old entries. Without DEC-1402 VN OT chain, manual entries become the cap-evasion path. Without DEC-1401 24h per-day cap, fat-finger errors (typed 80h instead of 8h) ship to invoices. The 6h effort covers the daily edge cases timer can't."
---

## §1 — Description (BCP-14 normative)

The TIME service **MUST** ship manual entry form at `services/time/src/manual_entry/` with date-window validation tiers (30d/90d/1y), 24h per-day hard cap + 16h soft-block, TASK-TIME-007 VN OT chain integration, 5 closed-enum reasons, and 5 memory audit kinds.

1. **MUST** define closed `manual_entry_reason` enum: `('forgot_to_start_timer','off_network','mobile_bulk_add','correction','retroactive_invoiced')` per DEC-1403. Cardinality 5.

2. **MUST** expose `POST /v1/time/entries/manual` body `{ engagement_id, project_id?, task_id?, entry_date, duration_seconds, description, reason, approval_override? }`. Handler:
   - Validates engagement membership.
   - Date-window check per §1 #3.
   - Per-day cap check per §1 #4.
   - TASK-TIME-007 OT cap check per §1 #5.
   - Creates TIME entry via TASK-TIME-001.
   - Emits `time.manual_entry_created` sev-2.

3. **MUST** enforce date-window tiers per DEC-1400 + DEC-1404:
   - `entry_date ≥ now() - 30d` → no approval required.
   - `30d < age ≤ 90d` → requires `engagement_admin` (caller has role OR `approval_override.subject_id` references one).
   - `90d < age ≤ 1y` → requires `cfo` approval.
   - `> 1y` → rejected with 412 + `entry_too_old`.
   - Approval-override path emits `time.manual_entry_past_90d_approved` or `time.manual_entry_past_1y_approved` sev-2.

4. **MUST** enforce 24h-per-day hard cap per DEC-1401. SUM(duration_seconds) for (member, entry_date) + new_duration > 86_400 → 412 + `daily_24h_cap_exceeded`. Emit `time.manual_entry_blocked_24h_cap` sev-2.

5. **MUST** emit 16h soft-block warning per DEC-1401. SUM + new > 57_600 (16h) AND ≤ 86_400 → require `approval_override.engagement_admin_subject_id`; without override → 412 + `daily_16h_softblock`. With override → audit `time.manual_entry_overrode_16h_softblock` sev-2.

6. **MUST** chain into TASK-TIME-007 OT cap check per DEC-1402. After per-day cap check, invoke `vn_labour::cap_check` for vn-1 Members. Breach → 412 + matching breach kind.

7. **MUST** validate `entry_date` not in future. `entry_date > today` → 400 + `future_date_invalid`.

8. **MUST** emit 5 memory audit kinds per DEC-1405. PII-scrub description via TASK-MEMORY-111.

9. **MUST** thread trace_id end-to-end.

10. **MUST NOT** allow > 1 year past entries (per DEC-1404).

11. **MUST NOT** bypass TASK-TIME-007 OT cap (per DEC-1402).

---

## §2 — Why this design (rationale)

**Why tiered approval (§1 #3, DEC-1404)?** Newer entries = legitimate forgetfulness; older entries = either bookkeeping cleanup or fraud. Tier escalation matches the suspicion gradient.

**Why 24h hard cap (§1 #4, DEC-1401)?** Physically impossible to work > 24h in 24h. Catches obvious typos before they pollute invoices.

**Why 16h soft-block (§1 #5)?** Possible but unusual; requires conscious approval. Catches plausible-but-suspicious entries.

**Why chain into TASK-TIME-007 (§1 #6, DEC-1402)?** Manual entry must respect all the same labour-law constraints as timer entries; otherwise it's the bypass path.

---

## §3 — API contract

```text
POST   /v1/time/entries/manual                       (member; with approval override)
GET    /v1/time/entries/manual/pending-approvals     (engagement_admin or cfo)
```

Body:
```json
{
  "engagement_id": "0190...",
  "project_id": "0190...",
  "entry_date": "2026-05-15",
  "duration_seconds": 7200,
  "description": "Sprint planning meeting",
  "reason": "forgot_to_start_timer",
  "approval_override": null
}
```

For past-90d:
```json
{
  ...
  "reason": "correction",
  "approval_override": { "engagement_admin_subject_id": "..." }
}
```

---

## §4 — Acceptance criteria

1. **manual_entry_reason cardinality 5**.
2. **30d default window** — entry 31d ago without override → 412.
3. **90d engagement_admin override** — entry 60d ago with override succeeds + sev-2 audit.
4. **1y cfo override** — entry 200d ago with cfo override succeeds.
5. **>1y rejected** — entry 400d ago → 412 + entry_too_old.
6. **24h cap** — Member with 23h59m already, +2min entry → 412.
7. **16h softblock** — Member with 14h, +3h entry without override → 412 + daily_16h_softblock.
8. **16h with override** — same scenario with engagement_admin override → succeeds.
9. **VN OT cap chained** — Member at 39h monthly OT, +90min entry (1.5h OT) → 412 monthly_40h_breach.
10. **Future date rejected** — entry_date = tomorrow → 400.
11. **5 memory audit kinds emitted**.
12. **Trace_id end-to-end**.
13. **PII scrub** — description hash in audit.
14. **Non-VN Member skips OT check** — sg-1 Member entries unaffected by OT chain.
15. **Same-day OT count post-entry** — entry creates row; subsequent timer-stop sees updated total.
16. **Engagement_admin not member** — admin from different engagement → 403.
17. **Approval override subject_id validated** — invalid override → 400.
18. **Audit kind per scenario** — happy → `time.manual_entry_created`; 24h → `_blocked_24h_cap`; etc.
19. **Description optional empty** — empty description allowed but warned.
20. **Reason required** — missing reason → 400.

---

## §5 — Verification

```rust
#[tokio::test]
async fn manual_entry_within_30d_succeeds() {
    let ctx = TestContext::with_member().await;
    let r = ctx.post_manual_entry(json!({
        "engagement_id": ctx.eng_id, "entry_date": yesterday(),
        "duration_seconds": 3600, "description": "test", "reason": "forgot_to_start_timer"
    })).await;
    assert_eq!(r.status(), 201);
}

#[tokio::test]
async fn 31d_requires_admin_override() {
    let ctx = TestContext::with_member().await;
    let r = ctx.post_manual_entry_at(31, None).await;
    assert_eq!(r.status(), 412);
    let r2 = ctx.post_manual_entry_at(31, Some(ctx.engagement_admin_id)).await;
    assert_eq!(r2.status(), 201);
}

#[tokio::test]
async fn 24h_per_day_hard_blocked() {
    let ctx = TestContext::with_member().await;
    ctx.seed_entries(ctx.member_id, today(), 23 * 3600 + 3540).await;  // 23h59m
    let r = ctx.post_manual_entry(json!({
        "engagement_id": ctx.eng_id, "entry_date": today(),
        "duration_seconds": 120, "description": "x", "reason": "forgot_to_start_timer"
    })).await;
    assert_eq!(r.status(), 412);
}

#[tokio::test]
async fn vn_ot_cap_chained() {
    let ctx = TestContext::with_vn_member().await;
    ctx.seed_ot_for_month(ctx.member_id, 39 * 3600).await;
    let r = ctx.post_manual_entry_with_duration(8.5 * 3600.0).await;  // 0.5h OT pushes monthly to 40.5
    assert_eq!(r.status(), 412);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["breach_kind"], "monthly_40h_breach");
}

// 5.5..5.10: future date, 5 cardinality, 16h softblock, override paths, audit emissions
```

---

## §7 — Dependencies

**Upstream:** TASK-TIME-001 (entry write).
**Cross-module:** TASK-TIME-007 (OT chain), TASK-AUTH-101 (engagement_admin + cfo roles), TASK-AI-003, TASK-MEMORY-111.

---

## §8 — Example payload

`time.manual_entry_overrode_16h_softblock`:
```json
{
  "kind": "time.manual_entry_overrode_16h_softblock",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.member.456",
  "trace_id": "...",
  "payload": {
    "member_subject_id_hash16": "f8a1...",
    "entry_date": "2026-05-17",
    "override_admin_subject_id_hash16": "9c4e...",
    "total_day_seconds_after": 75600
  }
}
```

---

## §9 — Open questions

Deferred:
- **Deferred:** Bulk manual entry CSV import — slice 2.
- **Deferred:** Calendar UI for retroactive bulk edit — slice 2.
- **Deferred:** Per-engagement custom date-window override — slice 2.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Date > 1 year | tier check | 412 | Caller rejects or escalates legally |
| 24h cap on bulk-add path | per-day check | 412 | Member spreads across days |
| Approval override missing | check | 412 with hint | Caller adds override |
| Invalid approval subject_id | role check | 400 | Caller fixes |
| TASK-TIME-007 OT breach | chained check | 412 with breach kind | Member splits to overtime-tier or new day |
| Future date | date check | 400 | Caller fixes |
| Cross-tenant engagement | RLS | 403 | Inherent |
| Description PII not scrubbed | TASK-MEMORY-111 | Audit dropped + sev-3 | Inherent |
| Same-day timer + manual race | concurrent inserts | Both checked individually; second may hit 24h cap | Inherent |
| Engagement membership lost mid-write | RLS | 403 at write | Inherent |
| Reason missing | validation | 400 | Inherent |
| Bulk import bypass | handler enforces same checks | Tested via TASK-TIME-007 §11.8 lint pattern | CI catches |
| Approval override expired | check expires_at | 412 | Re-request approval |
| Manual entry creates 0-duration | 0 allowed (placeholder) | Inherent | Member edits later |
| Duration > 24h single entry | per-day cap | 412 | Split into multiple entries |

---

## §11 — Implementation notes

**§11.1** Date window: `entry_date < today() - INTERVAL '30 days'` triggers approval check.

**§11.2** Approval override consumed at write time; `approval_override.engagement_admin_subject_id` validated against role table.

**§11.3** Per-day cap check uses TASK-TIME-007's aggregator (shared infrastructure).

**§11.4** TASK-TIME-007 chain invoked AFTER per-day cap (most-granular first).

**§11.5** UI form pre-fetches Member's today total to surface 16h warning client-side.

**§11.6** Bulk import (CSV) deferred to slice 2 but architecture supports it (same handler chain).

**§11.7** Audit row carries Member + admin override subject IDs as hashes.

**§11.8** Trace_id propagated from request through cap_check + write + audit.

**§11.9** Engagement_admin override validated by role table at write time (race-safe).

**§11.10** Future-date check uses tenant timezone (default Asia/Ho_Chi_Minh for VN).

---

*End of TASK-TIME-003 spec.*
