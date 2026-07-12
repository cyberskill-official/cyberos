---
workflow_id: chief-privacy-officer/breach-response-cycle
workflow_version: 1.0.0
purpose: Handle a personal-data breach — assess, notify regulators within 72h (GDPR/PDPD), notify affected individuals, document remediation.
persona: cuo/chief-privacy-officer
cadence: per-event
status: shipped
pattern: time_critical
sla_minutes: 240  # 4h — GDPR Art. 33 + PDPD 72h floor with 4h initial-notification target

inputs:
  - { name: incident_brief,     source: cuo/ciso / on-call (initial incident detection), format: markdown }
  - { name: forensic_findings,  source: cuo/ciso (security investigation),               format: markdown / penetration-test-report@1 if available }
  - { name: affected_inventory, source: data inventory + breach-scope analysis,          format: csv }

outputs:
  - { name: breach_notification, format: breach-notification@1, recipient: regulators (per jurisdiction) + affected individuals + cuo/cpo-privacy + cuo/clo-legal + cuo/ceo }

skill_chain:
  - { step: 1, skill: breach-notification-author, inputs_from: { incident_brief: incident_brief, forensic_findings: forensic_findings, affected_inventory: affected_inventory }, outputs_to: notif_draft }
  - { step: 2, skill: breach-notification-audit,  inputs_from: notif_draft, outputs_to: breach_notification }

escalates_to:
  - { persona: cuo/chief-executive-officer,         when: "breach is material — 8-K filing + investor communication required" }
  - { persona: cuo/chief-legal-officer,   when: "breach triggers litigation hold OR cross-border notification matrix complexity" }

consults:
  - { persona: cuo/chief-information-security-officer,            when: "remediation requires architecture change" }
  - { persona: cuo/chief-communications-officer, when: "public-facing notification needs PR positioning" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with breach_notification hash + jurisdictions-notified + affected-count + hours-from-detection
  - HITL pause at step 2 on QA-72H-001 (notification approaching/past 72-hour window) — same-day escalation
---

# Breach response cycle — `chief-privacy-officer/breach-response-cycle`

CPO-Privacy's per-breach workflow. Statutory 72-hour clock under GDPR Art. 33 (regulator) + Art. 34 (individuals if high risk) + Vietnam Decree 13/2023 PDPD + most US state laws. Speed matters: late notification compounds the penalty. This workflow is the ONLY Session-G workflow with same-day same-hour escalation discipline.

## When to invoke

- "Run the breach response for [incident]"
- "Personal data breach — start the clock"
- "GDPR Art. 33 notification needed"

## How to invoke

```bash
cyberos-cuo run cuo/chief-privacy-officer/breach-response-cycle \
  --input incident_brief=./incidents/2026-05-17-001/brief.md \
  --input forensic_findings=./incidents/2026-05-17-001/forensics.md \
  --input affected_inventory=./incidents/2026-05-17-001/scope.csv \
  --output-dir ./privacy/breaches/2026-05-17-001/
```

## Expected duration

- **Happy path:** 4-8 hours runtime (under time pressure)
- **Worst case:** material breach triggers same-day CEO + CFO + CLO + CCO-Communications war-room

## Skill chain

- **Step 1 `breach-notification-author`** — drafts per GDPR Art. 33+34 / PDPD / US state laws.
- **Step 2 `breach-notification-audit`** — validates per `breach_notification_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-72H-001 | Approaching/past 72h | Same-day escalation; do not wait |
| 2 | QA-SCOPE-001 | Affected inventory incomplete | CISO re-investigates |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.6 — CPO-Privacy role profile
- `../../chief-information-security-officer/workflows/monthly-vuln-management.md` — upstream signal source
- `../../../skill/breach-notification-{author,audit}/SKILL.md`
