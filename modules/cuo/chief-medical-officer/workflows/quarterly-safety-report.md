---
workflow_id: chief-medical-officer/quarterly-safety-report
workflow_version: 1.0.0
purpose: Author the quarterly safety report (PSUR/DSUR) — adverse-event review, signal detection, benefit-risk re-evaluation.
persona: cuo/chief-medical-officer
cadence: quarterly
status: shipped

inputs:
  - { name: prior_report,          source: last quarter's safety-report@1, format: safety-report@1 }
  - { name: adverse_events,        source: safety database (pharmacovigilance system), format: csv }
  - { name: literature_review,     source: published literature monitoring, format: markdown }
  - { name: trial_data,            source: ongoing clinical-trial interim data, format: markdown }

outputs:
  - { name: safety_report,         format: safety-report@1, recipient: cuo/chief-medical-officer + FDA/EMA (PSUR/DSUR) + clinical sites + cuo/clo-legal }

skill_chain:
  - { step: 1, skill: safety-report-author, inputs_from: { prior_report: prior_report, adverse_events: adverse_events, literature_review: literature_review, trial_data: trial_data }, outputs_to: report_draft }
  - { step: 2, skill: safety-report-audit,  inputs_from: report_draft, outputs_to: safety_report }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "signal detection triggers benefit-risk shift OR product label change" }
  - { persona: cuo/chief-legal-officer,      when: "regulatory submission triggers product-liability re-assessment" }

audit_hooks:
  - workflow_complete row on PASS with safety_report hash + signal count
  - HITL pause at step 2 on QA-ICH-E2D-001 (PSUR section ordering) or QA-SIGNAL-001
---

# Quarterly safety report — `chief-medical-officer/quarterly-safety-report`

Chief Medical Officer's quarterly safety report per ICH E2D PSUR + 21 CFR 314.80 + EMA Module VII Periodic Safety Update Report standards.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7
- `./per-clinical-protocol.md` — upstream feeder
- `../../../skill/safety-report-{author,audit}/SKILL.md`
