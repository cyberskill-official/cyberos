---
workflow_id: chief-operating-officer/annual-operating-model
workflow_version: 1.0.0
purpose: Refresh the annual operating model — org chart, decision rights (RACI), processes, escalation paths, governance cadence.
persona: cuo/chief-operating-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_model,        source: last year's operating-model@1,      format: operating-model@1 }
  - { name: org_chart,          source: cuo/chro (current),                  format: csv / markdown }
  - { name: ceo_priorities,     source: cuo/ceo (vision for the year),       format: markdown brief }
  - { name: incident_lessons,   source: prior-year postmortem@1 + retro@1,   format: postmortem@1 + retrospective@1 set }

outputs:
  - { name: operating_model,    format: operating-model@1, recipient: cuo/coo + cuo/ceo + entire C-suite }

skill_chain:
  - { step: 1, skill: operating-model-author, inputs_from: { prior_model: prior_model, org_chart: org_chart, ceo_priorities: ceo_priorities, incident_lessons: incident_lessons }, outputs_to: model_draft }
  - { step: 2, skill: operating-model-audit,  inputs_from: model_draft, outputs_to: operating_model }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "decision-rights changes alter authority for >2 functions" }
  - { persona: cuo/chief-human-resources-officer,        when: "org-chart changes imply layoffs / promotions / new roles" }

consults:
  - { persona: cuo/chief-communications-officer, when: "operating-model changes need internal-comms rollout" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with operating_model hash + delta-summary vs prior year
  - HITL pause at step 2 on QA-RACI-001 (ambiguous accountability) or QA-CHANGE-001 (change without rationale)
---

# Annual operating model — `chief-operating-officer/annual-operating-model`

COO's annual operating-model refresh. Combines prior model + current org chart + CEO priorities + incident lessons into the revised operating model — org chart / RACI / processes / escalations / governance cadence. The single source-of-truth for "how the company runs."

## When to invoke

- "Refresh the operating model"
- "Annual operating-model review"
- "Document how we run"

## How to invoke

```bash
cyberos-cuo run cuo/chief-operating-officer/annual-operating-model \
  --input prior_model=./ops/2025/operating-model.md \
  --input org_chart=./hr/2026-org-chart.csv \
  --input ceo_priorities=./engagements/2026/vision.md \
  --input incident_lessons=./incidents/2025/ \
  --output-dir ./ops/2026/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-8 weeks for cross-function consultation
- **Worst case:** decision-rights changes may require live exec-staff workshop + 1-quarter rollout

## Skill chain

- **Step 1 `operating-model-author`** — drafts per McKinsey operating-model framework + Spotify-model (squad/tribe/chapter/guild) where applicable.
- **Step 2 `operating-model-audit`** — validates per `operating_model_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-RACI-001 | Accountability ambiguous | Operator clarifies |
| 2 | QA-CHANGE-001 | Change no rationale | Operator adds |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.1 — COO role profile
- `../../chief-of-staff/workflows/weekly-rhythm-of-business.md` — operational peer (RoB references operating-model)
- `../../../skill/operating-model-{author,audit}/SKILL.md`
