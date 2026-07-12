---
workflow_id: chief-transformation-officer/quarterly-portfolio-review
workflow_version: 1.0.0
purpose: Review the transformation-program portfolio — program health, value realization, governance escalations, portfolio rebalancing.
persona: cuo/chief-transformation-officer
cadence: quarterly
status: shipped

inputs:
  - { name: roadmap,               source: cuo/chief-transformation-officer/annual-transformation-roadmap, format: transformation-roadmap@1 }
  - { name: program_charters,      source: all active per-program-charter@1, format: program-charter@1 (multiple) }
  - { name: program_status,        source: per-program PMO updates, format: markdown briefs }
  - { name: value_metrics,         source: per-program value-realization data, format: csv }

outputs:
  - { name: portfolio_review,      format: transformation-roadmap@1 (quarterly portfolio chapter), recipient: cuo/chief-transformation-officer + cuo/ceo + Board (transformation chapter) }

skill_chain:
  - { step: 1, skill: transformation-roadmap-author, inputs_from: { roadmap: roadmap, program_charters: program_charters, program_status: program_status, value_metrics: value_metrics }, outputs_to: review_draft }
  - { step: 2, skill: transformation-roadmap-audit,  inputs_from: review_draft, outputs_to: portfolio_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "program reaches red status without recovery plan OR portfolio value <60% of plan" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "portfolio rebalancing implies budget redirect" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with portfolio_review hash + green/yellow/red count + value-realization %
  - HITL pause at step 2 on QA-RED-001 (red program no recovery plan) or QA-VALUE-REAL-001 (value tracking missing)
---

# Quarterly portfolio review — `chief-transformation-officer/quarterly-portfolio-review`

Chief Transformation Officer's quarterly portfolio-review per Bain agile-PMO + McKinsey transformation-monitoring framework. Aggregates per-program status into portfolio health + value realization + rebalancing decisions.

## When to invoke

- "Run the Q<n> transformation portfolio review"
- "Transformation status check"
- "Portfolio health review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-transformation-officer/quarterly-portfolio-review \
  --input roadmap=./transformation/2026/roadmap.md \
  --input program_charters=./transformation/programs/charters/ \
  --input program_status=./transformation/programs/2026-Q1/status/ \
  --input value_metrics=./transformation/programs/2026-Q1/value.csv \
  --output-dir ./transformation/portfolio/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for program-lead round-trip
- **Worst case:** red program triggers same-quarter recovery design

## Skill chain

- **Step 1 `transformation-roadmap-author`** — drafts portfolio-review view.
- **Step 2 `transformation-roadmap-audit`** — validates per `transformation_roadmap_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-RED-001 | Red program no recovery plan | Escalate to CEO |
| 2 | QA-VALUE-REAL-001 | Value tracking missing | Operator instruments |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Transformation Officer role profile
- `./annual-transformation-roadmap.md` — upstream parent
- `../../../skill/transformation-roadmap-{author,audit}/SKILL.md`
