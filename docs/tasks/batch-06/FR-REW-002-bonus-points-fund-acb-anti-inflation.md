---
title: "REW — Bonus Points fund with anti-inflation interest at ACB savings rate; over-allocation rules; quarterly P3 payout cycle"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q1"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Implement the **Bonus Points (BP) fund** mechanics specified in CyberSkill's Total Rewards Appendix and named in PRD §2.3 Bet 5. The fund is the cash-collected pool from which P3 Performance pay is disbursed quarterly. Members earn Bonus Points through the LEARN-module Variable Performance roll-up (P2; cross-FR-LEARN-001) and Hội đồng Chuyên môn evaluation outcomes. **Anti-inflation interest** is applied to unredeemed BP at the **ACB Savings Rate** (the official savings interest rate of Asia Commercial Bank, sampled quarterly into the immutable parameter version from FR-REW-001 — a rate that anchors against Vietnamese inflation without speculation). **Over-allocation rules**: when total earned points exceed the cash fund's per-quarter ceiling, points roll forward to the next quarter rather than being diluted; the BP value-per-point is recomputed at each quarterly close. **Quarterly P3 payout cycle**: at quarter-close, the fund's cash + BP balances are reconciled; per-Member P3 cash is computed (Member's points × per-point value); the result feeds the FR-REW-003 payroll close. All compute is deterministic; AI is forbidden from the compute path. Lives in `hr_secure`; encrypted; trigger-protected like FR-REW-001.

## Problem

The PRD §2.3 Bet 5 names this directly as a moat: "Most platforms cannot model 3P income with a cash-collected pool, BP overflow, anti-inflation interest, four-year phantom-stock vesting with put options from Year 3, sabbaticals, anti-retroactive parameter versioning, and Good/Bad Leaver branches. CyberOS does — because it is required for the founder's own company to function."

Three failure modes the platform must prevent:

- **Inflation erosion of unredeemed BP.** A Member earning 100 BP today and redeeming next quarter, with Vietnamese inflation at ~4% / year, loses ~1% real value if the points are nominal-fixed. The ACB savings rate as the anchor is the contract's structural protection.
- **Over-allocation dilution.** If the fund collects 100 cash-units and members earn points worth a nominal 120, naïve dilution gives every Member 0.83× their nominal amount — silently reducing earned compensation. The Total Rewards Appendix specifies *roll-forward* instead: a Member's 100 BP this quarter remain 100 BP, redeemable next quarter (with anti-inflation interest applied) at the next quarter's per-point value.
- **AI in the compute path.** PRD §6.4 + §2.5: compensation compute is non-AI. The BP fund computations are deterministic + auditable; the only AI surface is the FR-REW-005 read-only narrator that *explains* the computation to a Member.

## Proposed Solution

The shape of the answer is `hr_secure.bp_*` schema + the deterministic compute pipeline + the quarterly close mechanics + the parameter-version anchoring.

**Schema.**

```sql
-- BP fund per quarter (the cash pool collected from company P&L for P3 distribution).
CREATE TABLE hr_secure.bp_fund_quarter (
  tenant_id UUID NOT NULL,
  quarter TEXT NOT NULL,                                       -- "2026-Q3"
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  fund_cash_collected_minor_encrypted BYTEA NOT NULL,           -- amount the company allocated; signed by Founder + Engineering Lead
  fund_cash_signed_by_founder_at TIMESTAMPTZ NOT NULL,
  fund_cash_signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  acb_savings_rate_snapshot_pct REAL NOT NULL,                  -- the published rate at the start of the quarter
  acb_rate_taken_at TIMESTAMPTZ NOT NULL,
  acb_rate_source_url TEXT NOT NULL,                             -- ACB's published-rates page; archived
  total_points_earned_this_quarter BIGINT NOT NULL DEFAULT 0,    -- aggregate; updated at close
  total_points_rolled_forward_in BIGINT NOT NULL DEFAULT 0,      -- carried in from prior quarter
  total_points_rolled_forward_out BIGINT,                         -- carried out at close (computed)
  per_point_value_minor_at_close_encrypted BYTEA,                 -- the cash-per-point at this quarter's close
  status TEXT NOT NULL DEFAULT 'open',                            -- "open" | "closed" | "paid_out"
  closed_at TIMESTAMPTZ,
  paid_out_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, quarter)
);

-- Per-Member BP balance per quarter (the canonical earnings ledger).
CREATE TABLE hr_secure.bp_balance (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  quarter TEXT NOT NULL REFERENCES hr_secure.bp_fund_quarter(quarter),
  points_earned_this_quarter BIGINT NOT NULL DEFAULT 0,
  points_rolled_forward_in BIGINT NOT NULL DEFAULT 0,
  points_redeemed_this_quarter BIGINT NOT NULL DEFAULT 0,        -- typically all points are redeemed at quarter-close
  points_rolled_forward_out BIGINT NOT NULL DEFAULT 0,
  cash_payout_minor_encrypted BYTEA,                              -- computed at close
  earnings_breakdown JSONB,                                       -- { source: "vp_evaluation", source_id: ..., points: ..., reason_md: "..." }[]
                                                                  -- shows where each point came from for transparency
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, employee_id, quarter)
);

CREATE INDEX bp_balance_quarter_idx ON hr_secure.bp_balance (tenant_id, quarter);
CREATE INDEX bp_balance_employee_idx ON hr_secure.bp_balance (tenant_id, employee_id);

-- Earnings: the immutable record of why a Member earned points (the source-of-truth).
CREATE TABLE hr_secure.bp_earning_event (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  quarter TEXT NOT NULL REFERENCES hr_secure.bp_fund_quarter(quarter),
  source TEXT NOT NULL,                                            -- "vp_evaluation" (LEARN P2)
                                                                  -- | "hoi_dong_chuyen_mon_promotion" (LEARN P2)
                                                                  -- | "ad_hoc_recognition" (Founder discretion)
                                                                  -- | "rolled_forward_from"  (system-generated)
  source_ref UUID,                                                  -- the LEARN evaluation id, etc.
  points BIGINT NOT NULL,                                          -- can be negative for clawback (rare; audited)
  reason_md TEXT NOT NULL,
  signed_by_founder_at TIMESTAMPTZ,
  signed_by_engineering_lead_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Forbid UPDATE on earning events after sign (immutable history).
CREATE OR REPLACE FUNCTION hr_secure.forbid_bp_earning_update()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.signed_by_founder_at IS NOT NULL THEN
    RAISE EXCEPTION 'bp_earning_event % is signed and immutable; create a clawback event instead', OLD.id
      USING ERRCODE = 'check_violation';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_bp_earning_immutable
  BEFORE UPDATE ON hr_secure.bp_earning_event
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_bp_earning_update();
```

**Quarterly close pipeline (deterministic; no AI).**

A scheduled job runs on the 1st day of the post-quarter month at 00:00 ICT (e.g. 1 October for Q3 close):

1. **Lock the quarter.** Set `bp_fund_quarter.status = 'closed'`. From this point, no new earning events for this quarter can be added (the trigger's `effective_from < quarter_start` check rejects).
2. **Aggregate per-Member points.**
   ```
   for each employee:
     points_total = bp_balance.points_rolled_forward_in
                  + sum(bp_earning_event.points where quarter = Q and signed)
     bp_balance.points_earned_this_quarter = sum(bp_earning_event.points)
   ```
3. **Apply anti-inflation interest to rolled-forward-in points.**
   ```
   for each employee:
     # the rolled-forward points carried from prior quarter accrue interest at the
     # ACB savings rate snapshot for THIS quarter
     interest_pct = bp_fund_quarter.acb_savings_rate_snapshot_pct
     adjusted_rolled_forward_in = round(points_rolled_forward_in * (1 + interest_pct/4 / 100))
                                  # quarterly compounding; interest_pct is annual
     # The rounding rule: round to nearest integer; bias toward Member (round up on .5).
   ```
4. **Compute per-point value at close.**
   ```
   total_redeemable_points = sum_over_employees(adjusted_rolled_forward_in + points_earned_this_quarter)
   per_point_value_minor = bp_fund_quarter.fund_cash_collected_minor / total_redeemable_points
                           # rounded down to the nearest minor unit; rounding residual
                           # accumulates into the next quarter's fund
   ```

   **No dilution.** If `total_redeemable_points * default_per_point_value > fund_cash`, the rule is **roll-forward**, not dilute. Specifically:

   ```
   default_per_point_value = parameter_version.parameters.p3_target_per_point_value_minor
   if total_redeemable_points * default_per_point_value <= fund_cash:
       # The fund covers everything; full payout
       per_point_value = default_per_point_value
       per_member_redemption = full
   else:
       # Roll-forward: each Member redeems a proportion equal to fund_cash / nominal_total
       redemption_ratio = fund_cash / (total_redeemable_points * default_per_point_value)
       # But roll-forward means: each Member's redeemed-this-quarter points = round(points * redemption_ratio)
       # remaining points → bp_balance.points_rolled_forward_out
       per_point_value = default_per_point_value
       for each employee:
         redeemed = round(employee_points * redemption_ratio)
         rolled_out = employee_points - redeemed
         bp_balance.points_redeemed_this_quarter = redeemed
         bp_balance.points_rolled_forward_out = rolled_out
         bp_balance.cash_payout = redeemed * per_point_value
   ```

   The rolled-forward points carry their `redemption_ratio_origin: <prior-quarter>` metadata so a Member can see why their rolled-forward came to be.

5. **Sign + commit.** Founder + Engineering Lead countersign the close: per-Member payouts + the per-point value + the roll-forward decisions. The Audit chain captures the close payload.
6. **Feed FR-REW-003 payroll close.** The per-Member `cash_payout_minor` is the `p3_cash_for_quarter` consumed by FR-REW-003 in the post-quarter payroll cycle.

**ACB savings rate snapshot.**

A small `cyberos-rew-rate-poll` job runs the day before each quarter starts:
1. Fetches the current ACB savings rate from `https://acb.com.vn/wps/portal/acb/rates/savings` (or the latest published rate page; the URL is parameterised in `acb_rate_source_url`).
2. Records the rate + URL + access timestamp.
3. The rate is signed by Founder + Engineering Lead + entered into the next `bp_fund_quarter.acb_savings_rate_snapshot_pct`.

If ACB's URL fails or the parsing fails, the rate is set to the *prior quarter's snapshot* and a sev-1 alert fires; the founder + Engineering Lead manually verify and sign within 48 hours.

**Earning-event ingestion.**

Earning events come from:

1. **LEARN P2 — Variable Performance evaluation roll-up.** A scheduled VP evaluation produces points per Member based on the structured rubric.
2. **LEARN P2 — Hội đồng Chuyên môn promotion.** A promotion decision can carry a one-time BP grant.
3. **Founder ad-hoc recognition.** The founder can grant points with explicit reason; signed; rare.
4. **Clawback** (negative points). Used in the Bad-Leaver branch (FR-REW-007). Requires founder + DPO sign + a documented reason.

Each event:
- Created in draft; HR/Ops Lead reviews for quarter-bounds.
- Signed by Founder; on sign, immutability trigger applies.
- Audit row; visible in the Member's `/rew/my` view (FR-REW-005) with the `reason_md` so the Member sees why points were granted.

**Member's view.**

`/rew/my` (FR-REW-005) shows the calling Member:
- Current quarter's points-earned-so-far.
- Rolled-forward points balance.
- Last quarter's payout + reason breakdown.
- Projected next-quarter payout (best-case + worst-case bands derived from the BP fund's cash + total points; informational).

The view requires step-up auth; only the calling Member sees their own data.

**Compliance Cockpit panel (FR-CP-001).**

- BP fund cash + points by quarter (aggregate; not per-Member).
- ACB rate timeline (visible inflation-tracker overlay).
- Roll-forward count + percentage by quarter.
- Earning-event signature compliance.

**MCP tool surface (read-only).**

- `cyberos.rew.my_bp_balance(quarter?)` — read; calling Member's own; step-up at gateway.
- `cyberos.rew.list_bp_fund_quarters(status?)` — read; HR/Ops + Founder + DPO; aggregates only.
- `cyberos.rew.get_bp_fund_quarter(quarter)` — read; aggregates only.

There are **no mutation MCP tools** for BP — same architectural rule as FR-REW-001. Earning-event creation is via HR/Ops Lead frontend; quarterly close is via a sealed-credentials cron + founder + Engineering Lead UI sign-off.

## Alternatives Considered

- **Dilute on over-allocation.** Rejected: Total Rewards Appendix Article 2c specifies roll-forward; the founder's contract with Members forbids dilution.
- **Use the State Bank of Vietnam (SBV) base rate instead of ACB savings rate.** Rejected: the Total Rewards Appendix specifies ACB. The architectural pattern (anchor to a published, conservative, savings-tier rate) is what matters; ACB is the chosen anchor.
- **Compound interest annually instead of quarterly.** Rejected: the BP cycle is quarterly; quarterly compounding aligns the math with the cycle; the rate's annual representation is divided by 4 for one-quarter accrual.
- **Allow AI to compute payout amounts.** Rejected — explicit architectural prohibition.
- **Skip earning-event signature.** Rejected: every comp event is contractual; signature is the floor.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) parameter version v1 includes the BP fund rules; (2) a synthetic Q3 close runs with 10 employees + a known-quantity earning ledger; the per-Member payouts match a hand-computed expected value; (3) a synthetic over-allocation triggers roll-forward (no dilution); (4) the ACB rate poll succeeds; (5) the immutability trigger rejects an attempted UPDATE on a signed earning event.
- **Compliance metric.** Zero instances of dilution applied to a Member's BP balance over the lifetime of the platform. Zero instances of an AI surface modifying a BP value.
- **Audit completeness.** Every earning event signed; every quarterly close signed by founder + engineering lead; every ACB rate snapshot has source URL + timestamp.

## Scope

**In-scope.**
- The 3 schema additions (`bp_fund_quarter`, `bp_balance`, `bp_earning_event`).
- Anti-retroactive immutability triggers on earning events.
- ACB rate poll job + sealed-source archiving.
- Quarterly close pipeline (deterministic compute).
- Roll-forward semantics with bias-toward-Member rounding.
- Earning-event ingestion paths from LEARN-P2 (stub) + Founder ad-hoc.
- Clawback path (negative points; rare; signed).
- Member's `/rew/my` view (read-only; step-up).
- Compliance Cockpit panel.
- The 3 read-only MCP tools.
- Audit integration in scope `rew.bp.{tenant}`.

**Out-of-scope (deferred).**
- Phantom Stock fund (P3 — separate cluster).
- Multi-currency BP funds (the company collects in tenant currency only).
- BP-to-ESOP conversion (P3+).
- Per-Member BP forecasting models (P3 informational; never determinative).
- Automated VP roll-up from LEARN P2 (this FR consumes the events; LEARN ships their authoring in batch-07).

## Dependencies

- FR-HR-001 / FR-REW-001 (the substrate + the parameter-version + signature primitives + KMS keys).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-001 (Compliance Cockpit panel).
- FR-LEARN-001..N (batch-07; produces the earning events).
- FR-OBS-001 / FR-OBS-002 (ACB rate poll alert; close-cycle dashboards).
- The signed Total Rewards Appendix as the legal source-of-truth.
- The ACB published-rates page (or a documented alternative pre-approved by founder + engineering lead + legal counsel).
- Compliance: Vietnamese Labour Code on bonuses + delayed compensation; PDPL Decree 13; EU AI Act high-risk classification (no AI in compute); GDPR Article 22 (deterministic; no auto decisions).
- Locked decisions referenced: DEC-169 (ACB savings rate as anti-inflation anchor), DEC-170 (roll-forward over dilution), DEC-171 (quarterly compounding), DEC-172 (no MCP write tools for BP), DEC-173 (earning-event sign-then-immutable).

## AI Risk Assessment

The BP fund mechanics explicitly forbid AI in the compute path; the only AI surface (FR-REW-005 narrator) is read-only on the data this FR produces. EU AI Act risk class: `high` (compensation domain).

### Data Sources

The schema stores deterministic computation results. No AI inputs in the close pipeline. The narrator (FR-REW-005) consumes the data via the AI Gateway with persona-stamping; per-tenant residency.

### Human Oversight

- Every earning event is human-signed before becoming immutable.
- Quarterly close is signed by founder + engineering lead.
- ACB rate snapshot is signed.
- Clawback requires founder + DPO + reason.
- The Compliance Cockpit panel surfaces every close + every roll-forward decision.
- The Member sees their breakdown + per-source reasons; can dispute via founder.

### Failure Modes

- **ACB rate poll fails.** Falls back to prior quarter's snapshot + sev-1 alert; founder + engineering lead manually verify within 48 hours.
- **Over-allocation drift.** Mitigation: the deterministic close enforces roll-forward; the parameter version's `default_per_point_value` is reviewed before each quarter starts.
- **Rounding-residual accumulation.** Mitigation: rounding bias is toward Member at the per-Member step; the residual is the company's; the annual reconciliation surfaces any abnormality.
- **Trigger bypass via DB direct access.** Mitigated by the same `cyberos_app` role grant restrictions as FR-REW-001.
- **Comp value leak into BRAIN.** Caught by the structural ingestion exclusion + nightly sweep.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, ACB rate snapshot, quarterly close pipeline, roll-forward semantics, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel will review the BP-fund encoding against the Total Rewards Appendix before P2 production deployment.
