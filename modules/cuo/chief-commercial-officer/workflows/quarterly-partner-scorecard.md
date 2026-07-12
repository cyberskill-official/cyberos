---
workflow_id: chief-commercial-officer/quarterly-partner-scorecard
workflow_version: 1.0.0
purpose: Score active partners — pipeline contribution, closed-won attribution, enablement engagement, tier promotion/demotion.
persona: cuo/chief-commercial-officer
cadence: quarterly
status: shipped

inputs:
  - { name: partner_program,       source: cuo/chief-commercial-officer/annual-partner-program, format: partner-program@1 }
  - { name: pipeline_attribution,  source: CRM partner-source data, format: csv }
  - { name: prior_scorecard,       source: last quarter's scorecard, format: partner-program@1 (quarterly chapter) }

outputs:
  - { name: partner_scorecard,     format: partner-program@1 (quarterly chapter), recipient: cuo/cco-commercial + cuo/cso-sales + partner-success managers }

skill_chain:
  - { step: 1, skill: partner-program-author, inputs_from: { partner_program: partner_program, pipeline_attribution: pipeline_attribution, prior_scorecard: prior_scorecard }, outputs_to: scorecard_draft }
  - { step: 2, skill: partner-program-audit,  inputs_from: scorecard_draft, outputs_to: partner_scorecard }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "top-3 partner contribution drops > 25% QoQ" }

audit_hooks:
  - workflow_complete row on PASS with partner_scorecard hash + tier-changes count
  - HITL pause at step 2 on QA-ATTRIBUTION-001
---

# Quarterly partner scorecard — `chief-commercial-officer/quarterly-partner-scorecard`

CCO-Commercial's quarterly partner scoring per Crossbeam ecosystem-led growth + Impartner partner-performance-management.

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4
- `./annual-partner-program.md` — upstream parent
- `../../../skill/partner-program-{author,audit}/SKILL.md`
