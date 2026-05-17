---
workflow_id: chief-financial-officer/quarterly-forecast
workflow_version: 1.0.0
purpose: Build the rolling quarterly forecast — revenue + expense + cash forecast with bridge from prior quarter and ±10% accuracy target.
persona: cuo/chief-financial-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_close,        source: cuo/chief-financial-officer/monthly-close (last 3 months), format: monthly-close@1 }
  - { name: prior_forecast,     source: last quarter's forecast@1,             format: forecast@1 }
  - { name: pipeline_snapshot,  source: cuo/cro-revenue (or sales head),       format: pipeline-report@1 or CRM extract }
  - { name: hire_plan,          source: cuo/chro (workforce-plan),             format: workforce-plan@1 }

outputs:
  - { name: forecast,           format: forecast@1, recipient: cuo/cfo + cuo/ceo + Board (board-deck financial chapter) }

skill_chain:
  - { step: 1, skill: forecast-author, inputs_from: { prior_close: prior_close, prior_forecast: prior_forecast, pipeline_snapshot: pipeline_snapshot, hire_plan: hire_plan }, outputs_to: forecast_draft }
  - { step: 2, skill: forecast-audit,  inputs_from: forecast_draft, outputs_to: forecast }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "forecast-audit fires QA-ACCURACY-001 — prior forecast missed ±10% target without narrative" }

consults:
  - { persona: cuo/chief-revenue-officer, when: "pipeline-coverage assumption < 3x — revenue forecast unrealistic" }
  - { persona: cuo/chief-human-resources-officer,        when: "hire plan slipping — opex forecast needs adjustment" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with forecast hash + per-line confidence band
  - HITL pause at step 2 on QA-DRIVER-001 (forecast line lacks named driver)
---

# Quarterly forecast — `chief-financial-officer/quarterly-forecast`

CFO's rolling quarterly forecast workflow. Combines prior close + prior forecast + pipeline + hire plan into a driver-based forecast with ±10% accuracy target. Per Adaptive / Anaplan / Pigment best practice: every line tied to a named driver, not a copy-forward from prior period.

## When to invoke

- "Run the Q<n> forecast"
- "Build the rolling 4Q forecast"
- "Refresh the quarterly outlook"

## How to invoke

```bash
cyberos-cuo run cuo/chief-financial-officer/quarterly-forecast \
  --input prior_close=./close/2026-Q1/ \
  --input prior_forecast=./forecast/2026-Q1/final.md \
  --input pipeline_snapshot=./sales/2026-Q1-apr/pipeline.md \
  --input hire_plan=./hr/2026-workforce-plan.md \
  --output-dir ./forecast/2026-Q2/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 1 week of cross-function input gathering
- **Worst case:** if pipeline-coverage assumption fails QA, revenue-only re-forecast adds 3-5 days

## Skill chain

- **Step 1 `forecast-author`** — drafts per Adaptive / Anaplan template: drivers / assumptions / revenue waterfall / opex stack / cash bridge.
- **Step 2 `forecast-audit`** — validates per `forecast_rubric@1.0` (FM + SEC + QA-DRIVER-001 (every line has named driver) + QA-ACCURACY-001 (prior-period look-back) + QA-CONFIDENCE-001 (best/likely/worst bands)).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-DRIVER-001 | Forecast line lacks named driver | Operator supplies driver basis |
| 2 | QA-ACCURACY-001 | Prior forecast off >10% with no narrative | Escalate to CEO |
| 2 | QA-CONFIDENCE-001 | No best/likely/worst bands | Operator adds confidence range |

## Cross-references
- `../README.md` §5 (Strategic) — output type "forecast"
- `../../../docs/The C-Suite Reference.md` §5.2
- `../../../skill/forecast-{author,audit}/SKILL.md`
