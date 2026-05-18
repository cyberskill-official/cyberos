---
workflow_id: chief-sales-officer/annual-gtm-plan
workflow_version: 1.0.0
purpose: Author the annual go-to-market plan — ICP, segmentation, channel strategy, pricing, sales motion, quota model, enablement.
persona: cuo/chief-sales-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_gtm,          source: last year's gtm-plan@1,                     format: go-to-market-plan@1 }
  - { name: ceo_priorities,     source: cuo/ceo (vision brief),                     format: markdown }
  - { name: prior_pipeline,     source: prior year's pipeline-report@1 set,         format: pipeline-report@1 (4 quarters) }
  - { name: win_loss_corpus,    source: prior-year deal post-mortems + CRM stage-out reasons, format: markdown / csv }

outputs:
  - { name: gtm_plan,           format: gtm-plan@1, recipient: cuo/cso-sales + cuo/ceo + cuo/cmo + cuo/cfo + Board (annual review) }

skill_chain:
  - { step: 1, skill: go-to-market-plan-author, inputs_from: { prior_gtm: prior_gtm, ceo_priorities: ceo_priorities, prior_pipeline: prior_pipeline, win_loss_corpus: win_loss_corpus }, outputs_to: gtm_draft }
  - { step: 2, skill: go-to-market-plan-audit,  inputs_from: gtm_draft, outputs_to: gtm_plan }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "GTM proposes channel-strategy change (sales-led → PLG, or vice versa)" }
  - { persona: cuo/chief-financial-officer,         when: "quota model implies > 20% rep capacity increase" }

consults:
  - { persona: cuo/chief-marketing-officer,         when: "ICP + segmentation reshapes marketing demand-gen" }
  - { persona: cuo/chief-product-officer, when: "pricing-posture change requires packaging revision" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with gtm_plan hash + ICP-count + channel-mix + quota-model summary
  - HITL pause at step 2 on QA-ICP-001 (ICP lacks measurable criteria) or QA-QUOTA-001 (quota model lacks attainment-history basis)
---

# Annual GTM plan — `chief-sales-officer/annual-gtm-plan`

CSO-Sales' annual go-to-market plan. Combines prior GTM + CEO priorities + prior pipeline + win/loss corpus into a refreshed ICP / segmentation / channel-strategy / pricing / sales-motion / quota-model / enablement plan. Board-reviewed annually.

## When to invoke

- "Build the 2026 GTM plan"
- "Annual go-to-market refresh"
- "Refresh ICP and channel strategy"

## How to invoke

```bash
cyberos-cuo run cuo/chief-sales-officer/annual-gtm-plan \
  --input prior_gtm=./gtm/2025/plan.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input prior_pipeline=./sales/2025/ \
  --input win_loss_corpus=./sales/2025/win-loss/ \
  --output-dir ./gtm/2026/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-8 weeks for CMO + CFO + product cross-input + board review
- **Worst case:** channel-strategy change requires 1-quarter rollout + 1-quarter outcomes assessment

## Skill chain

- **Step 1 `go-to-market-plan-author`** — drafts per Winning by Design + MEDDIC + Predictable Revenue + OpenView PLG.
- **Step 2 `go-to-market-plan-audit`** — validates per `gtm_plan_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ICP-001 | ICP lacks measurable criteria | Operator tightens |
| 2 | QA-QUOTA-001 | Quota model lacks attainment basis | Operator anchors to history |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4 — CSO-Sales / CGO role profile
- `../../cmo/README.md` — demand-gen peer
- `../../../skill/gtm-plan-{author,audit}/SKILL.md`
