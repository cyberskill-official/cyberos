---
workflow_id: chief-customer-officer/quarterly-cab-cycle
workflow_version: 1.0.0
purpose: Run the quarterly Customer Advisory Board cycle — agenda, attendee curation, synthesis, follow-through commitments.
persona: cuo/chief-customer-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_cab,             source: last quarter's customer-advisory-board@1, format: customer-advisory-board@1 }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }
  - { name: product_roadmap,       source: cuo/chief-product-officer/quarterly-roadmap-planning, format: product-roadmap@1 }
  - { name: attendee_signals,      source: CSM input + relationship-strength scoring, format: markdown }

outputs:
  - { name: cab_synthesis,         format: customer-advisory-board@1, recipient: cuo/cco-customer + cuo/ceo + cuo/cpo-product + cuo/cmo + CAB attendees }

skill_chain:
  - { step: 1, skill: customer-advisory-board-author, inputs_from: { prior_cab: prior_cab, ceo_priorities: ceo_priorities, product_roadmap: product_roadmap, attendee_signals: attendee_signals }, outputs_to: cab_draft }
  - { step: 2, skill: customer-advisory-board-audit,  inputs_from: cab_draft, outputs_to: cab_synthesis }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "CAB surfaces critical strategy gap OR customer churn intent" }

consults:
  - { persona: cuo/chief-product-officer,    when: "CAB feedback warrants roadmap update" }
  - { persona: cuo/chief-communications-officer, when: "CAB outcomes warrant public-positioning shift" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with cab_synthesis hash + attendee count + commitment count
  - HITL pause at step 2 on QA-COMMITMENT-001 (no follow-through commitments captured) or QA-DIVERSITY-001 (single-segment attendee bias)
---

# Quarterly CAB cycle — `chief-customer-officer/quarterly-cab-cycle`

CCO-Customer's quarterly Customer Advisory Board cycle. Per Salesforce CAB + Gartner CAB best practices. Combines prior CAB + CEO priorities + product roadmap + attendee signals into agenda + curated attendee list + synthesis + commitments.

## When to invoke

- "Run the Q<n> CAB cycle"
- "Customer advisory board prep"
- "CAB synthesis for [meeting]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-customer-officer/quarterly-cab-cycle \
  --input prior_cab=./customer/cab/2026-Q1/synthesis.md \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input product_roadmap=./product/2026-Q2/roadmap.md \
  --input attendee_signals=./customer/cab/2026-Q2/signals.md \
  --output-dir ./customer/cab/2026-Q2/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-6 weeks for attendee outreach + agenda design + post-meeting synthesis
- **Worst case:** critical-gap finding may trigger strategic-plan revision

## Skill chain

- **Step 1 `customer-advisory-board-author`** — drafts per Salesforce + Gartner CAB.
- **Step 2 `customer-advisory-board-audit`** — validates per `customer_advisory_board_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-COMMITMENT-001 | No follow-through | Operator captures |
| 2 | QA-DIVERSITY-001 | Single-segment bias | Operator extends attendees |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CCO-Customer role profile
- `../../chief-product-officer/workflows/quarterly-roadmap-planning.md` — peer (CAB feedback feeds roadmap)
- `../../../skill/customer-advisory-board-{author,audit}/SKILL.md`
