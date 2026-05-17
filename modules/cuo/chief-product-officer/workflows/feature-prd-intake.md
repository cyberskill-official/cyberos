---
workflow_id: chief-product-officer/feature-prd-intake
workflow_version: 1.0.0
purpose: Author a PRD for a new feature triggered by an opportunity (customer ask / metrics signal / strategy commitment).
persona: cuo/chief-product-officer
cadence: per-event
status: shipped

inputs:
  - { name: opportunity_brief,     source: PM (problem statement + evidence + target outcomes), format: markdown }
  - { name: research_synthesis,    source: discovery research / interviews, format: markdown }
  - { name: design_explorations,   source: design team (sketches / prototypes), format: markdown / image references }

outputs:
  - { name: prd,                   format: prd@1, recipient: cuo/cpo-product + cuo/cto + design + engineering }

skill_chain:
  - { step: 1, skill: prd-author, inputs_from: { opportunity_brief: opportunity_brief, research_synthesis: research_synthesis, design_explorations: design_explorations }, outputs_to: prd_draft }
  - { step: 2, skill: prd-audit,  inputs_from: prd_draft, outputs_to: prd }

escalates_to:
  - { persona: cuo/chief-product-officer,    when: "PRD audit fires repeated QA-OUTCOME-001 — outcome not measurable" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "PRD touches platform or architectural boundaries" }
  - { persona: cuo/chief-privacy-officer,    when: "PRD touches personal data — pia workflow follows" }
  - { persona: cuo/chief-ai-officer,           when: "PRD includes AI/ML features" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with prd hash + estimated-engineering-size + dependencies
  - HITL pause at step 2 on QA-OUTCOME-001 (outcome not measurable) or QA-OPPORTUNITY-001 (no opportunity-tree linkage)
---

# Feature PRD intake — `chief-product-officer/feature-prd-intake`

CPO-Product's per-feature PRD authoring workflow. Triggered when an opportunity meets the bar (customer ask × metrics signal × strategy commitment). Per Marty Cagan PRD framework + Reforge product-discovery. Feeds CTO's `architect-new-system` workflow downstream.

## When to invoke

- "Write the PRD for [feature]"
- "PRD intake for [opportunity]"
- "Build a PRD"

## How to invoke

```bash
cyberos-cuo run cuo/chief-product-officer/feature-prd-intake \
  --input opportunity_brief=./opportunities/2026-acme-feature/brief.md \
  --input research_synthesis=./research/2026-acme-feature/synthesis.md \
  --input design_explorations=./design/2026-acme-feature/ \
  --output-dir ./prds/2026-acme-feature/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for design + engineering review
- **Worst case:** opportunity-tree gap triggers discovery cycle (1-2 quarter)

## Skill chain

- **Step 1 `prd-author`** — drafts per Cagan PRD framework + Reforge discovery.
- **Step 2 `prd-audit`** — validates per `prd_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-OUTCOME-001 | Outcome not measurable | Operator quantifies |
| 2 | QA-OPPORTUNITY-001 | No opportunity-tree linkage | Operator discovers |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.3 — CPO-Product role profile
- `../../chief-technology-officer/workflows/architect-new-system.md` — downstream consumer (PRD → SRS → ADR chain)
- `../../chief-privacy-officer/workflows/privacy-impact-assessment.md` — peer (PIA when personal data touched)
- `../../../skill/prd-{author,audit}/SKILL.md`
