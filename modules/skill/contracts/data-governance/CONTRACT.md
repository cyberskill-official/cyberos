---
contract_id: data-governance
contract_version: v1
template_literal: data-governance@1
description: Canonical data-governance@1 — annual data-governance framework authored by cdo-data + cco-compliance; data classification + stewardship + access control + retention.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `data-governance@1` — canonical Data Governance Framework contract

> Frontmatter: `data-governance-audit/RUBRIC.md` §2.
> Required body sections: §3 (data classification scheme / stewardship model per domain / access-control policy / retention + deletion policy / lineage requirements / privacy controls / compliance mapping).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CDO-Data's data-quality + CCO-Compliance's regulatory-findings KPIs.

## Citations

- C-Suite Reference §5.3 / §5.6
- DAMA-DMBOK governance section
- GDPR + Vietnam Decree 13/2023 data-class requirements
- Consumers: `data-governance-author`, `data-governance-audit`.
