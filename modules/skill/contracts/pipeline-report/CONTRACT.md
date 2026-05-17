---
contract_id: pipeline-report
contract_version: v1
template_literal: pipeline-report@1
description: Canonical pipeline-report@1 — weekly pipeline report authored by cro-revenue or cso-sales; coverage / win-rate / stage progression.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `pipeline-report@1` — canonical Pipeline Report contract

> Frontmatter: `pipeline-report-audit/RUBRIC.md` §2.
> Required body sections: §3 (summary / pipeline by stage / coverage vs quota / win-rate trend / deal-velocity / at-risk deals / forecast roll-up).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CRO's pipeline-coverage (3-4× target) + win-rate KPIs.

## Citations

- C-Suite Reference §5.2 (CRO sales engine)
- Salesforce / Clari pipeline-discipline patterns
- Consumers: `pipeline-report-author`, `pipeline-report-audit`.
