---
workflow_id: chief-financial-officer/annual-strategic-plan
workflow_version: 1.0.0
purpose: Author the CFO's annual strategic-finance plan — capital structure, M&A capacity, hurdle rates, capital-allocation framework, investor-narrative arc.
persona: cuo/chief-financial-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,     source: last year's strategy-document@1 (CFO chapter), format: strategy-document@1 }
  - { name: ceo_priorities,     source: cuo/ceo (vision brief),                   format: markdown }
  - { name: prior_forecasts,    source: cuo/chief-financial-officer/quarterly-forecast (4Q),          format: forecast@1 (4 quarters) }
  - { name: prior_cap_alloc,    source: cuo/chief-executive-officer/capital-allocation-memo set,      format: capital-allocation-memo@1 (multiple) }

outputs:
  - { name: strategic_plan,     format: strategy-doc@1, recipient: cuo/cfo + cuo/ceo + Board (annual finance strategy chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, prior_forecasts: prior_forecasts, prior_cap_alloc: prior_cap_alloc }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: strategic_plan }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "strategy proposes capital-structure change (debt raise / equity raise / buyback)" }

consults:
  - { persona: cuo/chief-strategy-officer, when: "M&A capacity assumption needs strategic alignment" }
  - { persona: cuo/chief-legal-officer,    when: "financing structure needs legal review (Reg D / Reg S / covenant)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with strategic_plan hash + hurdle-rate + M&A-capacity-$
  - HITL pause at step 2 on QA-KERNEL-001 (Rumelt diagnosis/policy/action incomplete) or QA-HURDLE-001 (hurdle rate not benchmarked)
---

# Annual strategic plan (CFO) — `chief-financial-officer/annual-strategic-plan`

CFO's annual strategic-finance plan. Combines prior strategy + CEO priorities + 4Q forecasts + prior cap-alloc memos into capital-structure / M&A capacity / hurdle rates / capital-allocation framework / investor-narrative arc. Per Will Thorndike "outsider" capital-allocation framework + Rumelt good-strategy kernel.

## When to invoke

- "Build the 2026 CFO strategic plan"
- "Annual finance strategy"
- "Capital-allocation framework refresh"

## How to invoke

```bash
cyberos-cuo run cuo/chief-financial-officer/annual-strategic-plan \
  --input prior_strategy=./finance/2025/strategy.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input prior_forecasts=./forecast/2025/ \
  --input prior_cap_alloc=./cap-alloc/2025/ \
  --output-dir ./finance/2026/strategy/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-6 weeks for cross-function inputs + Board approval
- **Worst case:** capital-structure change may require 1-2 quarter execution

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per Rumelt + Roger Martin Playing-to-Win + Thorndike capital-allocation.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KERNEL-001 | Rumelt kernel incomplete | Operator extends |
| 2 | QA-HURDLE-001 | Hurdle rate not benchmarked | Operator benchmarks against industry WACC |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.2 — CFO role profile
- `../../chief-executive-officer/workflows/capital-allocation-memo.md` — per-event peer (single decision; this annual sets framework)
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
