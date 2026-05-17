---
workflow_id: chief-people-officer/annual-employee-value-proposition
workflow_version: 1.0.0
purpose: Author the annual Employee Value Proposition (EVP) — culture, total rewards, career growth, mission alignment.
persona: cuo/chief-people-officer
cadence: annual
status: shipped

inputs:
  - { name: prior_evp,             source: last year's EVP doc, format: strategy-doc@1 (EVP chapter) }
  - { name: enps_history,          source: 4 quarters of enps-program@1, format: enps-program@1 (4Q) }
  - { name: brand_strategy,        source: cuo/chief-brand-officer/annual-brand-strategy, format: brand-strategy@1 }

outputs:
  - { name: evp,                   format: strategy-doc@1 (EVP chapter), recipient: cuo/cpo-people + cuo/chro + cuo/cmo + cuo/chief-brand-officer + Board }

skill_chain:
  - { step: 1, skill: strategy-doc-author, inputs_from: { prior_evp: prior_evp, enps_history: enps_history, brand_strategy: brand_strategy }, outputs_to: evp_draft }
  - { step: 2, skill: strategy-doc-audit,  inputs_from: evp_draft, outputs_to: evp }

audit_hooks:
  - workflow_complete row on PASS with evp hash
  - HITL pause at step 2 on QA-KERNEL-001
---

# Annual Employee Value Proposition — `chief-people-officer/annual-employee-value-proposition`

CPO-People's annual EVP per Gartner EVP framework + LinkedIn Employer Brand + Universum Top Employers research.

## Cross-references
- `../../../docs/The C-Suite Reference.md` §5.5
- `../../chief-brand-officer/workflows/annual-brand-strategy.md` — brand alignment
- `../../../skill/strategy-doc-{author,audit}/SKILL.md`
