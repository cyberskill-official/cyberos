---
workflow_id: chief-customer-officer/quarterly-churn-collaboration
workflow_version: 1.0.0
purpose: Partner with CRO-Revenue on quarterly churn analysis — CS-side root-cause depth, win-back execution, leading-indicator integration into health scoring.
persona: cuo/chief-customer-officer
cadence: quarterly
status: shipped
pattern: persona_pair
peer_persona: chief-revenue-officer
peer_workflow: quarterly-churn-analysis
shared_artefact: churn-cohort-analysis
handoff_step: 3

inputs:
  - { name: cro_churn_analysis,    source: cuo/chief-revenue-officer/quarterly-churn-analysis, format: churn-analysis@1 }
  - { name: cs_exit_corpus,        source: CSM exit-interview notes per churned account, format: markdown }
  - { name: leading_indicators,    source: prior customer-health-review@1 (at-risk list 2 quarters prior), format: customer-health-review@1 }

outputs:
  - { name: cs_side_churn,         format: churn-analysis@1 (CS-augmented), recipient: cuo/cco-customer + cuo/cro-revenue + cuo/cpo-product + cuo/chro }

skill_chain:
  - { step: 1, skill: churn-analysis-author, inputs_from: { cro_churn_analysis: cro_churn_analysis, cs_exit_corpus: cs_exit_corpus, leading_indicators: leading_indicators }, outputs_to: cs_churn_draft }
  - { step: 2, skill: churn-analysis-audit,  inputs_from: cs_churn_draft, outputs_to: cs_side_churn }

escalates_to:
  - { persona: cuo/chief-product-officer,    when: "CS root-cause shows product-driven > 40% of churn" }
  - { persona: cuo/chief-human-resources-officer,           when: "CSM-execution issue surfaces (low book-coverage / low engagement frequency)" }

consults:
  - { persona: cuo/chief-revenue-officer,    when: "win-back program needs revenue-side design" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with cs_side_churn hash + CS-attributable-% + leading-indicator-accuracy %
  - HITL pause at step 2 on QA-ATTRIBUTION-001 (CS vs product vs external attribution muddy)
---

# Quarterly churn collaboration — `chief-customer-officer/quarterly-churn-collaboration`

CCO-Customer's CS-side partner workflow to CRO-Revenue's `quarterly-churn-analysis`. Augments the revenue-side churn analysis with CS-execution root-cause depth, win-back execution accountability, and leading-indicator validation against prior at-risk lists.

## When to invoke

- "Partner with CRO on Q<n> churn analysis"
- "CS-side churn deep-dive"
- "Validate at-risk leading indicators"

## How to invoke

```bash
cyberos-cuo run cuo/chief-customer-officer/quarterly-churn-collaboration \
  --input cro_churn_analysis=./churn/2026-Q1/cro-analysis.md \
  --input cs_exit_corpus=./cs/2026-Q1/exit-interviews/ \
  --input leading_indicators=./customer/2025-Q3/health.md \
  --output-dir ./customer/2026-Q1/churn-cs/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for CSM input
- **Worst case:** CSM-execution finding triggers 1-quarter intervention

## Skill chain

- **Step 1 `churn-analysis-author`** — augments CRO-side analysis with CS-execution depth.
- **Step 2 `churn-analysis-audit`** — validates per `churn_analysis_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ATTRIBUTION-001 | Attribution muddy | Operator clarifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CCO-Customer role profile
- `../../chief-revenue-officer/workflows/quarterly-churn-analysis.md` — upstream parent
- `../../../skill/churn-analysis-{author,audit}/SKILL.md`
