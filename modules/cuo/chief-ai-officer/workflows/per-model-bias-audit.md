---
workflow_id: chief-ai-officer/per-model-bias-audit
workflow_version: 1.0.0
purpose: Audit a model for bias across protected attributes — disparate impact analysis, fairness metrics, mitigation recommendations.
persona: cuo/chief-ai-officer
cadence: per-event
status: shipped

inputs:
  - { name: model_brief,           source: ML team, format: markdown }
  - { name: training_data,         source: data team (demographic-tagged sample), format: csv (anonymized aggregates) }
  - { name: production_predictions, source: deployed model's prediction log (or sample), format: csv (anonymized aggregates) }
  - { name: protected_attributes,  source: cuo/chief-ethics-officer (in-scope attributes by jurisdiction), format: markdown }

outputs:
  - { name: bias_audit,            format: bias-audit@1, recipient: cuo/caio + cuo/chief-ethics-officer + cuo/cpo-privacy + model owner }

skill_chain:
  - { step: 1, skill: bias-audit-author, inputs_from: { model_brief: model_brief, training_data: training_data, production_predictions: production_predictions, protected_attributes: protected_attributes }, outputs_to: audit_draft }
  - { step: 2, skill: bias-audit-audit,  inputs_from: audit_draft, outputs_to: bias_audit }

escalates_to:
  - { persona: cuo/chief-ethics-officer, when: "disparate impact ratio < 0.8 (4/5ths rule) on any protected attribute" }
  - { persona: cuo/chief-legal-officer,      when: "audit findings may trigger regulatory disclosure (EEOC / FHA / FCRA)" }

consults:
  - { persona: cuo/chief-privacy-officer,    when: "audit requires processing additional personal data" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with bias_audit hash + per-attribute disparate-impact ratio + mitigation count
  - HITL pause at step 2 on QA-FAIRNESS-001 (4/5ths violation) or QA-COVERAGE-001 (attributes incomplete)
---

# Per model bias audit — `chief-ai-officer/per-model-bias-audit`

CAIO's per-model bias audit workflow. Per AI Fairness 360 (IBM) + Fairlearn (Microsoft) + EEOC 4/5ths rule + ECOA / FCRA / FHA fairness standards. Triggered per major model release AND per quarter for production models with potential fairness concerns.

## When to invoke

- "Run bias audit on [model]"
- "Fairness audit for [model name]"
- "Disparate impact analysis"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ai-officer/per-model-bias-audit \
  --input model_brief=./models/2026-recommender/brief.md \
  --input training_data=./models/2026-recommender/data-demographics.csv \
  --input production_predictions=./models/2026-recommender/predictions.csv \
  --input protected_attributes=./ethics/protected-attributes.md \
  --output-dir ./models/2026-recommender/bias-audit/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for review
- **Worst case:** disparate impact violation triggers retraining + remediation (1-2 quarter)

## Skill chain

- **Step 1 `bias-audit-author`** — drafts per AI Fairness 360 + Fairlearn + EEOC 4/5ths.
- **Step 2 `bias-audit-audit`** — validates per `bias_audit_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-FAIRNESS-001 | 4/5ths violation | Escalate to chief-ethics-officer |
| 2 | QA-COVERAGE-001 | Attributes incomplete | Operator extends |

## Cross-references
- `../../../../modules/cuo/README.md` §5.3 — CAIO role profile
- `./per-model-card-release.md` — peer (feeds model-card bias section)
- `../../chief-ethics-officer/README.md` — ethics peer
- `../../../skill/bias-audit-{author,audit}/SKILL.md`
