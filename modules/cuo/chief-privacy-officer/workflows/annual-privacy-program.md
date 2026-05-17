---
workflow_id: chief-privacy-officer/annual-privacy-program
workflow_version: 1.0.0
purpose: Author the annual privacy program — ROPA + DPIA inventory + DSR metrics + breach lookback + regulator-by-regulator status + roadmap.
persona: cuo/chief-privacy-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_program,      source: last year's compliance-program@1 (privacy chapter), format: compliance-program@1 }
  - { name: dsr_log,            source: cuo/chief-privacy-officer/data-subject-request-cycle (4Q),    format: dsr-runbook@1 (multiple) }
  - { name: breach_log,         source: cuo/chief-privacy-officer/breach-response-cycle (4Q),         format: breach-notification@1 (multiple) }
  - { name: pia_inventory,      source: cuo/chief-privacy-officer/privacy-impact-assessment (4Q),     format: pia@1 (multiple) }

outputs:
  - { name: privacy_program,    format: compliance-program@1, recipient: cuo/cpo-privacy + cuo/clo-legal + cuo/ceo + Board (annual privacy chapter) }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { prior_program: prior_program, dsr_log: dsr_log, breach_log: breach_log, pia_inventory: pia_inventory }, outputs_to: program_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: program_draft, outputs_to: privacy_program }

escalates_to:
  - { persona: cuo/chief-legal-officer,   when: "regulator inquiry trend visible; need legal positioning" }
  - { persona: cuo/chief-financial-officer,         when: "program requires significant budget for tooling (PrivacyOps / consent mgmt)" }

consults:
  - { persona: cuo/chief-information-security-officer,        when: "privacy-by-design technical controls need engineering investment" }
  - { persona: cuo/chief-compliance-officer, when: "privacy program intersects SOC 2 / ISO 27701" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with privacy_program hash + DSR-count + breach-count + PIA-count + open-regulator-count
  - HITL pause at step 2 on QA-ROPA-001 (ROPA not current) or QA-METRICS-001 (DSR/breach metrics lack trend)
---

# Annual privacy program — `chief-privacy-officer/annual-privacy-program`

CPO-Privacy's annual privacy-program workflow. Combines prior program + DSR log + breach log + PIA inventory into the annual program document: ROPA refresh + DPIA inventory + DSR metrics + breach lookback + regulator-by-regulator status + program roadmap. Board-reviewed annually as part of the legal/risk chapter.

## When to invoke

- "Build the 2026 privacy program"
- "Annual privacy review"
- "Refresh the ROPA + DPIA inventory"

## How to invoke

```bash
cyberos-cuo run cuo/chief-privacy-officer/annual-privacy-program \
  --input prior_program=./privacy/2025/program.md \
  --input dsr_log=./privacy/dsr/2025/ \
  --input breach_log=./privacy/breaches/2025/ \
  --input pia_inventory=./privacy/pia/2025/ \
  --output-dir ./privacy/2026/program/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 4-6 weeks for cross-function inputs + board prep
- **Worst case:** regulator-inquiry-driven re-cut may add 1 quarter

## Skill chain

- **Step 1 `compliance-program-author`** — drafts per GDPR + PDPD + CCPA + ISO/IEC 27701 program structure.
- **Step 2 `compliance-program-audit`** — validates per `compliance_program_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-ROPA-001 | ROPA not current | Operator refreshes |
| 2 | QA-METRICS-001 | DSR/breach metrics no trend | Operator adds time series |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — CPO-Privacy role profile
- `../../chief-legal-officer/workflows/quarterly-regulatory-cycle.md` — peer for regulator-specific filings
- `../../cco-compliance/README.md` — compliance peer
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
