---
workflow_id: chief-innovation-officer/per-innovation-charter
workflow_version: 1.0.0
purpose: Charter a new innovation bet (Horizon-2 or Horizon-3) — hypothesis, experiment plan, success criteria, stage-gates.
persona: cuo/chief-innovation-officer
cadence: per-event
status: shipped

inputs:
  - { name: bet_brief,             source: bet sponsor, format: markdown }
  - { name: portfolio_context,     source: cuo/chief-innovation-officer/annual-innovation-portfolio, format: innovation-portfolio@1 }
  - { name: prior_charters,        source: similar bets' prior program-charter@1, format: program-charter@1 (set) }

outputs:
  - { name: innovation_charter,    format: program-charter@1, recipient: cuo/chief-innovation-officer + cuo/cpo-product + cuo/ceo + bet sponsor }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { bet_brief: bet_brief, portfolio_context: portfolio_context, prior_charters: prior_charters }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: innovation_charter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "bet exceeds $500K Horizon-3 envelope" }

consults:
  - { persona: cuo/chief-product-officer,    when: "bet may graduate to Horizon-1 roadmap" }
  - { persona: cuo/chief-technology-officer,            when: "bet requires platform investment" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with innovation_charter hash + horizon + stage-gate plan
  - HITL pause at step 2 on QA-HYPOTHESIS-001 (no testable hypothesis) or QA-STAGE-001 (no stage-gates defined)
---

# Per innovation charter — `chief-innovation-officer/per-innovation-charter`

Chief Innovation Officer's per-bet charter workflow. Each bet must have a testable hypothesis + experiment plan + stage-gates per Lean Startup + Christensen Jobs-to-Be-Done framework. Distinct from Chief-Transformation-Officer's `per-program-charter` (which charters transformation programs, not innovation bets).

## When to invoke

- "Charter the [bet name] innovation bet"
- "Start a new Horizon-2/3 experiment"
- "Innovation bet kickoff"

## How to invoke

```bash
cyberos-cuo run cuo/chief-innovation-officer/per-innovation-charter \
  --input bet_brief=./innovation/bets/2026-h3-quantum/brief.md \
  --input portfolio_context=./innovation/2026/portfolio.md \
  --input prior_charters=./innovation/bets/prior/ \
  --output-dir ./innovation/bets/2026-h3-quantum/charter/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for sponsor + cross-function input
- **Worst case:** Horizon-3 envelope review adds 1 quarter

## Skill chain

- **Step 1 `program-charter-author`** — drafts per Lean Startup + Christensen JTBD.
- **Step 2 `program-charter-audit`** — validates per `program_charter_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-HYPOTHESIS-001 | No testable hypothesis | Operator drafts |
| 2 | QA-STAGE-001 | No stage-gates | Operator defines |

## Cross-references
- `../../../../modules/cuo/README.md` §5.7 — Chief Innovation Officer role profile
- `./annual-innovation-portfolio.md` — upstream parent
- `../../chief-transformation-officer/workflows/per-program-charter.md` — distinct peer
- `../../../skill/program-charter-{author,audit}/SKILL.md`
