---
workflow_id: chief-sales-officer/weekly-pipeline-review
workflow_version: 1.0.0
purpose: Run the weekly pipeline review — stage-by-stage health, deal slippage, win/loss trends, quota-attainment forecast.
persona: cuo/chief-sales-officer
cadence: weekly
status: shipped

inputs:
  - { name: crm_extract,        source: Salesforce / HubSpot / Close / Pipedrive, format: csv export }
  - { name: prior_pipeline,     source: last week's pipeline-report@1,            format: pipeline-report@1 }
  - { name: rep_inputs,         source: each AE's 5-min commit/upside notes,      format: markdown (one per rep) }

outputs:
  - { name: pipeline_report,    format: pipeline-report@1, recipient: cuo/cso-sales + cuo/ceo + cuo/cfo }

skill_chain:
  - { step: 1, skill: pipeline-report-author, inputs_from: { crm_extract: crm_extract, prior_pipeline: prior_pipeline, rep_inputs: rep_inputs }, outputs_to: report_draft }
  - { step: 2, skill: pipeline-report-audit,  inputs_from: report_draft, outputs_to: pipeline_report }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "pipeline coverage < 3x of quarter quota" }
  - { persona: cuo/chief-financial-officer,         when: "forecast vs commit gap implies revenue miss > 10%" }

consults:
  - { persona: cuo/chief-customer-officer, when: "expansion-stage deals at risk — CS intervention" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with pipeline_report hash + coverage-ratio + commit-vs-quota
  - HITL pause at step 2 on QA-AGE-001 (stage age > stage-target without rationale)
---

# Weekly pipeline review — `chief-sales-officer/weekly-pipeline-review`

CSO-Sales' weekly pipeline discipline. Stage-by-stage health + slippage analysis + commit-upside-best-case forecast against quota. Standard MEDDIC / SPICED / Sandler qualification overlay (per the org's chosen method). Targets 3-4x coverage on the current quarter quota.

## When to invoke

- "Run the weekly pipeline review"
- "Pipeline health check"
- "Where are we on Q<n> quota"

## How to invoke

```bash
cyberos-cuo run cuo/chief-sales-officer/weekly-pipeline-review \
  --input crm_extract=./sales/2026-W20/sfdc.csv \
  --input prior_pipeline=./sales/2026-W19/pipeline.md \
  --input rep_inputs=./sales/2026-W20/rep-notes/ \
  --output-dir ./sales/2026-W20/
```

## Expected duration

- **Happy path:** 30-60 min runtime + Friday afternoon roll-up
- **Worst case:** coverage shortfall triggers same-day CEO escalation

## Skill chain

- **Step 1 `pipeline-report-author`** — drafts per Winning by Design + MEDDIC qualification.
- **Step 2 `pipeline-report-audit`** — validates per `pipeline_report_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-AGE-001 | Deal aged in stage | Operator surfaces with rep |
| 2 | QA-COVERAGE-001 | < 3x coverage | Escalate to CEO |

## Cross-references
- `../../../../modules/cuo/README.md` §5.4 — CSO-Sales / CGO role profile
- `../../cco-customer/README.md` — expansion-deal peer
- `../../../skill/pipeline-report-{author,audit}/SKILL.md`
