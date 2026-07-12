---
workflow_id: chief-ai-officer/quarterly-use-case-portfolio-review
workflow_version: 1.0.0
purpose: Review the AI use-case portfolio — pipeline stage, value × feasibility × risk score, sunset candidates.
persona: cuo/chief-ai-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_portfolio,       source: last quarter's ai-use-case-portfolio@1, format: ai-use-case-portfolio@1 }
  - { name: production_metrics,    source: deployed models' performance + drift data, format: csv }
  - { name: pipeline_intake,       source: new use-case proposals from business units, format: markdown briefs }
  - { name: incident_log,          source: model-incident postmortems, format: postmortem@1 (multiple) }

outputs:
  - { name: use_case_portfolio,    format: ai-use-case-portfolio@1, recipient: cuo/caio + cuo/cto + cuo/chief-ethics-officer + cuo/ceo }

skill_chain:
  - { step: 1, skill: ai-use-case-portfolio-author, inputs_from: { prior_portfolio: prior_portfolio, production_metrics: production_metrics, pipeline_intake: pipeline_intake, incident_log: incident_log }, outputs_to: portfolio_draft }
  - { step: 2, skill: ai-use-case-portfolio-audit,  inputs_from: portfolio_draft, outputs_to: use_case_portfolio }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "use case in production is recommended for sunset due to value/risk shift" }
  - { persona: cuo/chief-ethics-officer, when: "incident log surfaces fairness / explainability concerns" }

consults:
  - { persona: cuo/chief-data-officer,       when: "use case requires net-new data foundations" }
  - { persona: cuo/chief-product-officer,    when: "user-facing use case affects product surface" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with use_case_portfolio hash + new-pipeline count + sunset-count + production-MTTR
  - HITL pause at step 2 on QA-VALUE-001 (use-case value not measurable) or QA-RISK-001 (risk score lacks rationale)
---

# Quarterly use-case portfolio review — `chief-ai-officer/quarterly-use-case-portfolio-review`

CAIO's quarterly AI use-case portfolio review. Per Gartner AI Use-Case Prism + Stanford HAI use-case taxonomy. Each use case scored on value × feasibility × risk; pipeline managed gated through ethics + privacy + data-readiness reviews.

## When to invoke

- "Run the Q<n> AI use-case portfolio review"
- "AI portfolio refresh"
- "Use-case pipeline review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ai-officer/quarterly-use-case-portfolio-review \
  --input prior_portfolio=./ai/2026-Q1/portfolio.md \
  --input production_metrics=./ai/2026-Q1/prod-metrics.csv \
  --input pipeline_intake=./ai/2026-Q1/proposals/ \
  --input incident_log=./ai/2026-Q1/incidents/ \
  --output-dir ./ai/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for cross-function gating
- **Worst case:** sunset decision may require customer communication + transition plan

## Skill chain

- **Step 1 `ai-use-case-portfolio-author`** — drafts per Gartner AI Use-Case Prism.
- **Step 2 `ai-use-case-portfolio-audit`** — validates per `ai_use_case_portfolio_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-VALUE-001 | Value not measurable | Operator quantifies |
| 2 | QA-RISK-001 | Risk score no rationale | Operator drafts |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.3 — CAIO role profile
- `./annual-ai-strategy.md` — upstream parent
- `../../../skill/ai-use-case-portfolio-{author,audit}/SKILL.md`
