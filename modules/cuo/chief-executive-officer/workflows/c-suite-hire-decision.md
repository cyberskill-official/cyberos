---
workflow_id: chief-executive-officer/c-suite-hire-decision
workflow_version: 1.0.0
purpose: Drive a C-suite hire decision — interview-loop synthesis, reference-check synthesis, offer recommendation with rationale.
persona: cuo/chief-executive-officer
cadence: per-event
status: shipped

inputs:
  - { name: candidate_brief,    source: workflow-caller,                            format: markdown (CV summary + role JD) }
  - { name: interview_panel,    source: each interviewer (typically 5-7 panelists), format: scorecards (markdown) }
  - { name: ref_checks,         source: reference-check synthesis (CEO or CHRO),    format: markdown notes }
  - { name: role_jd,            source: role JD or workforce-plan output,           format: workforce-plan@1 chapter or JD markdown }

outputs:
  - { name: hire_decision,      format: hire-decision-record@1, recipient: cuo/ceo + cuo/chro + Board (info copy) }

skill_chain:
  - { step: 1, skill: hire-decision-author, inputs_from: { candidate_brief: candidate_brief, interview_panel: interview_panel, ref_checks: ref_checks, role_jd: role_jd }, outputs_to: decision_draft }
  - { step: 2, skill: hire-decision-audit,  inputs_from: decision_draft, outputs_to: hire_decision }

escalates_to:
  - { persona: cuo/chief-human-resources-officer,            when: "hire-decision-audit fires QA-BIAS-001 — panelist scoring shows disparate-impact pattern" }
  - { persona: cuo/chief-legal-officer,       when: "candidate's prior role triggers non-compete or trade-secret risk" }

consults:
  - { persona: cuo/chief-financial-officer,             when: "offer envelope exceeds the CFO-approved C-level comp band" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with hire_decision hash + final recommendation (offer / pass / parking-lot)
  - HITL pause at step 2 if QA-DISSENT-001 fires (panelist dissent unaddressed)
---

# C-suite hire decision — `chief-executive-officer/c-suite-hire-decision`

CEO-owned workflow for closing a C-level hire. Synthesises 5-7 panelist scorecards + 3-5 reference checks against the role JD, produces a structured hire-decision-record per the Topgrading / Who method, and audits for bias / dissent / comp-band fit.

## When to invoke

- "Make a hire decision on [candidate]"
- "Synthesise interview panel feedback for [name]"
- "Close out the [role] hire"

## How to invoke

```bash
cyberos-cuo run cuo/chief-executive-officer/c-suite-hire-decision \
  --input candidate_brief=./hires/2026-cfo/candidate-brief.md \
  --input interview_panel=./hires/2026-cfo/scorecards/ \
  --input ref_checks=./hires/2026-cfo/ref-checks.md \
  --input role_jd=./hires/2026-cfo/jd.md \
  --output-dir ./hires/2026-cfo/decision/
```

## Expected duration

- **Happy path:** 30-45 min runtime + 1 business day operator review
- **Worst case:** bias/dissent escalation may add 1 week (CHRO investigation + panel re-convene)

## Skill chain

- **Step 1 `hire-decision-author`** — drafts per Topgrading / Who recommendation template: must-haves / nice-to-haves / panelist-consensus / dissent / ref-check synthesis / risks / final rec.
- **Step 2 `hire-decision-audit`** — validates per `hire_decision_rubric@1.0` (FM + SEC + QA-BIAS-001 + QA-DISSENT-001 + QA-COMP-001 on offer envelope).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-BIAS-001 | Panel scoring shows disparate-impact pattern | Escalate to CHRO |
| 2 | QA-DISSENT-001 | Dissenting panelist not addressed | Operator addresses in writing |
| 2 | QA-COMP-001 | Offer exceeds CFO-approved band | Escalate to CFO for band waiver |

## Cross-references
- `../README.md` §5.4 — output type "C-suite hires + fires"
- `../../../../modules/cuo/README.md` §5.1
- `../../../skill/hire-decision-{author,audit}/SKILL.md`
