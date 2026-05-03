---
title: "REW — Vietnamese SI/PIT statutory engine: social insurance + health + unemployment + personal income tax with versioned rate tables"
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

Implement the Vietnamese statutory deduction engine consumed by FR-REW-003: **employee Social Insurance (SI)** at the prevailing rate (currently 8% on capped salary base; salary cap = 20× regional minimum wage); **employer SI contribution** (17.5%); **employee Health Insurance (HI)** at 1.5%; **employer HI** at 3%; **employee Unemployment Insurance (UI)** at 1% (capped at the same regional minimum-wage floor); **employer UI** at 1%; **Personal Income Tax (PIT)** computed via the seven-bracket progressive table after the personal deduction (currently 11M VND/month) + dependent deductions (4.4M VND/month/dependent); regional **minimum wage zones** I/II/III/IV; **versioned rate tables** under the parameter-version anti-retroactive contract from FR-REW-001 — every change to the statutory rates creates a new parameter version; **annual PIT reconciliation** at year-end (Vietnamese tax-year = calendar year); the engine is fully **deterministic**; AI is forbidden from the compute path; results are signed + traceable.

## Problem

Vietnamese SI/PIT is statutorily complex (multiple insurance funds + capped bases + tax-bracket progression + regional zones + dependent deductions) and changes annually. PRD §14.3.1 P2 scope: "Decree 13 full-regime graduation"; PRD §14.3.2 P2 → P3 gate requires payroll close runs entirely inside REW. Three failure modes the engine must structurally avoid:

- **Wrong cap on SI base.** The salary base for SI is capped at 20× regional minimum wage; computing on the full salary instead is the most common payroll-software error in Vietnam — it over-deducts SI and under-pays the Member.
- **Stale rate tables.** Vietnamese statutory rates change at New Year + occasionally mid-year; using an outdated rate produces wrong deductions every cycle until corrected. The parameter-version anti-retroactive contract is the architectural floor.
- **AI in compute.** Same PRD §2.5 + §6.4 invariants as FR-REW-001..003. Deterministic computation is the floor.

## Proposed Solution

The shape of the answer is `hr_secure.statutory_*` schema + the deterministic engine + versioned rate tables + the year-end reconciliation report.

**Schema.**

```sql
-- Statutory rate tables versioned alongside REW parameter versions.
CREATE TABLE hr_secure.statutory_rates (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  jurisdiction TEXT NOT NULL DEFAULT 'VN',
  effective_from DATE NOT NULL,
  effective_to DATE,
  -- Insurance rates as percentages (annualised)
  employee_si_pct REAL NOT NULL,                            -- 8.0
  employer_si_pct REAL NOT NULL,                            -- 17.5
  employee_hi_pct REAL NOT NULL,                            -- 1.5
  employer_hi_pct REAL NOT NULL,                            -- 3.0
  employee_ui_pct REAL NOT NULL,                            -- 1.0
  employer_ui_pct REAL NOT NULL,                            -- 1.0
  si_base_cap_multiplier REAL NOT NULL,                     -- 20.0 (× regional min wage)
  ui_base_cap_multiplier REAL NOT NULL,                     -- 20.0 (× regional min wage)
  pit_personal_deduction_monthly_minor BIGINT NOT NULL,     -- 11,000,000 in VND minor
  pit_dependent_deduction_monthly_minor BIGINT NOT NULL,    -- 4,400,000 in VND minor
  pit_brackets JSONB NOT NULL,                              -- the 7-bracket progressive table:
                                                            -- [{ up_to: 5_000_000, rate: 0.05 },
                                                            --  { up_to: 10_000_000, rate: 0.10 }, ...]
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  source_legal_ref TEXT NOT NULL,                            -- "Decree 12/2024/NĐ-CP" + "Resolution 954/2020/UBTVQH14"
  source_url TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Regional minimum wage table (zones I/II/III/IV per Vietnamese decree).
CREATE TABLE hr_secure.regional_minimum_wage (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  zone_code TEXT NOT NULL,                                  -- "I" | "II" | "III" | "IV"
  monthly_minimum_wage_minor BIGINT NOT NULL,
  hourly_minimum_wage_minor BIGINT,
  effective_from DATE NOT NULL,
  effective_to DATE,
  source_legal_ref TEXT NOT NULL,
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, zone_code, effective_from)
);

-- Per-employee statutory profile (their dependents, their region, their SI start date).
CREATE TABLE hr_secure.statutory_profile (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE CASCADE,
  region_zone_code TEXT NOT NULL,                           -- the zone of their primary workplace
  si_started_at DATE,                                        -- when their VN SI participation started
  ui_started_at DATE,
  hi_started_at DATE,
  registered_dependents_count INT NOT NULL DEFAULT 0,
  dependents_breakdown_encrypted BYTEA,                       -- encrypted JSONB: per-dependent details
  pit_id_encrypted BYTEA,                                     -- VN MST cá nhân
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (tenant_id, employee_id)
);

-- Year-end PIT reconciliation per Member.
CREATE TABLE hr_secure.pit_reconciliation (
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  year INT NOT NULL,
  total_taxable_minor_encrypted BYTEA NOT NULL,
  total_pit_withheld_minor_encrypted BYTEA NOT NULL,
  computed_pit_minor_encrypted BYTEA NOT NULL,
  pit_difference_minor_encrypted BYTEA NOT NULL,             -- (computed - withheld); positive = additional owed; negative = refund
  filing_status TEXT NOT NULL DEFAULT 'pending',             -- "pending" | "filed" | "settled" | "disputed"
  filing_signed_off_by_accountant_ref TEXT,                  -- external accountant's reference
  filed_at DATE,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, employee_id, year)
);
```

**Deterministic engine — monthly per-Member compute.**

Inputs to `compute_statutory(employee_id, gross_monthly_minor, parameter_version_id, cycle_month)`:

```pseudocode
1. Load `statutory_rates` row for the parameter_version + cycle_month effective date.
2. Load `regional_minimum_wage` for the employee's zone + same effective date.
3. Compute SI/HI/UI bases:
   si_base = min(gross_monthly_minor, statutory_rates.si_base_cap_multiplier * regional_min_wage)
   ui_base = min(gross_monthly_minor, statutory_rates.ui_base_cap_multiplier * regional_min_wage)
   hi_base = si_base                                # in VN, HI uses the same base as SI
4. Compute insurance deductions:
   employee_si  = round(si_base * statutory_rates.employee_si_pct / 100)
   employer_si  = round(si_base * statutory_rates.employer_si_pct / 100)
   employee_hi  = round(hi_base * statutory_rates.employee_hi_pct / 100)
   employer_hi  = round(hi_base * statutory_rates.employer_hi_pct / 100)
   employee_ui  = round(ui_base * statutory_rates.employee_ui_pct / 100)
   employer_ui  = round(ui_base * statutory_rates.employer_ui_pct / 100)
5. Compute PIT taxable income:
   taxable_pre_deduction = gross_monthly_minor - employee_si - employee_hi - employee_ui
   personal_deduction = statutory_rates.pit_personal_deduction_monthly_minor
   dependents_deduction = statutory_profile.registered_dependents_count
                          * statutory_rates.pit_dependent_deduction_monthly_minor
   taxable_income = max(0, taxable_pre_deduction - personal_deduction - dependents_deduction)
6. Apply progressive bracket table:
   pit = 0
   remaining = taxable_income
   prev_threshold = 0
   for bracket in pit_brackets:
       slice = min(remaining, bracket.up_to - prev_threshold)
       pit += round(slice * bracket.rate)
       remaining -= slice
       prev_threshold = bracket.up_to
       if remaining <= 0: break
   # Last bracket has rate 0.35; the table caps with up_to = +infinity for the top bracket.
7. Return:
   {
     employee_si, employer_si, employee_hi, employer_hi,
     employee_ui, employer_ui, pit, taxable_income,
     trace: { si_base, ui_base, hi_base, taxable_pre_deduction, ... }
   }
```

The engine is a pure function: same inputs → same outputs. Versioned. Reproducible. Inspectable. The `trace` is captured in `payroll_record.computation_trace_encrypted` (FR-REW-003).

**Default seed (Vietnam, 2026).**

The first parameter version's statutory_rates row, signed at deployment:

| Field | Value |
|---|---|
| employee_si_pct | 8.0 |
| employer_si_pct | 17.5 |
| employee_hi_pct | 1.5 |
| employer_hi_pct | 3.0 |
| employee_ui_pct | 1.0 |
| employer_ui_pct | 1.0 |
| si_base_cap_multiplier | 20.0 |
| ui_base_cap_multiplier | 20.0 |
| pit_personal_deduction_monthly_minor | 11,000,000 |
| pit_dependent_deduction_monthly_minor | 4,400,000 |
| pit_brackets | [`up_to:5M @ 5%`, `up_to:10M @ 10%`, `up_to:18M @ 15%`, `up_to:32M @ 20%`, `up_to:52M @ 25%`, `up_to:80M @ 30%`, `>80M @ 35%`] |

Regional minimum wage per Vietnamese Decree 12/2024/NĐ-CP:
- Zone I (HCMC, Hanoi major districts): 4,960,000 VND/month
- Zone II (HCMC outer, Hanoi outer, Da Nang): 4,410,000
- Zone III (provincial cities): 3,860,000
- Zone IV (rural): 3,450,000

(Values current as of 2026 expectations; update at year-end via parameter-version flow.)

**Annual PIT reconciliation.**

At year-end (typically March of the following year — Vietnamese tax law deadline):

1. **Aggregate per Member.** Sum of all monthly `payroll_record.gross_compensation_minor_encrypted` minus any non-taxable allowances; subtract sum of all monthly statutory deductions; recompute total taxable income.
2. **Recompute annual PIT** using the annualised brackets (annual = monthly × 12 with annual deductions × 12).
3. **Compare to total PIT withheld.** Difference = additional owed (positive) or refund (negative).
4. **Generate the annual PIT reconciliation report** per Member (signed PDF, like the monthly payslip).
5. **Filing.** The HR/Ops Lead + the company's external accountant file the reconciliation with the Vietnamese tax authority via the e-tax portal (manual upload; no public API). The accountant's filing reference is recorded.
6. **Settlement.** Additional owed → either deducted from the next month's payroll or paid directly by the Member; refund → returned via the next payroll. The settlement transaction is logged in `pit_reconciliation`.

**Rate-table publishing flow.**

When Vietnamese statutory rates change (typically at year-end):
1. The HR/Ops Lead drafts a new `parameter_version` (FR-REW-001 flow) with the new `statutory_rates` and `regional_minimum_wage` rows.
2. Legal counsel + Vietnamese accountant review.
3. Founder + Engineering Lead sign + publish.
4. The next payroll cycle automatically uses the new rates (cycle_month >= rates' effective_from).

**Testing.**

A regression test suite of synthetic Members across multiple gross levels + dependent counts + zones; the expected outputs are computed by hand by a Vietnamese accountant + signed off; the engine's outputs must match exactly. The suite runs on every parameter-version PR. A regression blocks the PR.

**MCP tool surface (read-only).**

- `cyberos.rew.list_statutory_rates(jurisdiction?, as_of_date?)` — read; HR/Ops + Founder + DPO + Auditor.
- `cyberos.rew.get_my_statutory_profile` — read; calling Member's own; step-up.
- `cyberos.rew.simulate_statutory_compute(gross_monthly_minor, region_zone, dependents_count, parameter_version_id?)` — read; HR/Ops + Founder; useful for "what's the take-home for a 30M VND gross?".
- `cyberos.rew.my_pit_reconciliation(year)` — read; calling Member; step-up.

There are no mutation MCP tools. Rate-table updates are HR/Ops Lead UI + Founder + Engineering Lead sign + parameter-version publish.

## Alternatives Considered

- **Use a hosted Vietnamese payroll service (KiotViet, MISA AMIS, Tanca).** Rejected: residency + the parameter-version contract + the Total Rewards Appendix integration + the audit-grade trace are not viable hosted.
- **Hardcode rates in code.** Rejected: rates change annually; the parameter-version flow is the floor.
- **Allow estimated rates** (e.g. "we know the rate changed but haven't updated yet"). Rejected: zero ambiguity; the cycle either uses the published rate or fails to compute.
- **AI-assisted PIT optimisation.** Rejected: PIT is statutory; "optimisation" is what tax-evasion looks like. The engine produces the legally-correct number; nothing else.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the regression test suite of 20 synthetic profiles produces outputs matching the accountant's hand-computed expected values to the minor unit; (2) a synthetic FR-REW-003 cycle calls this engine and produces correct net values; (3) the year-end reconciliation report is generated for the 10 employees with sensible deltas.
- **Compliance metric.** Zero statutory-rate computation errors over the lifetime of the platform (audit-checked quarterly by the external accountant).
- **Performance.** Engine compute per-Member ≤ 5 ms (deterministic; trivial).

## Scope

**In-scope.**
- The 4 schema additions (`statutory_rates`, `regional_minimum_wage`, `statutory_profile`, `pit_reconciliation`).
- Default seed of 2026 Vietnamese statutory rates + zones.
- The deterministic compute function.
- 7-bracket progressive PIT.
- SI/HI/UI base capping at the regional minimum-wage multiplier.
- Year-end PIT reconciliation flow with PDF report.
- Rate-table parameter-version publishing.
- 20+ regression test fixtures with hand-computed expected outputs.
- The 4 read-only MCP tools.
- Audit integration in scope `rew.statutory.{tenant}`.

**Out-of-scope (deferred).**
- Non-Vietnamese jurisdictions (P3+ when international hires).
- Year-end PIT auto-filing via e-tax portal (P3 if the portal exposes a stable API; manual upload is the floor).
- Personal-income-tax planning advice (would be high-risk; not within scope).
- Equity income tax handling (P3 — when ESOP ships).

## Dependencies

- FR-HR-001 / FR-REW-001.
- FR-REW-003 (consumer of this engine).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-001.
- A Vietnamese-licensed accountant for the regression-suite hand-computed expected outputs.
- Compliance: Vietnamese Tax Code + Decree 12/2024/NĐ-CP + Resolution 954/2020/UBTVQH14 + Law on Social Insurance + Law on Health Insurance + Law on Employment; PDPL Decree 13; EU AI Act high-risk classification (compensation domain — no AI in compute path).
- Locked decisions referenced: DEC-178 (deterministic SI/PIT engine; no AI), DEC-179 (regression-suite hand-computed by accountant gates every parameter version), DEC-180 (year-end reconciliation is human-filed).

## AI Risk Assessment

This FR explicitly forbids AI in the compute path. EU AI Act risk class: `high` (compensation domain).

### Data Sources

The compute uses statutory rate tables + per-Member profile data + the gross compensation. No AI inputs. No third-party data; per-tenant residency.

### Human Oversight

- Every rate-table version requires founder + engineering-lead sign + accountant review.
- Year-end reconciliation is filed by HR/Ops Lead + accountant manually.
- Member sees their own monthly + annual numbers; can dispute.

### Failure Modes

- **Stale rate table.** Mitigation: the rate-table version's `effective_to` is checked at every cycle; an expired version triggers a sev-1 alert demanding a new version.
- **Wrong zone for a Member.** Mitigation: zone is set at hire + reviewed annually; the simulate tool helps HR/Ops verify.
- **Bracket boundary off-by-one.** Mitigation: regression suite fixtures cover all bracket boundaries.
- **Rounding accumulation drift.** Mitigation: all rounding is consistent; the trace records every round.
- **Dependent-count miscount.** Mitigation: the Member declares dependents; HR/Ops Lead reviews supporting documents before recording.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, deterministic engine, rate-table seed, year-end reconciliation, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the regression suite + the actual Vietnamese statutory rates will be verified by the company's external accountant before P2 production.
