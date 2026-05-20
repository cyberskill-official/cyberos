---
workflow_id: chief-accounting-officer/annual-accounting-policy
workflow_version: 1.0.0
purpose: Refresh annual accounting-policy manual — revenue recognition, lease accounting, equity, segment reporting, recent standard adoptions.
persona: cuo/chief-accounting-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_policy,          source: last year's strategy-document@1 (accounting-policy chapter), format: strategy-document@1 }
  - { name: standard_updates,      source: FASB / IASB pronouncements + interpretations, format: markdown }
  - { name: business_changes,      source: M&A / new revenue streams / capital structure changes, format: markdown }

outputs:
  - { name: accounting_policy,     format: strategy-document@1 (accounting-policy chapter), recipient: cuo/cao-accounting + cuo/cfo + external auditors + Audit Committee }

skill_chain:
  - { step: 1, skill: strategy-document-author, inputs_from: { prior_policy: prior_policy, standard_updates: standard_updates, business_changes: business_changes }, outputs_to: policy_draft }
  - { step: 2, skill: strategy-document-audit,  inputs_from: policy_draft, outputs_to: accounting_policy }

audit_hooks:
  - workflow_complete row on PASS with accounting_policy hash
  - HITL pause at step 2 on QA-STANDARD-001 (new standard adoption without analysis)
---

# Annual accounting policy — `chief-accounting-officer/annual-accounting-policy`

CAO-Accounting's annual accounting-policy manual per FASB ASC + IFRS + SEC Reg S-X + PCAOB technical-accounting standards.

## Cross-references
- `../../../../modules/cuo/README.md` §5.2
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
