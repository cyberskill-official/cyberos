---
contract_id: compliance-program
contract_version: v1
template_literal: compliance-program@1
description: Canonical compliance-program@1 — compliance management system authored by cco-compliance + cpo-privacy; regulatory mapping + control catalog + training plan.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `compliance-program@1` — canonical Compliance Program (Compliance Management System) contract

> Frontmatter: `compliance-program-audit/RUBRIC.md` §2.
> Required body sections: §3 (applicable regulations / regulatory-mapping matrix / control-catalog with owner per control / training plan / incident-response procedure / audit cadence).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CCO-Compliance's regulatory-findings + training-completion KPIs.

## Citations

- C-Suite Reference §5.6 (CCO-Compliance)
- COSO Internal Control framework
- ISO 19600 / ISO 37301 compliance management system standards
- Consumers: `compliance-program-author`, `compliance-program-audit`.
