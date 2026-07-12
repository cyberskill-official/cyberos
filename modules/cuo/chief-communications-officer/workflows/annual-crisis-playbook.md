---
workflow_id: chief-communications-officer/annual-crisis-playbook
workflow_version: 1.0.0
purpose: Refresh the annual crisis-comms playbook — scenario inventory, holding statements, escalation matrix, media protocols, stakeholder maps.
persona: cuo/chief-communications-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_playbook,        source: last year's crisis-comms-playbook@1, format: crisis-communications-playbook@1 }
  - { name: incident_lookback,     source: 12 months of incident postmortems, format: postmortem@1 (multiple) }
  - { name: regulator_calendar,    source: cuo/clo-legal's regulatory calendar, format: markdown }
  - { name: stakeholder_register,  source: PR + IR stakeholder DB, format: markdown }

outputs:
  - { name: crisis_playbook,       format: crisis-comms-playbook@1, recipient: cuo/cco-communications + cuo/ceo + cuo/clo-legal + cuo/ciso + cuo/cpo-privacy }

skill_chain:
  - { step: 1, skill: crisis-communications-playbook-author, inputs_from: { prior_playbook: prior_playbook, incident_lookback: incident_lookback, regulator_calendar: regulator_calendar, stakeholder_register: stakeholder_register }, outputs_to: playbook_draft }
  - { step: 2, skill: crisis-communications-playbook-audit,  inputs_from: playbook_draft, outputs_to: crisis_playbook }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "playbook adds new scenario class (e.g. AI-incident, climate-related)" }
  - { persona: cuo/chief-legal-officer,      when: "scenario triggers SEC 8-K / Reg FD obligations" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "cyber-incident scenarios need security-incident-response integration" }
  - { persona: cuo/chief-privacy-officer,    when: "breach scenarios need privacy notification integration (Art. 33)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with crisis_playbook hash + scenario count + holding-statement count
  - HITL pause at step 2 on QA-SCENARIO-001 (scenario lacks holding statement) or QA-ESCALATION-001 (escalation matrix ambiguous)
---

# Annual crisis playbook — `chief-communications-officer/annual-crisis-playbook`

CCO-Communications' annual crisis-comms playbook per PRSA + Coombs SCCT (Situational Crisis Communication Theory) + Institute for Crisis Management. Maintains the pre-authored holding statements + escalation matrix + media protocols + stakeholder maps for known crisis scenarios.

## When to invoke

- "Refresh the 2026 crisis playbook"
- "Annual crisis-comms review"
- "Update crisis response playbook"

## How to invoke

```bash
cyberos-cuo run cuo/chief-communications-officer/annual-crisis-playbook \
  --input prior_playbook=./crisis/2025/playbook.md \
  --input incident_lookback=./incidents/2025/ \
  --input regulator_calendar=./legal/regulatory-calendar.md \
  --input stakeholder_register=./pr/stakeholders.md \
  --output-dir ./crisis/2026/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-6 weeks for cross-function input + tabletop
- **Worst case:** new scenario class adds 1-2 quarter (legal + stakeholder mapping)

## Skill chain

- **Step 1 `crisis-communications-playbook-author`** — drafts per PRSA + Coombs SCCT + ICM.
- **Step 2 `crisis-communications-playbook-audit`** — validates per `crisis_comms_playbook_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SCENARIO-001 | Scenario no holding statement | Operator drafts |
| 2 | QA-ESCALATION-001 | Escalation ambiguous | Operator clarifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CCO-Communications role profile
- `../../chief-privacy-officer/workflows/breach-response-cycle.md` — privacy-breach scenario peer
- `../../../skill/crisis-comms-playbook-{author,audit}/SKILL.md`
