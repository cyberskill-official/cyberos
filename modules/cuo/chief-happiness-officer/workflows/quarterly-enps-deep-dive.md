---
workflow_id: chief-happiness-officer/quarterly-enps-deep-dive
workflow_version: 1.0.0
purpose: Deep-dive analysis of eNPS results — verbatim coding, theme clustering, manager-distribution insights, root-cause synthesis.
persona: cuo/chief-happiness-officer
cadence: quarterly
status: shipped

inputs:
  - { name: enps_pulse,            source: cuo/chief-human-resources-officer/quarterly-enps-pulse, format: employee-net-promoter-score-program@1 }
  - { name: verbatims_corpus,      source: full survey verbatims, format: csv }

outputs:
  - { name: enps_deep_dive,        format: employee-net-promoter-score-program@1 (analytical chapter), recipient: cuo/chief-happiness-officer + cuo/chro + cuo/ceo + all managers }

skill_chain:
  - { step: 1, skill: employee-net-promoter-score-program-author, inputs_from: { enps_pulse: enps_pulse, verbatims_corpus: verbatims_corpus }, outputs_to: deep_dive_draft }
  - { step: 2, skill: employee-net-promoter-score-program-audit,  inputs_from: deep_dive_draft, outputs_to: enps_deep_dive }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "executive-team trust score < 50 OR detractor cluster identified" }

audit_hooks:
  - workflow_complete row on PASS with enps_deep_dive hash
  - HITL pause at step 2 on QA-VERBATIM-001 (verbatim themes not validated against response distribution)
---

# Quarterly eNPS deep-dive — `chief-happiness-officer/quarterly-enps-deep-dive`

Chief Happiness Officer's quarterly verbatim-deep-dive sister workflow to CHRO's `quarterly-enps-pulse`. CHRO runs the survey; Chief-Happiness does the qualitative deep-dive on verbatim themes.

## Cross-references
- `../../../../modules/cuo/README.md` §5.5
- `../../chief-human-resources-officer/workflows/quarterly-enps-pulse.md` — upstream peer
- `../../../skill/enps-program-{author,audit}/SKILL.md`
