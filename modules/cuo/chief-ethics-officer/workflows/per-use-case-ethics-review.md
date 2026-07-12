---
workflow_id: chief-ethics-officer/per-use-case-ethics-review
workflow_version: 1.0.0
purpose: Conduct ethics review for a proposed AI/data use case — stakeholder analysis, harm assessment, fairness check, mitigations, recommendation.
persona: cuo/chief-ethics-officer
cadence: per-event
status: shipped

inputs:
  - { name: use_case_brief,        source: cuo/caio (use-case-portfolio proposal), format: ai-use-case-portfolio@1 entry }
  - { name: data_flow_diagram,     source: engineering team, format: markdown }
  - { name: stakeholder_map,       source: requestor, format: markdown }
  - { name: prior_reviews,         source: similar prior ethics reviews, format: ethics-review@1 (set) }

outputs:
  - { name: ethics_review,         format: ethics-review@1, recipient: cuo/chief-ethics-officer + cuo/caio + cuo/cpo-privacy + cuo/clo-legal }

skill_chain:
  - { step: 1, skill: ethics-review-author, inputs_from: { use_case_brief: use_case_brief, data_flow_diagram: data_flow_diagram, stakeholder_map: stakeholder_map, prior_reviews: prior_reviews }, outputs_to: review_draft }
  - { step: 2, skill: ethics-review-audit,  inputs_from: review_draft, outputs_to: ethics_review }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "review recommends DECLINE on a strategic use case" }
  - { persona: cuo/chief-legal-officer,      when: "review surfaces regulatory exposure (EU AI Act high-risk, FCRA, ECOA)" }

consults:
  - { persona: cuo/chief-ai-officer,           when: "use-case modification proposed" }
  - { persona: cuo/chief-privacy-officer,    when: "personal-data PIA needed" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with ethics_review hash + recommendation (PROCEED / MODIFY / DECLINE) + stakeholder count
  - HITL pause at step 2 on QA-STAKEHOLDER-001 (affected stakeholder missed) or QA-HARM-001 (harm vector unexamined)
---

# Per use-case ethics review — `chief-ethics-officer/per-use-case-ethics-review`

Chief Ethics Officer's per-use-case ethics review. Per IEEE Ethically Aligned Design + Markkula Center for Applied Ethics + Anthropic Acceptable Use Policy as industry reference. Mandatory gate for use cases entering production per `chief-ai-officer/quarterly-use-case-portfolio-review`.

## When to invoke

- "Ethics review for [use case]"
- "Review the ethics of [proposed AI/data use]"
- "Ethics gate for [project]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-ethics-officer/per-use-case-ethics-review \
  --input use_case_brief=./ai/use-cases/2026-acme-recommender/brief.md \
  --input data_flow_diagram=./engineering/dfd/2026-acme.md \
  --input stakeholder_map=./ethics/use-cases/2026-acme-recommender/stakeholders.md \
  --input prior_reviews=./ethics/prior/ \
  --output-dir ./ethics/use-cases/2026-acme-recommender/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 1-2 weeks for stakeholder consultation
- **Worst case:** DECLINE recommendation requires CEO arbitration

## Skill chain

- **Step 1 `ethics-review-author`** — drafts per IEEE EAD + Markkula Center.
- **Step 2 `ethics-review-audit`** — validates per `ethics_review_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-STAKEHOLDER-001 | Stakeholder missed | Operator extends |
| 2 | QA-HARM-001 | Harm vector unexamined | Operator extends |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — Chief Ethics Officer role profile
- `../../chief-ai-officer/workflows/quarterly-use-case-portfolio-review.md` — upstream feeder
- `../../../skill/ethics-review-{author,audit}/SKILL.md`
