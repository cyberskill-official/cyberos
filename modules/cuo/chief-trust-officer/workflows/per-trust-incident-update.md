---
workflow_id: chief-trust-officer/per-trust-incident-update
workflow_version: 1.0.0
purpose: Update the trust portal after a security incident, breach, or trust event — what happened, what we did, what's next.
persona: cuo/chief-trust-officer
cadence: per-event
status: shipped
pattern: time_critical
sla_minutes: 240  # 4h — trust-incident customer-comms SLA

inputs:
  - { name: incident_brief,        source: cuo/ciso or cuo/cpo-privacy, format: markdown }
  - { name: prior_portal,          source: cuo/chief-trust-officer/quarterly-trust-portal-update, format: trust-portal-update@1 }
  - { name: response_actions,      source: response team, format: markdown }

outputs:
  - { name: trust_incident_update, format: trust-portal-update@1 (incident-augmented), recipient: cuo/chief-trust-officer + public + cuo/clo-legal + cuo/cco-communications }

skill_chain:
  - { step: 1, skill: trust-portal-update-author, inputs_from: { incident_brief: incident_brief, prior_portal: prior_portal, response_actions: response_actions }, outputs_to: update_draft }
  - { step: 2, skill: trust-portal-update-audit,  inputs_from: update_draft, outputs_to: trust_incident_update }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "incident material per 8-K thresholds" }
  - { persona: cuo/chief-legal-officer,      when: "incident triggers regulatory disclosure (GDPR Art. 33 / PDPD / state breach laws)" }

consults:
  - { persona: cuo/chief-privacy-officer,    when: "personal-data breach origin (coordinates with 72h Art. 33 clock)" }
  - { persona: cuo/chief-communications-officer, when: "incident requires coordinated PR response" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with trust_incident_update hash + scope + hours-from-detection
  - HITL pause at step 2 on QA-CONFIDENTIAL-001 (over-disclosure) or QA-SCOPE-001 (scope vague)
---

# Per trust incident update — `chief-trust-officer/per-trust-incident-update`

Chief Trust Officer's per-incident public-facing trust-portal update. Time-sensitive (paired with CPO-Privacy's `breach-response-cycle` and CCO-Communications' `per-crisis-response` if applicable). Per Santa Clara Principles + EFF best practices for incident disclosure.

## When to invoke

- "Update trust portal for [incident]"
- "Public trust update needed"
- "Trust incident disclosure"

## How to invoke

```bash
cyberos-cuo run cuo/chief-trust-officer/per-trust-incident-update \
  --input incident_brief=./incidents/2026-05-18-001/brief.md \
  --input prior_portal=./trust/2026-Q1/portal.md \
  --input response_actions=./incidents/2026-05-18-001/response.md \
  --output-dir ./trust/incidents/2026-05-18-001/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + same-day cross-function approval
- **Worst case:** material incident triggers same-hour war-room

## Skill chain

- **Step 1 `trust-portal-update-author`** — drafts per Santa Clara + EFF.
- **Step 2 `trust-portal-update-audit`** — validates per `trust_portal_update_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-CONFIDENTIAL-001 | Over-disclosure | Operator redacts |
| 2 | QA-SCOPE-001 | Vague scope | Operator clarifies |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — Chief Trust Officer role profile
- `../../chief-privacy-officer/workflows/breach-response-cycle.md` — peer (72h clock)
- `../../chief-communications-officer/workflows/per-crisis-response.md` — peer (PR coordination)
- `../../../skill/trust-portal-update-{author,audit}/SKILL.md`
