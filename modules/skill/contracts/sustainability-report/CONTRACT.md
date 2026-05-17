---
contract_id: sustainability-report
contract_version: v1
template_literal: sustainability-report@1
description: Canonical sustainability-report@1 — annual sustainability report authored by cso-sustainability or chief-esg-officer; CSRD/ESRS or ISSB-aligned disclosure.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `sustainability-report@1` — canonical Sustainability Report contract

> Frontmatter: `sustainability-report-audit/RUBRIC.md` §2.
> Required body sections: §3 (executive summary / governance / strategy / risk + opportunity / metrics (Scope 1/2/3) / targets / progress against prior year / assurance statement).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CSO-Sustainability's emissions-reduction + ESG-rating KPIs.

## Citations

- C-Suite Reference §5.7 (CSO-Sustainability / Chief ESG)
- CSRD / ESRS reporting standards (EU)
- ISSB IFRS S1 + S2 standards
- Consumers: `sustainability-report-author`, `sustainability-report-audit`.
