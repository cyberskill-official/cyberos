---
id: TASK-PROJ-007
title: "Three billing modes — Time & Materials, Fixed-Fee, Retainer — with mode-aware rollups and per-mode invoice generation hooks"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PROJ
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PROJ-005, TASK-PROJ-006, TASK-INV-001, TASK-TIME-005, TASK-TEN-203]
depends_on: [TASK-PROJ-005, TASK-PROJ-006]
blocks: []

source_pages:
  - website/docs/modules/proj.html#billing-modes
source_decisions:
  - DEC-280 (engagement has exactly one billing mode at any time; mode-change is a versioned event)
  - DEC-281 (T&M = hours × rate; Fixed-Fee = lump-sum with progress milestones; Retainer = capped monthly with rollover)
  - DEC-282 (mode-aware rollup is the canonical input to TASK-INV-001 invoice generation)

language: rust 1.81
service: cyberos/services/proj-sync/
new_files:
  - services/proj-sync/migrations/0007_billing_modes.sql
  - services/proj-sync/src/billing_mode/mod.rs
  - services/proj-sync/src/billing_mode/rollup.rs
  - services/proj-sync/src/billing_mode/fixed_fee.rs
  - services/proj-sync/src/billing_mode/retainer.rs
  - services/proj/tests/link_types_test.rs
modified_files:
  # billing_mode_id FK
  - services/proj-sync/src/engagement.rs
allowed_tools:
  - file_read: services/proj-sync/**
  - file_write: services/proj-sync/{src,tests,migrations}/**
  - bash: cd services/proj-sync && cargo test billing
disallowed_tools:
  - two active billing modes on one engagement (per DEC-280)
  - rollup non-billable time entries into invoiceable amount (per TASK-PROJ-006 snapshot)

effort_hours: 6
subtasks:
  - "0.5h: 0007_billing_modes.sql migration (table + mode enum + per-mode config columns)"
  - "0.5h: BillingMode enum (TimeAndMaterials | FixedFee | Retainer)"
  - "1.0h: rollup.rs — common interface: rollup_invoiceable(engagement, period) -> InvoiceLines"
  - "1.0h: fixed_fee.rs — milestone-based rollup; milestone completion (TASK-PROJ-004 status=done) triggers line"
  - "1.5h: retainer.rs — monthly cap with rollover (3-month default); track consumed_minutes; emit overage line"
  - "0.5h: memory audit rows (mode_changed, milestone_invoiced, retainer_overage)"
  - "1.0h: billing_mode_test.rs — three mode rollups with golden fixtures"
risk_if_skipped: "Without explicit billing modes, every invoice generation reinvents the rollup logic — drift between Kanban + Timeline + invoice batch jobs. Without milestone semantics for Fixed-Fee, project completion doesn't trigger billing → cash flow gaps. Without retainer overage tracking, clients silently consume more than contracted → margin erosion. Without mode-change audit, finance team has no trail for 'when did this engagement convert from T&M to retainer'."
---

## §1 — Description (BCP-14 normative)

The billing-mode layer **MUST** support exactly three modes per engagement with mode-aware rollups feeding TASK-INV-001 invoice generation. The contract:

1. **MUST** define `billing_modes` table with columns: `id UUID PK`, `engagement_id UUID FK`, `mode` (`time_and_materials | fixed_fee | retainer`), `effective_from DATE`, `effective_to DATE` (nullable), `config JSONB` (mode-specific), `created_at`, `created_by_subject_id`, `tenant_id`. Versioned same as rate cards (TASK-PROJ-005); INSERT-only.
2. **MUST** support `TimeAndMaterials` mode with config `{minimum_billable_increment_minutes: 15, weekly_cap_hours?: i32}`. Rollup = sum of billable time entries × applicable rate card.
3. **MUST** support `FixedFee` mode with config `{total_amount_minor: i64, currency, milestones: [{id, name, target_pct_complete: f32, target_date, amount_minor: i64}]}`. Rollup = sum of completed-milestone amounts (where milestone is "done" per TASK-PROJ-004 issue.status). Sum of milestone amounts MUST equal total_amount_minor (validated at config insert).
4. **MUST** support `Retainer` mode with config `{monthly_cap_minutes: i32, currency, base_amount_minor: i64, overage_rate_minor_per_hour: i64, rollover_months: u8}`. Rollup = always invoice base_amount_minor per month; if `consumed_minutes_this_period > cap + rollover_credit`, emit overage line at `overage_rate_minor_per_hour × (excess_minutes / 60)`.
5. **MUST** expose `rollup_invoiceable(engagement_id, period_start, period_end) -> InvoiceRollup` returning ordered list of `InvoiceLine { line_kind, description, quantity, unit_rate_minor, amount_minor, currency, source_refs }` where source_refs cross-references to time entries / milestone IDs.
6. **MUST** record mode changes via supersession (new row + close prior `effective_to`). Mode-change DURING an open invoice period prorates: portion before change billed under old mode, portion after under new.
7. **MUST** emit memory audit rows:
- `proj.billing_mode_set` on initial mode set.
- `proj.billing_mode_changed` on supersession with `{old_mode, new_mode, effective_from, prorating_applied}`.
- `proj.milestone_invoiced` per Fixed-Fee milestone rollup.
- `proj.retainer_overage_emitted` per Retainer overage line.
- `proj.retainer_rollover_carry_forward` per period transition with unused credit.
8. **MUST** validate Fixed-Fee `milestones[*].amount_minor` sums to `total_amount_minor` exactly (no rounding).
9. **MUST** track Retainer `consumed_minutes_this_period` + `rollover_credit_minutes` in dedicated table `retainer_state(engagement_id, period_year_month, consumed_minutes, rollover_credit_minutes, base_amount_invoiced, overage_minutes_invoiced, computed_at, tenant_id)`.
10. **MUST** support `cyberos engagements set-mode <eng> <mode>` CLI for ops with idempotency.
11. **MUST** emit OTel metrics:
- `proj_billing_rollup_duration_seconds{mode}` (histogram).
- `proj_retainer_overage_total{engagement_id_bucket}` (counter — overage frequency dashboard).
- `proj_fixed_fee_milestones_invoiced_total` (counter).
12. **MUST** RLS-enforce (TASK-AUTH-003).
13. **MUST** apply mid-period mode-change proration when the rollup period straddles a mode-change date. Specifically: split the period at the mode-change boundary; compute rollup independently for each sub-period under its mode; combine into a single `InvoiceRollup` with `prorated: true` marker and per-sub-period source_refs.
14. **MUST** support `cancel_milestone(milestone_id, reason)` — Fixed-Fee admin can cancel an in-progress milestone, redistributing its amount across remaining milestones OR refunding to client. Emits `proj.milestone_cancelled` audit + reduces total_amount_minor.
15. **MUST** support `add_milestone(after_milestone_id, milestone)` — Fixed-Fee admin can add a milestone post-contract (e.g. scope-creep agreement), with `total_amount_minor` adjustment. Emits `proj.milestone_added`. Invariant: new milestone count + amounts must satisfy §1 #8 sum.
16. **MUST** track `engagement_in_overage_streak` for Retainer mode: count of consecutive periods with overage; SEV-3 alert at 3 (suggests retainer cap is too low for actual usage).
17. **MUST** support per-engagement `retainer_holiday` window: operator can mark Dec 24 – Jan 2 as a holiday window where the retainer base is prorated (charged base × workdays_in_period / total_workdays_in_month). Stored in `retainer_holidays` table.
18. **MUST** complete `rollup_invoiceable` within 500ms p95 for engagements with ≤ 1000 time entries in the period. Larger engagements emit SEV-3 warning + the call still completes.
19. **MUST** support `?preview=true` flag on rollup endpoint: returns the rollup but does NOT persist `retainer_state` (used by operator UI for what-if scenarios).
20. **MUST** validate currency consistency across all time entries in a T&M period: if entries reference multiple currencies (e.g. mix of USD + VND rate cards), the rollup MUST emit separate InvoiceLines per currency with a `proj.billing_mixed_currency_period` SEV-2 warning.
21. **MUST** include an `engagement_metadata` field in `InvoiceRollup`: `{client_name, engagement_name, billing_address, vat_number}` from engagement (TASK-PROJ-001) for downstream invoice rendering (TASK-INV-001).

---

## §2 — Why this design (rationale for humans)

**Why exactly three modes (DEC-281)?** These are the industry-standard contract types. Hybrid models (e.g. "fixed-fee with overage T&M") compose from these primitives via mode-change + proration. Adding a 4th mode invites combinatorial complexity; we ship the empirical 95%.

**Why versioned mode changes (§1 #1, DEC-280)?** Same logic as rate cards (TASK-PROJ-005): UPDATE corrupts historical billing. A client invoiced at "T&M" in Q1 then "Fixed-Fee" in Q2 must show both modes in their respective periods.

**Why milestone sum invariant (§1 #8)?** Fixed-Fee clients pay a TOTAL; milestones are intermediate progress payments. If milestones sum ≠ total, the engagement either over-bills (sum > total) or leaves money on the table (sum < total). The invariant catches config errors early.

**Why retainer rollover (§1 #4)?** Clients hate "use it or lose it" — slow months should subsidise busy months. Configurable rollover (3 months default) balances client value with finance team's accrual closing. After rollover_months periods, unused credit drops (finite tracking).

**Why proration on mode change (§1 #6)?** Mid-period mode changes are common (small T&M engagement converts to retainer once volume stabilises). Without proration, the client either over-pays (charged full retainer + T&M hours for switching month) or under-pays (T&M month suddenly billed as flat retainer).

**Why InvoiceLine carries source_refs (§1 #5)?** Auditors and clients ask "what hours produced this invoice line?" Each line references the underlying time-entries (T&M), milestone (Fixed-Fee), or period (Retainer). Reverse-traceability built in.

**Why mid-period proration (§1 #13)?** Engagements convert modes mid-month often (small T&M proves out, converts to Fixed-Fee). Charging full retainer for a 2-week period feels unfair; charging T&M rates for a Fixed-Fee period feels wrong. Proration splits the difference correctly.

**Why milestone cancellation + add (§1 #14, #15)?** Real engagements scope-creep or descope. Cancellation refunds; addition extends. Both adjust total_amount_minor; both emit memory audits for finance reconciliation.

**Why overage-streak tracking (§1 #16)?** A retainer that consistently overages is mispriced; the operator should renegotiate. 3-period streak is the empirical signal that "this is structural, not noise."

**Why retainer holidays (§1 #17)?** Tet, Christmas, summer breaks: retainer base prorated for partial-availability periods. Without holidays, clients pay full retainer for half-month coverage. Operator config flexibility wins.

**Why 500ms p95 rollup budget (§1 #18)?** Rollup is in the invoice-generation hot path; operators triggering monthly invoices for 100 engagements need it fast. 500ms × 100 = 50s total — acceptable. Larger engagements get a warning but still complete.

**Why preview-only mode (§1 #19)?** What-if analysis: "if we switched to retainer next month, what would the invoice look like?" Without preview, operators have to do destructive state writes to find out.

**Why mixed-currency split (§1 #20)?** A single InvoiceLine can't carry multiple currencies; mixed-currency periods must split into per-currency lines. SEV-2 warning surfaces the rare case (operators may have set up multi-currency rate cards inadvertently).

**Why engagement_metadata in rollup (§1 #21)?** Downstream invoice rendering needs client name + billing address + VAT number. Pre-fetched in the rollup so TASK-INV-001 doesn't re-query engagement.

---

## §3 — API contract

### Migration

```sql
-- services/proj-sync/migrations/0007_billing_modes.sql

CREATE TABLE billing_modes (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    engagement_id   UUID NOT NULL REFERENCES engagements(id),
    mode            TEXT NOT NULL CHECK (mode IN ('time_and_materials','fixed_fee','retainer')),
    config          JSONB NOT NULL,
    effective_from  DATE NOT NULL,
    effective_to    DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by_subject_id UUID NOT NULL,
    tenant_id       UUID NOT NULL,
    UNIQUE (engagement_id, effective_from)
);
CREATE UNIQUE INDEX uniq_active_billing_mode
    ON billing_modes (engagement_id) WHERE effective_to IS NULL;
CREATE INDEX idx_billing_modes_lookup
    ON billing_modes (engagement_id, effective_from, effective_to);

CREATE TABLE retainer_state (
    engagement_id            UUID NOT NULL REFERENCES engagements(id),
    period_year_month        TEXT NOT NULL,  -- "2026-05"
    consumed_minutes         INT NOT NULL DEFAULT 0,
    rollover_credit_minutes  INT NOT NULL DEFAULT 0,
    base_amount_invoiced     BIGINT NOT NULL DEFAULT 0,
    overage_minutes_invoiced INT NOT NULL DEFAULT 0,
    computed_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id                UUID NOT NULL,
    PRIMARY KEY (engagement_id, period_year_month)
);

ALTER TABLE billing_modes  ENABLE ROW LEVEL SECURITY;
ALTER TABLE retainer_state ENABLE ROW LEVEL SECURITY;
CREATE POLICY billing_modes_tenant_isolation ON billing_modes
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
CREATE POLICY retainer_state_tenant_isolation ON retainer_state
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

### Rust API

```rust
// services/proj-sync/src/billing_mode/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum BillingMode { TimeAndMaterials, FixedFee, Retainer }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ModeConfig {
    TimeAndMaterials { minimum_billable_increment_minutes: i32, weekly_cap_hours: Option<i32> },
    FixedFee { total_amount_minor: i64, currency: crate::rate_card::Currency, milestones: Vec<Milestone> },
    Retainer { monthly_cap_minutes: i32, currency: crate::rate_card::Currency,
               base_amount_minor: i64, overage_rate_minor_per_hour: i64, rollover_months: u8 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub id:                 String,
    pub name:               String,
    pub target_pct_complete: f32,
    pub target_date:        chrono::NaiveDate,
    pub amount_minor:       i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct InvoiceRollup {
    pub engagement_id: uuid::Uuid,
    pub period_start:  chrono::NaiveDate,
    pub period_end:    chrono::NaiveDate,
    pub mode:          BillingMode,
    pub lines:         Vec<InvoiceLine>,
    pub total_minor:   i64,
    pub currency:      crate::rate_card::Currency,
}

#[derive(Clone, Debug, Serialize)]
pub struct InvoiceLine {
    pub line_kind:        LineKind,
    pub description:      String,
    pub quantity:         f64,
    pub unit_rate_minor:  i64,
    pub amount_minor:     i64,
    pub source_refs:      Vec<SourceRef>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineKind { Hours, Milestone, RetainerBase, RetainerOverage }

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceRef {
    TimeEntry { id: uuid::Uuid },
    Milestone { id: String },
    Period    { year_month: String },
}
```

### Rollup orchestrator

```rust
// services/proj-sync/src/billing_mode/rollup.rs
pub async fn rollup_invoiceable(
    pool: &sqlx::PgPool,
    engagement_id: uuid::Uuid,
    period_start: chrono::NaiveDate,
    period_end: chrono::NaiveDate,
) -> anyhow::Result<InvoiceRollup> {
    let mode_row = active_mode_at(pool, engagement_id, period_start).await?;
    let config: ModeConfig = serde_json::from_value(mode_row.config)?;

    let lines = match config {
        ModeConfig::TimeAndMaterials { .. } => {
            t_and_m::compute_lines(pool, engagement_id, period_start, period_end).await?
        }
        ModeConfig::FixedFee { milestones, currency, .. } => {
            fixed_fee::compute_lines(pool, engagement_id, &milestones, currency, period_start, period_end).await?
        }
        ModeConfig::Retainer { monthly_cap_minutes, currency, base_amount_minor, overage_rate_minor_per_hour, rollover_months } => {
            retainer::compute_lines(pool, engagement_id, monthly_cap_minutes, currency,
                                    base_amount_minor, overage_rate_minor_per_hour, rollover_months,
                                    period_start, period_end).await?
        }
    };
    let total: i64 = lines.iter().map(|l| l.amount_minor).sum();
    let currency = lines.first().map(|l| l.source_refs.first()).flatten().map(|_| determine_currency(&config)).unwrap_or(crate::rate_card::Currency::VND);
    Ok(InvoiceRollup { engagement_id, period_start, period_end,
                       mode: mode_row.mode, lines, total_minor: total, currency })
}
```

### Retainer overage compute

```rust
// services/proj-sync/src/billing_mode/retainer.rs
pub async fn compute_lines(
    pool: &sqlx::PgPool,
    eng: uuid::Uuid,
    cap_min: i32,
    currency: crate::rate_card::Currency,
    base_minor: i64,
    over_rate_per_hour_minor: i64,
    rollover_months: u8,
    period_start: chrono::NaiveDate,
    _period_end: chrono::NaiveDate,
) -> anyhow::Result<Vec<InvoiceLine>> {
    let ym = period_start.format("%Y-%m").to_string();
    let consumed: i32 = sqlx::query_scalar!(
        "SELECT COALESCE(SUM(duration_minutes), 0)::int4
         FROM time_entries WHERE engagement_id = $1
           AND start_at >= $2 AND start_at < $3
           AND billable_snapshot = true",
        eng, period_start, period_start + chrono::Duration::days(31)
    ).fetch_one(pool).await?;

    // Carry-over from prior periods, up to rollover_months back
    let credit = compute_rollover_credit(pool, eng, period_start, rollover_months, cap_min).await?;
    let effective_cap = cap_min + credit;
    let overage_min = (consumed - effective_cap).max(0);
    let overage_amount = (overage_min as i64 * over_rate_per_hour_minor) / 60;

    // Persist retainer_state for next period's rollover compute
    sqlx::query!(
        "INSERT INTO retainer_state (engagement_id, period_year_month, consumed_minutes, rollover_credit_minutes,
                                      base_amount_invoiced, overage_minutes_invoiced, tenant_id)
         VALUES ($1,$2,$3,$4,$5,$6, current_setting('app.tenant_id')::uuid)
         ON CONFLICT (engagement_id, period_year_month) DO UPDATE
           SET consumed_minutes = EXCLUDED.consumed_minutes,
               rollover_credit_minutes = EXCLUDED.rollover_credit_minutes,
               base_amount_invoiced = EXCLUDED.base_amount_invoiced,
               overage_minutes_invoiced = EXCLUDED.overage_minutes_invoiced,
               computed_at = NOW()",
        eng, ym, consumed, credit, base_minor, overage_min
    ).execute(pool).await?;

    let mut lines = vec![InvoiceLine {
        line_kind: LineKind::RetainerBase,
        description: format!("Retainer fee — {ym}"),
        quantity: 1.0,
        unit_rate_minor: base_minor,
        amount_minor: base_minor,
        source_refs: vec![SourceRef::Period { year_month: ym.clone() }],
    }];

    if overage_min > 0 {
        lines.push(InvoiceLine {
            line_kind: LineKind::RetainerOverage,
            description: format!("Retainer overage — {:.2} hours @ {}/h",
                                 overage_min as f64 / 60.0,
                                 format_minor(over_rate_per_hour_minor, currency)),
            quantity: overage_min as f64 / 60.0,
            unit_rate_minor: over_rate_per_hour_minor,
            amount_minor: overage_amount,
            source_refs: vec![SourceRef::Period { year_month: ym.clone() }],
        });
        emit_memory_row("proj.retainer_overage_emitted", serde_json::json!({
            "engagement_id": eng, "period": ym, "overage_minutes": overage_min,
            "overage_amount_minor": overage_amount,
        })).await;
    }
    Ok(lines)
}
```

---

## §4 — Acceptance criteria

1. **T&M rollup sums billable hours** — 40 billable hours @ 500_000 VND/h → 1 line × 20_000_000 VND.
2. **T&M ignores non-billable** — 40h billable + 10h non-billable → only 40h billed.
3. **Fixed-Fee milestones invariant** — config with milestones summing ≠ total → 422 at insert.
4. **Fixed-Fee rollup invoices completed milestones** — milestone X done in period → its amount_minor in InvoiceLine; not-done milestone absent.
5. **Retainer base always emitted** — period with 0 consumed → 1 line × base_amount_minor.
6. **Retainer overage** — consumed = 60h, cap = 40h → overage = 20h × overage_rate.
7. **Retainer rollover credit** — prior period unused 10h; cap 40h → effective cap = 50h; consumed 45h → no overage.
8. **Retainer rollover expires after N months** — unused credit > rollover_months old → drops.
9. **Mode change at period midpoint prorates** — T&M Jan 1-15, FixedFee Jan 16-31 → 2 InvoiceRollups OR 1 with proration markers.
10. **Two active modes forbidden** — second mode insert with overlapping effective dates → 409.
11. **memory audit on mode change** — POST new mode → `proj.billing_mode_changed` row.
12. **memory audit on milestone invoiced** — milestone done → `proj.milestone_invoiced` row.
13. **memory audit on retainer overage** — overage > 0 → `proj.retainer_overage_emitted`.
14. **InvoiceLine.source_refs traceable** — T&M line carries time_entry IDs; FixedFee carries milestone ID; Retainer carries period.
15. **CLI set-mode idempotent** — same Idempotency-Key + same body → returns prior.
16. **OTel metric `proj_retainer_overage_total`** — overage emitted → counter increments.
17. **RLS tenant isolation** — tenant A's retainer_state invisible to tenant B.
18. **Mid-period mode-change prorates** — period Jan 1-31, mode changed Jan 15 from T&M to FixedFee → InvoiceRollup `prorated: true` with sub-period lines (AC for §1 #13).
19. **Milestone cancellation redistributes** — cancel milestone (50M) → remaining milestones' amounts adjusted OR total reduced; `proj.milestone_cancelled` audit (AC for §1 #14).
20. **Milestone addition extends total** — add 25M milestone → total_amount_minor += 25M; invariant satisfied; `proj.milestone_added` audit (AC for §1 #15).
21. **Overage streak alarm at 3** — 3 consecutive overage periods → SEV-3 audit `proj.retainer_overage_streak` (AC for §1 #16).
22. **Retainer holiday prorates base** — Dec holiday window covers 7 of 22 workdays → base × 15/22 in line (AC for §1 #17).
23. **Rollup latency p95 < 500ms** — 1000 time entries → p95 < 500ms; > 1000 → SEV-3 warning (AC for §1 #18).
24. **Preview does not persist retainer_state** — POST /rollup?preview=true → retainer_state unchanged after call (AC for §1 #19).
25. **Mixed currency in T&M period** — entries in USD + VND → 2 InvoiceLines (one per currency) + SEV-2 `proj.billing_mixed_currency_period` (AC for §1 #20).
26. **engagement_metadata populated** — rollup response includes client_name + billing_address + vat_number (AC for §1 #21).
27. **CLI set-mode validates milestone sum** — bad config → CLI exits 1 with clear error (AC for §1 #8 + #15).
28. **Audit `proj.billing_mode_set` on first mode** — initial mode → memory row with mode + config (AC for §1 #7).

---

## §5 — Verification

```rust
#[tokio::test]
async fn t_and_m_rollup_excludes_non_billable() {
    let env = TestEnv::new().await;
    let eng = env.bootstrap_t_and_m().await;
    env.add_time_entry(eng, hours(40), billable=true).await;
    env.add_time_entry(eng, hours(10), billable=false).await;
    let r = rollup_invoiceable(&env.pool, eng, "2026-05-01".parse().unwrap(), "2026-06-01".parse().unwrap()).await.unwrap();
    let total_hours: f64 = r.lines.iter().filter(|l| l.line_kind == LineKind::Hours).map(|l| l.quantity).sum();
    assert!((total_hours - 40.0).abs() < 0.01);
}

#[tokio::test]
async fn fixed_fee_milestone_sum_invariant() {
    let env = TestEnv::new().await;
    let eng = env.create_engagement().await;
    let bad_config = ModeConfig::FixedFee {
        total_amount_minor: 100_000_000, currency: Currency::VND,
        milestones: vec![
            milestone("m1", 50_000_000), milestone("m2", 40_000_000),  // sums to 90M, not 100M
        ],
    };
    let err = set_mode(&env.pool, eng, bad_config).await.unwrap_err();
    assert!(matches!(err, BillingError::MilestoneSumMismatch { .. }));
}

#[tokio::test]
async fn retainer_overage_emitted_when_consumed_exceeds_cap() {
    let env = TestEnv::new().await;
    let eng = env.bootstrap_retainer(cap_minutes=2400, base=30_000_000, over_rate_per_h=750_000).await;
    env.add_time_entry(eng, hours(50)).await;  // 50h = 3000 min; cap = 2400; overage = 600 min = 10h
    let r = rollup_invoiceable(&env.pool, eng, "2026-05-01".parse().unwrap(), "2026-06-01".parse().unwrap()).await.unwrap();
    let overage = r.lines.iter().find(|l| l.line_kind == LineKind::RetainerOverage).unwrap();
    assert!((overage.quantity - 10.0).abs() < 0.01);
    assert_eq!(overage.amount_minor, 7_500_000);  // 10h × 750k
}

#[tokio::test]
async fn retainer_rollover_credit_applied() {
    let env = TestEnv::new().await;
    let eng = env.bootstrap_retainer(cap_minutes=2400, base=30_000_000, over_rate_per_h=750_000).await;
    env.set_retainer_state(eng, "2026-04", consumed=1800, rollover_credit=0).await;  // 10h unused
    env.add_time_entry_in_period(eng, hours(45), "2026-05").await;
    let r = rollup_invoiceable(&env.pool, eng, "2026-05-01".parse().unwrap(), "2026-06-01".parse().unwrap()).await.unwrap();
    let overage_line = r.lines.iter().find(|l| l.line_kind == LineKind::RetainerOverage);
    // Effective cap = 40h + 10h rollover = 50h; consumed 45h → no overage
    assert!(overage_line.is_none());
}
```

---

## §6 — Implementation skeleton

(API + DB schema above.)

---

## §7 — Dependencies

- **TASK-PROJ-005** — rate cards (T&M uses lookup_at).
- **TASK-PROJ-006** — billable cascade (snapshot consumed in T&M rollup).
- **TASK-INV-001 (downstream)** — invoice generation consumes `InvoiceRollup`.
- **TASK-TIME-005** — time-entry snapshot field.

---

## §8 — Example payloads

```json
{
  "kind": "proj.retainer_overage_emitted",
  "payload": {
    "engagement_id":         "eng-...",
    "period":                "2026-05",
    "overage_minutes":       600,
    "overage_amount_minor":  7500000
  }
}
```

```json
{
  "kind": "proj.milestone_invoiced",
  "payload": {
    "engagement_id":   "eng-...",
    "milestone_id":    "m2",
    "milestone_name":  "Phase 2 — Backend complete",
    "amount_minor":    25000000,
    "completed_at":    "2026-05-15"
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Hybrid mode (Fixed-Fee + T&M overage) — slice 4+; compose from mode-change.
- Multi-currency within one engagement — slice 4+; current MVP one currency per active mode.
- Variable retainer caps (Q1 high, Q2 low) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Milestone sum mismatch | invariant check at insert | 422 | Caller adjusts amounts |
| Two active modes | partial-unique constraint | 409 on insert | Caller supersedes prior first |
| No mode at period | active_mode_at returns None | 404 | Operator sets mode |
| Retainer rollover compute fails | sqlx Err | 500 | Operator investigates |
| Time entries in period missing currency | rollup defaults to engagement's currency | Mismatch warning | Caller validates currency consistency |
| Overage rate = 0 with cap exceeded | overage_amount = 0; line still emitted | client sees "0 VND overage" | By design |
| Retainer base = 0 | line emitted with amount 0 | client sees 0-amount line | Edge case for pro-bono retainer |
| Mode change without proration | period straddles change | rollup uses mode-at-period-start | Slice 4+ proration |
| memory audit fails | rollup still returns; audit lost | sev-2 alarm | Operator restores |
| Concurrent rollup calls | retainer_state ON CONFLICT DO UPDATE | last writer wins for state; rollup result deterministic | None |
| Invoice already generated for period | duplicate invoice attempt | caller's TASK-INV-001 dedup catches | None at this task |
| RLS bypass | RLS policy | 0 rows | None |
| Config JSON malformed | serde Err | 422 | Caller fixes |
| Milestone has target_pct_complete > 100 | should be rejected at insert | 422 | Caller fixes |
| Period straddles mode change without proration support | proration applied per §1 #13 | sub-period lines | None |
| Milestone cancelled during invoiced period | refund line emitted | downstream credit-note | Manual reconciliation |
| Milestone added with `amount > remaining_unbilled` | invariant check | 422 | Caller |
| Retainer overage streak = 0 then 1 then 0 | streak resets on no-overage period | None | None |
| Holiday window overlaps period boundary | prorate based on workdays in period × in-window | None | None |
| Rollup latency > 500ms but < 5s | SEV-3 warning; call completes | None | Operator |
| Rollup latency > 5s | timeout; 504 | None | Operator scales |
| Preview mode with concurrent real rollup | preview reads state but doesn't write | safe | None |
| Mixed-currency T&M without warning enabled | dedup warnings at OBS | None | None |
| engagement_metadata fetch fails | rollup degraded (metadata null) | downstream invoice has placeholder | Operator |
| Holiday window in past | applied at lookup time | as-is | None |
| Tax/VAT lookup deferred to TASK-INV-001 | rollup excludes taxes | None | By design |
| Two mode changes in one period | proration handles N segments | sub-period lines per change | None |
| Mode change effective in future | active_mode_at returns prior; new mode visible after effective_from | None | None |
| Retainer base prorated for partial-month engagement (new client mid-month) | proration applied | None | None |
| Retainer rollover with cap_change between periods | use cap-at-period for compute | None | None |
| Cancel milestone after invoice already issued | requires manual credit-note | SEV-2 | Operator |
| Add milestone retroactively (effective in past) | timestamp validates; rejected | 422 | Caller |
| Currency change in mode supersession | new mode's currency; warning emitted | None | None |
| Engagement archived during rollup | active_mode_at returns the archived-period mode | uses historical mode | None |
| Concurrent rollup + mode change | mode change waits for in-flight rollups | brief lock | None |
| Time entry without billable_snapshot (pre-TASK-PROJ-006) | excluded from rollup | SEV-3 warning if found | Operator backfills |
| Period start == period end | empty rollup | None | None |
| Period end before start | 422 | None | Caller |

---

## §11 — Implementation notes

- The `config JSONB` column carries mode-specific structure; serde tagged enum makes (de)serialisation type-safe.
- Retainer state is materialised per period (one row per engagement per month) — sled-style per-doc but in Postgres for transactional reads with billing rollups.
- Rollover credit compute walks back N months (configured per engagement); cap memoisation in retainer_state avoids recomputation.
- T&M rollup groups time entries by `(role, member_id)` to produce one InvoiceLine per cell; aggregation per TASK-INV-001 spec.
- Fixed-Fee milestones tied to issue IDs (each milestone has 1+ issues that must be "done" per TASK-PROJ-004 lifecycle); status change triggers rollup re-computation on next invoice.
- The `proj.retainer_rollover_carry_forward` audit row is emitted by the next period's rollup, not the current — bookkeeping aid for finance close.
- `format_minor(amount, currency)` is a helper from `cyberos-vn-common` that renders correctly per currency decimals (VND no decimals; USD 2).
- Mid-period proration splits the period at every mode-change boundary; for N changes in one period, rollup is computed for N+1 sub-periods. The `prorated: true` flag tells the invoice renderer to show sub-period detail.
- Milestone cancellation has two refund modes: (a) refund-to-client (issues credit note via TASK-INV-001); (b) redistribute remaining milestone amounts (e.g. cancel m3 worth 20M, add 20M to m4-m5 pro-rata). Default is (a); operator chooses at cancellation time.
- Milestone addition can occur post-contract (scope-creep agreement); the addition adjusts `total_amount_minor` and emits an audit row with `delta_amount` for finance reconciliation.
- Overage-streak tracking is a per-engagement counter persisted in `retainer_state`; a no-overage period resets it.
- Retainer holidays are inclusive intervals; the prorate formula uses workdays-in-period (Mon-Fri) excluding holidays.
- The 500ms p95 budget covers: 1 mode lookup + N time-entry queries + 1 milestone scan + retainer_state read/write. Typical engagement (100 entries) is ~50ms.
- Preview mode short-circuits the `INSERT ... ON CONFLICT` on `retainer_state`; everything else runs identically. Guarantees preview matches commit (no path divergence).
- Mixed-currency T&M periods are rare but legal; we emit separate lines + SEV-2 warning to surface the case (operators usually fix it by normalising rate-card currency).
- engagement_metadata is fetched in parallel with the rollup compute via `tokio::join!` to avoid serial latency.
- We considered keeping `total_amount_minor` mutable on Fixed-Fee config but rejected: every change generates a new mode row (supersession) to preserve audit history. UI sugar can hide the supersession from operators.
- Rollover credit is per-period (one row per month); we don't materialise the full credit-history graph — just the immediate prior period's surplus.
- The 3-month default rollover was chosen because: (a) most contracts allow 90-day rollover; (b) longer rollover bloats retainer_state queries; (c) finance teams want quarterly close.
- We chose Postgres JSONB over a separate per-mode table because the three modes have disjoint configurations; JOIN gymnastics weren't worth the schema purity.
- T&M weekly_cap_hours (optional config) lets operators cap a contractor's billable hours per week (anti-fraud); cap exceeded emits SEV-3 + still rolls up but with a warning line.
- We use `chrono::NaiveDate` for period boundaries (date-only); time-zone is the engagement's contracted billing timezone (tracked separately).
- Retainer base prorating for partial-month new clients: if engagement starts mid-month, base is `base × days_remaining / days_in_month`.
- Mode change effective in the future is allowed (e.g. scheduled Q1 → Q2 transition); rollup uses `active_mode_at` for the period start.
- Cancellation of an already-invoiced milestone requires manual credit-note via TASK-INV-001 — automated handling deferred to slice 4+.

---

*End of TASK-PROJ-007.*
