---
workflow_id: chief-ai-officer/annual-ai-strategy
workflow_version: 1.0.0
purpose: Author the annual AI strategy — use-case portfolio, build/buy/partner, model governance, MLOps maturity, EU AI Act risk classification, AI OKRs.
persona: cuo/chief-ai-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's ai-strategy@1, format: ai-strategy@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }
  - { name: use_case_portfolio,    source: cuo/caio's use-case register, format: ai-use-case-portfolio@1 }
  - { name: data_strategy,         source: cuo/chief-data-officer/annual-data-strategy, format: data-strategy@1 }

outputs:
  - { name: ai_strategy,           format: ai-strategy@1, recipient: cuo/caio + cuo/ceo + cuo/cto + cuo/cdo-data + cuo/chief-ethics-officer + Board (annual AI chapter) }

skill_chain:
  - { step: 1, skill: ai-strategy-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, use_case_portfolio: use_case_portfolio, data_strategy: data_strategy }, outputs_to: strategy_draft }
  - { step: 2, skill: ai-strategy-audit,  inputs_from: strategy_draft, outputs_to: ai_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes prohibited-tier use case (EU AI Act) OR high-risk-tier use case without full DPIA + conformity assessment" }
  - { persona: cuo/chief-legal-officer,      when: "EU AI Act conformity-assessment filings needed" }

consults:
  - { persona: cuo/chief-ethics-officer, when: "use case poses fairness / explainability concerns" }
  - { persona: cuo/chief-privacy-officer,    when: "training data includes personal data — PIA required" }
  - { persona: cuo/chief-data-officer,       when: "data-foundation dependencies for use cases" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with ai_strategy hash + use-case count by risk tier + MLOps maturity level
  - HITL pause at step 2 on QA-RISK-001 (high-risk use case without conformity plan) or QA-MATURITY-001 (MLOps level overstated)
---

# Annual AI strategy — `chief-ai-officer/annual-ai-strategy`

CAIO's annual AI strategy. Per NIST AI RMF 1.0 + EU AI Act Regulation (EU) 2024/1689 + ISO/IEC 42001:2023 + Stanford HAI policy framework + Google MLOps maturity + Anthropic Responsible Scaling Policy as industry reference. Critical for EU-market companies (CyberSkill targets global) — wrong risk classification triggers regulator fines up to 7% of global revenue.

## When to invoke

- "Build the 2026 AI strategy"
- "Annual AI strategic refresh"
- "AI portfolio + governance review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ai-officer/annual-ai-strategy \
  --input prior_strategy=./ai/2025/strategy.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input use_case_portfolio=./ai/2026/use-cases.md \
  --input data_strategy=./data/2026/strategy/data-strategy.md \
  --output-dir ./ai/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function + Board review
- **Worst case:** high-risk use case may add 1-2 quarter for conformity assessment

## Skill chain

- **Step 1 `ai-strategy-author`** — drafts per NIST AI RMF + EU AI Act + ISO/IEC 42001 + Google MLOps.
- **Step 2 `ai-strategy-audit`** — validates per `ai_strategy_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-RISK-001 | High-risk no conformity plan | Escalate to CEO + CLO-Legal |
| 2 | QA-MATURITY-001 | MLOps level overstated | Operator self-corrects |

## Cross-references
- `../../../../modules/cuo/README.md` §5.3 — CAIO role profile
- `../../chief-data-officer/workflows/annual-data-strategy.md` — peer upstream
- `../../chief-ethics-officer/README.md` — ethics peer
- `../../../skill/ai-strategy-{author,audit}/SKILL.md`
