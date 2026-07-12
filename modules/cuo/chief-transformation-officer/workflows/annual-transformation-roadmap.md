---
workflow_id: chief-transformation-officer/annual-transformation-roadmap
workflow_version: 1.0.0
purpose: Author the annual transformation roadmap — vision, value streams, programs, milestones, governance, change-impact.
persona: cuo/chief-transformation-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_roadmap,         source: last year's transformation-roadmap@1, format: transformation-roadmap@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }
  - { name: operating_model,       source: cuo/chief-operating-officer/annual-operating-model, format: operating-model@1 }
  - { name: prior_change_plans,    source: prior change-management-plan@1 set, format: change-management-plan@1 (multiple) }

outputs:
  - { name: transformation_roadmap, format: transformation-roadmap@1, recipient: cuo/chief-transformation-officer + cuo/ceo + cuo/coo + cuo/chro + Board (annual transformation chapter) }

skill_chain:
  - { step: 1, skill: transformation-roadmap-author, inputs_from: { prior_roadmap: prior_roadmap, ceo_priorities: ceo_priorities, operating_model: operating_model, prior_change_plans: prior_change_plans }, outputs_to: roadmap_draft }
  - { step: 2, skill: transformation-roadmap-audit,  inputs_from: roadmap_draft, outputs_to: transformation_roadmap }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "roadmap proposes program > $1M OR multi-function reorg" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "budget envelope needs alignment" }
  - { persona: cuo/chief-operating-officer,            when: "operating-model intersect" }
  - { persona: cuo/chief-human-resources-officer,           when: "change-impact on workforce" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with transformation_roadmap hash + program count + milestone count
  - HITL pause at step 2 on QA-VALUE-001 (program no value hypothesis) or QA-CHANGE-001 (no change-impact analysis)
---

# Annual transformation roadmap — `chief-transformation-officer/annual-transformation-roadmap`

Chief Transformation Officer's annual roadmap per McKinsey 7S + Kotter 8-step + Bain transformation framework. Combines prior roadmap + CEO priorities + operating-model + change-plan history into the year's vision / value streams / programs / milestones / governance.

## When to invoke

- "Build the 2026 transformation roadmap"
- "Annual transformation strategic refresh"
- "Refresh transformation priorities"

## How to invoke

```bash
cyberos-cuo run cuo/chief-transformation-officer/annual-transformation-roadmap \
  --input prior_roadmap=./transformation/2025/roadmap.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input operating_model=./ops/2026/operating-model.md \
  --input prior_change_plans=./transformation/2025/change-plans/ \
  --output-dir ./transformation/2026/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 8-12 weeks for cross-function + Board review
- **Worst case:** multi-function reorg extends to 1-2 quarter

## Skill chain

- **Step 1 `transformation-roadmap-author`** — drafts per McKinsey 7S + Kotter 8-step + Bain.
- **Step 2 `transformation-roadmap-audit`** — validates per `transformation_roadmap_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-VALUE-001 | Program no value hypothesis | Operator drafts |
| 2 | QA-CHANGE-001 | No change-impact analysis | Operator extends |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Transformation Officer role profile
- `../../chief-operating-officer/workflows/annual-operating-model.md` — upstream peer
- `../../../skill/transformation-roadmap-{author,audit}/SKILL.md`
