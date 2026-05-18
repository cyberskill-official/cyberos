---
workflow_id: chief-human-resources-officer/annual-comp-cycle
workflow_version: 1.0.0
purpose: Run the annual compensation cycle — band refresh, per-employee recommendation, equity refresh, manager-distribution + calibration.
persona: cuo/chief-human-resources-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_comp_plan,    source: last year's comp-plan@1,                  format: compensation-plan@1 }
  - { name: market_data,        source: Radford / Pave / Compa / Levels.fyi snap, format: csv export }
  - { name: budget_envelope,    source: cuo/cfo (annual budget comp line),        format: budget@1 chapter }
  - { name: perf_distribution,  source: prior-cycle performance ratings,          format: csv }

outputs:
  - { name: comp_plan,          format: comp-plan@1, recipient: cuo/chro + cuo/cfo + cuo/ceo + managers (cascade) }

skill_chain:
  - { step: 1, skill: compensation-plan-author, inputs_from: { prior_comp_plan: prior_comp_plan, market_data: market_data, budget_envelope: budget_envelope, perf_distribution: perf_distribution }, outputs_to: plan_draft }
  - { step: 2, skill: compensation-plan-audit,  inputs_from: plan_draft, outputs_to: comp_plan }

escalates_to:
  - { persona: cuo/chief-financial-officer,         when: "total recommendation > budget envelope by > 5%" }
  - { persona: cuo/chief-executive-officer,         when: "C-suite or VP-level comp changes need final sign-off" }

consults:
  - { persona: cuo/chief-communications-officer, when: "comp-cycle internal-comms (manager scripts + employee FAQs)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with comp_plan hash + band-update count + total-cost-vs-envelope %
  - HITL pause at step 2 on QA-BIAS-001 (disparate-impact pattern by protected class) or QA-PAY-EQUITY-001
---

# Annual comp cycle — `chief-human-resources-officer/annual-comp-cycle`

CHRO's annual compensation cycle. Refreshes bands per Radford / Pave / Compa benchmarks, generates per-employee recommendations against budget envelope + perf distribution, runs manager calibration. Audited for pay-equity + bias.

## When to invoke

- "Run the 2026 comp cycle"
- "Annual compensation review"
- "Refresh comp bands"

## How to invoke

```bash
cyberos-cuo run cuo/chief-human-resources-officer/annual-comp-cycle \
  --input prior_comp_plan=./hr/2025/comp-plan.md \
  --input market_data=./hr/2026/radford-snap.csv \
  --input budget_envelope=./budget/2026/comp-chapter.md \
  --input perf_distribution=./hr/2025-perf-distribution.csv \
  --output-dir ./hr/2026/comp/
```

## Expected duration

- **Happy path:** 3-6 hours runtime + 4-8 weeks for manager round-trip + calibration
- **Worst case:** pay-equity remediation may add 1 quarter

## Skill chain

- **Step 1 `compensation-plan-author`** — drafts per Radford / WorldatWork methodology.
- **Step 2 `compensation-plan-audit`** — validates per `comp_plan_rubric@1.0` (FM + SEC + QA-BIAS-001 + QA-PAY-EQUITY-001 + QA-BUDGET-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-BIAS-001 | Disparate-impact pattern by protected class | Escalate; remediate |
| 2 | QA-PAY-EQUITY-001 | Pay-equity gap unexplained | Operator investigates |
| 2 | QA-BUDGET-001 | Recommendation > envelope | Escalate to CFO |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5 — CHRO role profile
- `../../chief-financial-officer/workflows/annual-budget.md` — peer feeding budget envelope
- `../../../skill/comp-plan-{author,audit}/SKILL.md`
