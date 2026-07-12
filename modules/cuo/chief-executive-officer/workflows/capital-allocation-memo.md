---
workflow_id: chief-executive-officer/capital-allocation-memo
workflow_version: 1.0.0
purpose: Author a capital-allocation memo for the board — investment thesis, capital deployed, expected return, alternatives considered.
persona: cuo/chief-executive-officer
cadence: per-event
status: shipped

inputs:
  - { name: proposal_brief,     source: workflow-caller,                                 format: markdown (one-pager) }
  - { name: financials_context, source: cuo/cfo (latest cash position + runway),         format: monthly-close@1 + forecast@1 chapters }
  - { name: alternatives,       source: workflow-caller (the 2-3 alternative deployments considered), format: markdown }

outputs:
  - { name: cap_alloc_memo,     format: cap-alloc-memo@1, recipient: Board of Directors }

skill_chain:
  - { step: 1, skill: capital-allocation-memo-author, inputs_from: { proposal_brief: proposal_brief, financials_context: financials_context, alternatives: alternatives }, outputs_to: memo_draft }
  - { step: 2, skill: capital-allocation-memo-audit,  inputs_from: memo_draft, outputs_to: cap_alloc_memo }

escalates_to:
  - { persona: cuo/chief-financial-officer,         when: "cap-alloc amount > 10% of quarterly free cash flow OR triggers covenant in debt agreement" }
  - { persona: cuo/chief-legal-officer,   when: "cap-alloc involves M&A / related-party / regulated jurisdiction" }

consults:
  - { persona: cuo/chief-strategy-officer, when: "cap-alloc is for strategic acquisition or new market entry" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with cap_alloc_memo hash + amount + ROI assumption
  - HITL pause at step 2 if QA-ALTS-001 fires (fewer than 2 alternatives considered)
---

# Capital allocation memo — `chief-executive-officer/capital-allocation-memo`

CEO-owned workflow for documenting a non-trivial capital-allocation decision (M&A, large software purchase, share buyback, dividend, large hire wave). Per Warren Buffett / Will Thorndike "outsider" capital-allocation framework: every memo must consider 2-3 alternatives with explicit IRR assumptions.

## When to invoke

- "Write a cap-alloc memo for [decision]"
- "Document the M&A capital plan for [target]"
- "Memo for the board on the [decision] capital deployment"

## How to invoke

```bash
cyberos-cuo run cuo/chief-executive-officer/capital-allocation-memo \
  --input proposal_brief=./cap-alloc/2026-q2-acme-acquisition/proposal.md \
  --input financials_context=./engagements/2026-04/monthly-close.md \
  --input alternatives=./cap-alloc/2026-q2-acme-acquisition/alternatives.md \
  --output-dir ./cap-alloc/2026-q2-acme-acquisition/memo/
```

## Expected duration

- **Happy path:** 30-60 min runtime + 1-2 business days CFO review
- **Worst case:** if memo triggers debt-covenant check, CLO-Legal review may add 1 week

## Skill chain

- **Step 1 `capital-allocation-memo-author`** — drafts per Thorndike / Buffett template: thesis / amount / expected return / alternatives / risks / decision-criteria.
- **Step 2 `capital-allocation-memo-audit`** — validates per `cap_alloc_memo_rubric@1.0` (FM + SEC + QA-ALTS-001 (≥2 alts) + QA-IRR-001 (IRR assumption stated)).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ALTS-001 | Fewer than 2 alternatives considered | Operator supplies missing alts |
| 2 | QA-IRR-001 | No IRR / NPV assumption | Operator supplies |
| 2 | QA-COVENANT-001 | Debt-covenant trip risk flagged | Escalate to CLO-Legal |

## Cross-references
- `../README.md` §5.1 — output type "capital allocation"
- `../../../../modules/cuo/docs/module.md` §5.1
- `../../../skill/cap-alloc-memo-{author,audit}/SKILL.md`
