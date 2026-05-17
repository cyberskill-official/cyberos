---
workflow_id: chief-revenue-officer/annual-revenue-architecture
workflow_version: 1.0.0
purpose: Author the annual revenue architecture — new-biz motion + expansion motion + renewal motion + churn-prevention motion + cross-handoffs.
persona: cuo/chief-revenue-officer
cadence: annual
status: shipped

inputs:
  - { name: gtm_plan,           source: cuo/chief-sales-officer/annual-gtm-plan,                  format: gtm-plan@1 }
  - { name: prior_churn,        source: cuo/chief-revenue-officer/quarterly-churn-analysis (4Q),  format: churn-analysis@1 (4 quarters) }
  - { name: nps_history,        source: cuo/chief-sales-officer/quarterly-nps-program (4Q),       format: nps-program@1 (4 quarters) }
  - { name: ceo_priorities,     source: cuo/ceo (vision brief),                         format: markdown }

outputs:
  - { name: revenue_architecture, format: strategy-doc@1, recipient: cuo/cro-revenue + cuo/ceo + cuo/cso-sales + cuo/cco-customer + cuo/cmo + Board (annual) }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { gtm_plan: gtm_plan, prior_churn: prior_churn, nps_history: nps_history, ceo_priorities: ceo_priorities }, outputs_to: arch_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: arch_draft, outputs_to: revenue_architecture }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "architecture proposes motion-shift (e.g. add PLG + product-qualified leads layer)" }

consults:
  - { persona: cuo/chief-marketing-officer,         when: "architecture reshapes demand-gen" }
  - { persona: cuo/chief-product-officer, when: "architecture implies packaging / pricing change" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with revenue_architecture hash + motion count + cross-handoff count
  - HITL pause at step 2 on QA-HANDOFF-001 (motion-to-motion handoff missing owner)
---

# Annual revenue architecture — `chief-revenue-officer/annual-revenue-architecture`

CRO-Revenue's annual full-funnel revenue architecture. Combines GTM plan + 4Q churn + 4Q NPS + CEO priorities into the integrated new-biz / expansion / renewal / churn-prevention motion design with cross-handoffs. Per Winning by Design revenue-architecture framework + Rumelt good-strategy kernel.

## When to invoke

- "Build the 2026 revenue architecture"
- "Annual revenue motion design"
- "Integrated revenue strategy"

## How to invoke

```bash
cyberos-cuo run cuo/chief-revenue-officer/annual-revenue-architecture \
  --input gtm_plan=./gtm/2026/plan.md \
  --input prior_churn=./churn/2025/ \
  --input nps_history=./customer/2025/nps/ \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./revenue/2026/architecture/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-8 weeks for cross-function alignment + Board review
- **Worst case:** motion-shift may require 1-2 quarter pilot before full rollout

## Skill chain

- **Step 1 `strategy-doc-author`** — drafts per Rumelt good-strategy kernel + Winning by Design revenue-architecture.
- **Step 2 `strategy-doc-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-HANDOFF-001 | Motion handoff no owner | Operator assigns |
| 2 | QA-KERNEL-001 | Rumelt diagnosis/policy/action incomplete | Operator extends |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.2 — CRO-Revenue role profile
- `../../chief-sales-officer/workflows/annual-gtm-plan.md` — upstream feeder
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
