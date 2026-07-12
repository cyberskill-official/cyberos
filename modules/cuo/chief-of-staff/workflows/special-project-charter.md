---
workflow_id: chief-of-staff/special-project-charter
workflow_version: 1.0.0
purpose: Charter a CEO-sponsored special project — scope, owner, success criteria, milestones, escalation path.
persona: cuo/chief-of-staff
cadence: per-event
status: shipped

inputs:
  - { name: project_brief,      source: cuo/ceo (verbal or written),               format: markdown brief }
  - { name: function_inputs,    source: each function head impacted (consult phase), format: markdown (one per function) }
  - { name: capacity_signal,    source: cuo/cfo (budget envelope, if any),         format: markdown }

outputs:
  - { name: program_charter,    format: program-charter@1, recipient: cuo/ceo + project owner + impacted function heads }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { project_brief: project_brief, function_inputs: function_inputs, capacity_signal: capacity_signal }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: program_charter }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "program-charter-audit fires QA-SCOPE-001 — scope conflicts with active OKRs" }

consults:
  - { persona: cuo/chief-financial-officer,         when: "charter implies budget > $50K or > 5% of quarterly opex" }
  - { persona: cuo/chief-human-resources-officer,        when: "charter implies headcount > 2 FTE for >1 quarter" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with program_charter hash + project-owner + due-quarter
  - HITL pause at step 2 on QA-OWNER-001 (no named project owner) or QA-SUCCESS-001 (no success criteria)
---

# Special project charter — `chief-of-staff/special-project-charter`

CoS-owned charter authoring for CEO-sponsored special projects (acquisition integration, ERP rollout, leadership offsite, comp redesign, etc.). Forces scope / owner / success-criteria / milestones / escalation-path discipline so projects don't drift. Per PMI charter template + Bain agile-PMO practice.

## When to invoke

- "Charter the [project] for the CEO"
- "Set up a project charter for [initiative]"
- "Kick off [special project] with formal charter"

## How to invoke

```bash
cyberos-cuo run cuo/chief-of-staff/special-project-charter \
  --input project_brief=./projects/2026-acme-integration/brief.md \
  --input function_inputs=./projects/2026-acme-integration/function-inputs/ \
  --input capacity_signal=./projects/2026-acme-integration/budget.md \
  --output-dir ./projects/2026-acme-integration/charter/
```

## Expected duration

- **Happy path:** 30-45 min runtime + 3-5 business days for function-head round-trip
- **Worst case:** scope-conflict escalation may require live exec-staff meeting

## Skill chain

- **Step 1 `program-charter-author`** — drafts per PMI template: scope / owner / success criteria / milestones / escalation path / cross-function dependencies.
- **Step 2 `program-charter-audit`** — validates per `program_charter_rubric@1.0` (FM + SEC + QA-OWNER-001 + QA-SUCCESS-001 + QA-MILESTONE-001 + QA-SCOPE-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-OWNER-001 | No named owner | Operator assigns |
| 2 | QA-SUCCESS-001 | No success criteria | Operator drafts |
| 2 | QA-SCOPE-001 | Conflict with active OKR | Escalate to CEO |

## Cross-references
- `../README.md` §5 (Strategic) — "cross-functional special-projects"
- `../../../../modules/cuo/docs/module.md` §5.7
- `../../../skill/program-charter-{author,audit}/SKILL.md`
