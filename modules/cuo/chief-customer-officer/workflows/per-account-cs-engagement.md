---
workflow_id: chief-customer-officer/per-account-cs-engagement
workflow_version: 1.0.0
purpose: Author the per-account CS engagement plan — relationship map, success criteria, cadence, expansion thesis, risk indicators.
persona: cuo/chief-customer-officer
cadence: per-event
status: shipped

inputs:
  - { name: account_brief,         source: CSM, format: markdown }
  - { name: prior_engagement,      source: last engagement plan for the account (if any), format: cs-engagement@1 }
  - { name: crm_history,           source: CRM activity, format: csv (last 90 days) }
  - { name: product_usage,         source: telemetry, format: csv }

outputs:
  - { name: cs_engagement,         format: cs-engagement@1, recipient: cuo/cco-customer + CSM + account team }

skill_chain:
  - { step: 1, skill: cs-engagement-author, inputs_from: { account_brief: account_brief, prior_engagement: prior_engagement, crm_history: crm_history, product_usage: product_usage }, outputs_to: engagement_draft }
  - { step: 2, skill: cs-engagement-audit,  inputs_from: engagement_draft, outputs_to: cs_engagement }

escalates_to:
  - { persona: cuo/chief-customer-officer,   when: "account is top-tier and engagement plan lacks executive sponsor" }

consults:
  - { persona: cuo/chief-sales-officer,      when: "expansion thesis maps to active sales opportunity" }
  - { persona: cuo/chief-product-officer,    when: "account requested feature commitments" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with cs_engagement hash + account-tier + expansion-thesis-$
  - HITL pause at step 2 on QA-SUCCESS-001 (no success criteria) or QA-CADENCE-001 (no defined cadence)
---

# Per-account CS engagement — `chief-customer-officer/per-account-cs-engagement`

CCO-Customer's per-account CS engagement-plan workflow. Per Gainsight Customer Success methodology. Triggered per major account onboarding, per renewal cycle, or when health-score drops.

## When to invoke

- "Build CS engagement plan for [account]"
- "Account engagement refresh for [name]"
- "CSM playbook for [customer]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-customer-officer/per-account-cs-engagement \
  --input account_brief=./customer/accounts/2026-acme/brief.md \
  --input prior_engagement=./customer/accounts/2026-acme/prior-engagement.md \
  --input crm_history=./crm/2026-Q1/acme-activity.csv \
  --input product_usage=./customer/accounts/2026-acme/usage.csv \
  --output-dir ./customer/accounts/2026-acme/engagement/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 1 week for CSM + AE validation
- **Worst case:** executive-sponsor gap requires CCO intervention

## Skill chain

- **Step 1 `cs-engagement-author`** — drafts per Gainsight methodology.
- **Step 2 `cs-engagement-audit`** — validates per `cs_engagement_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SUCCESS-001 | No success criteria | Operator drafts |
| 2 | QA-CADENCE-001 | No cadence | Operator defines |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4 — CCO-Customer role profile
- `../../chief-sales-officer/workflows/quarterly-account-plan.md` — sales peer (expansion thesis overlap)
- `../../../skill/cs-engagement-{author,audit}/SKILL.md`
