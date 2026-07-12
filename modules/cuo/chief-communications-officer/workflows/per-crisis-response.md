---
workflow_id: chief-communications-officer/per-crisis-response
workflow_version: 1.0.0
purpose: Execute crisis-comms response — invoke playbook scenario, draft + distribute statements, coordinate stakeholder calls.
persona: cuo/chief-communications-officer
cadence: per-event
status: shipped
pattern: time_critical
sla_minutes: 120  # 2h — initial public statement window during reputational crisis

inputs:
  - { name: crisis_playbook,       source: cuo/chief-communications-officer/annual-crisis-playbook, format: crisis-communications-playbook@1 }
  - { name: incident_brief,        source: incident-detection source (CISO / CPO-Privacy / ops), format: markdown }
  - { name: stakeholder_status,    source: CRM + IR DB + media status, format: markdown }

outputs:
  - { name: crisis_response,       format: crisis-communications-playbook@1 (incident-augmented log), recipient: cuo/cco-communications + cuo/ceo + cuo/clo-legal + affected stakeholders }

skill_chain:
  - { step: 1, skill: crisis-communications-playbook-author, inputs_from: { crisis_playbook: crisis_playbook, incident_brief: incident_brief, stakeholder_status: stakeholder_status }, outputs_to: response_draft }
  - { step: 2, skill: crisis-communications-playbook-audit,  inputs_from: response_draft, outputs_to: crisis_response }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "any tier-1 crisis (material breach / litigation / safety / executive)" }
  - { persona: cuo/chief-legal-officer,      when: "response triggers SEC 8-K / Reg FD obligations OR litigation hold needed" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "cyber-incident origin" }
  - { persona: cuo/chief-privacy-officer,    when: "personal-data-breach origin (interacts with 72h Art. 33 clock)" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with crisis_response hash + scenario class + hours-from-detection
  - HITL pause at step 2 on QA-PLAYBOOK-001 (scenario not in playbook — improvising) or QA-STAKEHOLDER-001 (key stakeholder missed)
---

# Per crisis response — `chief-communications-officer/per-crisis-response`

CCO-Communications' per-incident crisis response. Invokes the annual playbook's pre-authored scenarios; if scenario is new, escalates immediately. Time-critical workflow (similar SLA pressure to `chief-privacy-officer/breach-response-cycle`).

## When to invoke

- "Activate crisis playbook for [incident]"
- "Crisis comms response for [issue]"
- "PR crisis intervention"

## How to invoke

```bash
cyberos-cuo run cuo/chief-communications-officer/per-crisis-response \
  --input crisis_playbook=./crisis/2026/playbook.md \
  --input incident_brief=./incidents/2026-05-18-001/brief.md \
  --input stakeholder_status=./crisis/2026-05-18-001/stakeholders.md \
  --output-dir ./crisis/2026-05-18-001/response/
```

## Expected duration

- **Happy path:** 1-2 hours runtime (under time pressure)
- **Worst case:** new scenario triggers same-hour CEO + CLO-Legal war-room

## Skill chain

- **Step 1 `crisis-communications-playbook-author`** — augments playbook with incident specifics.
- **Step 2 `crisis-communications-playbook-audit`** — validates per `crisis_comms_playbook_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-PLAYBOOK-001 | Scenario not in playbook | Escalate to CEO immediately |
| 2 | QA-STAKEHOLDER-001 | Key stakeholder missed | Operator extends notify list |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CCO-Communications role profile
- `./annual-crisis-playbook.md` — upstream parent
- `../../chief-privacy-officer/workflows/breach-response-cycle.md` — privacy-specific peer (72h discipline)
- `../../../skill/crisis-comms-playbook-{author,audit}/SKILL.md`
