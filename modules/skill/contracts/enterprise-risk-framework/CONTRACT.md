---
contract_id: enterprise-risk-framework
contract_version: v1
template_literal: enterprise-risk-framework@1
description: Canonical enterprise-risk-framework@1 — annual enterprise-risk framework authored by cro-risk; risk-appetite vs risk-tolerance + KRI taxonomy + scenario library.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `enterprise-risk-framework@1` — canonical Enterprise Risk Framework contract

> Frontmatter: `enterprise-risk-framework-audit/RUBRIC.md` §2.
> Required body sections: §3 (risk appetite / risk tolerance / KRI taxonomy / scenario library / stress-test catalog / governance cadence / appetite-vs-tolerance reconciliation).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CRO-Risk's KRI-breach-count + capital-adequacy + scenario-test-coverage KPIs.

## Citations

- C-Suite Reference §5.6 (CRO-Risk)
- COSO Enterprise Risk Management framework
- ISO 31000 Risk Management
- Consumers: `enterprise-risk-framework-author`, `enterprise-risk-framework-audit`.
