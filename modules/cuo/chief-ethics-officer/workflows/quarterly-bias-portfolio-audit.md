---
workflow_id: chief-ethics-officer/quarterly-bias-portfolio-audit
workflow_version: 1.0.0
purpose: Audit the bias-audit portfolio across all production models — disparate-impact aggregate, trend analysis, intervention prioritization.
persona: cuo/chief-ethics-officer
cadence: quarterly
status: shipped

inputs:
  - { name: model_bias_audits,     source: cuo/chief-ai-officer/per-model-bias-audit (all in-production), format: bias-audit@1 (multiple) }
  - { name: protected_attributes,  source: cuo/chief-ethics-officer (in-scope attributes by jurisdiction), format: markdown }
  - { name: prior_portfolio_audit, source: last quarter's bias-audit portfolio summary, format: bias-audit@1 (aggregate) }

outputs:
  - { name: portfolio_bias_audit,  format: bias-audit@1 (portfolio aggregate), recipient: cuo/chief-ethics-officer + cuo/caio + cuo/ceo + cuo/clo-legal }

skill_chain:
  - { step: 1, skill: bias-audit-author, inputs_from: { model_bias_audits: model_bias_audits, protected_attributes: protected_attributes, prior_portfolio_audit: prior_portfolio_audit }, outputs_to: portfolio_draft }
  - { step: 2, skill: bias-audit-audit,  inputs_from: portfolio_draft, outputs_to: portfolio_bias_audit }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "portfolio-level disparate-impact pattern (>1 model) on same attribute" }
  - { persona: cuo/chief-legal-officer,      when: "pattern triggers regulatory disclosure (EEOC / FHA / FCRA)" }

consults:
  - { persona: cuo/chief-ai-officer,           when: "remediation intervention design" }
  - { persona: cuo/chief-data-officer,       when: "training-data bias suspected (data-side root cause)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with portfolio_bias_audit hash + per-attribute aggregate + intervention count
  - HITL pause at step 2 on QA-PATTERN-001 (portfolio-level pattern unflagged)
---

# Quarterly bias portfolio audit — `chief-ethics-officer/quarterly-bias-portfolio-audit`

Chief Ethics Officer's quarterly aggregate review of bias-audit results across all production models. Identifies systemic bias patterns (multiple models, same attribute) that single-model audits miss. Per AI Fairness 360 portfolio analysis + EEOC + FHA portfolio-disparate-impact case law.

## When to invoke

- "Run the Q<n> bias portfolio audit"
- "Aggregate bias review"
- "Cross-model fairness pattern check"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ethics-officer/quarterly-bias-portfolio-audit \
  --input model_bias_audits=./ai/bias-audits/2026-Q1/ \
  --input protected_attributes=./ethics/protected-attributes.md \
  --input prior_portfolio_audit=./ethics/portfolio/2025-Q4/bias.md \
  --output-dir ./ethics/portfolio/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for cross-team review
- **Worst case:** portfolio-pattern triggers cross-model remediation (1-2 quarter)

## Skill chain

- **Step 1 `bias-audit-author`** — augments individual audits with portfolio-level patterns.
- **Step 2 `bias-audit-audit`** — validates per `bias_audit_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-PATTERN-001 | Portfolio pattern unflagged | Operator surfaces |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — Chief Ethics Officer role profile
- `../../chief-ai-officer/workflows/per-model-bias-audit.md` — upstream per-model feeders
- `../../../skill/bias-audit-{author,audit}/SKILL.md`
