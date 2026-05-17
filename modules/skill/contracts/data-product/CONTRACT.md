---
contract_id: data-product
contract_version: v1
template_literal: data-product@1
description: Canonical data-product@1 — per-data-product spec authored by cdo-data; user / dataset / SLAs / lineage / quality contract.
contract_kind: artefact_schema
locked_at: 2026-05-17

steward_persona: cuo-ceo
escalation_on_breach: { legal: cuo-clo, security: cuo-cseco, compliance: cuo-clo }
determinism: { reproducible: false, fixity_notes: "Content is judgement-shaped; section set + frontmatter shape are stable." }
emitted_source_freshness_tier: 12
---

# `data-product@1` — canonical Data Product Spec contract

> Frontmatter: `data-product-audit/RUBRIC.md` §2.
> Required body sections: §3 (product summary / target users / dataset description / SLAs (freshness / completeness / accuracy) / lineage / quality contract / consumption interface).
> Conditional sections: §4 per audit RUBRIC.
> KPI tie: moves CDO-Data's model-deployment-count + data-product-revenue KPIs.

## Citations

- C-Suite Reference §5.3 (CDO-Data)
- Data Mesh data-as-product principles
- Open Data Product Specification
- Consumers: `data-product-author`, `data-product-audit`.
