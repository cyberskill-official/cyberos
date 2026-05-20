---
workflow_id: chief-restructuring-officer/per-turnaround-plan
workflow_version: 1.0.0
purpose: Author the per-engagement turnaround plan — situation assessment, stabilization, value creation, exit strategy.
persona: cuo/chief-restructuring-officer
cadence: per-event
status: shipped

inputs:
  - { name: company_brief,         source: client engagement intake, format: markdown }
  - { name: financial_diligence,   source: TWCF + balance-sheet diligence, format: markdown }
  - { name: prior_plans,           source: similar prior turnaround-plan@1, format: turnaround-plan@1 (set) }

outputs:
  - { name: turnaround_plan,       format: turnaround-plan@1, recipient: cuo/cro-restructuring + board / lender / sponsor + cuo/clo-legal }

skill_chain:
  - { step: 1, skill: turnaround-plan-author, inputs_from: { company_brief: company_brief, financial_diligence: financial_diligence, prior_plans: prior_plans }, outputs_to: plan_draft }
  - { step: 2, skill: turnaround-plan-audit,  inputs_from: plan_draft, outputs_to: turnaround_plan }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "plan recommends Chapter 11 filing OR forced asset sale" }
  - { persona: cuo/chief-legal-officer,      when: "plan triggers fiduciary-duty considerations (zone of insolvency)" }

audit_hooks:
  - workflow_complete row on PASS with turnaround_plan hash + projected exit value
  - HITL pause at step 2 on QA-STABILIZATION-001 (no 13-week stabilization plan)
---

# Per turnaround plan — `chief-restructuring-officer/per-turnaround-plan`

CRO-Restructuring's per-engagement turnaround plan per AlixPartners / FTI / Alvarez & Marsal / Berkeley Research Group playbooks.

## Cross-references
- `../../../../modules/cuo/README.md` §5.7 — CRO-Restructuring role profile
- `../../../skill/turnaround-plan-{author,audit}/SKILL.md`
