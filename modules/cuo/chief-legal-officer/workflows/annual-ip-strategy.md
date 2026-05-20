---
workflow_id: chief-legal-officer/annual-ip-strategy
workflow_version: 1.0.0
purpose: Author the annual IP portfolio strategy — patent filings, trademark portfolio, trade-secret protection, defensive vs offensive posture, FTO assessment, enforcement budget.
persona: cuo/chief-legal-officer
cadence: annual
status: shipped

inputs:
  - { name: portfolio_inventory, source: IP register (USPTO / WIPO / firm IP-mgmt tool), format: csv export }
  - { name: prior_strategy,      source: last year's ip-strategy@1,                     format: intellectual-property-strategy@1 }
  - { name: product_roadmap,     source: cuo/cpo-product or cuo/cto,                    format: product-roadmap@1 }
  - { name: budget_envelope,     source: cuo/cfo (annual budget IP line),               format: budget@1 chapter }

outputs:
  - { name: ip_strategy,         format: ip-strategy@1, recipient: cuo/clo-legal + cuo/ceo + Board (annual review) }

skill_chain:
  - { step: 1, skill: intellectual-property-strategy-author, inputs_from: { portfolio_inventory: portfolio_inventory, prior_strategy: prior_strategy, product_roadmap: product_roadmap, budget_envelope: budget_envelope }, outputs_to: strategy_draft }
  - { step: 2, skill: intellectual-property-strategy-audit,  inputs_from: strategy_draft, outputs_to: ip_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,           when: "strategy proposes offensive posture (enforcement litigation) with budget > $500K" }
  - { persona: cuo/chief-financial-officer,           when: "filing plan exceeds annual IP budget envelope" }

consults:
  - { persona: cuo/chief-technology-officer,           when: "patent filings derive from R&D — inventor-attribution + tech disclosure quality matters" }
  - { persona: cuo/chief-product-officer,   when: "trademark portfolio touches active brand campaigns" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with ip_strategy hash + per-portfolio counts (patents / trademarks / trade-secrets)
  - HITL pause at step 2 on QA-FTO-001 (FTO assessment missing for new products) or QA-BUDGET-001 (filing plan exceeds envelope)
---

# Annual IP strategy — `chief-legal-officer/annual-ip-strategy`

CLO-Legal's annual IP portfolio strategy. Inventories existing portfolio, layers new product-roadmap → filings plan, decides defensive vs offensive posture per portfolio segment, runs FTO assessment on new products, allocates enforcement budget. Output reviewed by Board annually.

## When to invoke

- "Build the 2026 IP strategy"
- "Refresh the annual IP portfolio plan"
- "Annual IP review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-legal-officer/annual-ip-strategy \
  --input portfolio_inventory=./ip/2026/inventory.csv \
  --input prior_strategy=./ip/2025/strategy.md \
  --input product_roadmap=./product/2026-roadmap.md \
  --input budget_envelope=./budget/2026/ip-chapter.md \
  --output-dir ./ip/2026/strategy/
```

## Expected duration

- **Happy path:** 3-6 hours runtime + 4-8 weeks for inventor interviews + FTO opinions + outside-counsel review
- **Worst case:** FTO assessment uncovers blocking patents; product roadmap re-cut adds 1-2 quarters

## Skill chain

- **Step 1 `intellectual-property-strategy-author`** — drafts per WIPO + USPTO MPEP structure: inventory / posture / patent plan / trademark plan / trade-secret controls / FTO / budget / risks.
- **Step 2 `intellectual-property-strategy-audit`** — validates per `ip_strategy_rubric@1.0` (FM + SEC + QA-FTO-001 + QA-BUDGET-001 + QA-POSTURE-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-FTO-001 | FTO assessment missing for new product | Operator commissions FTO opinion |
| 2 | QA-BUDGET-001 | Filing plan exceeds budget | Escalate to CFO |
| 2 | QA-POSTURE-001 | Offensive posture without enforcement rationale | Operator documents rationale or downshifts to defensive |

## Cross-references
- `../README.md` §5 (Strategic) — "IP portfolio strategy"
- `../../../../modules/cuo/README.md` §5.2
- `../../../skill/ip-strategy-{author,audit}/SKILL.md`
- `../../../skill/product-roadmap-{author,audit}/SKILL.md` — upstream input
