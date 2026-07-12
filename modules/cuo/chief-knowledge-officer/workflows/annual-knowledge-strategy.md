---
workflow_id: chief-knowledge-officer/annual-knowledge-strategy
workflow_version: 1.0.0
purpose: Author the annual knowledge strategy — vision, asset roadmap, codification investments, reuse-economics targets, technology stack.
persona: cuo/chief-knowledge-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_strategy,        source: last year's strategy-document@1 (knowledge chapter), format: strategy-document@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }
  - { name: pipeline_history,      source: 4 quarters of knowledge-pipeline@1, format: knowledge-pipeline@1 (4Q) }
  - { name: practice_input,        source: practice leads (per-practice knowledge needs), format: markdown briefs }

outputs:
  - { name: knowledge_strategy,    format: strategy-doc@1, recipient: cuo/chief-knowledge-officer + cuo/ceo + cuo/coo + practice leads + Board (annual KM chapter) }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_strategy: prior_strategy, ceo_priorities: ceo_priorities, pipeline_history: pipeline_history, practice_input: practice_input }, outputs_to: strategy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: strategy_draft, outputs_to: knowledge_strategy }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "strategy proposes IP-asset productization (knowledge → product transition)" }
  - { persona: cuo/chief-financial-officer,            when: "investment plan exceeds budget envelope" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "tech-stack changes affect engineering" }
  - { persona: cuo/chief-marketing-officer,            when: "asset-publishing intersects external thought leadership" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with knowledge_strategy hash + reuse-economics target + tech-stack changes
  - HITL pause at step 2 on QA-KERNEL-001 (Rumelt incomplete) or QA-REUSE-001 (no measurable reuse target)
---

# Annual knowledge strategy — `chief-knowledge-officer/annual-knowledge-strategy`

Chief Knowledge Officer's annual KM strategy. Per Rumelt good-strategy kernel + Davenport Working Knowledge + McKinsey internal KM playbook. For consulting firms (CyberSkill commercial baseline §7 high-ROI), this is THE moat strategy.

## When to invoke

- "Build the 2026 knowledge strategy"
- "Annual KM strategic refresh"
- "Knowledge strategy review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-knowledge-officer/annual-knowledge-strategy \
  --input prior_strategy=./knowledge/2025/strategy.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input pipeline_history=./knowledge/2025/ \
  --input practice_input=./knowledge/2026/practice-briefs/ \
  --output-dir ./knowledge/2026/strategy/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 4-8 weeks for cross-practice + Board review
- **Worst case:** IP-productization shift may require 1-2 year transition

## Skill chain

- **Step 1 `strategy-document-author`** — drafts per Rumelt + Davenport + McKinsey KM.
- **Step 2 `strategy-document-audit`** — validates per `strategy_doc_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-KERNEL-001 | Rumelt incomplete | Operator extends |
| 2 | QA-REUSE-001 | No measurable reuse target | Operator quantifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Knowledge Officer role profile
- `./quarterly-knowledge-pipeline.md` — downstream consumer
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
