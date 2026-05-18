---
workflow_id: chief-sales-officer/quarterly-account-plan
workflow_version: 1.0.0
purpose: Build the quarterly strategic-account plan for a top-tier account — stakeholder map, white-space analysis, growth thesis, action plan.
persona: cuo/chief-sales-officer
cadence: quarterly
status: shipped
pattern: per_instance
instance_descriptor:
  - { account_id: ACCT-0001, account_name: "<populate at runtime from CRM top-tier query>", account_tier: enterprise }
  # In production: operator pre-populates this list each quarter from CRM top-tier filter.

inputs:
  - { name: account_brief,      source: account-team brief,                       format: markdown }
  - { name: prior_plan,         source: last quarter's account-plan@1,            format: account-plan@1 }
  - { name: crm_activity,       source: Salesforce / HubSpot (last 90 days),      format: csv export }
  - { name: customer_signals,   source: customer-success-engagement@1 (if existing customer),   format: customer-success-engagement@1 }

outputs:
  - { name: account_plan,       format: account-plan@1, recipient: cuo/cso-sales + AE + CS owner + cuo/cco-customer }

skill_chain:
  - { step: 1, skill: account-plan-author, inputs_from: { account_brief: account_brief, prior_plan: prior_plan, crm_activity: crm_activity, customer_signals: customer_signals }, outputs_to: plan_draft }
  - { step: 2, skill: account-plan-audit,  inputs_from: plan_draft, outputs_to: account_plan }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "account is top 10 by ARR — CEO-sponsored relationship" }
  - { persona: cuo/chief-customer-officer, when: "account is at-risk / churn signal in customer_signals" }

consults:
  - { persona: cuo/chief-product-officer, when: "account asks for feature commitments — product input needed" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with account_plan hash + account-tier + growth-thesis-$
  - HITL pause at step 2 on QA-STAKEHOLDER-001 (no decision-maker mapped) or QA-WHITESPACE-001 (no growth thesis)
---

# Quarterly strategic-account plan — `chief-sales-officer/quarterly-account-plan`

CSO-Sales' quarterly account-planning workflow for top-tier accounts. Per Winning by Design + Sales Excellence (Holden) strategic-account-management discipline: stakeholder map (champions / detractors / blockers) + white-space analysis + named growth thesis + 90-day action plan.

## When to invoke

- "Build account plan for [customer]"
- "Quarterly account review for [name]"
- "Strategic account plan refresh"

## How to invoke

```bash
cyberos-cuo run cuo/chief-sales-officer/quarterly-account-plan \
  --input account_brief=./accounts/2026-Q2/acme/brief.md \
  --input prior_plan=./accounts/2026-Q1/acme/plan.md \
  --input crm_activity=./sales/2026-Q1/acme-activity.csv \
  --input customer_signals=./cs/2026-Q1/acme-engagement.md \
  --output-dir ./accounts/2026-Q2/acme/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + 1 week for AE + CS validation
- **Worst case:** at-risk classification triggers same-day CCO-Customer intervention

## Skill chain

- **Step 1 `account-plan-author`** — drafts per Winning by Design + Holden strategic-account-mgmt.
- **Step 2 `account-plan-audit`** — validates per `account_plan_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-STAKEHOLDER-001 | No decision-maker mapped | AE supplies |
| 2 | QA-WHITESPACE-001 | No growth thesis | AE drafts |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.4 — CSO-Sales role profile
- `../../cco-customer/README.md` — peer for at-risk handoff
- `../../../skill/account-plan-{author,audit}/SKILL.md`
