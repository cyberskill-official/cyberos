---
workflow_id: chief-accounting-officer/quarterly-audit-readiness
workflow_version: 1.0.0
purpose: Prep quarterly audit-readiness — control testing, evidence collection, walkthroughs, technical-accounting position papers.
persona: cuo/chief-accounting-officer
cadence: quarterly
status: shipped

inputs:
  - { name: control_testing,       source: cuo/chief-compliance-officer/quarterly-control-testing, format: compliance-program@1 chapter }
  - { name: prior_readiness,       source: last quarter's readiness assessment, format: compliance-program@1 (audit-readiness chapter) }
  - { name: technical_papers,      source: technical-accounting position papers, format: markdown }

outputs:
  - { name: audit_readiness,       format: compliance-program@1 (audit-readiness chapter), recipient: cuo/cao-accounting + cuo/cfo + external auditors + cuo/cco-compliance }

skill_chain:
  - { step: 1, skill: compliance-program-author, inputs_from: { control_testing: control_testing, prior_readiness: prior_readiness, technical_papers: technical_papers }, outputs_to: readiness_draft }
  - { step: 2, skill: compliance-program-audit,  inputs_from: readiness_draft, outputs_to: audit_readiness }

escalates_to:
  - { persona: cuo/chief-financial-officer,            when: "material weakness identified pre-audit" }

audit_hooks:
  - workflow_complete row on PASS with audit_readiness hash
  - HITL pause at step 2 on QA-CONTROL-001 or QA-WALKTHROUGH-001
---

# Quarterly audit readiness — `chief-accounting-officer/quarterly-audit-readiness`

CAO-Accounting's quarterly audit-readiness per PCAOB AS 5 + AICPA SOC 1 + ICFR (SOX 404) walkthrough standards.

## Cross-references
- `../../../../modules/cuo/README.md` §5.2
- `../../chief-compliance-officer/workflows/quarterly-control-testing.md` — upstream feeder
- `../../../skill/compliance-program-{author,audit}/SKILL.md`
