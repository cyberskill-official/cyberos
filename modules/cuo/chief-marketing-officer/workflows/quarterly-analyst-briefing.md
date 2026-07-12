---
workflow_id: chief-marketing-officer/quarterly-analyst-briefing
workflow_version: 1.0.0
purpose: Author a quarterly analyst briefing — Gartner / Forrester / IDC / IDG analyst-relations narrative + supporting evidence.
persona: cuo/chief-marketing-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_briefing,        source: last quarter's analyst-briefing@1, format: analyst-briefing@1 }
  - { name: product_updates,       source: cuo/cpo-product (last 90 days), format: markdown }
  - { name: customer_wins,         source: cuo/cso-sales (notable closed-won + expansion), format: markdown }
  - { name: market_movements,      source: competitive-brief + analyst-questionnaire intake, format: markdown }

outputs:
  - { name: analyst_briefing,      format: analyst-briefing@1, recipient: cuo/cmo + cuo/ceo + cuo/cpo-product + target analysts }

skill_chain:
  - { step: 1, skill: analyst-briefing-author, inputs_from: { prior_briefing: prior_briefing, product_updates: product_updates, customer_wins: customer_wins, market_movements: market_movements }, outputs_to: briefing_draft }
  - { step: 2, skill: analyst-briefing-audit,  inputs_from: briefing_draft, outputs_to: analyst_briefing }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "briefing includes strategic-shift content (M&A, pivot, exec change)" }

consults:
  - { persona: cuo/chief-product-officer,    when: "product narrative is the lead story" }
  - { persona: cuo/chief-communications-officer, when: "briefing dovetails with PR campaign" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with analyst_briefing hash + analyst-firm-target + key-message count
  - HITL pause at step 2 on QA-EVIDENCE-001 (claim lacks evidence)
---

# Quarterly analyst briefing — `chief-marketing-officer/quarterly-analyst-briefing`

CMO's quarterly AR (analyst-relations) briefing per ARchitect / Kea Company AR best practices. Drives positioning with Gartner / Forrester / IDC / IDG analysts. Critical for enterprise-segment companies where MQ / Wave / MarketScape placement affects deal velocity.

## When to invoke

- "Build the Q<n> analyst briefing"
- "Analyst briefing for [firm]"
- "AR refresh"

## How to invoke

```bash
cyberos-cuo run cuo/chief-marketing-officer/quarterly-analyst-briefing \
  --input prior_briefing=./ar/2026-Q1/briefing.md \
  --input product_updates=./product/2026-Q1/updates.md \
  --input customer_wins=./sales/2026-Q1/wins.md \
  --input market_movements=./market/2026-Q2/intel.md \
  --output-dir ./ar/2026-Q2/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2 weeks for analyst-call scheduling
- **Worst case:** competitive intel may require rewrite

## Skill chain

- **Step 1 `analyst-briefing-author`** — drafts per ARchitect + Kea Company AR.
- **Step 2 `analyst-briefing-audit`** — validates per `analyst_briefing_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-EVIDENCE-001 | Claim no evidence | Operator supplies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CMO role profile
- `../../cco-communications/README.md` — PR partner
- `../../../skill/analyst-briefing-{author,audit}/SKILL.md`
