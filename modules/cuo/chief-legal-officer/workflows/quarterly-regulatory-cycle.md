---
workflow_id: chief-legal-officer/quarterly-regulatory-cycle
workflow_version: 1.0.0
purpose: Author the quarter's regulatory filings — SEC quarterlies (10-Q/8-K), GDPR/PDPD compliance reviews, FDA quarterly safety reports, AI Act conformity refreshes.
persona: cuo/chief-legal-officer
cadence: quarterly
status: shipped
pattern: multi_output
output_recipients:
  - { recipient_id: vn-mst,  format: filing-pdf,  delivery_method: portal }   # Vietnam Ministry of Science & Technology
  - { recipient_id: vn-mof,  format: filing-pdf,  delivery_method: email }    # Vietnam Ministry of Finance
  - { recipient_id: vn-sbv,  format: filing-pdf,  delivery_method: portal }   # State Bank of Vietnam (if applicable)
  - { recipient_id: vn-mic,  format: filing-pdf,  delivery_method: portal }   # Ministry of Information & Communications (PDPD)

inputs:
  - { name: filings_calendar,   source: cuo/clo-legal's regulatory-calendar register, format: markdown }
  - { name: compliance_program, source: cuo/cco-compliance (annual compliance-program@1 with quarterly status), format: compliance-program@1 }
  - { name: prior_filings,      source: prior quarter's regulatory-filing@1 set,      format: regulatory-filing@1 (one per regulator) }
  - { name: material_events,    source: workflow-caller (CFO + CEO + CISO inputs for material events in quarter), format: markdown brief }

outputs:
  - { name: quarterly_filings,  format: regulatory-filing@1 (multiple, one per regulator), recipient: regulators + cuo/clo-legal + Board }

skill_chain:
  - { step: 1, skill: regulatory-filing-author, inputs_from: { filings_calendar: filings_calendar, compliance_program: compliance_program, prior_filings: prior_filings, material_events: material_events }, outputs_to: filings_draft }
  - { step: 2, skill: regulatory-filing-audit,  inputs_from: filings_draft, outputs_to: quarterly_filings }

escalates_to:
  - { persona: cuo/chief-financial-officer,         when: "10-Q financial chapter requires CFO certification" }
  - { persona: cuo/chief-executive-officer,         when: "10-Q/10-K requires CEO certification (Sarbanes-Oxley §302)" }
  - { persona: cuo/chief-privacy-officer, when: "GDPR/PDPD filing surfaces a breach not previously reported (≤72h window per Art. 33)" }

consults:
  - { persona: cuo/chief-information-security-officer,        when: "security incident affects 10-K Item 1C cybersecurity disclosure" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with one row per regulator-filing (hash + regulator + filing-type + due-date)
  - HITL pause at step 2 on QA-SOURCING-001 (material assertion lacks primary-record citation)
---

# Quarterly regulatory cycle — `chief-legal-officer/quarterly-regulatory-cycle`

CLO-Legal's quarterly regulatory-filing cycle. Combines the regulator calendar + compliance-program status + material-events brief to author the quarter's filings (SEC 10-Q / 8-K, GDPR-Art-30 + PDPD updates, FDA safety reports per persona, EU AI Act conformity refresh). Each filing's section ordering matches its regulator's structural mandate.

## When to invoke

- "Run the Q<n> regulatory cycle"
- "Author this quarter's regulatory filings"
- "Prep the 10-Q and supporting filings"

## How to invoke

```bash
cyberos-cuo run cuo/chief-legal-officer/quarterly-regulatory-cycle \
  --input filings_calendar=./legal/regulatory-calendar.md \
  --input compliance_program=./compliance/2026-program.md \
  --input prior_filings=./regulatory/2026-Q1/ \
  --input material_events=./regulatory/2026-Q2/material-events.md \
  --output-dir ./regulatory/2026-Q2/
```

## Expected duration

- **Happy path:** 3-8 hours runtime + 1-3 weeks for CFO/CEO certifications + counsel review
- **Worst case:** breach-discovery during filing triggers Art. 33 72-hour clock; same-day escalation

## Skill chain

- **Step 1 `regulatory-filing-author`** — drafts per regulator (SEC Reg S-K / FDA 21 CFR / AI Act Annex IV / GDPR Art. 30+33 / PDPD).
- **Step 2 `regulatory-filing-audit`** — validates per `regulatory_filing_rubric@1.0` (FM + SEC ordering per regulator + QA-SOURCING-001 + QA-CERTIFICATION-001).

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-SOURCING-001 | Material assertion lacks primary record | Operator supplies record |
| 2 | QA-CERTIFICATION-001 | Missing SOX §302 cert sign-off | Escalate to CFO + CEO |
| 2 | QA-DEADLINE-001 | Filing past regulator deadline | Same-day escalation to CLO + outside counsel |

## Cross-references
- `../README.md` §5 (Operational) — "regulatory submissions"
- `../../../docs/The C-Suite Reference.md` §5.2
- `../../cco-compliance/README.md` — peer persona whose compliance-program@1 is the upstream input
- `../../../skill/regulatory-filing-{author,audit}/SKILL.md`
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
