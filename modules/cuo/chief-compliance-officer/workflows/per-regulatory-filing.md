---
workflow_id: chief-compliance-officer/per-regulatory-filing
workflow_version: 1.0.0
purpose: Author a per-event regulatory filing — compliance-driven (vs CLO-Legal's litigation-driven) — annual reports, attestations, registration renewals.
persona: cuo/chief-compliance-officer
cadence: per-event
status: shipped

inputs:
  - { name: filing_brief,          source: regulator calendar + filing template, format: markdown }
  - { name: compliance_program,    source: cuo/chief-compliance-officer/annual-compliance-program, format: compliance-program@1 }
  - { name: control_evidence,      source: GRC evidence repository, format: directory tree }

outputs:
  - { name: regulatory_filing,     format: regulatory-filing@1, recipient: regulator + cuo/cco-compliance + cuo/clo-legal }

skill_chain:
  - { step: 1, skill: regulatory-filing-author, inputs_from: { filing_brief: filing_brief, compliance_program: compliance_program, control_evidence: control_evidence }, outputs_to: filing_draft }
  - { step: 2, skill: regulatory-filing-audit,  inputs_from: filing_draft, outputs_to: regulatory_filing }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "filing has litigation-adjacent risk OR cross-jurisdiction complexity" }

consults:
  - { persona: cuo/chief-information-security-officer,           when: "filing certifies security controls" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with regulatory_filing hash + regulator + filing-type + due-date
  - HITL pause at step 2 on QA-DEADLINE-001 (approaching deadline) or QA-EVIDENCE-001 (evidence reference missing)
---

# Per regulatory filing — `chief-compliance-officer/per-regulatory-filing`

CCO-Compliance's per-event regulatory filing workflow. Distinct from `chief-legal-officer/quarterly-regulatory-cycle` (which is litigation/regulator-action driven) — this is compliance-program driven: annual reports, attestations, registration renewals, SOC reports.

## When to invoke

- "File the [regulator] [filing type]"
- "Compliance filing for [requirement]"
- "Submit attestation"

## How to invoke

```bash
cyberos-cuo run cuo/chief-compliance-officer/per-regulatory-filing \
  --input filing_brief=./compliance/filings/2026-soc2-annual/brief.md \
  --input compliance_program=./compliance/2026/program.md \
  --input control_evidence=./grc/2026/evidence/ \
  --output-dir ./compliance/filings/2026-soc2-annual/
```

## Expected duration

- **Happy path:** 4-8 hours runtime + 2-4 weeks for evidence gathering + auditor review
- **Worst case:** deadline approach triggers expedited filing + risk acceptance

## Skill chain

- **Step 1 `regulatory-filing-author`** — drafts per regulator template.
- **Step 2 `regulatory-filing-audit`** — validates per `regulatory_filing_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-DEADLINE-001 | Approaching deadline | Operator expedites |
| 2 | QA-EVIDENCE-001 | Evidence missing | Operator gathers |

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.6 — CCO-Compliance role profile
- `../../chief-legal-officer/workflows/quarterly-regulatory-cycle.md` — litigation-driven peer
- `../../../skill/regulatory-filing-{author,audit}/SKILL.md`
