---
workflow_id: chief-compliance-officer/annual-compliance-program
workflow_version: 1.0.0
purpose: Author the annual compliance program — applicable regulations, control framework, training plan, monitoring + testing, escalations.
persona: cuo/chief-compliance-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_program,         source: last year's compliance-program@1, format: compliance-program@1 }
  - { name: regulator_calendar,    source: cuo/clo-legal regulatory calendar, format: markdown }
  - { name: incident_lookback,     source: 12 months compliance incidents + findings, format: markdown }
  - { name: ceo_priorities,        source: cuo/ceo (vision brief), format: markdown }

outputs:
  - { name: compliance_program,    format: compliance-program@1, recipient: cuo/cco-compliance + cuo/ceo + cuo/clo-legal + Board (annual compliance chapter) }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { prior_program: prior_program, regulator_calendar: regulator_calendar, incident_lookback: incident_lookback, ceo_priorities: ceo_priorities }, outputs_to: program_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: program_draft, outputs_to: compliance_program }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "program adds new regulator (e.g. CCPA → CPRA, FedRAMP, HIPAA expansion)" }
  - { persona: cuo/chief-legal-officer,      when: "regulatory interpretation needed" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "control framework intersects security" }
  - { persona: cuo/chief-privacy-officer,    when: "privacy regulations included" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with compliance_program hash + regulator count + control count + training hours target
  - HITL pause at step 2 on QA-REGULATOR-001 (regulator not enumerated) or QA-CONTROL-001 (regulator without mapped control)
---

# Annual compliance program — `chief-compliance-officer/annual-compliance-program`

CCO-Compliance's annual program per COSO Internal Control - Integrated Framework + Federal Sentencing Guidelines Chapter 8 + ISO 37301:2021. The master document for compliance posture across all applicable regulations.

## When to invoke

- "Build the 2026 compliance program"
- "Annual compliance refresh"
- "Refresh compliance framework"

## How to invoke

```bash
cyberos-cuo run cuo/chief-compliance-officer/annual-compliance-program \
  --input prior_program=./compliance/2025/program.md \
  --input regulator_calendar=./legal/regulatory-calendar.md \
  --input incident_lookback=./compliance/2025/incidents/ \
  --input ceo_priorities=./engagements/2026/vision.md \
  --output-dir ./compliance/2026/program/
```

## Expected duration

- **Happy path:** 8-16 hours runtime + 6-12 weeks for cross-function + Board review
- **Worst case:** new regulator addition requires full control-design cycle (1-2 quarter)

## Skill chain

- **Step 1 `compliance-program-author`** — drafts per COSO + FSG Ch 8 + ISO 37301.
- **Step 2 `compliance-program-audit`** — validates per `compliance_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-REGULATOR-001 | Regulator missing | Operator adds |
| 2 | QA-CONTROL-001 | No mapped control | Operator drafts |

## Cross-references
- `../../../../modules/cuo/README.md` §5.6 — CCO-Compliance role profile
- `../../chief-legal-officer/workflows/quarterly-regulatory-cycle.md` — peer (filings vs program)
- `../../chief-information-security-officer/workflows/soc2-audit-readiness.md` — security-compliance peer
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
