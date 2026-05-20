---
workflow_id: chief-of-staff/weekly-rhythm-of-business
workflow_version: 1.0.0
purpose: Maintain the operating rhythm — set the week's meeting calendar, action-item triage, exec-staff agenda, CEO time audit.
persona: cuo/chief-of-staff
cadence: weekly
status: shipped

inputs:
  - { name: prior_rob,          source: last week's rhythm-of-business@1, format: rhythm-of-business@1 }
  - { name: open_decisions,     source: cuo/chief-of-staff/decision-log,  format: decision-log@1 outstanding items }
  - { name: ceo_calendar,       source: CEO's calendar tool,              format: ics or markdown extract }
  - { name: exec_inputs,        source: each C-level (5-min status),      format: markdown (one per exec) }

outputs:
  - { name: rob,                format: rhythm-of-business@1, recipient: cuo/ceo + cuo/chief-of-staff + entire C-suite }

skill_chain:
  - { step: 1, skill: rhythm-of-business-author, inputs_from: { prior_rob: prior_rob, open_decisions: open_decisions, ceo_calendar: ceo_calendar, exec_inputs: exec_inputs }, outputs_to: rob_draft }
  - { step: 2, skill: rhythm-of-business-audit,  inputs_from: rob_draft, outputs_to: rob }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "rhythm-of-business-audit fires QA-CEO-TIME-001 — CEO calendar exceeds 80% booked or has no focus blocks" }

consults:
  - { persona: cuo/chief-communications-officer, when: "rhythm includes external-comms cadence items (PR, customer events)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with rob hash + open-decision count + CEO-time-reclaimed metric
  - HITL pause at step 2 on QA-DECISION-AGE-001 (decision open >2 weeks)
---

# Weekly rhythm of business — `chief-of-staff/weekly-rhythm-of-business`

CoS-owned weekly operating-rhythm refresh. Combines last week's RoB + open decisions + CEO calendar + 5-min exec-staff inputs into the week's RoB doc (exec-staff agenda, decisions-to-close-this-week, CEO time-blocks, function-status). Per First Round / Visible.vc CoS playbook.

## When to invoke

- "Run the weekly RoB refresh"
- "Set this week's exec staff agenda"
- "What's on the CEO's plate this week"

## How to invoke

```bash
cyberos-cuo run cuo/chief-of-staff/weekly-rhythm-of-business \
  --input prior_rob=./rob/2026-W19.md \
  --input open_decisions=./decisions/open.md \
  --input ceo_calendar=./calendar/ceo-2026-W20.ics \
  --input exec_inputs=./exec-staff/2026-W20/ \
  --output-dir ./rob/2026-W20/
```

## Expected duration

- **Happy path:** 30-45 min runtime + Friday afternoon close-out
- **Worst case:** decision-age escalation may require ad-hoc 1:1s

## Skill chain

- **Step 1 `rhythm-of-business-author`** — drafts per First Round CoS template: agenda / decisions / focus blocks / cross-function dependencies.
- **Step 2 `rhythm-of-business-audit`** — validates per `rhythm_of_business_rubric@1.0` (FM + SEC + QA-CEO-TIME-001 + QA-DECISION-AGE-001 + QA-DEPENDENCY-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CEO-TIME-001 | CEO calendar > 80% booked / no focus blocks | Escalate to CEO; reshuffle |
| 2 | QA-DECISION-AGE-001 | Decision open >2 weeks | Operator schedules close-out 1:1 |

## Cross-references
- `../README.md` §5 (Strategic) — "rhythm-of-business calendar"
- `../../../../modules/cuo/README.md` §5.7
- `../../../skill/rhythm-of-business-{author,audit}/SKILL.md`
