---
workflow_id: chief-revenue-officer/quarterly-churn-analysis
workflow_version: 1.0.0
purpose: Author the quarterly churn-analysis report — cohort, reason segmentation, root-cause, win-back program, leading indicators.
persona: cuo/chief-revenue-officer
cadence: quarterly
status: shipped
pattern: persona_pair
peer_persona: chief-customer-officer
peer_workflow: quarterly-churn-collaboration
shared_artefact: churn-cohort-analysis
handoff_step: 3

inputs:
  - { name: prior_analysis,     source: last quarter's churn-analysis@1,                        format: churn-analysis@1 }
  - { name: churned_accounts,   source: CRM export filtered for churned accounts in quarter,    format: csv }
  - { name: exit_interviews,    source: CS exit-interview corpus,                               format: markdown / csv }
  - { name: cs_engagements,     source: cs-engagement@1 for churned accounts (last 4 quarters), format: cs-engagement@1 (multiple) }

outputs:
  - { name: churn_analysis,     format: churn-analysis@1, recipient: cuo/cro-revenue + cuo/cco-customer + cuo/cpo-product + cuo/ceo }

skill_chain:
  - { step: 1, skill: churn-analysis-author, inputs_from: { prior_analysis: prior_analysis, churned_accounts: churned_accounts, exit_interviews: exit_interviews, cs_engagements: cs_engagements }, outputs_to: analysis_draft }
  - { step: 2, skill: churn-analysis-audit,  inputs_from: analysis_draft, outputs_to: churn_analysis }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "gross revenue retention (GRR) drops > 5pts QoQ" }
  - { persona: cuo/chief-product-officer, when: "reason segmentation shows >40% product-driven" }

consults:
  - { persona: cuo/chief-marketing-officer,         when: "reason segmentation shows fit-driven (wrong ICP attracted)" }
  - { persona: cuo/chief-customer-officer, when: "execution-driven churn pattern" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with churn_analysis hash + GRR + NRR + top-reason
  - HITL pause at step 2 on QA-ROOT-001 (root-cause no verbatim evidence) or QA-LEAD-001 (no leading indicators)
---

# Quarterly churn analysis — `chief-revenue-officer/quarterly-churn-analysis`

CRO-Revenue's quarterly churn-analysis workflow. Combines prior analysis + churned accounts + exit interviews + last-4-quarter CS engagements into the standard cohort / reason segmentation / root-cause / win-back / leading-indicators output. Per Reichheld customer-economics + Gainsight + Catalyst + TSIA churn-benchmarking + Bessemer Cloud Index.

## When to invoke

- "Run the Q<n> churn analysis"
- "Why are customers leaving"
- "Churn root-cause review"

## How to invoke

```bash
cyberos-cuo run cuo/chief-revenue-officer/quarterly-churn-analysis \
  --input prior_analysis=./churn/2026-Q1/analysis.md \
  --input churned_accounts=./crm/2026-Q1-churn.csv \
  --input exit_interviews=./cs/2026-Q1/exit-interviews/ \
  --input cs_engagements=./customer/2025-2026/cs-history/ \
  --output-dir ./churn/2026-Q1/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for verbatim coding + win-back design
- **Worst case:** GRR drop > 5pts triggers cross-function intervention program

## Skill chain

- **Step 1 `churn-analysis-author`** — drafts per Reichheld / Gainsight / Catalyst structure.
- **Step 2 `churn-analysis-audit`** — validates per `churn_analysis_rubric@1.0` (FM + SEC + QA-ROOT-001 + QA-LEAD-001 + QA-WINBACK-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ROOT-001 | Root-cause no evidence | Operator codes verbatims |
| 2 | QA-LEAD-001 | No leading indicators | Operator drafts at-risk signals |
| 2 | QA-WINBACK-001 | Win-back program no offer | Operator designs |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.2 — CRO-Revenue role profile
- `../../cco-customer/README.md` — CS peer for execution-driven churn
- `../../../skill/churn-analysis-{author,audit}/SKILL.md`
