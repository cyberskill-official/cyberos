---
workflow_id: chief-revenue-officer/quarterly-revenue-review
workflow_version: 1.0.0
purpose: Author the quarterly revenue review — ARR / NDR / GRR / logo churn / new-biz vs expansion vs renewal mix vs plan.
persona: cuo/chief-revenue-officer
cadence: quarterly
status: shipped

inputs:
  - { name: financials,         source: cuo/chief-financial-officer/quarterly-board-financials, format: monthly-close@1 + forecast@1 }
  - { name: pipeline_history,   source: prior quarter pipeline-report@1 set, format: pipeline-report@1 (13 weeks) }
  - { name: cs_engagements,     source: cs-engagement@1 for all customers,   format: cs-engagement@1 (multiple) }
  - { name: gtm_plan,           source: cuo/chief-sales-officer/annual-gtm-plan,       format: gtm-plan@1 }

outputs:
  - { name: revenue_review,     format: board-deck@1 chapter (revenue), recipient: cuo/cro-revenue + cuo/ceo + Board (chapter) }

skill_chain:
  - { step: 1, skill: board-deck-author, inputs_from: { financials: financials, pipeline_history: pipeline_history, cs_engagements: cs_engagements, gtm_plan: gtm_plan }, outputs_to: review_draft }
  - { step: 2, skill: board-deck-audit,  inputs_from: review_draft, outputs_to: revenue_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "ARR miss > 10% OR NDR < 110% (SaaS target)" }
  - { persona: cuo/chief-financial-officer,         when: "revenue mix shift implies operating-model change" }

consults:
  - { persona: cuo/chief-marketing-officer,         when: "review surfaces demand-gen needing recalibration" }
  - { persona: cuo/chief-customer-officer, when: "expansion / churn delta is CS-driven" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with revenue_review hash + ARR + NDR + GRR + logo-churn
  - HITL pause at step 2 on QA-MIX-001 (new-biz/expansion/renewal mix shifts > 10pts without narrative)
---

# Quarterly revenue review — `chief-revenue-officer/quarterly-revenue-review`

CRO-Revenue's quarterly revenue chapter for the board deck. Combines CFO financials + 13 weeks pipeline history + CS engagement portfolio + GTM plan into ARR/NDR/GRR/logo-churn analysis with mix decomposition (new-biz vs expansion vs renewal). Bessemer / OpenView NDR benchmarks: 110%+ = healthy SaaS; <100% = leaking bucket.

## When to invoke

- "Build the Q<n> revenue review"
- "Revenue chapter for next board"
- "ARR + NDR review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-revenue-officer/quarterly-revenue-review \
  --input financials=./close/2026-Q1/segment.md \
  --input pipeline_history=./sales/2026-Q1/ \
  --input cs_engagements=./customer/2026-Q1/cs/ \
  --input gtm_plan=./gtm/2026/plan.md \
  --output-dir ./revenue/2026-Q1/board-chapter/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 days for CRO + CEO review
- **Worst case:** ARR miss triggers same-quarter re-plan + CEO+board call

## Skill chain

- **Step 1 `board-deck-author`** — drafts revenue chapter per Bessemer State of the Cloud + Winning by Design RevOps.
- **Step 2 `board-deck-audit`** — validates per `board_deck_rubric@1.0` chapter-mode rules.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-MIX-001 | Mix shift > 10pts no narrative | Operator drafts |
| 2 | QA-NDR-001 | NDR < 110% no remediation | Escalate to CCO-Customer |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.2 — CRO-Revenue role profile
- `../../chief-executive-officer/workflows/quarterly-board-update.md` — board-deck consumer
- `../../cco-customer/README.md` — CS peer for NDR remediation
- `../../../skill/board-deck-{author,audit}/SKILL.md`
