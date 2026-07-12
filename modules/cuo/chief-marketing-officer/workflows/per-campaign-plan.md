---
workflow_id: chief-marketing-officer/per-campaign-plan
workflow_version: 1.0.0
purpose: Build a campaign plan for a launch / theme / sustained-program — objective, audience, channel mix, creative brief, measurement.
persona: cuo/chief-marketing-officer
cadence: per-event
status: shipped
pattern: persona_pair
peer_persona: chief-communications-officer
peer_workflow: per-press-release
shared_artefact: campaign-plan
handoff_step: 4

inputs:
  - { name: brand_strategy,        source: cuo/chief-marketing-officer/quarterly-brand-strategy, format: brand-strategy@1 }
  - { name: campaign_brief,        source: marketing requestor (PM / sales / CS), format: markdown }
  - { name: budget_envelope,       source: marketing budget allocation, format: markdown }
  - { name: prior_campaigns,       source: similar prior campaigns + performance, format: markdown }

outputs:
  - { name: campaign_plan,         format: campaign-plan@1, recipient: cuo/cmo + creative team + channel owners + cuo/cso-sales (alignment) }

skill_chain:
  - { step: 1, skill: campaign-plan-author, inputs_from: { brand_strategy: brand_strategy, campaign_brief: campaign_brief, budget_envelope: budget_envelope, prior_campaigns: prior_campaigns }, outputs_to: plan_draft }
  - { step: 2, skill: campaign-plan-audit,  inputs_from: plan_draft, outputs_to: campaign_plan }

escalates_to:
  - { persona: cuo/chief-marketing-officer,            when: "audit fires QA-MEASURE-001 — campaign lacks measurable success criteria" }

consults:
  - { persona: cuo/chief-sales-officer,      when: "campaign expects pipeline impact" }
  - { persona: cuo/chief-communications-officer, when: "campaign involves PR / earned media" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with campaign_plan hash + channel mix + target metrics
  - HITL pause at step 2 on QA-MEASURE-001 (no measurable target)
---

# Per campaign plan — `chief-marketing-officer/per-campaign-plan`

CMO's per-campaign planning workflow. Per AAF campaign-planning framework + AMA campaign-evaluation standards. Triggered per launch, per quarterly theme, or per sustained program.

## When to invoke

- "Build the [campaign name] plan"
- "Campaign planning for [theme]"
- "Plan the launch campaign"

## How to invoke

```bash
cyberos-cuo run cuo/chief-marketing-officer/per-campaign-plan \
  --input brand_strategy=./brand/2026-Q2/strategy.md \
  --input campaign_brief=./campaigns/2026-acme-launch/brief.md \
  --input budget_envelope=./marketing/2026-Q2/budget.md \
  --input prior_campaigns=./campaigns/prior/ \
  --output-dir ./campaigns/2026-acme-launch/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1-2 weeks for creative + channel input
- **Worst case:** measurement gap triggers re-plan

## Skill chain

- **Step 1 `campaign-plan-author`** — drafts per AAF + AMA standards.
- **Step 2 `campaign-plan-audit`** — validates per `campaign_plan_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-MEASURE-001 | No measurable target | Operator quantifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CMO role profile
- `./quarterly-brand-strategy.md` — upstream feeder
- `../../../skill/campaign-plan-{author,audit}/SKILL.md`
