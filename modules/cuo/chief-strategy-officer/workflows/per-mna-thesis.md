---
workflow_id: chief-strategy-officer/per-mna-thesis
workflow_version: 1.0.0
purpose: Author an M&A thesis for a target — strategic rationale, synergy hypothesis, integration plan, deal economics, alternatives.
persona: cuo/chief-strategy-officer
cadence: per-event
status: shipped

inputs:
  - { name: target_brief,          source: corporate-development team, format: markdown }
  - { name: strategy_context,      source: cuo/chief-strategy-officer/annual-corporate-strategy, format: strategy-document@1 }
  - { name: financial_diligence,   source: cuo/cfo (preliminary financial review), format: markdown }
  - { name: comparable_deals,      source: prior mergers-and-acquisitions-thesis@1 + market deal database, format: mergers-and-acquisitions-thesis@1 (set) }

outputs:
  - { name: mna_thesis,            format: mna-thesis@1, recipient: cuo/cso-strategy + cuo/ceo + cuo/cfo + cuo/clo-legal + Board (per-deal review) }

skill_chain:
  - { step: 1, skill: mergers-and-acquisitions-thesis-author, inputs_from: { target_brief: target_brief, strategy_context: strategy_context, financial_diligence: financial_diligence, comparable_deals: comparable_deals }, outputs_to: thesis_draft }
  - { step: 2, skill: mergers-and-acquisitions-thesis-audit,  inputs_from: thesis_draft, outputs_to: mna_thesis }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "thesis recommends PROCEED on a deal > 10% of market cap" }
  - { persona: cuo/chief-legal-officer,      when: "deal needs regulatory review (antitrust / CFIUS / etc.)" }

consults:
  - { persona: cuo/chief-financial-officer,            when: "deal economics need full diligence" }
  - { persona: cuo/chief-operating-officer,            when: "integration complexity needs ops review" }
  - { persona: cuo/chief-human-resources-officer,           when: "people-integration risk needs CHRO review" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with mna_thesis hash + deal-size + synergy hypothesis + recommendation
  - HITL pause at step 2 on QA-SYNERGY-001 (synergy claim no underwriting) or QA-INTEGRATION-001 (no integration plan)
---

# Per M&A thesis — `chief-strategy-officer/per-mna-thesis`

CSO-Strategy's per-target M&A thesis workflow per McKinsey / Bain M&A playbook + Bruner Applied M&A + Damodaran valuation framework. Drives the GO/NO-GO recommendation with explicit synergy + integration discipline.

## When to invoke

- "Build M&A thesis for [target]"
- "Acquisition analysis for [company]"
- "Deal thesis"

## How to invoke

```bash
cyberos-cuo run cuo/chief-strategy-officer/per-mna-thesis \
  --input target_brief=./mna/2026-target-acme/brief.md \
  --input strategy_context=./strategy/2026/corporate.md \
  --input financial_diligence=./mna/2026-target-acme/financial.md \
  --input comparable_deals=./mna/prior/ \
  --output-dir ./mna/2026-target-acme/thesis/
```

## Expected duration

- **Happy path:** 16-32 hours runtime + 4-8 weeks for diligence + Board approval
- **Worst case:** deep diligence may extend 3-6 months pre-LOI

## Skill chain

- **Step 1 `mergers-and-acquisitions-thesis-author`** — drafts per McKinsey/Bain M&A playbook + Bruner + Damodaran.
- **Step 2 `mergers-and-acquisitions-thesis-audit`** — validates per `mna_thesis_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SYNERGY-001 | Synergy not underwritten | Operator quantifies |
| 2 | QA-INTEGRATION-001 | No integration plan | Operator drafts |

## Cross-references
- `../../../../modules/cuo/README.md` §5.1 — CSO-Strategy role profile
- `../../chief-executive-officer/workflows/capital-allocation-memo.md` — deal-financing peer
- `../../chief-financial-officer/workflows/annual-strategic-plan.md` — capital-structure peer
- `../../../skill/mna-thesis-{author,audit}/SKILL.md`
