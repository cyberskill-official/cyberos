---
workflow_id: chief-sales-officer/quarterly-nps-program
workflow_version: 1.0.0
purpose: Run the quarterly customer NPS program — survey distribution, results analysis, root-cause synthesis, action plan.
persona: cuo/chief-sales-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_nps,          source: last quarter's nps-program@1,                          format: net-promoter-score-program@1 }
  - { name: survey_results,     source: Delighted / Qualtrics / SurveyMonkey / built-in CRM,   format: csv export }
  - { name: open_actions,       source: prior-quarter action plan + status,                    format: markdown }

outputs:
  - { name: nps_pulse,          format: nps-program@1, recipient: cuo/cso-sales + cuo/cco-customer + cuo/ceo + Board (chapter) }

skill_chain:
  - { step: 1, skill: net-promoter-score-program-author, inputs_from: { prior_nps: prior_nps, survey_results: survey_results, open_actions: open_actions }, outputs_to: pulse_draft }
  - { step: 2, skill: net-promoter-score-program-audit,  inputs_from: pulse_draft, outputs_to: nps_pulse }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "company NPS drops >10 pts QoQ" }
  - { persona: cuo/chief-customer-officer, when: "detractor cluster identified — CS intervention" }

consults:
  - { persona: cuo/chief-product-officer, when: "NPS theme is product-driven (specific feature/UX complaint)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with nps_pulse hash + company NPS + promoter/passive/detractor split
  - HITL pause at step 2 on QA-RESPONSE-001 (response rate < 25%) or QA-ROOT-001 (no root-cause synthesis)
---

# Quarterly NPS program — `chief-sales-officer/quarterly-nps-program`

CSO-Sales' quarterly customer-NPS workflow. Per Reichheld / Bain NPS methodology: survey via Delighted / Qualtrics / built-in CRM, analyse promoters / passives / detractors, root-cause synthesis on verbatims, action plan.

## When to invoke

- "Run the Q<n> NPS"
- "Customer NPS results"
- "NPS quarterly review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-sales-officer/quarterly-nps-program \
  --input prior_nps=./customer/2026-Q1/nps.md \
  --input survey_results=./customer/2026-Q2/delighted.csv \
  --input open_actions=./customer/2026-Q1/nps-actions.md \
  --output-dir ./customer/2026-Q2/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 3 weeks for survey window + verbatim coding
- **Worst case:** detractor-cluster intervention may span 1 quarter

## Skill chain

- **Step 1 `net-promoter-score-program-author`** — drafts per Reichheld / Bain NPS + Net Promoter System.
- **Step 2 `net-promoter-score-program-audit`** — validates per `nps_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-RESPONSE-001 | Response < 25% | Operator extends + re-prompts |
| 2 | QA-ROOT-001 | No root-cause | Operator codes verbatims |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CSO-Sales role profile
- `../../cco-customer/README.md` — CS peer for detractor intervention
- `../../../skill/nps-program-{author,audit}/SKILL.md`
