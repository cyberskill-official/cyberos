---
contract_id: kri-dashboard
contract_version: v1
template_literal: kri-dashboard@1
description: Canonical kri-dashboard@1 — quarterly KRI dashboard authored by cro-risk; key risk indicators with breach thresholds + trend analysis.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `kri-dashboard@1` — canonical KRI Dashboard contract

> Frontmatter: `kri-dashboard-audit/RUBRIC.md` §2.
> Required body sections: §3 (summary / KRI table per risk category / breach detail per indicator / trend charts / control effectiveness / mitigation status).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CRO-Risk's KRI-breach-count + risk-incident-MTTR KPIs.

## Citations

- C-Suite Reference §5.6 (CRO-Risk)
- Workiva / Archer / Riskonnect ERM patterns
- Consumers: `kri-dashboard-author`, `kri-dashboard-audit`.
