---
id: FR-INV-011
title: "INV revenue recognition — ASC 606 / IFRS 15 compliant deferred-revenue rollforward with monthly journal entries + per-engagement schedule"
module: INV
priority: MUST
status: draft
verify: T
phase: P2
milestone: P2 · slice 2
slice: 2
owner: Stephen Cheng (CFO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-INV-001, FR-INV-002, FR-INV-009, FR-AI-003, FR-MEMORY-111]
depends_on: [FR-INV-001]
blocks: []

source_pages:
  - website/docs/modules/inv.html#revenue
  - https://asc.fasb.org/topic&trid=49044639  # ASC 606
  - https://www.ifrs.org/issued-standards/list-of-standards/ifrs-15-revenue/  # IFRS 15

source_decisions:
  - DEC-1560 2026-05-17 — Revenue recognition method per engagement: {time-based (subscription), milestone-based (project), pct-completion (consulting), point-in-time (one-off)}; closed enum cardinality 4
  - DEC-1561 2026-05-17 — Closed enum `recognition_method` = {time_based, milestone_based, pct_completion, point_in_time}; cardinality 4
  - DEC-1562 2026-05-17 — Monthly rollforward job at end-of-month tenant_timezone — computes period revenue + deferred revenue change
  - DEC-1563 2026-05-17 — Journal entry pairs: Debit Deferred Revenue / Credit Revenue (recognition); Debit AR / Credit Deferred Revenue (invoice issuance)
  - DEC-1564 2026-05-17 — Per-engagement recognition_schedule table: monthly buckets with planned + recognized amounts; reconcile_status tracked
  - DEC-1565 2026-05-17 — Recognition triggers memory audit: inv.revenue_recognized, inv.revenue_deferred, inv.revenue_rollforward_completed, inv.revenue_rollforward_failed; PII scrubbed (amount → SHA256)
  - DEC-1566 2026-05-17 — Snapshots: recognition rollforward at each EOM is IMMUTABLE — never re-computed (audit lineage); corrections via prior-period adjustment journal entry

build_envelope:
  language: rust 1.81
  service: cyberos/services/invoicing/
  new_files:
    - services/invoicing/migrations/0010_recognition.sql
    - services/invoicing/src/recognition/mod.rs
    - services/invoicing/src/recognition/scheduler.rs
    - services/invoicing/src/recognition/rollforward.rs
    - services/invoicing/src/recognition/journal_entry.rs
    - services/invoicing/src/handlers/recognition_routes.rs
    - services/invoicing/src/audit/recognition_events.rs
    - services/invoicing/tests/recognition_time_based_test.rs
    - services/invoicing/tests/recognition_milestone_based_test.rs
    - services/invoicing/tests/recognition_pct_completion_test.rs
    - services/invoicing/tests/recognition_point_in_time_test.rs
    - services/invoicing/tests/recognition_method_enum_cardinality_test.rs
    - services/invoicing/tests/recognition_rollforward_immutable_test.rs
    - services/invoicing/tests/recognition_journal_entry_test.rs
    - services/invoicing/tests/recognition_audit_emission_test.rs

  modified_files:
    - services/invoicing/src/lib.rs

  allowed_tools:
    - file_read: services/invoicing/**
    - file_write: services/invoicing/{src,tests,migrations}/**
    - bash: cd services/invoicing && cargo test recognition

  disallowed_tools:
    - mutate prior-period rollforward (per DEC-1566)
    - use float for amounts (rust_decimal only)
    - skip journal entry on recognition (per DEC-1563)

effort_hours: 5
sub_tasks:
  - "0.4h: 0010_recognition.sql"
  - "0.3h: recognition/mod.rs"
  - "0.5h: scheduler.rs (per-engagement schedule build)"
  - "0.7h: rollforward.rs (monthly EOM job)"
  - "0.5h: journal_entry.rs (DR/CR pairs)"
  - "0.4h: handlers/recognition_routes.rs"
  - "0.3h: audit/recognition_events.rs"
  - "1.6h: tests — 8 test files"
  - "0.3h: cron registration"

risk_if_skipped: "Without revenue recognition, financial statements violate ASC 606/IFRS 15 — uninvestable. Without DEC-1562 rollforward, deferred revenue grows unbounded. Without DEC-1566 immutable snapshots, prior-period restatements untraceable (audit failure)."
---

## §1 — Description (BCP-14 normative)

The INV service **MUST** ship revenue recognition at `services/invoicing/src/recognition/` supporting 4 methods, monthly EOM rollforward, journal entry pairs, immutable snapshots, 4 memory audit kinds.

1. **MUST** support 4 recognition methods per DEC-1560:
   - `time_based`: equal monthly amount over engagement.start_date → end_date
   - `milestone_based`: recognize when milestone achieved (FR-PROJ-002 milestone marked complete)
   - `pct_completion`: recognize by % of project hours billed (uses FR-TIME-002 time entries)
   - `point_in_time`: full recognition on invoice send (one-off services)

2. **MUST** validate `recognition_method` against closed enum per DEC-1561.

3. **MUST** build schedule at engagement creation via `scheduler.rs::build(engagement)`:
   - time_based → equal split across N months
   - milestone_based → per-milestone bucket
   - pct_completion → recompute on hours change
   - point_in_time → single bucket at send_date

4. **MUST** run rollforward at EOM tenant_timezone per DEC-1562 via FR-MCP-007 task:
   - For each engagement, compute period revenue recognized
   - Emit journal entry pair per DEC-1563
   - Insert immutable snapshot row

5. **MUST** generate journal entry pairs per DEC-1563:
   - On invoice issuance: `DR AR / CR Deferred Revenue`
   - On recognition: `DR Deferred Revenue / CR Revenue`
   - Pairs balance (sum DR == sum CR per entry)

6. **MUST** define tables at migration `0010`:
   ```sql
   CREATE TABLE recognition_schedules (
     schedule_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     engagement_id UUID NOT NULL,
     recognition_method TEXT NOT NULL
       CHECK (recognition_method IN ('time_based','milestone_based','pct_completion','point_in_time')),
     period_start DATE NOT NULL,
     period_end DATE NOT NULL,
     planned_amount NUMERIC(18,4) NOT NULL,
     recognized_amount NUMERIC(18,4) NOT NULL DEFAULT 0,
     currency TEXT NOT NULL,
     reconcile_status TEXT NOT NULL DEFAULT 'pending'
       CHECK (reconcile_status IN ('pending','partial','complete')),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE UNIQUE INDEX recognition_schedule_period_idx
     ON recognition_schedules(tenant_id, engagement_id, period_start, period_end);
   ALTER TABLE recognition_schedules ENABLE ROW LEVEL SECURITY;
   CREATE POLICY rec_sched_rls ON recognition_schedules
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON recognition_schedules FROM cyberos_app;
   GRANT UPDATE (recognized_amount, reconcile_status, updated_at) ON recognition_schedules TO cyberos_app;

   CREATE TABLE recognition_snapshots (
     snapshot_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     period_end DATE NOT NULL,
     engagement_id UUID NOT NULL,
     period_revenue NUMERIC(18,4) NOT NULL,
     deferred_revenue_change NUMERIC(18,4) NOT NULL,
     ending_deferred_revenue NUMERIC(18,4) NOT NULL,
     currency TEXT NOT NULL,
     journal_entry_id UUID,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE UNIQUE INDEX rec_snap_period_idx
     ON recognition_snapshots(tenant_id, engagement_id, period_end);
   ALTER TABLE recognition_snapshots ENABLE ROW LEVEL SECURITY;
   CREATE POLICY rec_snap_rls ON recognition_snapshots
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON recognition_snapshots FROM cyberos_app;
   -- No GRANT UPDATE — snapshots immutable per DEC-1566

   CREATE TABLE journal_entries (
     journal_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     entry_date DATE NOT NULL,
     description TEXT NOT NULL,
     debit_account TEXT NOT NULL,
     credit_account TEXT NOT NULL,
     amount NUMERIC(18,4) NOT NULL CHECK (amount > 0),
     currency TEXT NOT NULL,
     engagement_id UUID,
     invoice_id UUID,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   ALTER TABLE journal_entries ENABLE ROW LEVEL SECURITY;
   CREATE POLICY journal_rls ON journal_entries
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON journal_entries FROM cyberos_app;
   ```

7. **MUST** make `recognition_snapshots` immutable per DEC-1566 — corrections via NEW `prior_period_adjustment` snapshot, never UPDATE.

8. **MUST** emit 4 memory audit kinds per DEC-1565. PII scrub per FR-MEMORY-111: amount/period_revenue SHA-256 hashed; engagement_id (uuid) ok.

9. **MUST** thread trace_id from EOM cron → rollforward → schedule update → journal → snapshot.

10. **MUST** use `rust_decimal::Decimal` for ALL amounts — never f64.

11. **MUST NOT** mutate prior-period snapshot per DEC-1566.

12. **MUST NOT** skip journal entry on any recognition event per DEC-1563.

---

## §2 — Why this design

**Why 4 methods (DEC-1560)?** Covers consultancy spectrum — subscription, project (milestone), T&M consulting (pct-completion), one-off (point-in-time). ASC 606 enumerates these patterns.

**Why immutable snapshots (DEC-1566)?** Audit lineage requires period close to be unrevisable; restatements happen via prior-period adjustment journal entries.

**Why monthly EOM (DEC-1562)?** Aligns with month-end close cycle; weekly/quarterly are less common.

**Why per-engagement schedule (DEC-1564)?** Each engagement has independent contract terms; aggregation done at report time.

---

## §3 — API contract

```text
POST   /v1/inv/recognition/schedules             (CFO assigns method to engagement)
GET    /v1/inv/recognition/schedules/{eng_id}    (view schedule)
GET    /v1/inv/recognition/snapshots/{period}    (period rollforward)
POST   /v1/inv/recognition/rollforward           (manual trigger, CFO)
GET    /v1/inv/recognition/journal-entries       (audit query)
```

Sample schedule (time_based, 12-month $120k):
```json
[
  {"period_end": "2026-01-31", "planned_amount": 10000, "recognized_amount": 10000, "status": "complete"},
  {"period_end": "2026-02-28", "planned_amount": 10000, "recognized_amount": 10000, "status": "complete"},
  ...
  {"period_end": "2026-12-31", "planned_amount": 10000, "recognized_amount": 0, "status": "pending"}
]
```

Sample journal pair on rollforward:
```json
[
  {"date": "2026-05-31", "description": "May revenue recognition", "DR": "Deferred Revenue", "CR": "Revenue", "amount": 10000, "engagement_id": "..."},
  {"date": "2026-05-31", "description": "May AR offset (already booked at issue)", "DR": "AR", "CR": "Deferred Revenue", "amount": 0}  // no-op for pre-billed
]
```

---

## §4 — Acceptance criteria
1. **4 methods supported + cardinality test**. 2. **Schedule built at engagement creation**. 3. **EOM rollforward via FR-MCP-007 cron**. 4. **Journal entries balance (DR sum = CR sum)**. 5. **Snapshots immutable (no UPDATE/DELETE grant)**. 6. **Prior-period adjustment via new snapshot**. 7. **rust_decimal for amounts (no f64)**. 8. **4 memory audit kinds**. 9. **PII scrubbed (amount SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Multi-currency: schedule in engagement currency; report base via FR-INV-002**. 13. **time_based: equal monthly split**. 14. **milestone_based: recognize on milestone complete event**. 15. **pct_completion: recompute on FR-TIME-002 entry change**. 16. **point_in_time: full on invoice send**. 17. **Manual rollforward CFO-only**. 18. **Idempotent (UNIQUE on tenant+engagement+period)**. 19. **Failure mid-rollforward → snapshot=failed + sev-1**. 20. **Reconcile_status auto-updates (pending→partial→complete)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn time_based_equal_split() {
    let ctx = TestContext::engagement_with_terms(120000, 12, "time_based").await;
    let schedule = ctx.fetch_schedule(ctx.engagement_id).await;
    assert_eq!(schedule.len(), 12);
    for s in schedule { assert_eq!(s.planned_amount, dec!(10000)); }
}

#[tokio::test]
async fn snapshots_immutable() {
    let ctx = TestContext::with_completed_period().await;
    let snap = ctx.fetch_snapshot(ctx.period_end).await;
    let result = ctx.try_mutate_snapshot(snap.snapshot_id).await;
    assert!(result.is_err());  // RLS + REVOKE blocks
}

#[tokio::test]
async fn journal_entries_balance() {
    let ctx = TestContext::with_rollforward_run().await;
    let entries = ctx.fetch_journal_entries_for_period().await;
    let total_dr: Decimal = entries.iter().map(|e| e.amount).sum();
    let total_cr: Decimal = entries.iter().map(|e| e.amount).sum();
    assert_eq!(total_dr, total_cr);
}

// 5.4..5.10
```

---

## §6 — Skeleton

```rust
pub async fn rollforward(period_end: NaiveDate, tenant: &Tenant, db: &Db) -> Result<RollforwardResult> {
    let engagements = db.fetch_active_engagements(tenant.id).await?;
    let trace = current_span_trace_id();
    for eng in engagements {
        let period_revenue = match eng.recognition_method.as_str() {
            "time_based" => time_based_amount(&eng, period_end),
            "milestone_based" => milestone_revenue(&eng, period_end, db).await?,
            "pct_completion" => pct_completion_revenue(&eng, period_end, db).await?,
            "point_in_time" => point_in_time_revenue(&eng, period_end, db).await?,
            _ => unreachable!()
        };
        let journal_id = journal_entry::create_recognition_pair(&eng, period_revenue, period_end, db).await?;
        db.insert_snapshot(eng.id, period_end, period_revenue, journal_id, trace).await?;
        db.update_schedule_recognized(&eng, period_end, period_revenue).await?;
        audit::emit("inv.revenue_recognized", json!({...}), trace).await?;
    }
    audit::emit("inv.revenue_rollforward_completed", json!({"period_end": period_end}), trace).await?;
    Ok(RollforwardResult{snapshots_created: engagements.len()})
}
```

---

## §7 — Dependencies
**Upstream:** FR-INV-001.
**Cross-module:** FR-MCP-007 (cron), FR-TIME-002 (hours for pct_completion), FR-PROJ-002 (milestones), FR-MEMORY-111 (PII), FR-AUTH-101 (CFO role).

## §8 — Sample payloads (see §3)

## §9 — Open questions
None blocking — ASC 606 + IFRS 15 patterns well-established.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Engagement no method set | scheduler check | sev-2 + skip rollforward | CFO assigns method |
| Journal entries don't balance | post-condition check | rollback + sev-1 | data investigation |
| Snapshot mutation attempt | RLS + REVOKE | DB error | inherent |
| pct_completion negative hours | scheduler validate | sev-2 + skip period | FR-TIME-002 fix |
| Milestone never completes | indefinite deferral | annual sweep alerts | CFO review |
| Mid-month engagement add | next EOM rollforward picks up | inherent | inherent |
| Mid-rollforward crash | partial snapshots | resume from last successful eng | re-run |
| Multi-currency engagement w/o FX | rollforward fail | sev-1 | provide FX |
| Engagement modified mid-rollforward | snapshot SQL | uses snapshot row state | inherent |
| Decimal precision drift | rust_decimal | exact | inherent |
| Period not yet ended | guard | reject | wait |
| Duplicate rollforward run | UNIQUE constraint | ON CONFLICT DO NOTHING | inherent |

## §11 — Implementation notes
- §11.1 EOM tenant_timezone: midnight of first of next month; jitter ±30min to avoid thundering herd.
- §11.2 pct_completion formula: `recognized = (hours_completed / hours_estimated) * total_contract`; cap at 100%.
- §11.3 memory audit body: engagement_id (uuid), period_end (date), period_revenue SHA256 hashed.
- §11.4 Snapshots are reportable via FR-INV-009 + future financial-statements FR.
- §11.5 Prior-period adjustment pattern: new snapshot with `description: 'PPA: <reason>'`; original snapshot unchanged.

---

*End of FR-INV-011 spec.*
