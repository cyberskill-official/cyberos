---
workflow_id: chief-architect/per-architecture-decision
workflow_version: 1.0.0
purpose: Author an Architecture Decision Record (ADR) per Nygard format with chain to threat-model + SDD.
persona: cuo/chief-architect
cadence: per-event
status: shipped

inputs:
  - { name: decision_brief,        source: requestor (engineer / CTO / Chief-Architect), format: markdown }
  - { name: srs_context,           source: upstream SRS or PRD, format: markdown }
  - { name: prior_adrs,            source: existing ADRs (decision history), format: architecture-decision-record@1 (set) }

outputs:
  - { name: adr,                   format: adr@1, recipient: cuo/chief-architect + cuo/cto + engineering team + future engineers }

skill_chain:
  - { step: 1, skill: architecture-decision-record-author, inputs_from: { decision_brief: decision_brief, srs_context: srs_context, prior_adrs: prior_adrs }, outputs_to: adr_draft }
  - { step: 2, skill: architecture-decision-record-audit,  inputs_from: adr_draft, outputs_to: adr }

escalates_to:
  - { persona: cuo/chief-information-security-officer,           when: "decision touches security boundary (auth / crypto / data-handling)" }

audit_hooks:
  - workflow_complete row on PASS with adr hash
  - HITL pause at step 2 on QA-OPT-001 (single-option ADR)
---

# Per architecture decision — `chief-architect/per-architecture-decision`

Chief-Architect's per-decision ADR workflow per Michael Nygard ADR format + ISO/IEC 25010 quality attributes.

## Cross-references
- `../../../../modules/cuo/README.md` §5.3 — Chief Architect role profile
- `../../chief-technology-officer/workflows/adr-quick-capture.md` — CTO peer (CTO captures most ADRs; Chief-Architect authors major architecture-class ADRs)
- `../../../skill/adr-{author,audit}/SKILL.md`
