---
workflow_id: chief-human-resources-officer/new-hire-onboarding
workflow_version: 1.0.0
purpose: Generate the onboarding pack for a new hire — Day-1 / Week-1 / 30-60-90 plan, buddy assignment, accounts list, role-specific learning path.
persona: cuo/chief-human-resources-officer
cadence: per-event
status: shipped

inputs:
  - { name: hire_record,        source: ATS / hire-decision@1, format: hire-decision-record@1 }
  - { name: role_jd,            source: workforce-plan@1 or JD, format: markdown }
  - { name: team_context,       source: hiring manager,         format: markdown brief }

outputs:
  - { name: onboarding_pack,    format: onboarding-pack@1, recipient: new hire + hiring manager + IT + cuo/chro }

skill_chain:
  - { step: 1, skill: onboarding-pack-author, inputs_from: { hire_record: hire_record, role_jd: role_jd, team_context: team_context }, outputs_to: pack_draft }
  - { step: 2, skill: onboarding-pack-audit,  inputs_from: pack_draft, outputs_to: onboarding_pack }

escalates_to:
  - { persona: cuo/chief-information-security-officer,        when: "role requires elevated security access (privileged credentials, prod, customer data)" }

consults:
  - { persona: cuo/chief-technology-officer,         when: "engineering hire — adds repo / on-call / IDE setup chapter" }
  - { persona: cuo/chief-customer-officer, when: "customer-facing hire — adds customer-shadowing schedule" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with onboarding_pack hash + buddy-assigned flag + 30-60-90-defined flag
  - HITL pause at step 2 on QA-BUDDY-001 (no named buddy) or QA-GOALS-001 (no 30-60-90 goals)
---

# New-hire onboarding — `chief-human-resources-officer/new-hire-onboarding`

CHRO's standard onboarding-pack workflow. Per-hire output: Day-1 / Week-1 / 30-60-90 plan, buddy assignment, accounts list, role-specific learning path. Audited for buddy + goals + accounts completeness. Triggered automatically when a hire-decision-record@1 is shipped (or manually on demand).

## When to invoke

- "Build onboarding for [new hire]"
- "Onboarding pack for [name] starting [date]"
- "New hire prep"

## How to invoke

```bash
cyberos-cuo run cuo/chief-human-resources-officer/new-hire-onboarding \
  --input hire_record=./hires/2026-cfo/decision.md \
  --input role_jd=./hires/2026-cfo/jd.md \
  --input team_context=./hires/2026-cfo/team-context.md \
  --output-dir ./onboarding/2026-cfo/
```

## Expected duration

- **Happy path:** 30-60 min runtime + same-day manager review
- **Worst case:** security-access escalation may delay account provisioning by 1 week

## Skill chain

- **Step 1 `onboarding-pack-author`** — drafts per First Round + Lattice onboarding templates.
- **Step 2 `onboarding-pack-audit`** — validates per `onboarding_pack_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-BUDDY-001 | No named buddy | Hiring manager assigns |
| 2 | QA-GOALS-001 | No 30-60-90 goals | Hiring manager drafts |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.5 — CHRO role profile
- `../../chief-executive-officer/workflows/c-suite-hire-decision.md` — upstream hire-decision feeder
- `../../../skill/onboarding-pack-{author,audit}/SKILL.md`
