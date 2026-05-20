---
workflow_id: chief-risk-officer/per-incident-postmortem
workflow_version: 1.0.0
purpose: Author the risk-side postmortem for a significant incident — root cause, risk-class attribution, control-failure analysis, lessons.
persona: cuo/chief-risk-officer
cadence: per-event
status: shipped
pattern: persona_pair
peer_persona: chief-technology-officer
peer_workflow: post-incident-review
shared_artefact: incident-report
handoff_step: 3

inputs:
  - { name: incident_brief,        source: incident-detection source, format: markdown }
  - { name: technical_postmortem,  source: cuo/chief-technology-officer/post-incident-review (engineering postmortem), format: postmortem@1 }
  - { name: erm_framework,         source: cuo/chief-risk-officer/annual-erm-framework, format: enterprise-risk-framework@1 }

outputs:
  - { name: risk_postmortem,       format: postmortem@1 (risk lens), recipient: cuo/cro-risk + cuo/ceo + cuo/clo-legal + Board (if material) }

skill_chain:
  - { step: 1, skill: postmortem-author, inputs_from: { incident_brief: incident_brief, technical_postmortem: technical_postmortem, erm_framework: erm_framework }, outputs_to: pm_draft }
  - { step: 2, skill: postmortem-audit,  inputs_from: pm_draft, outputs_to: risk_postmortem }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "incident material (8-K trigger / brand impact / customer impact >threshold)" }
  - { persona: cuo/chief-legal-officer,      when: "incident implicates regulatory disclosure" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "cyber incident root cause" }
  - { persona: cuo/chief-compliance-officer, when: "control-failure intersects compliance-program controls" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with risk_postmortem hash + risk-class attribution + control failures count
  - HITL pause at step 2 on QA-ROOT-001 (root cause shallow) or QA-CONTROL-001 (control failure unidentified)
---

# Per incident postmortem (risk lens) — `chief-risk-officer/per-incident-postmortem`

CRO-Risk's risk-lens postmortem for a significant incident. Distinct from the engineering postmortem (chief-technology-officer/post-incident-review): this one classifies the incident against the ERM framework, attributes to control failures, and feeds lessons back into ERM + KRI refresh.

## When to invoke

- "Risk postmortem for [incident]"
- "ERM-lens incident review"
- "Classify the [incident] against ERM"

## How to invoke

```bash
cyberos-cuo run cuo/chief-risk-officer/per-incident-postmortem \
  --input incident_brief=./incidents/2026-05-18-001/brief.md \
  --input technical_postmortem=./incidents/2026-05-18-001/eng-pm.md \
  --input erm_framework=./risk/2026/erm/framework.md \
  --output-dir ./risk/incidents/2026-05-18-001/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for cross-function input
- **Worst case:** material classification triggers board reporting + control-redesign cycle

## Skill chain

- **Step 1 `postmortem-author`** — augments technical PM with risk-class + control-failure analysis.
- **Step 2 `postmortem-audit`** — validates per `postmortem_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ROOT-001 | Root cause shallow | Operator extends |
| 2 | QA-CONTROL-001 | Control failure unidentified | Operator traces |

## Cross-references
- `../../../../modules/cuo/README.md` §5.6 — CRO-Risk role profile
- `../../chief-technology-officer/workflows/post-incident-review.md` — upstream engineering peer
- `../../../skill/postmortem-{author,audit}/SKILL.md`
