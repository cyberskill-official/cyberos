---
contract_id: change-mgmt-plan
contract_version: v1
template_literal: change-management-plan@1
description: Canonical change-management-plan@1 — per-program change-mgmt plan authored by chief-transformation-officer; stakeholder analysis + comms plan + training plan + adoption tracking.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `change-management-plan@1` — canonical Change Management Plan contract

> Frontmatter: `change-management-plan-audit/RUBRIC.md` §2.
> Required body sections: §3 (program context / stakeholder analysis (RACI + sentiment) / comms plan per stakeholder group / training plan / adoption tracking + leading indicators / resistance mitigation).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves Chief-Transformation-Officer's stakeholder-NPS + cross-functional-friction KPIs.

## Citations

- C-Suite Reference §5.7
- Prosci ADKAR change-mgmt model
- Kotter 8-step process
- Consumers: `change-management-plan-author`, `change-management-plan-audit`.
